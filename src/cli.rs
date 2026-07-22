use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::Serialize;
use tokio::time::sleep;

use crate::config::{local_base, AppPaths};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[derive(Debug, Clone, Serialize)]
pub struct DaemonControlReport {
    pub action: String,
    pub ok: bool,
    pub healthy: bool,
    pub gateway: String,
    pub pids: Vec<u32>,
    pub detail: String,
}

pub async fn ensure_daemon() -> Result<()> {
    if is_healthy().await {
        return Ok(());
    }

    let exe = env::current_exe()?;
    let mut command = Command::new(exe);
    command
        .arg("daemon")
        .arg("run")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    spawn_background(&mut command)?;

    for _ in 0..40 {
        if is_healthy().await {
            return Ok(());
        }
        sleep(Duration::from_millis(150)).await;
    }

    Err(anyhow!("qorx daemon did not become healthy"))
}

pub async fn start_daemon() -> Result<DaemonControlReport> {
    if is_healthy().await {
        return Ok(control_report("start", true, daemon_pids(), "daemon already healthy").await);
    }

    ensure_daemon().await?;
    Ok(control_report("start", true, daemon_pids(), "daemon started").await)
}

pub async fn stop_daemon() -> Result<DaemonControlReport> {
    let pids = daemon_pids();
    if pids.is_empty() {
        let healthy = is_healthy().await;
        return Ok(DaemonControlReport {
            action: "stop".to_string(),
            ok: !healthy,
            healthy,
            gateway: local_base(),
            pids,
            detail: if healthy {
                "gateway is healthy, but no local daemon process owned by this user was found; it may be supervised by systemd, Docker, or another wrapper".to_string()
            } else {
                "daemon was not running".to_string()
            },
        });
    }

    for pid in &pids {
        stop_pid(*pid);
    }

    for _ in 0..40 {
        if !is_healthy().await {
            return Ok(DaemonControlReport {
                action: "stop".to_string(),
                ok: true,
                healthy: false,
                gateway: local_base(),
                pids,
                detail: "daemon stopped".to_string(),
            });
        }
        sleep(Duration::from_millis(150)).await;
    }

    Ok(DaemonControlReport {
        action: "stop".to_string(),
        ok: false,
        healthy: is_healthy().await,
        gateway: local_base(),
        pids,
        detail: "stop command sent, but gateway still answers health checks".to_string(),
    })
}

pub async fn daemon_status() -> DaemonControlReport {
    control_report("status", true, daemon_pids(), "daemon status").await
}

async fn control_report(
    action: &str,
    ok: bool,
    pids: Vec<u32>,
    detail: &str,
) -> DaemonControlReport {
    DaemonControlReport {
        action: action.to_string(),
        ok,
        healthy: is_healthy().await,
        gateway: local_base(),
        pids,
        detail: detail.to_string(),
    }
}

fn spawn_background(command: &mut Command) -> Result<()> {
    configure_background(command);
    command.spawn()?;
    Ok(())
}

#[cfg(windows)]
fn configure_background(command: &mut Command) {
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn configure_background(_command: &mut Command) {}

pub async fn is_healthy() -> bool {
    Client::new()
        .get(format!("{}/health", local_base()))
        .timeout(Duration::from_millis(700))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

fn daemon_pids() -> Vec<u32> {
    if cfg!(windows) {
        return daemon_pids_windows();
    }
    daemon_pids_unix()
}

#[cfg(windows)]
fn daemon_pids_windows() -> Vec<u32> {
    let current = std::process::id();
    let script = format!(
        "Get-CimInstance Win32_Process -Filter \"name = 'qorx.exe'\" | Where-Object {{ $_.ProcessId -ne {current} -and $_.CommandLine -match '(^|\\s)daemon(\\s|$)' }} | ForEach-Object {{ $_.ProcessId }}"
    );
    Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(script)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| parse_pid_lines(&String::from_utf8_lossy(&output.stdout)))
        .unwrap_or_default()
}

#[cfg(not(windows))]
fn daemon_pids_windows() -> Vec<u32> {
    Vec::new()
}

fn daemon_pids_unix() -> Vec<u32> {
    if cfg!(windows) {
        return Vec::new();
    }
    let current = std::process::id();
    Command::new("ps")
        .args(["-eo", "pid=,args="])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter_map(|line| parse_unix_daemon_pid(line, current))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_unix_daemon_pid(line: &str, current: u32) -> Option<u32> {
    let mut parts = line.trim().splitn(2, char::is_whitespace);
    let pid = parts.next()?.parse::<u32>().ok()?;
    if pid == current {
        return None;
    }
    let args = parts.next().unwrap_or_default();
    (args.contains("qorx") && has_daemon_arg(args)).then_some(pid)
}

fn has_daemon_arg(command_line: &str) -> bool {
    command_line.split_whitespace().any(|arg| arg == "daemon")
}

fn parse_pid_lines(text: &str) -> Vec<u32> {
    text.lines()
        .filter_map(|line| line.trim().parse::<u32>().ok())
        .collect()
}

fn stop_pid(pid: u32) {
    if cfg!(windows) {
        let _ = Command::new("powershell.exe")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(format!("Stop-Process -Id {pid} -Force"))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    } else {
        let _ = Command::new("kill")
            .arg(pid.to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

pub async fn print_stats(paths: &AppPaths) -> Result<()> {
    if env::var_os("QORX_HOME").is_none() && is_healthy().await {
        let text = Client::new()
            .get(format!("{}/stats", local_base()))
            .send()
            .await?
            .text()
            .await?;
        println!("{text}");
        return Ok(());
    }

    let legacy = paths.stats_file.with_extension("json");
    let stats: crate::stats::Stats =
        crate::proto_store::load_or_default(&paths.stats_file, &[legacy.as_path()])?;
    println!("{}", serde_json::to_string(&stats)?);
    Ok(())
}

pub async fn reset_stats(paths: &AppPaths) -> Result<()> {
    if env::var_os("QORX_HOME").is_none() && is_healthy().await {
        let text = Client::new()
            .post(format!("{}/stats/reset", local_base()))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        println!("{text}");
        return Ok(());
    }

    let stats = crate::stats::reset(&paths.stats_file)?;
    println!("{}", serde_json::to_string(&stats)?);
    Ok(())
}

pub async fn run_provider(provider: &str, args: Vec<String>) -> Result<i32> {
    ensure_daemon().await?;
    let mut command = match provider {
        "codex" => {
            let mut cmd = provider_command("codex")?;
            cmd.env("QORX_CODEX_HOOK", "1");
            cmd.env("QORX_GATEWAY", local_base());
            cmd.args(codex_tight_args(
                qorx_codex_proxy_enabled(),
                qorx_codex_disable_memories_enabled(),
            ));
            cmd
        }
        "claude" => {
            let mut cmd = provider_command("claude")?;
            cmd.env("ANTHROPIC_BASE_URL", format!("{}/anthropic", local_base()));
            cmd
        }
        "gemini" => {
            let mut cmd = provider_command("gemini")?;
            cmd.env("GOOGLE_GEMINI_BASE_URL", format!("{}/gemini", local_base()));
            cmd
        }
        other => return Err(anyhow!("unknown provider: {other}")),
    };

    command.args(args);
    let status = command.status()?;
    Ok(status.code().unwrap_or(1))
}

fn provider_command(name: &str) -> Result<Command> {
    let path = resolve_on_path(name).ok_or_else(|| anyhow!("program not found: {name}"))?;
    if path
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("ps1"))
    {
        let mut cmd = Command::new("powershell.exe");
        cmd.arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-File")
            .arg(path);
        return Ok(cmd);
    }

    Ok(Command::new(path))
}

fn qorx_codex_proxy_enabled() -> bool {
    truthy_env("QORX_CODEX_PROXY")
}

fn qorx_codex_disable_memories_enabled() -> bool {
    truthy_env("QORX_CODEX_DISABLE_MEMORIES") && !truthy_env("QORX_CODEX_RAW_MEMORIES")
}

fn truthy_env(key: &str) -> bool {
    env::var(key)
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn codex_tight_args(proxy_enabled: bool, disable_memories: bool) -> Vec<String> {
    let mut args = Vec::new();
    if proxy_enabled {
        args.push("-c".to_string());
        args.push(format!("openai_base_url=\"{}/v1\"", local_base()));
    }
    if disable_memories {
        args.push("--disable".to_string());
        args.push("memories".to_string());
    }
    args
}

fn resolve_on_path(name: &str) -> Option<PathBuf> {
    let candidates = if cfg!(windows) {
        vec![
            OsString::from(format!("{name}.exe")),
            OsString::from(format!("{name}.cmd")),
            OsString::from(format!("{name}.bat")),
            OsString::from(format!("{name}.ps1")),
            OsString::from(name),
        ]
    } else {
        vec![OsString::from(name)]
    };

    let path_env = env::var_os("PATH")?;
    for dir in env::split_paths(&path_env) {
        for candidate in &candidates {
            let full = dir.join(candidate);
            if is_file(&full) {
                return Some(full);
            }
        }
    }
    None
}

fn is_file(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|m| m.is_file())
        .unwrap_or(false)
}

pub fn patch_provider(provider: &str, apply: bool) -> Result<()> {
    match provider {
        "codex" => patch_codex(apply),
        "claude" => {
            println!("Claude Code wrapper env:");
            println!("ANTHROPIC_BASE_URL={}/anthropic", local_base());
            Ok(())
        }
        "gemini" => {
            println!("Gemini CLI wrapper env:");
            println!("GOOGLE_GEMINI_BASE_URL={}/gemini", local_base());
            Ok(())
        }
        other => Err(anyhow!("unknown provider: {other}")),
    }
}

fn patch_codex(apply: bool) -> Result<()> {
    let config_path = codex_config_path()?;

    if !apply {
        println!("Codex can be launched with Qorx hook context using:");
        println!("qorx run codex -- <prompt or options>");
        println!();
        println!("No OpenAI API proxy profile is installed by default.");
        println!(
            "Only set QORX_CODEX_PROXY=1 manually when you intentionally want {}/v1.",
            local_base()
        );
        return Ok(());
    }

    let mut existing = if config_path.exists() {
        std::fs::read_to_string(&config_path)?
    } else {
        String::new()
    };
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let updated = remove_toml_section(&existing, "[profiles.qorx]");
    if updated == existing {
        println!(
            "No Codex Qorx API profile found in {}",
            config_path.display()
        );
        return Ok(());
    }

    existing = updated;
    std::fs::write(&config_path, existing)?;
    println!(
        "Removed Codex Qorx API profile from {}",
        config_path.display()
    );
    Ok(())
}

fn codex_config_path() -> Result<PathBuf> {
    let home = env::var("USERPROFILE").or_else(|_| env::var("HOME"))?;
    Ok(PathBuf::from(home).join(".codex").join("config.toml"))
}

fn remove_toml_section(input: &str, section: &str) -> String {
    let mut output = Vec::new();
    let mut skipping = false;

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed == section {
            skipping = true;
            continue;
        }
        if skipping && trimmed.starts_with('[') && trimmed.ends_with(']') {
            skipping = false;
        }
        if !skipping {
            output.push(line);
        }
    }

    let mut result = output.join("\n");
    if input.ends_with('\n') {
        result.push('\n');
    }
    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn codex_tight_args_keep_qorx_memory_without_proxy_by_default() {
        let args = super::codex_tight_args(false, false);

        assert!(args.is_empty());
        assert!(!args.iter().any(|arg| arg.contains("openai_base_url")));
        assert!(!args
            .windows(2)
            .any(|pair| pair == ["--disable", "memories"]));
    }

    #[test]
    fn codex_tight_args_can_enable_proxy_explicitly() {
        let args = super::codex_tight_args(true, false);

        assert!(args.iter().any(|arg| arg == "-c"));
        assert!(args.iter().any(|arg| arg.contains("openai_base_url")));
        assert!(!args
            .windows(2)
            .any(|pair| pair == ["--disable", "memories"]));
    }

    #[test]
    fn codex_tight_args_can_disable_memories_explicitly() {
        let args = super::codex_tight_args(false, true);

        assert!(args
            .windows(2)
            .any(|pair| pair == ["--disable", "memories"]));
    }

    #[test]
    fn raw_memories_env_overrides_disable_memories_env() {
        unsafe {
            std::env::set_var("QORX_CODEX_DISABLE_MEMORIES", "1");
            std::env::set_var("QORX_CODEX_RAW_MEMORIES", "1");
        }

        assert!(!super::qorx_codex_disable_memories_enabled());

        unsafe {
            std::env::remove_var("QORX_CODEX_DISABLE_MEMORIES");
            std::env::remove_var("QORX_CODEX_RAW_MEMORIES");
        }
    }

    #[test]
    fn remove_toml_section_removes_only_target_section() {
        let input =
            "model = \"gpt-5.5\"\n[profiles.qorx]\nmodel_provider = \"openai\"\nopenai_base_url = \"http://127.0.0.1:47187/v1\"\n[features]\nmemories = true\n";

        let output = super::remove_toml_section(input, "[profiles.qorx]");

        assert!(!output.contains("[profiles.qorx]"));
        assert!(!output.contains("openai_base_url"));
        assert!(output.contains("[features]"));
        assert!(output.contains("memories = true"));
    }
}
