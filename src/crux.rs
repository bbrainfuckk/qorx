use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use anyhow::{anyhow, Result};
use chrono::{SecondsFormat, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    config::{self, AppPaths},
    integrations,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CruxState {
    pub schema: String,
    pub pid: u32,
    pub started_at_utc: String,
    pub planned_iterations: u64,
    pub interval_seconds: u64,
    pub log: String,
    pub summary: String,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CruxRunReport {
    pub schema: String,
    pub action: String,
    pub ok: bool,
    pub pid: Option<u32>,
    pub planned_iterations: u64,
    pub interval_seconds: u64,
    pub log: String,
    pub summary: String,
    pub state_file: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CruxSummary {
    pub schema: String,
    pub started_at_utc: String,
    pub finished_at_utc: String,
    pub iterations: u64,
    pub failures: u64,
    pub ok: bool,
    pub first_failure: Option<CruxIteration>,
    pub log_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CruxIteration {
    pub iteration: u64,
    pub timestamp_utc: String,
    pub ok: bool,
    pub elapsed_seconds: f64,
    pub version: Option<String>,
    pub bind: Option<String>,
    pub handle: Option<String>,
    pub checks: BTreeMap<String, bool>,
    pub errors: Vec<String>,
    pub status: Value,
    pub configs: Value,
    pub mcp_self_test: integrations::McpSelfTestReport,
}

#[derive(Debug, Clone, Serialize)]
pub struct CruxReport {
    pub schema: String,
    pub running: bool,
    pub pid: Option<u32>,
    pub iterations: u64,
    pub failures: u64,
    pub last_ok: Option<bool>,
    pub latest_log: Option<String>,
    pub summary: Option<CruxSummary>,
    pub state: Option<CruxState>,
}

pub fn start(
    paths: &AppPaths,
    hours: f64,
    interval_seconds: u64,
    log: Option<PathBuf>,
) -> Result<CruxRunReport> {
    let interval_seconds = interval_seconds.max(1);
    let planned_iterations = iterations_for_hours(hours, interval_seconds);
    let dir = crux_dir(paths);
    fs::create_dir_all(&dir)?;
    let stamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let log = log.unwrap_or_else(|| dir.join(format!("crux-stress-{stamp}.jsonl")));
    let summary = dir.join(format!("crux-stress-{stamp}.summary.json"));
    let stdout = dir.join(format!("crux-stress-{stamp}.stdout.log"));
    let stderr = dir.join(format!("crux-stress-{stamp}.stderr.log"));
    let state_file = state_file(paths);
    let exe = env::current_exe()?;

    let mut command = Command::new(&exe);
    command
        .arg("crux")
        .arg("worker")
        .arg("--iterations")
        .arg(planned_iterations.to_string())
        .arg("--interval-seconds")
        .arg(interval_seconds.to_string())
        .arg("--log")
        .arg(&log)
        .arg("--summary")
        .arg(&summary)
        .stdin(Stdio::null())
        .stdout(fs::File::create(&stdout)?)
        .stderr(fs::File::create(&stderr)?);
    configure_background(&mut command);
    let child = command.spawn()?;
    let state = CruxState {
        schema: "qorx.crux-state.v1".to_string(),
        pid: child.id(),
        started_at_utc: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        planned_iterations,
        interval_seconds,
        log: log.display().to_string(),
        summary: summary.display().to_string(),
        stdout: stdout.display().to_string(),
        stderr: stderr.display().to_string(),
    };
    fs::write(&state_file, serde_json::to_string_pretty(&state)?)?;

    Ok(CruxRunReport {
        schema: "qorx.crux-run.v1".to_string(),
        action: "run".to_string(),
        ok: true,
        pid: Some(state.pid),
        planned_iterations,
        interval_seconds,
        log: state.log,
        summary: state.summary,
        state_file: state_file.display().to_string(),
        detail: "Crux stress worker started in the background".to_string(),
    })
}

pub fn stop(paths: &AppPaths) -> Result<CruxRunReport> {
    let state_file = state_file(paths);
    let state = load_state(paths)?;
    let mut stopped = false;
    if let Some(state) = &state {
        stopped = stop_pid(state.pid);
    }
    let report = report(paths)?;
    Ok(CruxRunReport {
        schema: "qorx.crux-run.v1".to_string(),
        action: "stop".to_string(),
        ok: true,
        pid: state.as_ref().map(|state| state.pid),
        planned_iterations: state
            .as_ref()
            .map(|state| state.planned_iterations)
            .unwrap_or(0),
        interval_seconds: state
            .as_ref()
            .map(|state| state.interval_seconds)
            .unwrap_or(0),
        log: report.latest_log.unwrap_or_default(),
        summary: state
            .as_ref()
            .map(|state| state.summary.clone())
            .unwrap_or_default(),
        state_file: state_file.display().to_string(),
        detail: if stopped {
            "Crux stress worker stopped".to_string()
        } else {
            "No running Crux stress worker was found".to_string()
        },
    })
}

pub fn report(paths: &AppPaths) -> Result<CruxReport> {
    let state = load_state(paths)?;
    let latest_log = state
        .as_ref()
        .map(|state| PathBuf::from(&state.log))
        .filter(|path| path.exists())
        .or_else(|| latest_log(paths));
    let records = latest_log
        .as_ref()
        .map(|path| read_iterations(path))
        .transpose()?
        .unwrap_or_default();
    let failures = records.iter().filter(|record| !record.ok).count() as u64;
    let summary = state
        .as_ref()
        .and_then(|state| read_summary(Path::new(&state.summary)).ok());
    let running = state
        .as_ref()
        .is_some_and(|state| process_is_alive(state.pid));
    Ok(CruxReport {
        schema: "qorx.crux-report.v1".to_string(),
        running,
        pid: state.as_ref().map(|state| state.pid),
        iterations: records.len() as u64,
        failures,
        last_ok: records.last().map(|record| record.ok),
        latest_log: latest_log.map(|path| path.display().to_string()),
        summary,
        state,
    })
}

pub fn rollback(checkpoint: &Path) -> Result<integrations::CheckpointReport> {
    integrations::restore_checkpoint(checkpoint)
}

pub async fn worker(
    iterations: u64,
    interval_seconds: u64,
    log: PathBuf,
    summary: PathBuf,
) -> Result<()> {
    let start = Instant::now();
    let started_at_utc = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let mut failures = Vec::new();
    if let Some(parent) = log.parent() {
        fs::create_dir_all(parent)?;
    }
    let exe = env::current_exe()?;
    let client = Client::new();
    for iteration in 1..=iterations {
        let target =
            Duration::from_secs(interval_seconds.saturating_mul(iteration.saturating_sub(1)));
        if start.elapsed() < target {
            tokio::time::sleep(target - start.elapsed()).await;
        }
        let record = one_iteration(&client, &exe, iteration, start.elapsed().as_secs_f64()).await;
        append_jsonl(&log, &record)?;
        if !record.ok {
            failures.push(record);
        }
    }
    let report = CruxSummary {
        schema: "qorx.crux-stress.v1".to_string(),
        started_at_utc,
        finished_at_utc: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        iterations,
        failures: failures.len() as u64,
        ok: failures.is_empty(),
        first_failure: failures.into_iter().next(),
        log_path: log.display().to_string(),
    };
    fs::write(summary, serde_json::to_string_pretty(&report)?)?;
    Ok(())
}

fn iterations_for_hours(hours: f64, interval_seconds: u64) -> u64 {
    let seconds = (hours.max(0.0) * 3600.0).ceil() as u64;
    seconds.div_ceil(interval_seconds.max(1)).max(1)
}

async fn one_iteration(
    client: &Client,
    exe: &Path,
    iteration: u64,
    elapsed_seconds: f64,
) -> CruxIteration {
    let mut checks = BTreeMap::new();
    let mut errors = Vec::new();
    let mut version = None;
    let mut bind = None;
    let mut handle = None;

    let doctor = run_json(exe, &["doctor", "--json"], None);
    let doctor_ok = match doctor {
        Ok(value) => {
            version = value
                .get("version")
                .and_then(Value::as_str)
                .map(str::to_string);
            bind = value
                .get("bind")
                .and_then(Value::as_str)
                .map(str::to_string);
            value.get("version").and_then(Value::as_str) == Some(crate::version::QORX_VERSION)
                && value
                    .get("gateway_healthy")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
        }
        Err(err) => {
            errors.push(format!("doctor:{err}"));
            false
        }
    };
    checks.insert("doctor".to_string(), doctor_ok);

    let health_ok = match bind.as_deref().and_then(config::gateway_base_from_bind) {
        Some(gateway) => health_ok(client, &gateway).await,
        None => false,
    };
    if !health_ok {
        errors.push("health:gateway did not answer expected product/version".to_string());
    }
    checks.insert("health".to_string(), health_ok);

    let context = run_json(
        exe,
        &[
            "context",
            "inject",
            &format!("Crux stress context iteration {iteration}"),
        ],
        None,
    );
    let context_ok = match context {
        Ok(value) => {
            handle = value
                .get("handle")
                .and_then(Value::as_str)
                .map(str::to_string);
            value.get("local_only").and_then(Value::as_bool) == Some(true)
                && value.get("provider_calls").and_then(Value::as_u64) == Some(0)
                && handle
                    .as_deref()
                    .is_some_and(|value| value.starts_with("qorx://"))
        }
        Err(err) => {
            errors.push(format!("context:{err}"));
            false
        }
    };
    checks.insert("context_inject".to_string(), context_ok);

    let mcp_self_test = integrations::mcp_self_test(exe);
    checks.insert("mcp".to_string(), mcp_self_test.ok);
    if !mcp_self_test.ok {
        errors.push(format!("mcp:{}", mcp_self_test.detail));
    }

    let configs = config_checks(exe);
    let configs_ok = configs
        .as_object()
        .map(|object| {
            object
                .values()
                .all(|value| value.get("ok").and_then(Value::as_bool) == Some(true))
        })
        .unwrap_or(false);
    checks.insert("configs".to_string(), configs_ok);

    let status = integration_status(exe);
    let status_ok = status.get("ok").and_then(Value::as_bool) == Some(true);
    checks.insert("status".to_string(), status_ok);

    let ok = checks.values().all(|value| *value);
    if !ok {
        for (name, passed) in &checks {
            if !passed {
                errors.push(format!("failed:{name}"));
            }
        }
    }

    CruxIteration {
        iteration,
        timestamp_utc: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        ok,
        elapsed_seconds,
        version,
        bind,
        handle,
        checks,
        errors,
        status,
        configs,
        mcp_self_test,
    }
}

async fn health_ok(client: &Client, gateway: &str) -> bool {
    match client
        .get(format!("{gateway}/health"))
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            response.json::<Value>().await.is_ok_and(|value| {
                value.get("ok").and_then(Value::as_bool) == Some(true)
                    && value.get("product").and_then(Value::as_str)
                        == Some(crate::version::product_name())
                    && value.get("version").and_then(Value::as_str)
                        == Some(crate::version::QORX_VERSION)
            })
        }
        _ => false,
    }
}

fn config_checks(exe: &Path) -> Value {
    let exe_text = exe.display().to_string();
    let antigravity_compat = crux_antigravity_compat();
    let (antigravity_primary, antigravity_legacy) = if antigravity_compat {
        (
            mcp_json_contains(
                appdata_file(&["Antigravity", "User", "mcp.json"]),
                &exe_text,
            ),
            mcp_json_contains(
                home_file(&[".gemini", "antigravity", "mcp_config.json"]),
                &exe_text,
            ),
        )
    } else {
        (
            mcp_json_lacks_qorx(appdata_file(&["Antigravity", "User", "mcp.json"])),
            mcp_json_lacks_qorx(home_file(&[".gemini", "antigravity", "mcp_config.json"])),
        )
    };
    json!({
        "codex_mcp": text_contains(home_file(&[".codex", "config.toml"]), &["[mcp_servers.qorx]", "qorx.exe", "args = [\"mcp\"]"]),
        "codex_hook": text_contains(home_file(&[".codex", "hooks.json"]), &["UserPromptSubmit", "qorx_user_prompt_submit.py"]),
        "gemini": text_contains(home_file(&[".gemini", "settings.json"]), &["\"qorx\"", "qorx.exe", "\"mcp\"", "qorx-middleware.cjs"]),
        "antigravity_mode": {"ok": true, "mode": if antigravity_compat { "compat" } else { "active" }},
        "antigravity_primary": antigravity_primary,
        "antigravity_legacy": antigravity_legacy,
    })
}

fn text_contains(path: Option<PathBuf>, needles: &[&str]) -> Value {
    let Some(path) = path else {
        return json!({"ok": false, "error": "path unavailable"});
    };
    match fs::read_to_string(&path) {
        Ok(text) => json!({
            "ok": needles.iter().all(|needle| text.contains(needle)),
            "path": path,
        }),
        Err(err) => json!({"ok": false, "path": path, "error": err.to_string()}),
    }
}

fn mcp_json_contains(path: Option<PathBuf>, exe: &str) -> Value {
    let Some(path) = path else {
        return json!({"ok": false, "error": "path unavailable"});
    };
    let value = fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok());
    let server = value
        .as_ref()
        .and_then(|value| value.pointer("/mcpServers/qorx"));
    let ok = server.is_some_and(|server| {
        server.get("command").and_then(Value::as_str) == Some(exe)
            && server
                .get("args")
                .and_then(Value::as_array)
                .is_some_and(|args| {
                    args.iter().map(Value::as_str).collect::<Option<Vec<_>>>() == Some(vec!["mcp"])
                })
    });
    json!({"ok": ok, "path": path})
}

fn mcp_json_lacks_qorx(path: Option<PathBuf>) -> Value {
    let Some(path) = path else {
        return json!({"ok": true, "absent": true});
    };
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return json!({"ok": true, "path": path, "absent": true});
        }
        Err(err) => return json!({"ok": false, "path": path, "error": err.to_string()}),
    };
    let value = match serde_json::from_str::<Value>(&text) {
        Ok(value) => value,
        Err(err) => return json!({"ok": false, "path": path, "error": err.to_string()}),
    };
    let has_qorx = ["mcpServers", "servers"].iter().any(|section| {
        ["qorx", "qorx_edge", "mcp_qorx_qorx"]
            .iter()
            .any(|name| value.pointer(&format!("/{section}/{name}")).is_some())
    });
    json!({"ok": !has_qorx, "path": path, "absent": !has_qorx})
}

fn integration_status(exe: &Path) -> Value {
    let mut envs = BTreeMap::new();
    let antigravity_compat = crux_antigravity_compat();
    if antigravity_compat {
        envs.insert("QORX_ANTIGRAVITY_MCP".to_string(), "1".to_string());
        envs.insert("QORX_ANTIGRAVITY_CONTEXT_RULE".to_string(), "1".to_string());
    }
    let envs = if envs.is_empty() { None } else { Some(&envs) };
    match run_json(exe, &["integrate", "status"], envs) {
        Ok(value) => {
            let targets = value
                .get("targets")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let platform = |name: &str| {
                targets
                    .iter()
                    .find(|target| target.get("platform").and_then(Value::as_str) == Some(name))
            };
            let active = |name: &str, field: &str| {
                platform(name)
                    .and_then(|target| target.get(field))
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            };
            let antigravity_ok = if antigravity_compat {
                active("antigravity", "mcp_active") && active("antigravity", "hook_active")
            } else {
                platform("antigravity").is_some()
                    && !active("antigravity", "active")
                    && !active("antigravity", "mcp_active")
                    && !active("antigravity", "hook_active")
            };
            let ok = active("codex", "mcp_active")
                && active("codex", "hook_active")
                && active("gemini", "mcp_active")
                && active("gemini", "hook_active")
                && antigravity_ok;
            json!({
                "ok": ok,
                "antigravity_mode": if antigravity_compat { "compat" } else { "active" },
                "targets": {
                    "codex": platform("codex").cloned(),
                    "gemini": platform("gemini").cloned(),
                    "antigravity": platform("antigravity").cloned(),
                }
            })
        }
        Err(err) => json!({"ok": false, "error": err.to_string()}),
    }
}

fn crux_antigravity_compat() -> bool {
    env::var("QORX_CRUX_ANTIGRAVITY_COMPAT")
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
        .unwrap_or(false)
}

fn run_json(exe: &Path, args: &[&str], envs: Option<&BTreeMap<String, String>>) -> Result<Value> {
    let mut command = Command::new(exe);
    command
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::piped());
    if let Some(envs) = envs {
        command.envs(envs);
    }
    configure_no_window(&mut command);
    let output = command.output()?;
    if !output.status.success() {
        return Err(anyhow!(
            "{} {:?} failed: {}",
            exe.display(),
            args,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn append_jsonl(path: &Path, record: &CruxIteration) -> Result<()> {
    use std::io::Write;

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{}", serde_json::to_string(record)?)?;
    Ok(())
}

fn read_iterations(path: &Path) -> Result<Vec<CruxIteration>> {
    let text = fs::read_to_string(path)?;
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(serde_json::from_str::<CruxIteration>)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(Into::into)
}

fn read_summary(path: &Path) -> Result<CruxSummary> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn crux_dir(paths: &AppPaths) -> PathBuf {
    paths.data_dir.join("crux")
}

fn state_file(paths: &AppPaths) -> PathBuf {
    crux_dir(paths).join("state.json")
}

fn load_state(paths: &AppPaths) -> Result<Option<CruxState>> {
    let path = state_file(paths);
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(&fs::read_to_string(path)?)?))
}

fn latest_log(paths: &AppPaths) -> Option<PathBuf> {
    fs::read_dir(crux_dir(paths))
        .ok()?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("crux-stress-") && name.ends_with(".jsonl"))
        })
        .max_by_key(|path| fs::metadata(path).and_then(|meta| meta.modified()).ok())
}

fn home_file(parts: &[&str]) -> Option<PathBuf> {
    let home = env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)?;
    Some(parts.iter().fold(home, |path, part| path.join(part)))
}

fn appdata_file(parts: &[&str]) -> Option<PathBuf> {
    let appdata = env::var_os("APPDATA").map(PathBuf::from)?;
    Some(parts.iter().fold(appdata, |path, part| path.join(part)))
}

fn stop_pid(pid: u32) -> bool {
    if !process_is_alive(pid) {
        return false;
    }
    if cfg!(windows) {
        Command::new("powershell.exe")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(format!("Stop-Process -Id {pid} -Force"))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    } else {
        Command::new("kill")
            .arg(pid.to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }
}

fn process_is_alive(pid: u32) -> bool {
    if cfg!(windows) {
        Command::new("powershell.exe")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(format!("if (Get-Process -Id {pid} -ErrorAction SilentlyContinue) {{ exit 0 }} else {{ exit 1 }}"))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    } else {
        Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }
}

#[cfg(windows)]
fn configure_background(command: &mut Command) {
    use std::os::windows::process::CommandExt;

    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn configure_background(_command: &mut Command) {}

#[cfg(windows)]
fn configure_no_window(command: &mut Command) {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn configure_no_window(_command: &mut Command) {}

#[cfg(test)]
mod tests {
    #[test]
    fn crux_hours_convert_to_minute_iterations() {
        assert_eq!(super::iterations_for_hours(1.0, 60), 60);
        assert_eq!(super::iterations_for_hours(0.01, 60), 1);
        assert_eq!(super::iterations_for_hours(0.5, 300), 6);
    }
}
