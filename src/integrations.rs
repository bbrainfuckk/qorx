use std::{
    env,
    ffi::OsString,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{anyhow, Result};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::config::{load_drive_home_config, local_base, AppPaths};

const CODEX_USER_PROMPT_EVENT: &str = "UserPromptSubmit";
const CODEX_USER_PROMPT_EVENT_KEY: &str = "user_prompt_submit";
const CODEX_HOOK_TIMEOUT_SECONDS: u64 = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationReport {
    pub qorx_exe: String,
    pub gateway: String,
    pub settings: IntegrationSettings,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint: Option<CheckpointReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_self_test: Option<McpSelfTestReport>,
    pub autostart: IntegrationStatus,
    pub tray_runtime: IntegrationStatus,
    pub codex_hook: IntegrationStatus,
    pub shims: IntegrationStatus,
    pub capabilities: Vec<PlatformCapability>,
    pub targets: Vec<IntegrationStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointReport {
    pub schema: String,
    pub label: String,
    pub created_at_utc: String,
    pub path: String,
    pub files: Vec<CheckpointFile>,
    pub git_status: Option<String>,
    pub git_diff: Option<String>,
    pub restore_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointFile {
    pub label: String,
    pub source: String,
    pub backup: Option<String>,
    pub existed: bool,
}

#[derive(Debug, Clone)]
struct CheckpointTarget {
    label: String,
    path: PathBuf,
}

impl CheckpointTarget {
    fn new(label: impl Into<String>, path: PathBuf) -> Self {
        Self {
            label: label.into(),
            path,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSelfTestReport {
    pub schema: String,
    pub ok: bool,
    pub line_json_ok: bool,
    pub content_length_ok: bool,
    pub server_name: Option<String>,
    pub server_version: Option<String>,
    pub tool_count: usize,
    pub detail: String,
}

const CODEX_CONTEXT_MODE_AUTO: &str = "auto";
const CODEX_CONTEXT_MODE_READABLE: &str = "readable";
const CODEX_CONTEXT_MODE_DEEP: &str = "deep";
const CODEX_CONTEXT_MODE_OFF: &str = "off";

fn default_codex_context_mode() -> String {
    CODEX_CONTEXT_MODE_AUTO.to_string()
}

pub fn normalize_codex_context_mode(mode: &str) -> String {
    match mode.trim().to_ascii_lowercase().as_str() {
        CODEX_CONTEXT_MODE_READABLE => CODEX_CONTEXT_MODE_READABLE.to_string(),
        CODEX_CONTEXT_MODE_DEEP => CODEX_CONTEXT_MODE_DEEP.to_string(),
        CODEX_CONTEXT_MODE_OFF => CODEX_CONTEXT_MODE_OFF.to_string(),
        "verbose" => CODEX_CONTEXT_MODE_READABLE.to_string(),
        _ => CODEX_CONTEXT_MODE_AUTO.to_string(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationSettings {
    #[serde(default)]
    pub automcp_enabled: bool,
    #[serde(default)]
    pub autohook_enabled: bool,
    #[serde(default = "default_codex_context_mode")]
    pub codex_context_mode: String,
}

impl Default for IntegrationSettings {
    fn default() -> Self {
        Self {
            automcp_enabled: false,
            autohook_enabled: false,
            codex_context_mode: CODEX_CONTEXT_MODE_OFF.to_string(),
        }
    }
}

impl IntegrationSettings {
    pub fn all_enabled() -> Self {
        Self {
            automcp_enabled: true,
            autohook_enabled: true,
            codex_context_mode: CODEX_CONTEXT_MODE_AUTO.to_string(),
        }
    }

    pub fn any_enabled(&self) -> bool {
        self.automcp_enabled || self.autohook_enabled
    }

    pub fn normalized(mut self) -> Self {
        self.codex_context_mode = normalize_codex_context_mode(&self.codex_context_mode);
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntegrationStatus {
    #[serde(default)]
    pub platform: String,
    pub name: String,
    pub installed: bool,
    pub active: bool,
    #[serde(default)]
    pub supports_mcp: bool,
    #[serde(default)]
    pub mcp_active: bool,
    #[serde(default)]
    pub supports_hooks: bool,
    #[serde(default)]
    pub hook_active: bool,
    #[serde(default)]
    pub hook_mode: String,
    #[serde(default)]
    pub supports_proxy: bool,
    #[serde(default)]
    pub proxy_mode: String,
    #[serde(default)]
    pub install_scope: String,
    #[serde(default)]
    pub reload_hint: String,
    pub mechanism: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformCapability {
    pub platform: String,
    pub name: String,
    pub supports_mcp: bool,
    pub supports_hooks: bool,
    pub hook_mode: String,
    pub supports_proxy: bool,
    pub proxy_mode: String,
    pub install_scope: String,
    pub reload_hint: String,
}

#[derive(Debug, Clone, Copy)]
struct CapabilitySpec<'a> {
    platform: IntegrationPlatform,
    name: &'a str,
    supports_mcp: bool,
    supports_hooks: bool,
    hook_mode: &'a str,
    supports_proxy: bool,
    proxy_mode: &'a str,
    install_scope: &'a str,
    reload_hint: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntegrationPlatform {
    All,
    Windows,
    Codex,
    Claude,
    OpenCode,
    Copilot,
    VsCodeCopilot,
    Aider,
    OpenClaw,
    FactoryDroid,
    Trae,
    TraeCn,
    Gemini,
    Hermes,
    Kiro,
    Pi,
    Cursor,
    Antigravity,
}

impl IntegrationPlatform {
    pub fn from_slug(slug: &str) -> Option<Self> {
        let normalized = slug.trim().to_ascii_lowercase().replace('_', "-");
        match normalized.as_str() {
            "all" => Some(Self::All),
            "windows" | "win" => Some(Self::Windows),
            "codex" => Some(Self::Codex),
            "claude" | "claude-code" => Some(Self::Claude),
            "opencode" | "open-code" => Some(Self::OpenCode),
            "copilot" | "github-copilot" | "github-copilot-cli" => Some(Self::Copilot),
            "vscode" | "vs-code" | "vscode-copilot" | "vs-code-copilot" => {
                Some(Self::VsCodeCopilot)
            }
            "aider" => Some(Self::Aider),
            "claw" | "openclaw" | "open-claw" => Some(Self::OpenClaw),
            "droid" | "factory" | "factory-droid" => Some(Self::FactoryDroid),
            "trae" => Some(Self::Trae),
            "trae-cn" | "traecn" => Some(Self::TraeCn),
            "gemini" | "gemini-cli" => Some(Self::Gemini),
            "hermes" => Some(Self::Hermes),
            "kiro" | "kiro-ide" | "kiro-cli" => Some(Self::Kiro),
            "pi" | "pi-agent" | "pi-coding-agent" => Some(Self::Pi),
            "cursor" => Some(Self::Cursor),
            "antigravity" | "google-antigravity" => Some(Self::Antigravity),
            _ => None,
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Windows => "windows",
            Self::Codex => "codex",
            Self::Claude => "claude",
            Self::OpenCode => "opencode",
            Self::Copilot => "copilot",
            Self::VsCodeCopilot => "vscode",
            Self::Aider => "aider",
            Self::OpenClaw => "claw",
            Self::FactoryDroid => "droid",
            Self::Trae => "trae",
            Self::TraeCn => "trae-cn",
            Self::Gemini => "gemini",
            Self::Hermes => "hermes",
            Self::Kiro => "kiro",
            Self::Pi => "pi",
            Self::Cursor => "cursor",
            Self::Antigravity => "antigravity",
        }
    }

    fn agent_platforms() -> &'static [Self] {
        &[
            Self::Codex,
            Self::Claude,
            Self::OpenCode,
            Self::Copilot,
            Self::VsCodeCopilot,
            Self::Aider,
            Self::OpenClaw,
            Self::FactoryDroid,
            Self::Trae,
            Self::TraeCn,
            Self::Gemini,
            Self::Hermes,
            Self::Kiro,
            Self::Pi,
            Self::Cursor,
            Self::Antigravity,
        ]
    }
}

pub fn install_autostart() -> Result<IntegrationStatus> {
    let startup = startup_file()?;
    let exe = current_exe()?;
    if let Some(parent) = startup.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&startup, autostart_script(&exe)?)?;
    remove_legacy_tray_startup()?;
    remove_redundant_drive_startup()?;
    Ok(IntegrationStatus {
        name: "Windows daemon + tray login startup".to_string(),
        installed: true,
        active: startup.exists(),
        mechanism: "Startup folder VBS hidden daemon + tray launch".to_string(),
        detail: startup.display().to_string(),
        ..Default::default()
    })
}

fn autostart_script(exe: &Path) -> Result<String> {
    let exe = exe.display();
    let mut script = "Set shell = CreateObject(\"WScript.Shell\")\r\n".to_string();
    if let Some(drive_home) = load_drive_home_config()? {
        let ram_arg = if drive_home.backend.eq_ignore_ascii_case("imdisk") {
            format!(" --ram --size {}", drive_home.size)
        } else {
            String::new()
        };
        let drive_args = format!("drive mount --letter {}{}", drive_home.letter, ram_arg);
        script.push_str(&format!(
            "shell.Run \"\"\"{exe}\"\" {drive_args}\", 0, True\r\n"
        ));
        script.push_str("WScript.Sleep 5000\r\n");
        script.push_str(&format!(
            "shell.Run \"\"\"{exe}\"\" {drive_args}\", 0, True\r\n"
        ));
    }
    script.push_str(&format!(
        "shell.Run \"\"\"{exe}\"\" daemon start\", 0, True\r\n"
    ));
    script.push_str("WScript.Sleep 1000\r\n");
    script.push_str(&format!("shell.Run \"\"\"{exe}\"\" tray\", 0, False\r\n"));
    Ok(script)
}

fn remove_legacy_tray_startup() -> Result<()> {
    let startup = legacy_tray_startup_file()?;
    if startup.exists() {
        fs::remove_file(startup)?;
    }
    Ok(())
}

fn remove_redundant_drive_startup() -> Result<()> {
    let Some(drive_home) = load_drive_home_config()? else {
        return Ok(());
    };
    let startup = drive_startup_file(&drive_home.letter)?;
    if startup.exists() {
        fs::remove_file(startup)?;
    }
    Ok(())
}

fn drive_startup_file(letter: &str) -> Result<PathBuf> {
    let appdata = env::var("APPDATA")?;
    let normalized = letter
        .trim()
        .trim_end_matches('\\')
        .trim_end_matches('/')
        .trim_end_matches(':');
    Ok(PathBuf::from(appdata)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Startup")
        .join(format!("Qorx Drive {normalized}.vbs")))
}

pub fn remove_autostart() -> Result<IntegrationStatus> {
    let startup = startup_file()?;
    if startup.exists() {
        fs::remove_file(&startup)?;
    }
    remove_legacy_tray_startup()?;
    Ok(IntegrationStatus {
        name: "Windows daemon + tray login startup".to_string(),
        installed: true,
        active: startup.exists(),
        mechanism: "Startup folder VBS hidden daemon + tray launch".to_string(),
        detail: startup.display().to_string(),
        ..Default::default()
    })
}

pub fn activate_all(paths: &AppPaths) -> Result<IntegrationReport> {
    let exe = current_exe()?;
    let checkpoint = make_integration_checkpoint(paths, "activate-all").ok();
    install_runtime(paths)?;
    let settings = IntegrationSettings::all_enabled();
    save_settings(paths, &settings)?;
    install_all_platform_connectors(&exe, &settings)?;

    let mut report = current_report(paths, integration_statuses(&exe))?;
    attach_change_evidence(&mut report, checkpoint, &exe);
    write_report(paths, &report)?;
    Ok(report)
}

pub fn install_platform(
    paths: &AppPaths,
    platform: IntegrationPlatform,
) -> Result<IntegrationReport> {
    if platform == IntegrationPlatform::All {
        return activate_all(paths);
    }

    let exe = current_exe()?;
    let checkpoint =
        make_integration_checkpoint(paths, &format!("activate-{}", platform.slug())).ok();
    install_runtime(paths)?;
    let mut settings = load_settings(paths)?;
    let capability = capability_for(platform);
    settings.automcp_enabled |= capability.supports_mcp;
    settings.autohook_enabled |= capability.supports_hooks;
    save_settings(paths, &settings)?;
    if platform != IntegrationPlatform::Windows {
        install_platform_connector(platform, &exe, &settings)?;
    }

    let mut report = current_report(paths, integration_statuses(&exe))?;
    attach_change_evidence(&mut report, checkpoint, &exe);
    write_report(paths, &report)?;
    Ok(report)
}

pub fn activate_enabled(paths: &AppPaths) -> Result<IntegrationReport> {
    let settings = load_settings(paths)?;
    let checkpoint = if settings.any_enabled() {
        make_integration_checkpoint(paths, "activate-enabled").ok()
    } else {
        None
    };
    if settings.any_enabled() {
        let exe = current_exe()?;
        install_runtime(paths)?;
        install_all_platform_connectors(&exe, &settings)?;
    }

    let exe = current_exe()?;
    let mut report = current_report(paths, integration_statuses(&exe))?;
    if checkpoint.is_some() {
        attach_change_evidence(&mut report, checkpoint, &exe);
    }
    write_report(paths, &report)?;
    Ok(report)
}

fn install_runtime(paths: &AppPaths) -> Result<()> {
    let _ = install_autostart();
    let _ = set_user_env("QORX_GATEWAY", &local_base());
    let _ = set_user_env("QORX_PROXY_MODE", "aim-proxy");
    let _ = set_user_env("QORX_SHIMS", &paths.shim_dir.display().to_string());
    let _ = crate::aim::resolve_aim_path().map(|path| {
        let _ = set_user_env("QORX_AIM_PATH", &path.display().to_string());
    });
    let _ = write_shims(paths);
    let _ = ensure_daemon_running();
    let _ = ensure_tray_running();
    Ok(())
}

fn install_all_platform_connectors(exe: &Path, settings: &IntegrationSettings) -> Result<()> {
    for platform in IntegrationPlatform::agent_platforms() {
        if *platform == IntegrationPlatform::Antigravity && !antigravity_mcp_opt_in() {
            let _ = remove_antigravity_qorx_mcp_entries();
            let _ = remove_antigravity_context_rule();
            continue;
        }
        install_platform_connector(*platform, exe, settings)?;
    }
    Ok(())
}

fn install_platform_connector(
    platform: IntegrationPlatform,
    exe: &Path,
    settings: &IntegrationSettings,
) -> Result<()> {
    match platform {
        IntegrationPlatform::All => install_all_platform_connectors(exe, settings),
        IntegrationPlatform::Windows => Ok(()),
        IntegrationPlatform::Codex => {
            if settings.autohook_enabled {
                let _ = install_codex_hook();
            }
            if settings.automcp_enabled {
                register_codex_mcp(exe);
            }
            Ok(())
        }
        IntegrationPlatform::Claude => {
            if settings.automcp_enabled {
                register_claude_mcp(exe);
                install_claude_plugin_mcp_connector(exe)?;
            }
            if settings.autohook_enabled {
                install_claude_plugin_hook_connector()?;
            }
            Ok(())
        }
        IntegrationPlatform::Gemini => {
            if settings.automcp_enabled {
                register_gemini_mcp(exe);
                install_gemini_mcp_file(exe)?;
            }
            if settings.autohook_enabled {
                install_gemini_hook()?;
            }
            Ok(())
        }
        IntegrationPlatform::Antigravity => {
            if !antigravity_mcp_opt_in() {
                let _ = remove_antigravity_qorx_mcp_entries();
                let _ = remove_antigravity_context_rule();
                return Ok(());
            }
            if settings.automcp_enabled {
                install_antigravity_mcp(exe)?;
            }
            if settings.autohook_enabled && antigravity_context_rule_opt_in() {
                install_antigravity_context_rule()?;
            } else {
                let _ = remove_antigravity_context_rule();
            }
            Ok(())
        }
        IntegrationPlatform::OpenCode => install_if_mcp_enabled(settings, || {
            install_named_mcp_platform(
                exe,
                &[
                    home_dir()?.join(".opencode").join("mcp.json"),
                    appdata_dir()?.join("OpenCode").join("mcp.json"),
                ],
                "opencode",
            )
        }),
        IntegrationPlatform::Copilot => install_if_mcp_enabled(settings, || {
            install_named_mcp_platform(
                exe,
                &[home_dir()?.join(".copilot").join("mcp.json")],
                "copilot",
            )
        }),
        IntegrationPlatform::VsCodeCopilot => install_if_mcp_enabled(settings, || {
            install_named_mcp_platform(
                exe,
                &[appdata_dir()?.join("Code").join("User").join("mcp.json")],
                "vscode-copilot",
            )
        }),
        IntegrationPlatform::Aider => install_if_mcp_enabled(settings, || {
            install_named_mcp_platform(
                exe,
                &[
                    home_dir()?.join(".aider").join("mcp.json"),
                    home_dir()?.join(".aider-desk").join("mcp.json"),
                ],
                "aider",
            )
        }),
        IntegrationPlatform::OpenClaw => {
            install_agent_home_connector(exe, &home_dir()?.join(".openclaw"), "openclaw", settings)
        }
        IntegrationPlatform::FactoryDroid => install_agent_home_connector(
            exe,
            &home_dir()?.join(".factory"),
            "factory-droid",
            settings,
        ),
        IntegrationPlatform::Trae => {
            install_agent_home_connector(exe, &home_dir()?.join(".trae"), "trae", settings)
        }
        IntegrationPlatform::TraeCn => {
            install_agent_home_connector(exe, &home_dir()?.join(".trae-cn"), "trae-cn", settings)
        }
        IntegrationPlatform::Hermes => {
            install_agent_home_connector(exe, &home_dir()?.join(".hermes"), "hermes", settings)
        }
        IntegrationPlatform::Kiro => {
            install_agent_home_connector(exe, &home_dir()?.join(".kiro"), "kiro", settings)
        }
        IntegrationPlatform::Pi => install_agent_home_connector(
            exe,
            &home_dir()?.join(".pi").join("agent"),
            "pi",
            settings,
        ),
        IntegrationPlatform::Cursor => install_if_mcp_enabled(settings, || {
            install_named_mcp_platform(
                exe,
                &[home_dir()?.join(".cursor").join("mcp.json")],
                "cursor",
            )
        }),
    }
}

fn install_if_mcp_enabled<F>(settings: &IntegrationSettings, install: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    if settings.automcp_enabled {
        install()?;
    }
    Ok(())
}

pub fn set_settings(paths: &AppPaths, settings: IntegrationSettings) -> Result<IntegrationReport> {
    let settings = settings.normalized();
    let checkpoint = make_integration_checkpoint(paths, "settings").ok();
    save_settings(paths, &settings)?;
    if settings.any_enabled() {
        let exe = current_exe()?;
        install_runtime(paths)?;
        install_all_platform_connectors(&exe, &settings)?;
    }
    if !settings.automcp_enabled {
        remove_all_mcp_connectors()?;
    }
    if !settings.autohook_enabled {
        remove_all_hook_connectors()?;
        let _ = remove_antigravity_context_rule();
    }

    let exe = current_exe()?;
    let mut report = current_report(paths, integration_statuses(&exe))?;
    attach_change_evidence(&mut report, checkpoint, &exe);
    write_report(paths, &report)?;
    Ok(report)
}

pub fn deactivate_all(paths: &AppPaths) -> Result<IntegrationReport> {
    let checkpoint = make_integration_checkpoint(paths, "deactivate-all").ok();
    save_settings(paths, &IntegrationSettings::default())?;
    let _ = remove_autostart();
    let _ = remove_all_hook_connectors();
    let _ = remove_all_mcp_connectors();
    let _ = remove_antigravity_context_rule();
    if paths.shim_dir.exists() {
        for entry in fs::read_dir(&paths.shim_dir)? {
            let entry = entry?;
            if entry.file_name().to_string_lossy().starts_with("qorx-") {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    let exe = current_exe()?;
    let mut report = current_report(paths, integration_statuses(&exe))?;
    attach_change_evidence(&mut report, checkpoint, &exe);
    write_report(paths, &report)?;
    Ok(report)
}

pub fn report(paths: &AppPaths) -> Result<IntegrationReport> {
    current_report(paths, integration_statuses(&current_exe()?))
}

pub fn restore_checkpoint(checkpoint: &Path) -> Result<CheckpointReport> {
    let manifest = checkpoint.join("manifest.json");
    let report: CheckpointReport = serde_json::from_str(&fs::read_to_string(&manifest)?)?;
    for file in &report.files {
        let source = PathBuf::from(&file.source);
        if file.existed {
            let backup = file
                .backup
                .as_ref()
                .ok_or_else(|| anyhow!("checkpoint file {} has no backup path", file.label))?;
            if let Some(parent) = source.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(backup, &source)?;
        } else if source.exists() {
            fs::remove_file(&source)?;
        }
    }
    Ok(report)
}

fn current_report(paths: &AppPaths, targets: Vec<IntegrationStatus>) -> Result<IntegrationReport> {
    Ok(IntegrationReport {
        qorx_exe: current_exe()?.display().to_string(),
        gateway: local_base(),
        settings: load_settings(paths)?,
        checkpoint: None,
        mcp_self_test: None,
        autostart: autostart_status()?,
        tray_runtime: tray_runtime_status(&current_exe()?),
        codex_hook: codex_hook_status(),
        shims: shims_status(paths),
        capabilities: platform_capabilities(),
        targets,
    })
}

fn attach_change_evidence(
    report: &mut IntegrationReport,
    checkpoint: Option<CheckpointReport>,
    exe: &Path,
) {
    report.checkpoint = checkpoint;
    report.mcp_self_test = Some(mcp_self_test(exe));
}

fn make_integration_checkpoint(paths: &AppPaths, label: &str) -> Result<CheckpointReport> {
    let mut targets = vec![
        CheckpointTarget::new("qorx-exe", current_exe()?),
        CheckpointTarget::new("codex-config", codex_config_path()?),
        CheckpointTarget::new("codex-hooks", codex_hooks_path()?),
        CheckpointTarget::new(
            "gemini-settings",
            home_dir()?.join(".gemini").join("settings.json"),
        ),
        CheckpointTarget::new("gemini-hook", gemini_hook_script_path()?),
        CheckpointTarget::new("antigravity-primary-mcp", antigravity_mcp_path()?),
        CheckpointTarget::new("antigravity-context-rule", antigravity_context_rule_path()?),
        CheckpointTarget::new("claude-user-config", claude_user_config_path()?),
    ];
    if let Ok(path) = legacy_gemini_antigravity_mcp_path() {
        targets.push(CheckpointTarget::new("antigravity-legacy-mcp", path));
    }
    for path in all_mcp_config_paths() {
        targets.push(CheckpointTarget::new("mcp-config", path));
    }
    write_checkpoint_at(
        &paths.data_dir.join("checkpoints"),
        label,
        &targets,
        env::current_dir().ok().as_deref(),
    )
}

fn write_checkpoint_at(
    root: &Path,
    label: &str,
    targets: &[CheckpointTarget],
    git_cwd: Option<&Path>,
) -> Result<CheckpointReport> {
    let mut seen = Vec::<PathBuf>::new();
    let targets = targets
        .iter()
        .filter(|target| {
            if seen.iter().any(|path| path == &target.path) {
                false
            } else {
                seen.push(target.path.clone());
                true
            }
        })
        .cloned()
        .collect::<Vec<_>>();
    let created_at_utc = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let stamp = created_at_utc
        .replace([':', '-'], "")
        .replace('T', "-")
        .trim_end_matches('Z')
        .to_string();
    let checkpoint = root.join(format!("{stamp}-{}", sanitize_filename(label)));
    let file_dir = checkpoint.join("files");
    fs::create_dir_all(&file_dir)?;

    let mut files = Vec::new();
    for (index, target) in targets.iter().enumerate() {
        let existed = target.path.exists();
        let backup = if existed {
            let filename = target
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .filter(|name| !name.trim().is_empty())
                .unwrap_or("file");
            let backup = file_dir.join(format!(
                "{index:03}-{}-{}",
                sanitize_filename(&target.label),
                sanitize_filename(filename)
            ));
            fs::create_dir_all(backup.parent().unwrap_or(&file_dir))?;
            fs::copy(&target.path, &backup)?;
            Some(backup.display().to_string())
        } else {
            None
        };
        files.push(CheckpointFile {
            label: target.label.clone(),
            source: target.path.display().to_string(),
            backup,
            existed,
        });
    }

    let git_status = git_cwd.and_then(|cwd| git_capture(cwd, &["status", "--short"]));
    let git_diff = git_cwd.and_then(|cwd| git_capture(cwd, &["diff", "--binary"]));
    if let Some(status) = &git_status {
        fs::write(checkpoint.join("git-status.txt"), status)?;
    }
    if let Some(diff) = &git_diff {
        fs::write(checkpoint.join("repo.diff"), diff)?;
    }

    let report = CheckpointReport {
        schema: "qorx.integration-checkpoint.v1".to_string(),
        label: label.to_string(),
        created_at_utc,
        path: checkpoint.display().to_string(),
        files,
        git_status,
        git_diff,
        restore_command: format!(
            "qorx crux rollback --checkpoint \"{}\"",
            checkpoint.display()
        ),
    };
    fs::write(
        checkpoint.join("manifest.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    fs::write(checkpoint.join("restore.ps1"), restore_script(&report))?;
    Ok(report)
}

fn restore_script(report: &CheckpointReport) -> String {
    let mut script = "$ErrorActionPreference = 'Stop'\r\n".to_string();
    for file in &report.files {
        let source = ps_single_quote(&file.source);
        if let Some(backup) = &file.backup {
            let backup = ps_single_quote(backup);
            script.push_str(&format!(
                "New-Item -ItemType Directory -Force -Path (Split-Path -LiteralPath '{source}' -Parent) | Out-Null\r\n"
            ));
            script.push_str(&format!(
                "Copy-Item -LiteralPath '{backup}' -Destination '{source}' -Force\r\n"
            ));
        } else {
            script.push_str(&format!(
                "if (Test-Path -LiteralPath '{source}') {{ Remove-Item -LiteralPath '{source}' -Force }}\r\n"
            ));
        }
    }
    script
}

fn ps_single_quote(value: &str) -> String {
    value.replace('\'', "''")
}

fn sanitize_filename(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    let trimmed = out.trim_matches('_');
    if trimmed.is_empty() {
        "checkpoint".to_string()
    } else {
        trimmed.to_string()
    }
}

fn git_capture(cwd: &Path, args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn mcp_self_test(exe: &Path) -> McpSelfTestReport {
    let line = mcp_line_self_test(exe);
    let framed = mcp_content_length_self_test(exe);
    let server_name = line
        .as_ref()
        .ok()
        .and_then(|value| value.pointer("/server/name").and_then(Value::as_str))
        .map(str::to_string);
    let server_version = line
        .as_ref()
        .ok()
        .and_then(|value| value.pointer("/server/version").and_then(Value::as_str))
        .map(str::to_string);
    let tool_count = line
        .as_ref()
        .ok()
        .and_then(|value| value.get("tool_count").and_then(Value::as_u64))
        .unwrap_or(0) as usize;
    let line_json_ok = line.as_ref().is_ok_and(|value| {
        value.pointer("/server/name").and_then(Value::as_str) == Some("qorx-void")
            && value.pointer("/server/version").and_then(Value::as_str)
                == Some(crate::version::QORX_VERSION)
            && value
                .get("has_context_inject")
                .and_then(Value::as_bool)
                .unwrap_or(false)
    });
    let content_length_ok = framed.as_ref().is_ok_and(|value| {
        value
            .get("has_content_length")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            && value
                .get("has_server")
                .and_then(Value::as_bool)
                .unwrap_or(false)
    });
    McpSelfTestReport {
        schema: "qorx.mcp-self-test.v1".to_string(),
        ok: line_json_ok && content_length_ok,
        line_json_ok,
        content_length_ok,
        server_name,
        server_version,
        tool_count,
        detail: if line_json_ok && content_length_ok {
            "qorx.exe mcp responded over line JSON and Content-Length framing".to_string()
        } else {
            format!(
                "line={}; content_length={}",
                mcp_result_detail(&line),
                mcp_result_detail(&framed)
            )
        },
    }
}

fn mcp_result_detail(result: &Result<Value>) -> String {
    match result {
        Ok(value) => serde_json::to_string(value).unwrap_or_else(|_| "ok".to_string()),
        Err(err) => err.to_string(),
    }
}

fn mcp_line_self_test(exe: &Path) -> Result<Value> {
    let init = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "qorx-crux-self-test", "version": "1"}
        }
    });
    let list = json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}});
    let output = run_mcp_payload(
        exe,
        format!(
            "{}\n{}\n",
            serde_json::to_string(&init)?,
            serde_json::to_string(&list)?
        )
        .into_bytes(),
    )?;
    let text = String::from_utf8_lossy(&output);
    let responses = text
        .lines()
        .filter(|line| line.trim_start().starts_with('{'))
        .map(serde_json::from_str::<Value>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let init = responses
        .first()
        .ok_or_else(|| anyhow!("mcp line test did not return initialize response"))?;
    let tools = responses
        .get(1)
        .ok_or_else(|| anyhow!("mcp line test did not return tools/list response"))?;
    let tool_names = tools
        .pointer("/result/tools")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|tool| tool.get("name").and_then(Value::as_str).map(str::to_string))
        .collect::<Vec<_>>();
    Ok(json!({
        "server": init.pointer("/result/serverInfo").cloned().unwrap_or(Value::Null),
        "tool_count": tool_names.len(),
        "has_context_inject": tool_names.iter().any(|name| name == "qorx.context_inject"),
    }))
}

fn mcp_content_length_self_test(exe: &Path) -> Result<Value> {
    let init = serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "qorx-crux-frame-self-test", "version": "1"}
        }
    }))?;
    let payload = format!("Content-Length: {}\r\n\r\n{}", init.len(), init);
    let output = run_mcp_payload(exe, payload.into_bytes())?;
    let text = String::from_utf8_lossy(&output);
    Ok(json!({
        "has_content_length": text.contains("Content-Length:"),
        "has_server": text.contains("qorx-void") && text.contains(crate::version::QORX_VERSION),
        "bytes": output.len(),
    }))
}

fn run_mcp_payload(exe: &Path, payload: Vec<u8>) -> Result<Vec<u8>> {
    let mut command = Command::new(exe);
    command
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    configure_no_window_process(&mut command);
    let mut child = command.spawn()?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(&payload)?;
    }
    drop(child.stdin.take());
    let output = child.wait_with_output()?;
    if !output.status.success() && output.stdout.is_empty() {
        return Err(anyhow!(
            "mcp self-test exited {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output.stdout)
}

fn integration_statuses(exe: &Path) -> Vec<IntegrationStatus> {
    vec![
        codex_status(exe),
        claude_status(exe),
        gemini_status(exe),
        antigravity_status(exe),
        named_mcp_platform_status(
            "OpenCode",
            "opencode",
            "opencode",
            "Qorx MCP stdio server",
            &mcp_paths_for(IntegrationPlatform::OpenCode),
            exe,
        ),
        named_mcp_platform_status(
            "GitHub Copilot CLI",
            "copilot",
            "copilot",
            "Qorx MCP stdio server",
            &mcp_paths_for(IntegrationPlatform::Copilot),
            exe,
        ),
        named_mcp_platform_status(
            "VS Code Copilot Chat",
            "code",
            "vscode",
            "Qorx MCP stdio server",
            &mcp_paths_for(IntegrationPlatform::VsCodeCopilot),
            exe,
        ),
        named_mcp_platform_status(
            "Aider",
            "aider",
            "aider",
            "Qorx MCP stdio server + shim lane",
            &mcp_paths_for(IntegrationPlatform::Aider),
            exe,
        ),
        agent_home_status("OpenClaw", "openclaw", IntegrationPlatform::OpenClaw, exe),
        agent_home_status(
            "Factory Droid",
            "droid",
            IntegrationPlatform::FactoryDroid,
            exe,
        ),
        agent_home_status("Trae", "trae", IntegrationPlatform::Trae, exe),
        agent_home_status("Trae CN", "trae-cn", IntegrationPlatform::TraeCn, exe),
        agent_home_status("Hermes", "hermes", IntegrationPlatform::Hermes, exe),
        agent_home_status("Kiro IDE/CLI", "kiro", IntegrationPlatform::Kiro, exe),
        agent_home_status("Pi coding agent", "pi", IntegrationPlatform::Pi, exe),
        named_mcp_platform_status(
            "Cursor",
            "cursor",
            "cursor",
            "Qorx MCP stdio server",
            &mcp_paths_for(IntegrationPlatform::Cursor),
            exe,
        ),
    ]
}

pub fn platform_capabilities() -> Vec<PlatformCapability> {
    IntegrationPlatform::agent_platforms()
        .iter()
        .map(|platform| capability_for(*platform))
        .collect()
}

fn capability_for(platform: IntegrationPlatform) -> PlatformCapability {
    match platform {
        IntegrationPlatform::Codex => capability(CapabilitySpec {
            platform,
            name: "Codex",
            supports_mcp: true,
            supports_hooks: true,
            hook_mode: "managed",
            supports_proxy: false,
            proxy_mode: "opt-in env: QORX_CODEX_PROXY=1",
            install_scope: "Codex user MCP + UserPromptSubmit hook",
            reload_hint: "Restart Codex after MCP config changes; hooks apply on the next prompt.",
        }),
        IntegrationPlatform::Claude => capability(CapabilitySpec {
            platform,
            name: "Claude Code",
            supports_mcp: true,
            supports_hooks: true,
            hook_mode: "plugin-kit",
            supports_proxy: true,
            proxy_mode: "optional ANTHROPIC_BASE_URL lane",
            install_scope: "Claude user MCP + local Qorx plugin files",
            reload_hint: "Run `/mcp` or restart Claude Code after changing MCP/plugin config.",
        }),
        IntegrationPlatform::Gemini => capability(CapabilitySpec {
            platform,
            name: "Gemini CLI",
            supports_mcp: true,
            supports_hooks: true,
            hook_mode: "managed",
            supports_proxy: true,
            proxy_mode: "optional GOOGLE_GEMINI_BASE_URL lane",
            install_scope: "Gemini settings MCP + BeforeAgent hook",
            reload_hint:
                "Restart Gemini CLI after MCP config changes; hooks apply on the next run.",
        }),
        IntegrationPlatform::Antigravity => capability(CapabilitySpec {
            platform,
            name: "Google Antigravity",
            supports_mcp: true,
            supports_hooks: false,
            hook_mode: "mcp-pull-only",
            supports_proxy: false,
            proxy_mode: "not supported",
            install_scope:
                "Antigravity MCP config; Qorx context is pulled with qorx.context_inject",
            reload_hint: "Reload or restart Antigravity after MCP config changes.",
        }),
        IntegrationPlatform::OpenCode => capability(CapabilitySpec {
            platform,
            name: "OpenCode",
            supports_mcp: true,
            supports_hooks: false,
            hook_mode: "mcp-only",
            supports_proxy: false,
            proxy_mode: "not supported",
            install_scope: "OpenCode MCP config file",
            reload_hint: "Restart OpenCode after MCP config changes.",
        }),
        IntegrationPlatform::Copilot => capability(CapabilitySpec {
            platform,
            name: "GitHub Copilot CLI",
            supports_mcp: true,
            supports_hooks: false,
            hook_mode: "mcp-only",
            supports_proxy: false,
            proxy_mode: "not supported",
            install_scope: "Copilot MCP config file",
            reload_hint: "Restart the Copilot CLI session after MCP config changes.",
        }),
        IntegrationPlatform::VsCodeCopilot => capability(CapabilitySpec {
            platform,
            name: "VS Code Copilot Chat",
            supports_mcp: true,
            supports_hooks: false,
            hook_mode: "mcp-only",
            supports_proxy: false,
            proxy_mode: "not supported",
            install_scope: "VS Code user MCP config",
            reload_hint: "Reload VS Code after MCP config changes.",
        }),
        IntegrationPlatform::Aider => capability(CapabilitySpec {
            platform,
            name: "Aider",
            supports_mcp: true,
            supports_hooks: false,
            hook_mode: "mcp-only",
            supports_proxy: false,
            proxy_mode: "not supported",
            install_scope: "Aider MCP config file",
            reload_hint: "Restart Aider after MCP config changes.",
        }),
        IntegrationPlatform::Cursor => capability(CapabilitySpec {
            platform,
            name: "Cursor",
            supports_mcp: true,
            supports_hooks: false,
            hook_mode: "mcp-only",
            supports_proxy: false,
            proxy_mode: "not supported",
            install_scope: "Cursor MCP config file",
            reload_hint: "Reload Cursor after MCP config changes.",
        }),
        IntegrationPlatform::OpenClaw => manual_kit_capability(platform, "OpenClaw"),
        IntegrationPlatform::FactoryDroid => manual_kit_capability(platform, "Factory Droid"),
        IntegrationPlatform::Trae => manual_kit_capability(platform, "Trae"),
        IntegrationPlatform::TraeCn => manual_kit_capability(platform, "Trae CN"),
        IntegrationPlatform::Hermes => manual_kit_capability(platform, "Hermes"),
        IntegrationPlatform::Kiro => manual_kit_capability(platform, "Kiro IDE/CLI"),
        IntegrationPlatform::Pi => manual_kit_capability(platform, "Pi coding agent"),
        IntegrationPlatform::All | IntegrationPlatform::Windows => capability(CapabilitySpec {
            platform,
            name: platform.slug(),
            supports_mcp: false,
            supports_hooks: false,
            hook_mode: "runtime-only",
            supports_proxy: false,
            proxy_mode: "not supported",
            install_scope: "Qorx runtime",
            reload_hint: "No agent MCP/hook reload needed.",
        }),
    }
}

fn manual_kit_capability(platform: IntegrationPlatform, name: &str) -> PlatformCapability {
    capability(CapabilitySpec {
        platform,
        name,
        supports_mcp: true,
        supports_hooks: true,
        hook_mode: "manual-kit",
        supports_proxy: false,
        proxy_mode: "not supported",
        install_scope: "Qorx MCP config, hooks.json, and skill kit in the agent home",
        reload_hint:
            "Restart the agent, then enable its local MCP/hook file if the client requires a manual toggle.",
    })
}

fn capability(spec: CapabilitySpec<'_>) -> PlatformCapability {
    PlatformCapability {
        platform: spec.platform.slug().to_string(),
        name: spec.name.to_string(),
        supports_mcp: spec.supports_mcp,
        supports_hooks: spec.supports_hooks,
        hook_mode: spec.hook_mode.to_string(),
        supports_proxy: spec.supports_proxy,
        proxy_mode: spec.proxy_mode.to_string(),
        install_scope: spec.install_scope.to_string(),
        reload_hint: spec.reload_hint.to_string(),
    }
}

fn status_from_capability(
    capability: PlatformCapability,
    installed: bool,
    mcp_active: bool,
    hook_active: bool,
    mechanism: &str,
    detail: String,
) -> IntegrationStatus {
    let active = installed
        && ((capability.supports_mcp && mcp_active) || (capability.supports_hooks && hook_active));
    IntegrationStatus {
        platform: capability.platform,
        name: capability.name,
        installed,
        active,
        supports_mcp: capability.supports_mcp,
        mcp_active,
        supports_hooks: capability.supports_hooks,
        hook_active,
        hook_mode: capability.hook_mode,
        supports_proxy: capability.supports_proxy,
        proxy_mode: capability.proxy_mode,
        install_scope: capability.install_scope,
        reload_hint: capability.reload_hint,
        mechanism: mechanism.to_string(),
        detail,
    }
}

fn codex_status(_exe: &Path) -> IntegrationStatus {
    let installed = command_exists("codex");
    let hook_active = codex_hook_status().active;
    let mcp_active = codex_mcp_config_active(_exe);
    status_from_capability(
        capability_for(IntegrationPlatform::Codex),
        installed,
        mcp_active,
        hook_active,
        "Qorx global hook + MCP stdio server",
        "Codex receives compact Qorx hook context and can also call the global `qorx.exe mcp` server; OpenAI proxy routing stays opt-in through QORX_CODEX_PROXY=1".to_string(),
    )
}

fn gemini_status(exe: &Path) -> IntegrationStatus {
    let installed = command_exists("gemini");
    let mcp_active = installed && gemini_mcp_config_active(exe);
    let hook_active = gemini_hook_active();
    status_from_capability(
        capability_for(IntegrationPlatform::Gemini),
        installed,
        mcp_active,
        hook_active,
        "Qorx MCP stdio server + prompt hook + proxy env lane",
        format!(
            "MCP: {} mcp; optional proxy: {} run gemini; GOOGLE_GEMINI_BASE_URL={}/gemini",
            exe.display(),
            exe.display(),
            local_base()
        ),
    )
}

fn claude_status(exe: &Path) -> IntegrationStatus {
    let installed = command_exists("claude");
    let mcp_active = claude_mcp_config_active(exe);
    let hook_active = claude_plugin_hook_active();
    status_from_capability(
        capability_for(IntegrationPlatform::Claude),
        installed,
        mcp_active,
        hook_active,
        "Qorx MCP stdio server + plugin hook kit + proxy env lane",
        format!(
            "MCP: {} mcp; optional proxy: {} run claude; ANTHROPIC_BASE_URL={}/anthropic",
            exe.display(),
            exe.display(),
            local_base()
        ),
    )
}

fn antigravity_status(exe: &Path) -> IntegrationStatus {
    let installed = command_exists("antigravity");
    let opt_in = antigravity_mcp_opt_in();
    let mcp_active = installed && opt_in && antigravity_mcp_config_active(exe);
    let hook_active = installed
        && opt_in
        && antigravity_context_rule_active()
        && antigravity_context_rule_opt_in();
    status_from_capability(
        capability_for(IntegrationPlatform::Antigravity),
        installed,
        mcp_active,
        hook_active,
        "Qorx MCP stdio server + pull-only context tools",
        if mcp_active {
            format!(
                "MCP: {} mcp; context is pulled through qorx.context_inject; gateway={}",
                exe.display(),
                local_base()
            )
        } else if !opt_in {
            "Antigravity AutoMCP is explicitly disabled by environment variables.".to_string()
        } else {
            "Use `qorx install --platform antigravity` or `qorx integrate activate --platform antigravity` to write Antigravity MCP config".to_string()
        },
    )
}

fn named_mcp_platform_status(
    display_name: &str,
    command_name: &str,
    install_slug: &str,
    mechanism: &str,
    paths: &[PathBuf],
    exe: &Path,
) -> IntegrationStatus {
    let installed = command_exists(command_name) || paths.iter().any(|path| path.exists());
    let mcp_active = paths
        .iter()
        .any(|path| mcp_config_active(path, "mcpServers", "qorx", exe));
    let platform = IntegrationPlatform::from_slug(install_slug).unwrap_or(IntegrationPlatform::All);
    let mut status = status_from_capability(
        capability_for(platform),
        installed,
        mcp_active,
        false,
        mechanism,
        if mcp_active {
            format!("MCP: {} mcp", exe.display())
        } else {
            format!(
                "Run `qorx install --platform {}` to write Qorx MCP config",
                install_slug
            )
        },
    );
    status.name = display_name.to_string();
    status
}

fn agent_home_status(
    display_name: &str,
    command_name: &str,
    platform: IntegrationPlatform,
    exe: &Path,
) -> IntegrationStatus {
    let paths = mcp_paths_for(platform);
    let installed = command_exists(command_name) || paths.iter().any(|path| path.exists());
    let mcp_active = paths
        .iter()
        .any(|path| mcp_config_active(path, "mcpServers", "qorx", exe));
    let hook_active = hook_paths_for(platform)
        .iter()
        .any(|path| hook_config_active(path));
    let capability = capability_for(platform);
    let name = capability.name.clone();
    let mut status = status_from_capability(
        capability,
        installed,
        mcp_active,
        hook_active,
        "Qorx MCP config + prompt hook connector",
        if mcp_active || hook_active {
            format!("MCP/hook connector written for {}", exe.display())
        } else {
            format!(
                "Run `qorx install --platform {}` to write Qorx connector files",
                platform.slug()
            )
        },
    );
    status.name = display_name.to_string();
    if status.name.is_empty() {
        status.name = name;
    }
    status
}

fn register_codex_mcp(exe: &Path) {
    let _ = install_codex_mcp_config(exe);
}

fn register_gemini_mcp(exe: &Path) {
    let _ = install_gemini_mcp_file(exe);
}

fn register_claude_mcp(exe: &Path) {
    let _ = install_claude_user_mcp_config(exe);
}

fn install_antigravity_mcp(exe: &Path) -> Result<()> {
    remove_json_object_entries(&antigravity_mcp_path()?, "servers", &["qorx", "qorx_edge"])?;
    let cwd = env::current_dir().ok();
    install_antigravity_mcp_configs(
        &mcp_paths_for(IntegrationPlatform::Antigravity),
        exe,
        cwd.as_deref(),
    )
}

fn remove_antigravity_qorx_mcp_entries() -> Result<()> {
    remove_json_object_entries(&antigravity_mcp_path()?, "servers", &["qorx", "qorx_edge"])?;
    for path in mcp_paths_for(IntegrationPlatform::Antigravity) {
        remove_json_object_entries(&path, "mcpServers", &["qorx", "qorx_edge", "mcp_qorx_qorx"])?;
    }
    Ok(())
}

fn install_antigravity_mcp_configs(
    paths: &[PathBuf],
    exe: &Path,
    cwd: Option<&Path>,
) -> Result<()> {
    for path in paths {
        install_mcp_server_config(path, "mcpServers", "qorx", exe, cwd)?;
        remove_json_object_entries(path, "mcpServers", &["qorx_edge", "mcp_qorx_qorx"])?;
    }
    Ok(())
}

fn install_antigravity_context_rule() -> Result<()> {
    let path = antigravity_context_rule_path()?;
    remove_managed_block(
        &path,
        LEGACY_AYIE_ANTIGRAVITY_CONTEXT_RULE_START,
        LEGACY_AYIE_ANTIGRAVITY_CONTEXT_RULE_END,
    )?;
    upsert_managed_block(
        &path,
        ANTIGRAVITY_CONTEXT_RULE_START,
        ANTIGRAVITY_CONTEXT_RULE_END,
        &antigravity_context_rule_block(),
    )?;
    migrate_legacy_gemini_qorx_rule()?;
    Ok(())
}

fn remove_antigravity_context_rule() -> Result<()> {
    remove_managed_block(
        &antigravity_context_rule_path()?,
        ANTIGRAVITY_CONTEXT_RULE_START,
        ANTIGRAVITY_CONTEXT_RULE_END,
    )
}

fn antigravity_context_rule_active() -> bool {
    antigravity_context_rule_path()
        .ok()
        .and_then(|path| fs::read_to_string(path).ok())
        .is_some_and(|text| text.contains(ANTIGRAVITY_CONTEXT_RULE_START))
        || home_dir()
            .map(|home| home.join(".gemini").join("GEMINI.md"))
            .ok()
            .and_then(|path| fs::read_to_string(path).ok())
            .is_some_and(|text| {
                (text.contains("QORX VOID CONTEXT") || text.contains("QORX AYIE CONTEXT"))
                    && text.contains("qorx.context_inject")
            })
}

fn antigravity_context_rule_opt_in() -> bool {
    env::var("QORX_ANTIGRAVITY_CONTEXT_RULE")
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(true)
}

fn antigravity_mcp_opt_in() -> bool {
    env::var("QORX_ANTIGRAVITY_MCP")
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(true)
}

const ANTIGRAVITY_CONTEXT_RULE_START: &str = "<!-- QORX_VOID_ANTIGRAVITY_CONTEXT_START -->";
const ANTIGRAVITY_CONTEXT_RULE_END: &str = "<!-- QORX_VOID_ANTIGRAVITY_CONTEXT_END -->";
const LEGACY_AYIE_ANTIGRAVITY_CONTEXT_RULE_START: &str =
    "<!-- QORX_AYIE_ANTIGRAVITY_CONTEXT_START -->";
const LEGACY_AYIE_ANTIGRAVITY_CONTEXT_RULE_END: &str = "<!-- QORX_AYIE_ANTIGRAVITY_CONTEXT_END -->";

fn antigravity_context_rule_block() -> String {
    format!(
        r#"{ANTIGRAVITY_CONTEXT_RULE_START}
## Qorx Void Context

Normal chat comes first. Do not block a response while trying Qorx.

- Use the `qorx` MCP server only when the user asks for local repo, workspace, metrics, or evidence-backed context.
- Start with `qorx.context_inject`, then use `qorx.squeeze`, `qorx.map`, `qorx.orcl`, or `qorx.session` only if needed.
- Prefer `qorx://` session handles over pasting bulk repository or vault contents.
- If Qorx tools are unavailable or slow, skip Qorx for that turn and answer normally.
- Do not retry Qorx more than once in a single turn.
- Do not spawn background shells for Qorx context. Antigravity should pull context through MCP.
{ANTIGRAVITY_CONTEXT_RULE_END}
"#
    )
}

fn migrate_legacy_gemini_qorx_rule() -> Result<()> {
    let path = home_dir()?.join(".gemini").join("GEMINI.md");
    let Ok(text) = fs::read_to_string(&path) else {
        return Ok(());
    };
    if !text.contains("QORX EDGE PRIORITIZATION") && !text.contains("Qorx Edge MCP") {
        return Ok(());
    }
    let updated = r#"# Qorx Void Context

Normal chat comes first. Do not block a response while trying Qorx.

- Use Qorx only when the user asks for local repo, workspace, metrics, or evidence-backed context.
- Use `qorx.context_inject` first, then `qorx.squeeze`, `qorx.map`, or `qorx.session` only if needed.
- Prefer `qorx://` handles over bulk file or vault dumps.
- If Qorx tools are unavailable or slow, skip Qorx for that turn and answer normally.

This keeps local context small without blocking normal chat.
"#;
    fs::write(path, updated)?;
    Ok(())
}

fn remove_all_mcp_connectors() -> Result<()> {
    let _ = remove_codex_mcp_config();
    let _ = remove_claude_user_mcp_config();
    let _ = remove_antigravity_qorx_mcp_entries();

    for path in all_mcp_config_paths() {
        remove_json_object_entries(&path, "mcpServers", &["qorx", "qorx_edge", "mcp_qorx_qorx"])?;
        remove_json_object_entries(&path, "servers", &["qorx", "qorx_edge"])?;
    }
    Ok(())
}

fn remove_all_hook_connectors() -> Result<()> {
    let _ = disable_codex_hook();
    let _ = remove_antigravity_context_rule();
    if let Ok(home) = home_dir() {
        let gemini_settings = home.join(".gemini").join("settings.json");
        remove_json_hook_entries(&gemini_settings, "BeforeAgent")?;
        let _ = fs::remove_file(
            home.join(".gemini")
                .join("hooks")
                .join("qorx-middleware.cjs"),
        );

        let claude_root = home.join(".claude").join("plugins").join("qorx");
        let _ = fs::remove_file(claude_root.join("hooks").join("hooks.json"));
        let _ = fs::remove_file(claude_root.join("hooks").join("qorx_user_prompt_submit.py"));
    }
    for platform in IntegrationPlatform::agent_platforms() {
        for path in hook_paths_for(*platform) {
            remove_json_hook_entries(&path, "UserPromptSubmit")?;
        }
    }
    Ok(())
}

fn remove_json_object_entries(path: &Path, object_key: &str, names: &[&str]) -> Result<()> {
    let Ok(text) = fs::read_to_string(path) else {
        return Ok(());
    };
    let Ok(mut value) = serde_json::from_str::<Value>(&text) else {
        return Ok(());
    };
    let Some(object) = value.get_mut(object_key).and_then(Value::as_object_mut) else {
        return Ok(());
    };

    let mut changed = false;
    for name in names {
        changed |= object.remove(*name).is_some();
    }
    if changed {
        fs::write(path, serde_json::to_string_pretty(&value)?)?;
    }
    Ok(())
}

fn remove_json_hook_entries(path: &Path, event: &str) -> Result<()> {
    let Ok(text) = fs::read_to_string(path) else {
        return Ok(());
    };
    let Ok(mut value) = serde_json::from_str::<Value>(&text) else {
        return Ok(());
    };
    let mut changed = remove_qorx_hook_entries_from_object(&mut value, event);
    if let Some(hooks) = value.get_mut("hooks").and_then(Value::as_object_mut) {
        if let Some(event_value) = hooks.get_mut(event) {
            changed |= remove_qorx_hook_entries(event_value);
            if event_value
                .as_array()
                .is_some_and(|entries| entries.is_empty())
            {
                hooks.remove(event);
            }
        }
        if hooks.is_empty() {
            value.as_object_mut().map(|object| object.remove("hooks"));
            changed = true;
        }
    }
    if changed {
        fs::write(path, serde_json::to_string_pretty(&value)?)?;
    }
    Ok(())
}

fn remove_qorx_hook_entries_from_object(value: &mut Value, event: &str) -> bool {
    let Some(event_value) = value.get_mut(event) else {
        return false;
    };
    let changed = remove_qorx_hook_entries(event_value);
    if event_value
        .as_array()
        .is_some_and(|entries| entries.is_empty())
    {
        value.as_object_mut().map(|object| object.remove(event));
        return true;
    }
    changed
}

fn remove_qorx_hook_entries(value: &mut Value) -> bool {
    let Some(entries) = value.as_array_mut() else {
        return false;
    };
    let before = entries.len();
    entries.retain(|entry| {
        !serde_json::to_string(entry)
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains("qorx")
    });
    entries.len() != before
}

fn upsert_managed_block(path: &Path, start: &str, end: &str, block: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = fs::read_to_string(path).unwrap_or_default();
    let without = remove_managed_block_text(&text, start, end);
    let mut next = without.trim_end().to_string();
    if !next.is_empty() {
        next.push_str("\n\n");
    }
    next.push_str(block.trim_end());
    next.push('\n');
    fs::write(path, next)?;
    Ok(())
}

fn remove_managed_block(path: &Path, start: &str, end: &str) -> Result<()> {
    let Ok(text) = fs::read_to_string(path) else {
        return Ok(());
    };
    let next = remove_managed_block_text(&text, start, end);
    if next != text {
        fs::write(path, next.trim_end().to_string() + "\n")?;
    }
    Ok(())
}

fn remove_managed_block_text(text: &str, start: &str, end: &str) -> String {
    let mut out = Vec::new();
    let mut skipping = false;
    for line in text.lines() {
        if line.trim() == start {
            skipping = true;
            continue;
        }
        if skipping {
            if line.trim() == end {
                skipping = false;
            }
            continue;
        }
        out.push(line);
    }
    let mut next = out.join("\n");
    if text.ends_with('\n') && !next.ends_with('\n') {
        next.push('\n');
    }
    next
}

fn install_codex_mcp_config(exe: &Path) -> Result<()> {
    let path = codex_config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = fs::read_to_string(&path).unwrap_or_default();
    let mut text = remove_toml_section(&text, "mcp_servers.qorx");
    if !text.ends_with('\n') && !text.is_empty() {
        text.push('\n');
    }
    text.push_str(&format!(
        "\n[mcp_servers.qorx]\ncommand = {}\nargs = [\"mcp\"]\n",
        toml_string(&exe.display().to_string())
    ));
    fs::write(path, text)?;
    Ok(())
}

fn remove_codex_mcp_config() -> Result<()> {
    let path = codex_config_path()?;
    let Ok(text) = fs::read_to_string(&path) else {
        return Ok(());
    };
    fs::write(path, remove_toml_section(&text, "mcp_servers.qorx"))?;
    Ok(())
}

fn install_claude_user_mcp_config(exe: &Path) -> Result<()> {
    let path = claude_user_config_path()?;
    let mut value = fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .unwrap_or_else(|| json!({}));
    if !value.is_object() {
        value = json!({});
    }
    if value.get("mcpServers").and_then(Value::as_object).is_none() {
        value["mcpServers"] = json!({});
    }
    value["mcpServers"]["qorx"] = json!({
        "type": "stdio",
        "command": exe.display().to_string(),
        "args": ["mcp"],
        "env": {}
    });
    fs::write(path, serde_json::to_string_pretty(&value)?)?;
    Ok(())
}

fn remove_claude_user_mcp_config() -> Result<()> {
    remove_json_object_entries(
        &claude_user_config_path()?,
        "mcpServers",
        &["qorx", "qorx_edge"],
    )
}

fn remove_toml_section(text: &str, section: &str) -> String {
    let header = format!("[{section}]");
    let mut kept = Vec::new();
    let mut skipping = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == header {
            skipping = true;
            continue;
        }
        if skipping && trimmed.starts_with('[') && trimmed.ends_with(']') {
            skipping = false;
        }
        if !skipping {
            kept.push(line);
        }
    }
    let mut result = kept.join("\n");
    if text.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

fn toml_string(value: &str) -> String {
    if !value.contains('\'') {
        format!("'{value}'")
    } else {
        serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
    }
}

fn install_mcp_server_config(
    path: &Path,
    object_key: &str,
    server_name: &str,
    exe: &Path,
    cwd: Option<&Path>,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut value = fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .unwrap_or_else(|| json!({}));
    if !value.is_object() {
        value = json!({});
    }
    if value.get(object_key).and_then(Value::as_object).is_none() {
        value[object_key] = json!({});
    }
    if object_key == "servers" && value.get("inputs").is_none() {
        value["inputs"] = json!([]);
    }
    value[object_key][server_name] = mcp_server_json_with_cwd(exe, cwd);
    fs::write(path, serde_json::to_string_pretty(&value)?)?;
    Ok(())
}

fn mcp_server_json_with_cwd(exe: &Path, cwd: Option<&Path>) -> Value {
    let mut server = json!({
        "command": exe.display().to_string(),
        "args": ["mcp"]
    });
    if let Some(cwd) = cwd {
        server["cwd"] = json!(cwd.display().to_string());
    }
    server
}

fn install_named_mcp_platform(exe: &Path, paths: &[PathBuf], _platform_slug: &str) -> Result<()> {
    for path in paths {
        install_mcp_server_config(path, "mcpServers", "qorx", exe, None)?;
    }
    Ok(())
}

fn install_agent_home_connector(
    exe: &Path,
    home: &Path,
    platform_slug: &str,
    settings: &IntegrationSettings,
) -> Result<()> {
    fs::create_dir_all(home)?;
    if settings.automcp_enabled {
        install_mcp_server_config(&home.join("mcp.json"), "mcpServers", "qorx", exe, None)?;
        install_mcp_server_config(&home.join(".mcp.json"), "mcpServers", "qorx", exe, None)?;
    }
    if settings.autohook_enabled {
        install_command_hook_json(
            &home.join("hooks.json"),
            "UserPromptSubmit",
            &python_hook_command(&ensure_shared_hook_script()?),
            &format!("Loading Qorx context for {platform_slug}"),
        )?;
    }
    if settings.any_enabled() {
        write_agent_skill(home, platform_slug)?;
    }
    Ok(())
}

fn install_claude_plugin_mcp_connector(exe: &Path) -> Result<()> {
    let root = home_dir()?.join(".claude").join("plugins").join("qorx");
    fs::create_dir_all(&root)?;
    install_mcp_server_config(&root.join(".mcp.json"), "mcpServers", "qorx", exe, None)?;
    write_claude_plugin_manifest(&root)
}

fn install_claude_plugin_hook_connector() -> Result<()> {
    let root = home_dir()?.join(".claude").join("plugins").join("qorx");
    fs::create_dir_all(root.join("hooks"))?;
    fs::write(
        root.join("hooks").join("qorx_user_prompt_submit.py"),
        codex_hook_script(),
    )?;
    fs::write(
        root.join("hooks").join("hooks.json"),
        serde_json::to_string_pretty(&json!({
            "hooks": {
                "UserPromptSubmit": [
                    {
                        "matcher": "*",
                        "hooks": [
                            {
                                "type": "command",
                                "command": python_hook_command(&root.join("hooks").join("qorx_user_prompt_submit.py")),
                                "statusMessage": "Loading Qorx context"
                            }
                        ]
                    }
                ]
            }
        }))?,
    )?;
    write_claude_plugin_manifest(&root)
}

fn write_claude_plugin_manifest(root: &Path) -> Result<()> {
    fs::write(
        root.join("plugin.json"),
        serde_json::to_string_pretty(&json!({
            "name": "qorx",
            "version": crate::version::QORX_VERSION,
            "description": "Qorx local context MCP and prompt hook connector",
            "hooks": "./hooks/hooks.json",
            "mcpServers": "./.mcp.json"
        }))?,
    )?;
    Ok(())
}

fn install_gemini_mcp_file(exe: &Path) -> Result<()> {
    install_mcp_server_config(
        &home_dir()?.join(".gemini").join("settings.json"),
        "mcpServers",
        "qorx",
        exe,
        None,
    )
}

fn install_gemini_hook() -> Result<()> {
    let hook_path = home_dir()?
        .join(".gemini")
        .join("hooks")
        .join("qorx-middleware.cjs");
    if let Some(parent) = hook_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&hook_path, gemini_hook_script())?;
    install_command_hook_json(
        &home_dir()?.join(".gemini").join("settings.json"),
        "BeforeAgent",
        &format!("node {}", hook_path.display()),
        "Injects Qorx context into every prompt",
    )
}

fn write_agent_skill(home: &Path, platform_slug: &str) -> Result<()> {
    let skill_dir = home.join("skills").join("qorx");
    let gateway = local_base();
    fs::create_dir_all(&skill_dir)?;
    fs::write(
        skill_dir.join("SKILL.md"),
        format!(
            r#"---
name: qorx
description: Use Qorx local context, MCP tools, and prompt hooks when repository or local evidence matters.
---

# Qorx

Qorx is installed for `{platform_slug}`.

- MCP server: `qorx mcp`
- Local gateway: `{gateway}`
- Prompt hook: `hooks.json` in this platform home when the client supports hook loading
- Use Qorx to fetch compact cited local context instead of pasting whole files into prompts.
"#
        ),
    )?;
    Ok(())
}

fn gemini_mcp_config_active(exe: &Path) -> bool {
    home_dir()
        .map(|home| home.join(".gemini").join("settings.json"))
        .ok()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|value| value.pointer("/mcpServers/qorx").cloned())
        .is_some_and(|server| mcp_server_matches(&server, exe))
}

fn codex_mcp_config_active(exe: &Path) -> bool {
    codex_config_path()
        .ok()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|text| toml_section(&text, "mcp_servers.qorx"))
        .is_some_and(|section| {
            section.to_ascii_lowercase().contains("mcp")
                && section_contains_command_path(&section, exe)
        })
}

fn claude_mcp_config_active(exe: &Path) -> bool {
    let user_config_active = claude_user_config_path()
        .ok()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|value| value.pointer("/mcpServers/qorx").cloned())
        .is_some_and(|server| mcp_server_matches(&server, exe));

    user_config_active
        || home_dir()
            .map(|home| {
                home.join(".claude")
                    .join("plugins")
                    .join("qorx")
                    .join(".mcp.json")
            })
            .ok()
            .is_some_and(|path| mcp_config_active(&path, "mcpServers", "qorx", exe))
}

fn antigravity_mcp_config_active(exe: &Path) -> bool {
    antigravity_mcp_config_active_in_paths(&mcp_paths_for(IntegrationPlatform::Antigravity), exe)
}

fn antigravity_mcp_config_active_in_paths(paths: &[PathBuf], exe: &Path) -> bool {
    paths.iter().any(|path| {
        antigravity_mcp_server_active(Some(path.as_path()), "mcpServers", "qorx", exe)
            || antigravity_mcp_server_active(
                Some(path.as_path()),
                "mcpServers",
                "mcp_qorx_qorx",
                exe,
            )
            || antigravity_mcp_server_active(Some(path.as_path()), "mcpServers", "qorx_edge", exe)
    })
}

fn antigravity_mcp_server_active(
    path: Option<&Path>,
    object_key: &str,
    server_name: &str,
    exe: &Path,
) -> bool {
    path.and_then(|path| fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|value| value.get(object_key)?.get(server_name).cloned())
        .is_some_and(|server| mcp_server_matches(&server, exe))
}

fn mcp_config_active(path: &Path, object_key: &str, server_name: &str, exe: &Path) -> bool {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|value| value.get(object_key)?.get(server_name).cloned())
        .is_some_and(|server| mcp_server_matches(&server, exe))
}

fn toml_section(text: &str, section: &str) -> Option<String> {
    let header = format!("[{section}]");
    let mut found = false;
    let mut lines = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == header {
            found = true;
            lines.push(line);
            continue;
        }
        if found && trimmed.starts_with('[') && trimmed.ends_with(']') {
            break;
        }
        if found {
            lines.push(line);
        }
    }
    found.then(|| lines.join("\n"))
}

fn section_contains_command_path(section: &str, exe: &Path) -> bool {
    let wanted = normalize_path_text(&exe.display().to_string());
    section
        .lines()
        .filter_map(|line| line.split_once('='))
        .filter(|(key, _)| key.trim().eq_ignore_ascii_case("command"))
        .map(|(_, value)| {
            value
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string()
        })
        .any(|command| normalize_path_text(&command) == wanted)
}

fn gemini_hook_active() -> bool {
    home_dir()
        .map(|home| home.join(".gemini").join("settings.json"))
        .ok()
        .is_some_and(|path| hook_config_active(&path))
}

fn claude_plugin_hook_active() -> bool {
    home_dir()
        .map(|home| {
            home.join(".claude")
                .join("plugins")
                .join("qorx")
                .join("hooks")
                .join("hooks.json")
        })
        .ok()
        .is_some_and(|path| hook_config_active(&path))
}

fn hook_paths_for(platform: IntegrationPlatform) -> Vec<PathBuf> {
    let home = home_dir().ok();
    match platform {
        IntegrationPlatform::OpenClaw => home
            .map(|home| vec![home.join(".openclaw").join("hooks.json")])
            .unwrap_or_default(),
        IntegrationPlatform::FactoryDroid => home
            .map(|home| vec![home.join(".factory").join("hooks.json")])
            .unwrap_or_default(),
        IntegrationPlatform::Trae => home
            .map(|home| vec![home.join(".trae").join("hooks.json")])
            .unwrap_or_default(),
        IntegrationPlatform::TraeCn => home
            .map(|home| vec![home.join(".trae-cn").join("hooks.json")])
            .unwrap_or_default(),
        IntegrationPlatform::Hermes => home
            .map(|home| vec![home.join(".hermes").join("hooks.json")])
            .unwrap_or_default(),
        IntegrationPlatform::Kiro => home
            .map(|home| vec![home.join(".kiro").join("hooks.json")])
            .unwrap_or_default(),
        IntegrationPlatform::Pi => home
            .map(|home| vec![home.join(".pi").join("agent").join("hooks.json")])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn hook_config_active(path: &Path) -> bool {
    fs::read_to_string(path).ok().is_some_and(|text| {
        let lower = text.to_ascii_lowercase();
        lower.contains("qorx") && lower.contains("hook")
    })
}

fn mcp_server_matches(server: &Value, exe: &Path) -> bool {
    let Some(command) = server.get("command").and_then(Value::as_str) else {
        return false;
    };
    let command_matches =
        normalize_path_text(command) == normalize_path_text(&exe.display().to_string());
    let args_match = server
        .get("args")
        .and_then(Value::as_array)
        .is_some_and(|args| {
            args.iter().any(|arg| {
                arg.as_str()
                    .is_some_and(|arg| arg.eq_ignore_ascii_case("mcp"))
            })
        });
    command_matches && args_match
}

fn write_shims(paths: &AppPaths) -> Result<IntegrationStatus> {
    let exe = current_exe()?;
    let shims = [
        ("qorx-codex.cmd", qorx_codex_shim_body(&exe)),
        ("qorx-codex.ps1", qorx_codex_shim_script_body(&exe)),
        (
            "qorx-gemini.cmd",
            format!("@echo off\r\n\"{}\" run gemini -- %*\r\n", exe.display()),
        ),
        (
            "qorx-claude.cmd",
            format!("@echo off\r\n\"{}\" run claude -- %*\r\n", exe.display()),
        ),
        (
            "qorx-mcp.cmd",
            format!("@echo off\r\n\"{}\" mcp\r\n", exe.display()),
        ),
        (
            "qorx-antigravity.cmd",
            "@echo off\r\nantigravity chat %*\r\n".to_string(),
        ),
        (
            "qorx-opencode.cmd",
            "@echo off\r\nopencode %*\r\n".to_string(),
        ),
        ("qorx-aider.cmd", "@echo off\r\naider %*\r\n".to_string()),
        ("qorx-goose.cmd", "@echo off\r\ngoose %*\r\n".to_string()),
    ];
    for dir in reachable_shim_dirs(paths) {
        fs::create_dir_all(&dir)?;
        for (shim, body) in &shims {
            fs::write(dir.join(shim), body)?;
        }
    }
    Ok(shims_status(paths))
}

fn qorx_codex_shim_body(exe: &Path) -> String {
    let _ = exe;
    "@echo off\r\npowershell -NoProfile -ExecutionPolicy Bypass -File \"%~dp0qorx-codex.ps1\" %*\r\n"
        .to_string()
}

fn qorx_codex_shim_script_body(exe: &Path) -> String {
    let _ = exe;
    let gateway = local_base();
    r#"$ErrorActionPreference = "Stop"
$CodexArgs = [string[]]$args

function Test-Truthy {
  param([string]$Value)
  $clean = ([string]$Value).Trim().ToLowerInvariant()
  return @("1", "true", "yes", "on") -contains $clean
}

function Split-ExecArgs {
  param([string[]]$InputArgs)
  $options = New-Object System.Collections.Generic.List[string]
  $prompt = New-Object System.Collections.Generic.List[string]
  $takesValue = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
  @(
    "-c", "--config", "-i", "--image", "-m", "--model", "-p", "--profile",
    "-s", "--sandbox", "-C", "--cd", "--add-dir", "--output-schema", "--color",
    "-o", "--output-last-message", "--enable", "--disable"
  ) | ForEach-Object { [void]$takesValue.Add($_) }
  for ($i = 1; $i -lt $InputArgs.Count; $i++) {
    $arg = $InputArgs[$i]
    if ($prompt.Count -gt 0) {
      $prompt.Add($arg)
      continue
    }
    if ($arg -eq "--") {
      $options.Add($arg)
      for ($j = $i + 1; $j -lt $InputArgs.Count; $j++) { $prompt.Add($InputArgs[$j]) }
      break
    }
    if ($takesValue.Contains($arg)) {
      $options.Add($arg)
      if ($i + 1 -lt $InputArgs.Count) {
        $i++
        $options.Add($InputArgs[$i])
      }
      continue
    }
    if ($arg.StartsWith("-")) {
      $options.Add($arg)
      continue
    }
    $prompt.Add($arg)
  }
  return [pscustomobject]@{ Options = [string[]]$options; Prompt = [string[]]$prompt }
}

if (-not $env:QORX_GATEWAY) {
  $env:QORX_GATEWAY = "__QORX_GATEWAY__"
}
$codexCommand = if ($env:QORX_CODEX_BIN) { $env:QORX_CODEX_BIN } else { "codex" }

if ($CodexArgs.Count -gt 0 -and $CodexArgs[0] -eq "exec" -and -not (Test-Truthy $env:QORX_CODEX_CONTEXT_OFF)) {
  $split = Split-ExecArgs $CodexArgs
  $promptText = ($split.Prompt -join " ").Trim()
  $isSubcommand = @("resume", "review", "help") -contains $promptText.Split(" ")[0]
  $hookPath = Join-Path $HOME ".codex\hooks\qorx_user_prompt_submit.py"
  if ($promptText -and -not $isSubcommand -and (Test-Path $hookPath)) {
    try {
      $payload = @{ cwd = (Get-Location).Path; prompt = $promptText } | ConvertTo-Json -Compress
      $hookText = (($payload | py -3 $hookPath) -join "`n").Trim()
      if ($hookText) {
        $hookJson = $hookText | ConvertFrom-Json
        $context = [string]$hookJson.hookSpecificOutput.additionalContext
        if ($context.Trim()) {
          $wrappedPrompt = "$context`n`nUser prompt:`n$promptText"
          $savedOff = $env:QORX_CODEX_CONTEXT_OFF
          $env:QORX_CODEX_CONTEXT_OFF = "1"
          try {
            $execArgs = @("exec") + [string[]]$split.Options + @($wrappedPrompt)
            & $codexCommand @execArgs
            exit $LASTEXITCODE
          } finally {
            $env:QORX_CODEX_CONTEXT_OFF = $savedOff
          }
        }
      }
    } catch {
      Write-Warning "Qorx Codex fallback inject failed: $($_.Exception.Message)"
    }
  }
}

& $codexCommand @CodexArgs
exit $LASTEXITCODE
"#
    .replace("__QORX_GATEWAY__", &gateway)
}

fn shims_status(paths: &AppPaths) -> IntegrationStatus {
    let active = command_exists("qorx-codex") || paths.shim_dir.join("qorx-codex.cmd").exists();
    IntegrationStatus {
        name: "Qorx CLI shims".to_string(),
        installed: true,
        active,
        mechanism: "qorx-* wrapper commands".to_string(),
        detail: reachable_shim_dirs(paths)
            .into_iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join("; "),
        ..Default::default()
    }
}

fn reachable_shim_dirs(paths: &AppPaths) -> Vec<PathBuf> {
    let mut dirs = vec![paths.shim_dir.clone()];
    if cfg!(windows) {
        if let Ok(appdata) = env::var("APPDATA") {
            dirs.push(PathBuf::from(appdata).join("npm"));
        }
    }
    dirs
}

fn install_codex_hook() -> Result<IntegrationStatus> {
    let hooks_path = codex_hooks_path()?;
    let hook_script = codex_hook_script_path()?;
    if let Some(parent) = hook_script.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&hook_script, codex_hook_script())?;
    let command = python_hook_command(&hook_script);
    install_codex_command_hook_json(
        &hooks_path,
        CODEX_USER_PROMPT_EVENT,
        &command,
        "Loading Qorx context",
    )?;
    install_codex_hooks_feature_flag()?;
    install_codex_hook_trust_state(
        &hooks_path,
        CODEX_USER_PROMPT_EVENT_KEY,
        &command,
        CODEX_HOOK_TIMEOUT_SECONDS,
        "Loading Qorx context",
    )?;
    Ok(codex_hook_status())
}

fn disable_codex_hook() -> Result<IntegrationStatus> {
    let hooks_path = codex_hooks_path()?;
    if hooks_path.exists() {
        let text = fs::read_to_string(&hooks_path)?;
        let mut value = serde_json::from_str::<Value>(&text).unwrap_or_else(|_| json!({}));
        value
            .as_object_mut()
            .map(|object| object.remove("UserPromptSubmit"));
        if let Some(hooks) = value.get_mut("hooks").and_then(Value::as_object_mut) {
            hooks.remove("UserPromptSubmit");
            if hooks.is_empty() {
                value.as_object_mut().map(|object| object.remove("hooks"));
            }
        }
        fs::write(&hooks_path, serde_json::to_string_pretty(&value)?)?;
    }
    Ok(codex_hook_status())
}

fn install_command_hook_json(
    path: &Path,
    event: &str,
    command: &str,
    status_message: &str,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut value = fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .unwrap_or_else(|| json!({}));
    if !value.is_object() {
        value = json!({});
    }
    if value.get("hooks").and_then(Value::as_object).is_none() {
        value["hooks"] = json!({});
    }
    if value["hooks"]
        .get(event)
        .and_then(Value::as_array)
        .is_none()
    {
        value["hooks"][event] = json!([]);
    }

    if let Some(entries) = value["hooks"][event].as_array_mut() {
        entries.retain(|entry| {
            !serde_json::to_string(entry)
                .unwrap_or_default()
                .to_ascii_lowercase()
                .contains("qorx")
        });
        entries.push(json!({
            "matcher": "*",
            "hooks": [
                {
                    "name": "qorx-injector",
                    "type": "command",
                    "command": command,
                    "timeout": 5000,
                    "statusMessage": status_message,
                    "description": status_message
                }
            ]
        }));
    }

    fs::write(path, serde_json::to_string_pretty(&value)?)?;
    Ok(())
}

fn install_codex_command_hook_json(
    path: &Path,
    event: &str,
    command: &str,
    status_message: &str,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut value = fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .unwrap_or_else(|| json!({}));
    if !value.is_object() {
        value = json!({});
    }

    let mut entries = Vec::new();
    if let Some(existing) = value.get(event).and_then(Value::as_array) {
        entries.extend(existing.iter().cloned());
    }
    if let Some(nested) = value
        .get("hooks")
        .and_then(Value::as_object)
        .and_then(|hooks| hooks.get(event))
        .and_then(Value::as_array)
    {
        entries.extend(nested.iter().cloned());
    }
    entries.retain(|entry| {
        !serde_json::to_string(entry)
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains("qorx")
    });
    entries.push(json!({
        "hooks": [
            {
                "type": "command",
                "command": command,
                "timeout": CODEX_HOOK_TIMEOUT_SECONDS,
                "statusMessage": status_message
            }
        ]
    }));
    value.as_object_mut().map(|object| object.remove(event));
    if value.get("hooks").and_then(Value::as_object).is_none() {
        value["hooks"] = json!({});
    }
    value["hooks"][event] = Value::Array(entries);

    fs::write(path, serde_json::to_string_pretty(&value)?)?;
    Ok(())
}

fn codex_hook_status() -> IntegrationStatus {
    let active = codex_hooks_path().ok().is_some_and(|path| {
        let hook_config_active = fs::read_to_string(&path)
            .ok()
            .is_some_and(|text| codex_hook_config_active(&text));
        let trust_state_active = codex_config_path().ok().is_some_and(|config_path| {
            fs::read_to_string(config_path)
                .ok()
                .is_some_and(|text| codex_hook_trust_state_active(&text, &path))
        });
        hook_config_active && trust_state_active
    });
    IntegrationStatus {
        name: "Codex prompt hook".to_string(),
        installed: true,
        active,
        mechanism: "UserPromptSubmit additionalContext".to_string(),
        detail: codex_hooks_path()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|err| err.to_string()),
        ..Default::default()
    }
}

fn codex_hook_config_active(text: &str) -> bool {
    let Ok(value) = serde_json::from_str::<Value>(text) else {
        return false;
    };
    let Some(entries) = value
        .get("hooks")
        .and_then(Value::as_object)
        .and_then(|hooks| hooks.get(CODEX_USER_PROMPT_EVENT))
        .and_then(Value::as_array)
    else {
        return false;
    };
    entries.iter().any(|entry| {
        serde_json::to_string(entry)
            .unwrap_or_default()
            .contains("qorx_user_prompt_submit.py")
    })
}

fn install_codex_hooks_feature_flag() -> Result<()> {
    let path = codex_config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = fs::read_to_string(&path).unwrap_or_default();
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .unwrap_or_else(|_| toml_edit::DocumentMut::new());
    doc["features"]["hooks"] = toml_edit::value(true);
    fs::write(path, doc.to_string())?;
    Ok(())
}

fn install_codex_hook_trust_state(
    hooks_path: &Path,
    event_key: &str,
    command: &str,
    timeout_seconds: u64,
    status_message: &str,
) -> Result<()> {
    let path = codex_config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let key = codex_hook_state_key(hooks_path, event_key);
    let section = format!("hooks.state.{}", toml_string(&key));
    let trusted_hash = codex_command_hook_hash(event_key, command, timeout_seconds, status_message);
    let mut text = remove_toml_section(&fs::read_to_string(&path).unwrap_or_default(), &section);
    if !text.ends_with('\n') && !text.is_empty() {
        text.push('\n');
    }
    text.push_str(&format!(
        "\n[{section}]\nenabled = true\ntrusted_hash = {}\n",
        toml_string(&trusted_hash)
    ));
    fs::write(path, text)?;
    Ok(())
}

fn codex_hook_trust_state_active(config_toml: &str, hooks_path: &Path) -> bool {
    let key = codex_hook_state_key(hooks_path, CODEX_USER_PROMPT_EVENT_KEY);
    let expected_hash = codex_command_hook_hash(
        CODEX_USER_PROMPT_EVENT_KEY,
        &python_hook_command(&codex_hook_script_path().unwrap_or_else(|_| {
            home_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(".codex")
                .join("hooks")
                .join("qorx_user_prompt_submit.py")
        })),
        CODEX_HOOK_TIMEOUT_SECONDS,
        "Loading Qorx context",
    );
    let Ok(doc) = config_toml.parse::<toml_edit::DocumentMut>() else {
        return false;
    };
    let Some(state) = doc
        .get("hooks")
        .and_then(|hooks| hooks.get("state"))
        .and_then(|state| state.get(&key))
    else {
        return false;
    };
    let enabled = state
        .get("enabled")
        .and_then(|enabled| enabled.as_bool())
        .unwrap_or(true);
    let trusted = state
        .get("trusted_hash")
        .and_then(|hash| hash.as_str())
        .is_some_and(|hash| hash == expected_hash);
    enabled && trusted
}

fn codex_hook_state_key(hooks_path: &Path, event_key: &str) -> String {
    format!("{}:{event_key}:0:0", hooks_path.display())
}

fn codex_command_hook_hash(
    event_key: &str,
    command: &str,
    timeout_seconds: u64,
    status_message: &str,
) -> String {
    let identity = json!({
        "event_name": event_key,
        "hooks": [
            {
                "async": false,
                "command": command,
                "statusMessage": status_message,
                "timeout": timeout_seconds,
                "type": "command"
            }
        ]
    });
    version_for_json(&identity)
}

fn version_for_json(value: &Value) -> String {
    let canonical = canonical_json(value);
    let serialized = serde_json::to_vec(&canonical).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(serialized);
    let hash = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("sha256:{hash}")
}

fn canonical_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted = serde_json::Map::new();
            let mut keys = map.keys().collect::<Vec<_>>();
            keys.sort();
            for key in keys {
                if let Some(value) = map.get(key) {
                    sorted.insert(key.to_string(), canonical_json(value));
                }
            }
            Value::Object(sorted)
        }
        Value::Array(items) => Value::Array(items.iter().map(canonical_json).collect()),
        other => other.clone(),
    }
}

fn autostart_status() -> Result<IntegrationStatus> {
    let startup = startup_file()?;
    Ok(IntegrationStatus {
        name: "Windows daemon + tray login startup".to_string(),
        installed: true,
        active: startup.exists(),
        mechanism: "Startup folder VBS hidden daemon + tray launch".to_string(),
        detail: startup.display().to_string(),
        ..Default::default()
    })
}

fn tray_runtime_status(exe: &Path) -> IntegrationStatus {
    let active = tray_process_running(exe);
    IntegrationStatus {
        name: "Qorx tray runtime".to_string(),
        installed: true,
        active,
        mechanism: "qorx.exe tray notification-area process".to_string(),
        detail: if active {
            format!("{} tray is running", exe.display())
        } else {
            format!("{} tray is not running", exe.display())
        },
        ..Default::default()
    }
}

fn ensure_daemon_running() -> Result<()> {
    let exe = current_exe()?;
    if daemon_process_running(&exe) {
        return Ok(());
    }

    let mut command = Command::new(&exe);
    command
        .arg("daemon")
        .arg("start")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    spawn_hidden(&mut command)
}

fn ensure_tray_running() -> Result<()> {
    if !cfg!(windows) {
        return Ok(());
    }

    let exe = current_exe()?;
    if tray_process_running(&exe) {
        return Ok(());
    }

    let mut command = Command::new(&exe);
    command
        .arg("tray")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    spawn_hidden(&mut command)
}

fn spawn_hidden(command: &mut Command) -> Result<()> {
    configure_hidden_process(command);
    command.spawn()?;
    Ok(())
}

#[cfg(windows)]
fn configure_hidden_process(command: &mut Command) {
    use std::os::windows::process::CommandExt;

    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn configure_hidden_process(_command: &mut Command) {}

#[cfg(windows)]
fn configure_no_window_process(command: &mut Command) {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn configure_no_window_process(_command: &mut Command) {}

fn tray_process_running(exe: &Path) -> bool {
    qorx_process_command_lines()
        .into_iter()
        .any(|line| is_tray_command_line(&line, exe))
}

fn daemon_process_running(exe: &Path) -> bool {
    qorx_process_command_lines()
        .into_iter()
        .any(|line| is_daemon_command_line(&line, exe))
}

fn qorx_process_command_lines() -> Vec<String> {
    if !cfg!(windows) {
        return Vec::new();
    }

    let mut command = Command::new("powershell.exe");
    command
        .arg("-NoProfile")
        .arg("-Command")
        .arg("Get-CimInstance Win32_Process -Filter \"name = 'qorx.exe'\" | ForEach-Object { $_.CommandLine }")
        .stdin(Stdio::null())
        .stderr(Stdio::null());
    configure_no_window_process(&mut command);
    let output = command.output();

    output
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn is_tray_command_line(command_line: &str, exe: &Path) -> bool {
    let args = split_command_line(command_line);
    command_line_matches_exe(command_line, &args, exe)
        && command_line_has_arg(command_line, &args, "tray")
}

fn is_daemon_command_line(command_line: &str, exe: &Path) -> bool {
    let args = split_command_line(command_line);
    command_line_matches_exe(command_line, &args, exe)
        && command_line_has_arg(command_line, &args, "daemon")
        && !command_line_has_arg(command_line, &args, "stop")
}

fn command_line_matches_exe(command_line: &str, args: &[String], exe: &Path) -> bool {
    let wanted = normalize_path_text(&exe.display().to_string());
    args.first()
        .is_some_and(|command| normalize_path_text(command) == wanted)
        || normalize_path_text(command_line).contains(&wanted)
}

fn command_line_has_arg(command_line: &str, args: &[String], wanted: &str) -> bool {
    args.iter()
        .skip(1)
        .any(|arg| arg.eq_ignore_ascii_case(wanted))
        || normalize_path_text(command_line)
            .split_whitespace()
            .any(|arg| arg.trim_matches('"').eq_ignore_ascii_case(wanted))
}

fn split_command_line(command_line: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;

    for ch in command_line.chars() {
        match ch {
            '"' => in_quote = !in_quote,
            ch if ch.is_whitespace() && !in_quote => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

fn normalize_path_text(path: &str) -> String {
    path.trim()
        .trim_start_matches(r"\\?\")
        .replace('/', "\\")
        .to_lowercase()
}

fn startup_file() -> Result<PathBuf> {
    let appdata = env::var("APPDATA")?;
    Ok(PathBuf::from(appdata)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Startup")
        .join("Qorx Daemon.vbs"))
}

fn legacy_tray_startup_file() -> Result<PathBuf> {
    let appdata = env::var("APPDATA")?;
    Ok(PathBuf::from(appdata)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Startup")
        .join("Qorx Tray.vbs"))
}

fn codex_hooks_path() -> Result<PathBuf> {
    Ok(home_dir()?.join(".codex").join("hooks.json"))
}

fn codex_config_path() -> Result<PathBuf> {
    Ok(home_dir()?.join(".codex").join("config.toml"))
}

fn codex_hook_script_path() -> Result<PathBuf> {
    Ok(home_dir()?
        .join(".codex")
        .join("hooks")
        .join("qorx_user_prompt_submit.py"))
}

fn home_dir() -> Result<PathBuf> {
    env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .map(PathBuf::from)
        .map_err(Into::into)
}

fn appdata_dir() -> Result<PathBuf> {
    env::var("APPDATA").map(PathBuf::from).map_err(Into::into)
}

fn current_exe() -> Result<PathBuf> {
    env::current_exe().map_err(Into::into)
}

fn antigravity_mcp_path() -> Result<PathBuf> {
    Ok(PathBuf::from(env::var("APPDATA")?)
        .join("Antigravity")
        .join("User")
        .join("mcp.json"))
}

fn legacy_gemini_antigravity_mcp_path() -> Result<PathBuf> {
    Ok(home_dir()?
        .join(".gemini")
        .join("antigravity")
        .join("mcp_config.json"))
}

fn gemini_hook_script_path() -> Result<PathBuf> {
    Ok(home_dir()?
        .join(".gemini")
        .join("hooks")
        .join("qorx-middleware.cjs"))
}

fn antigravity_context_rule_path() -> Result<PathBuf> {
    Ok(home_dir()?.join(".gemini").join("AGENTS.md"))
}

fn claude_user_config_path() -> Result<PathBuf> {
    Ok(home_dir()?.join(".claude.json"))
}

fn shared_hook_script_path() -> Result<PathBuf> {
    Ok(home_dir()?
        .join(".qorx")
        .join("hooks")
        .join("qorx_user_prompt_submit.py"))
}

fn ensure_shared_hook_script() -> Result<PathBuf> {
    let path = shared_hook_script_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, codex_hook_script())?;
    Ok(path)
}

fn python_hook_command(path: &Path) -> String {
    if cfg!(windows) {
        format!("py -3 \"{}\"", path.display())
    } else {
        format!("python3 \"{}\"", path.display())
    }
}

fn mcp_paths_for(platform: IntegrationPlatform) -> Vec<PathBuf> {
    let home = home_dir().ok();
    let appdata = appdata_dir().ok();
    match platform {
        IntegrationPlatform::OpenCode => {
            let mut paths = Vec::new();
            if let Some(home) = home.as_ref() {
                paths.push(home.join(".opencode").join("mcp.json"));
            }
            if let Some(appdata) = appdata.as_ref() {
                paths.push(appdata.join("OpenCode").join("mcp.json"));
            }
            paths
        }
        IntegrationPlatform::Copilot => home
            .map(|home| vec![home.join(".copilot").join("mcp.json")])
            .unwrap_or_default(),
        IntegrationPlatform::VsCodeCopilot => appdata
            .map(|appdata| vec![appdata.join("Code").join("User").join("mcp.json")])
            .unwrap_or_default(),
        IntegrationPlatform::Aider => home
            .map(|home| {
                vec![
                    home.join(".aider").join("mcp.json"),
                    home.join(".aider-desk").join("mcp.json"),
                ]
            })
            .unwrap_or_default(),
        IntegrationPlatform::OpenClaw => home
            .map(|home| vec![home.join(".openclaw").join("mcp.json")])
            .unwrap_or_default(),
        IntegrationPlatform::FactoryDroid => home
            .map(|home| vec![home.join(".factory").join("mcp.json")])
            .unwrap_or_default(),
        IntegrationPlatform::Trae => home
            .map(|home| vec![home.join(".trae").join("mcp.json")])
            .unwrap_or_default(),
        IntegrationPlatform::TraeCn => home
            .map(|home| vec![home.join(".trae-cn").join("mcp.json")])
            .unwrap_or_default(),
        IntegrationPlatform::Hermes => home
            .map(|home| vec![home.join(".hermes").join("mcp.json")])
            .unwrap_or_default(),
        IntegrationPlatform::Kiro => home
            .map(|home| vec![home.join(".kiro").join("mcp.json")])
            .unwrap_or_default(),
        IntegrationPlatform::Pi => home
            .map(|home| vec![home.join(".pi").join("agent").join("mcp.json")])
            .unwrap_or_default(),
        IntegrationPlatform::Cursor => home
            .map(|home| vec![home.join(".cursor").join("mcp.json")])
            .unwrap_or_default(),
        IntegrationPlatform::Antigravity => {
            let mut paths = Vec::new();
            if let Some(appdata) = appdata.as_ref() {
                paths.push(appdata.join("Antigravity").join("User").join("mcp.json"));
            }
            if let Ok(path) = legacy_gemini_antigravity_mcp_path() {
                paths.push(path);
            } else if let Some(home) = home.as_ref() {
                paths.push(
                    home.join(".gemini")
                        .join("antigravity")
                        .join("mcp_config.json"),
                );
            }
            paths
        }
        _ => Vec::new(),
    }
}

fn all_mcp_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for platform in IntegrationPlatform::agent_platforms() {
        paths.extend(mcp_paths_for(*platform));
    }
    if let Ok(home) = home_dir() {
        paths.push(home.join(".gemini").join("settings.json"));
        paths.push(
            home.join(".claude")
                .join("plugins")
                .join("qorx")
                .join(".mcp.json"),
        );
        for home_name in [
            ".openclaw",
            ".factory",
            ".trae",
            ".trae-cn",
            ".hermes",
            ".kiro",
        ] {
            paths.push(home.join(home_name).join(".mcp.json"));
        }
        paths.push(home.join(".pi").join("agent").join(".mcp.json"));
    }
    paths.sort();
    paths.dedup();
    paths
}

fn set_user_env(key: &str, value: &str) -> Result<()> {
    let mut command = Command::new("reg");
    command
        .args([
            "add",
            "HKCU\\Environment",
            "/v",
            key,
            "/t",
            "REG_SZ",
            "/d",
            value,
            "/f",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    configure_hidden_process(&mut command);
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("failed to set user env {key}"))
    }
}

pub fn load_settings(paths: &AppPaths) -> Result<IntegrationSettings> {
    let path = integration_settings_path(paths);
    if path.exists() {
        crate::proto_store::load_required::<IntegrationSettings>(&path, &[])
            .map(IntegrationSettings::normalized)
    } else {
        Ok(IntegrationSettings::all_enabled())
    }
}

fn save_settings(paths: &AppPaths, settings: &IntegrationSettings) -> Result<()> {
    crate::proto_store::save(&integration_settings_path(paths), settings)
}

fn integration_settings_path(paths: &AppPaths) -> PathBuf {
    paths.data_dir.join("integration-settings.pb")
}

fn write_report(paths: &AppPaths, report: &IntegrationReport) -> Result<()> {
    crate::proto_store::save(&paths.integration_report_file, report)
}

fn command_exists(name: &str) -> bool {
    resolve_on_path(name).is_some()
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
            if full.is_file() {
                return Some(full);
            }
        }
    }
    None
}

fn gemini_hook_script() -> &'static str {
    r#"const http = require('http');

const gateway = (process.env.QORX_GATEWAY || 'http://127.0.0.1:47187').replace(/\/$/, '');

function getJson(path) {
  return new Promise((resolve, reject) => {
    const req = http.get(gateway + path, { timeout: 900 }, (res) => {
      let body = '';
      res.setEncoding('utf8');
      res.on('data', (chunk) => { body += chunk; });
      res.on('end', () => {
        try { resolve(JSON.parse(body || '{}')); } catch (error) { reject(error); }
      });
    });
    req.on('timeout', () => req.destroy(new Error('timeout')));
    req.on('error', reject);
  });
}

async function buildAdditionalContext(payload) {
  const prompt = String(payload.prompt || payload.user_prompt || payload.message || '').trim();
  const cwd = String(payload.cwd || process.env.GEMINI_CWD || process.cwd() || '').trim();
  const query = new URLSearchParams({
    objective: prompt.slice(0, 500) || 'Gemini prompt context',
    cwd,
    budget_tokens: '900',
    limit: '3',
  }).toString();
  try {
    const data = await getJson('/context/inject?' + query);
    if (data.additional_context) return data.additional_context;
    if (data.handle) return [
      'Qorx is ready for this turn.',
      'Local context stays on this computer until exact proof is needed.',
      `Gateway: ${gateway}`,
      `Handle: ${data.handle}`,
      `Proof endpoint: ${gateway}/context/fault`,
      'Ask Qorx for cited proof pages before trusting local claims.',
    ].join('\n');
  } catch (error) {}
  return '';
}

let input = '';
process.stdin.on('data', (chunk) => { input += chunk; });
process.stdin.on('end', async () => {
  try {
    const payload = JSON.parse(input || '{}');
    const additionalContext = await buildAdditionalContext(payload);
    const response = additionalContext
      ? {
          hookSpecificOutput: { additionalContext },
          systemMessage: 'Qorx context injected [qorx-injector]',
          suppressOutput: true,
        }
      : { suppressOutput: true };
    console.log(JSON.stringify(response));
  } catch (error) {
    console.log(JSON.stringify({ suppressOutput: true }));
  }
});
"#
}

fn codex_hook_script() -> &'static str {
    r#"import json
import os
import sys
import urllib.parse
import urllib.request

GATEWAY = os.environ.get("QORX_GATEWAY", "http://127.0.0.1:47187").rstrip("/")
HTTP_TIMEOUT_SECONDS = float(os.environ.get("QORX_CODEX_HTTP_TIMEOUT", "2.5"))
DEFAULT_CONTEXT_BUDGET = os.environ.get("QORX_CODEX_CONTEXT_BUDGET", "180")
DEFAULT_CONTEXT_LIMIT = os.environ.get("QORX_CODEX_CONTEXT_LIMIT", "1")
DEEP_CONTEXT_BUDGET = os.environ.get("QORX_CODEX_DEEP_CONTEXT_BUDGET", "600")
DEEP_CONTEXT_LIMIT = os.environ.get("QORX_CODEX_DEEP_CONTEXT_LIMIT", "3")


def truthy_env(key):
    return os.environ.get(key, "").strip().lower() in {"1", "true", "yes", "on"}


def normalize_context_mode(mode):
    raw = str(mode or "").strip().lower()
    if raw == "verbose":
        return "readable"
    if raw in {"auto", "readable", "deep", "off", "quetta"}:
        return raw
    return "auto"


def env_context_mode():
    if truthy_env("QORX_CODEX_CONTEXT_OFF"):
        return "off"
    explicit = os.environ.get("QORX_CODEX_CONTEXT_MODE")
    if explicit:
        return normalize_context_mode(explicit)
    if truthy_env("QORX_CODEX_DEEP_CONTEXT"):
        return "deep"
    if truthy_env("QORX_CODEX_VERBOSE_CONTEXT"):
        return "readable"
    if truthy_env("QORX_CODEX_QUETTA"):
        return "quetta"
    return ""


def saved_context_mode():
    override = env_context_mode()
    if override:
        return override
    try:
        settings = get_json("/integrations/settings")
        return normalize_context_mode(settings.get("codex_context_mode", "auto"))
    except Exception:
        return "auto"


def hook_enabled():
    return not truthy_env("QORX_CODEX_CONTEXT_OFF")


def compact_context_enabled():
    return not truthy_env("QORX_CODEX_VERBOSE_CONTEXT")


def deep_context_enabled():
    return truthy_env("QORX_CODEX_DEEP_CONTEXT")


def hook_payload(additional_context):
    return {
        "hookSpecificOutput": {
            "hookEventName": "UserPromptSubmit",
            "additionalContext": additional_context,
        }
    }


def emit(response):
    if response:
        print(json.dumps(response, separators=(",", ":")))


def get_json(path):
    with urllib.request.urlopen(GATEWAY + path, timeout=HTTP_TIMEOUT_SECONDS) as response:
        return json.loads(response.read().decode("utf-8"))


def summarize_context_vm(data):
    context = data.get("additional_context", "")
    if context:
        return context
    handle = data.get("handle", "qorx://s/unavailable")
    return "\n".join([
        "Qorx is ready for this turn.",
        "Local context stays on this computer until exact proof is needed.",
        f"Gateway: {GATEWAY}",
        f"Handle: {handle}",
        f"Fault endpoint: {GATEWAY}/context/fault",
        "Ask Qorx for cited proof pages before trusting local claims.",
    ])


def summarize_context_nano(data):
    carrier = data.get("carrier", "")
    handle = data.get("handle", "")
    if carrier and compact_context_enabled():
        if handle.startswith("qorx://"):
            return f"Qorx Auto is on. Local context stays here. Handle: {handle}. Pull exact proof only when needed."
        return "Qorx Auto is on. Local context stays here. Pull exact proof only when needed."
    return summarize_context_vm(data)


def summarize_capsule(data):
    handle = data.get("handle", "qorx://c/unavailable")
    quarks = data.get("quark_count", data.get("atom_count", "?"))
    indexed = data.get("indexed_tokens", "?")
    sources = data.get("source_count", len(data.get("sources") or []))
    if compact_context_enabled():
        return f"Qorx {handle} src={sources} q={quarks} t={indexed}; pull only if needed."
    return "\n".join([
        "Qorx context is available for this session. Normal chat comes first.",
        f"Gateway: {GATEWAY}",
        f"Capsule handle: {handle}",
        f"Local context: {sources} source(s), {quarks} quarks, {indexed} estimated tokens.",
        "qosm + qshf are available when they help.",
        "Keep it human; pull targeted Qorx context when the task benefits from it.",
    ])


def summarize_session(data):
    handle = data.get("handle", "qorx://s/unavailable")
    quarks = data.get("quark_count", data.get("atom_count", "?"))
    indexed = data.get("indexed_tokens", "?")
    if compact_context_enabled():
        return f"Qorx {handle} q={quarks} t={indexed}; pull only if needed."
    return "\n".join([
        "Qorx context is available for this session. Normal chat comes first.",
        f"Gateway: {GATEWAY}",
        f"Session handle: {handle}",
        f"Local context: {quarks} quarks, {indexed} estimated tokens.",
        "qosm + qshf are available when they help.",
        "Keep it human; pull targeted Qorx context when the task benefits from it.",
    ])


def offline_context(error, cwd="", prompt=""):
    detail = str(error).replace("\n", " ")[:160]
    return "\n".join([
        "Qorx fallback inject is active because the gateway did not answer.",
        f"Gateway: {GATEWAY}",
        f"CWD: {cwd or os.getcwd()}",
        f"Prompt hint: {(prompt or '')[:160]}",
        f"Proceed with normal local inspection until Qorx is back. error={detail}",
    ])


def build_additional_context(cwd=None, prompt="", mode=None):
    cwd = (cwd or os.getcwd() or "").strip()
    prompt = (prompt or "").strip()
    mode = normalize_context_mode(mode or saved_context_mode())
    if mode == "off":
        return ""
    if prompt:
        try:
            query = urllib.parse.urlencode({
                "objective": prompt[:240],
                "cwd": cwd,
                "budget_tokens": DEFAULT_CONTEXT_BUDGET,
                "limit": DEFAULT_CONTEXT_LIMIT,
            })
            if mode == "deep" or deep_context_enabled():
                try:
                    deep_query = urllib.parse.urlencode({
                        "objective": prompt[:500],
                        "cwd": cwd,
                        "budget_tokens": DEEP_CONTEXT_BUDGET,
                        "limit": DEEP_CONTEXT_LIMIT,
                    })
                    return summarize_context_vm(get_json("/context/inject?" + deep_query))
                except Exception:
                    pass
            if mode == "readable":
                return summarize_context_vm(get_json("/context/inject?" + query))
            if mode == "quetta" or truthy_env("QORX_CODEX_QUETTA"):
                return summarize_context_nano(get_json("/context/quetta?" + query))
            return summarize_context_nano(get_json("/context/nano?" + query))
        except Exception:
            pass
        try:
            return summarize_context_nano(get_json("/context/nano?" + query))
        except Exception:
            pass
        try:
            return summarize_context_vm(get_json("/context/inject?" + query))
        except Exception:
            pass
    try:
        return summarize_capsule(get_json("/capsule/session"))
    except Exception:
        try:
            return summarize_session(get_json("/session"))
        except Exception as exc:
            return offline_context(exc, cwd, prompt)


def handle_user_prompt_submit(payload):
    mode = saved_context_mode()
    if not hook_enabled() or mode == "off":
        return {}
    context = build_additional_context(payload.get("cwd"), payload.get("prompt", ""), mode)
    if not context:
        return {}
    return hook_payload(context)


def read_payload():
    if sys.stdin.isatty():
        return {}
    text = sys.stdin.read()
    if not text.strip():
        return {}
    return json.loads(text)


def main():
    try:
        response = handle_user_prompt_submit(read_payload())
    except Exception:
        response = {}
    emit(response)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
"#
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeSet,
        fs,
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_app_paths(prefix: &str) -> crate::config::AppPaths {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let data_dir = std::env::temp_dir().join(format!("{prefix}-{nonce}"));
        crate::config::AppPaths {
            data_dir: data_dir.clone(),
            portable: false,
            stats_file: data_dir.join("stats.pb"),
            atom_file: data_dir.join("quarks.pb"),
            index_file: data_dir.join("repo_index.pb"),
            context_protobuf_file: data_dir.join("qorx-context.pb"),
            response_cache_file: data_dir.join("response_cache.pb"),
            integration_report_file: data_dir.join("integrations.pb"),
            provenance_file: data_dir.join("qorx-provenance.pb"),
            security_keys_file: data_dir.join("qorx-security-keys.pb"),
            shim_dir: data_dir.join("shims"),
        }
    }

    #[test]
    fn integration_settings_default_to_operator_opt_in() {
        let settings = super::IntegrationSettings::default();

        assert!(!settings.automcp_enabled);
        assert!(!settings.autohook_enabled);
        assert_eq!(settings.codex_context_mode, super::CODEX_CONTEXT_MODE_OFF);
        assert!(!settings.any_enabled());

        let enabled = super::IntegrationSettings::all_enabled();
        assert!(enabled.automcp_enabled);
        assert!(enabled.autohook_enabled);
        assert_eq!(enabled.codex_context_mode, super::CODEX_CONTEXT_MODE_AUTO);
        assert!(enabled.any_enabled());
    }

    #[test]
    fn missing_integration_settings_default_to_first_run_enabled() {
        let paths = temp_app_paths("qorx-integration-default-on");

        let settings = super::load_settings(&paths).unwrap();

        assert!(settings.automcp_enabled);
        assert!(settings.autohook_enabled);
        assert_eq!(settings.codex_context_mode, super::CODEX_CONTEXT_MODE_AUTO);
        assert!(settings.any_enabled());
        assert!(!paths.data_dir.join("integration-settings.pb").exists());

        let _ = fs::remove_dir_all(paths.data_dir);
    }

    #[test]
    fn saved_disabled_integration_settings_stay_disabled() {
        let paths = temp_app_paths("qorx-integration-disabled");
        let disabled = super::IntegrationSettings::default();

        super::save_settings(&paths, &disabled).unwrap();
        let settings = super::load_settings(&paths).unwrap();

        assert!(!settings.automcp_enabled);
        assert!(!settings.autohook_enabled);
        assert_eq!(settings.codex_context_mode, super::CODEX_CONTEXT_MODE_OFF);
        assert!(!settings.any_enabled());

        let _ = fs::remove_dir_all(paths.data_dir);
    }

    #[test]
    fn removing_hook_entries_keeps_non_qorx_hooks() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("qorx-hook-removal-{nonce}.json"));
        fs::write(
            &path,
            serde_json::to_string_pretty(&serde_json::json!({
                "hooks": {
                    "UserPromptSubmit": [
                        {"hooks": [{"command": "py -3 qorx_user_prompt_submit.py"}]},
                        {"hooks": [{"command": "other-hook"}]}
                    ]
                }
            }))
            .unwrap(),
        )
        .unwrap();

        super::remove_json_hook_entries(&path, "UserPromptSubmit").unwrap();

        let text = fs::read_to_string(&path).unwrap();
        assert!(!text.contains("qorx_user_prompt_submit.py"));
        assert!(text.contains("other-hook"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn codex_hook_json_uses_current_nested_hooks_shape() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("qorx-codex-hook-install-{nonce}.json"));
        fs::write(
            &path,
            serde_json::to_string_pretty(&serde_json::json!({
                "hooks": {
                    "UserPromptSubmit": [
                        {"hooks": [{"command": "py -3 old_qorx_user_prompt_submit.py"}]},
                        {"hooks": [{"command": "other-hook"}]}
                    ]
                }
            }))
            .unwrap(),
        )
        .unwrap();

        super::install_codex_command_hook_json(
            &path,
            "UserPromptSubmit",
            r#"py -3 "C:\Users\Marvin\.codex\hooks\qorx_user_prompt_submit.py""#,
            "Loading Qorx context",
        )
        .unwrap();

        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert!(value.get("UserPromptSubmit").is_none());
        let entries = value["hooks"]["UserPromptSubmit"]
            .as_array()
            .expect("nested hooks event");
        assert_eq!(entries.len(), 2);
        assert!(serde_json::to_string(&entries[0])
            .unwrap()
            .contains("other-hook"));
        assert!(serde_json::to_string(&entries[1])
            .unwrap()
            .contains("qorx_user_prompt_submit.py"));
        assert_eq!(entries[1]["hooks"][0]["timeout"], serde_json::json!(8));
        assert!(entries[1].get("matcher").is_none());
        assert!(super::codex_hook_config_active(
            &fs::read_to_string(&path).unwrap()
        ));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn codex_hook_status_accepts_current_nested_event_shape() {
        let text = serde_json::to_string_pretty(&serde_json::json!({
            "hooks": {
                "UserPromptSubmit": [
                    {"hooks": [{"command": "py -3 qorx_user_prompt_submit.py"}]}
                ]
            }
        }))
        .unwrap();

        assert!(super::codex_hook_config_active(&text));
    }

    #[test]
    fn codex_hook_status_rejects_legacy_top_level_event_shape() {
        let text = serde_json::to_string_pretty(&serde_json::json!({
            "UserPromptSubmit": [
                {"hooks": [{"command": "py -3 qorx_user_prompt_submit.py"}]}
            ]
        }))
        .unwrap();

        assert!(!super::codex_hook_config_active(&text));
    }

    #[test]
    fn codex_hook_hash_matches_codex_current_hash_identity() {
        let hash = super::codex_command_hook_hash(
            super::CODEX_USER_PROMPT_EVENT_KEY,
            r#"py -3 "C:\Users\Marvin\.codex\hooks\qorx_user_prompt_submit.py""#,
            super::CODEX_HOOK_TIMEOUT_SECONDS,
            "Loading Qorx context",
        );

        assert_eq!(
            hash,
            "sha256:7f9d505bb8056d7ffb373fa1f3928c4c5ea42ca5ad327f3c8af7edc75e81868e"
        );
    }

    #[test]
    fn managed_rule_blocks_can_be_replaced_and_removed() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("qorx-rule-block-{nonce}.md"));
        fs::write(&path, "Keep this\n").unwrap();

        super::upsert_managed_block(
            &path,
            super::ANTIGRAVITY_CONTEXT_RULE_START,
            super::ANTIGRAVITY_CONTEXT_RULE_END,
            &super::antigravity_context_rule_block(),
        )
        .unwrap();
        super::upsert_managed_block(
            &path,
            super::ANTIGRAVITY_CONTEXT_RULE_START,
            super::ANTIGRAVITY_CONTEXT_RULE_END,
            &super::antigravity_context_rule_block(),
        )
        .unwrap();

        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("Keep this"));
        assert_eq!(
            text.matches(super::ANTIGRAVITY_CONTEXT_RULE_START).count(),
            1
        );
        assert!(text.contains("Qorx Void Context"));
        assert!(text.contains("QORX_VOID_ANTIGRAVITY_CONTEXT_START"));
        assert!(text.contains("qorx.context_inject"));
        assert!(text.contains("Do not spawn background shells"));

        super::remove_managed_block(
            &path,
            super::ANTIGRAVITY_CONTEXT_RULE_START,
            super::ANTIGRAVITY_CONTEXT_RULE_END,
        )
        .unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("Keep this"));
        assert!(!text.contains("Qorx Void Context"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn mcp_server_json_uses_real_qorx_mcp_subcommand() {
        let server = super::mcp_server_json_with_cwd(
            Path::new(r"C:\Qorx Void\qorx.exe"),
            Some(Path::new(r"C:\repo")),
        );

        assert_eq!(server["command"], r"C:\Qorx Void\qorx.exe");
        assert_eq!(server["args"], serde_json::json!(["mcp"]));
        assert_eq!(server["cwd"], r"C:\repo");
    }

    #[test]
    fn antigravity_mcp_install_writes_primary_and_legacy_configs() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("qorx-antigravity-mcp-{nonce}"));
        let primary = dir.join("Antigravity").join("User").join("mcp.json");
        let legacy = dir
            .join(".gemini")
            .join("antigravity")
            .join("mcp_config.json");
        let exe = Path::new(r"C:\Qorx Void\qorx.exe");

        super::install_antigravity_mcp_configs(
            &[primary.clone(), legacy.clone()],
            exe,
            Some(Path::new(r"C:\repo")),
        )
        .unwrap();

        assert!(super::mcp_config_active(
            &primary,
            "mcpServers",
            "qorx",
            exe
        ));
        assert!(super::mcp_config_active(&legacy, "mcpServers", "qorx", exe));
        assert!(super::antigravity_mcp_config_active_in_paths(
            &[primary.clone(), legacy.clone()],
            exe
        ));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn integration_checkpoint_writes_restore_manifest_and_script() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("qorx-checkpoint-{nonce}"));
        let target = dir.join("configs").join("mcp.json");
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, "{\"mcpServers\":{}}\n").unwrap();

        let report = super::write_checkpoint_at(
            &dir.join("checkpoints"),
            "test",
            &[
                super::CheckpointTarget::new("test-mcp", target.clone()),
                super::CheckpointTarget::new("duplicate-mcp", target.clone()),
            ],
            None,
        )
        .unwrap();

        let checkpoint = Path::new(&report.path);
        assert!(checkpoint.join("manifest.json").exists());
        assert!(checkpoint.join("restore.ps1").exists());
        assert_eq!(
            report
                .files
                .iter()
                .filter(|file| file.source == target.display().to_string())
                .count(),
            1
        );
        assert!(fs::read_to_string(checkpoint.join("manifest.json"))
            .unwrap()
            .contains("test-mcp"));
        assert!(fs::read_to_string(checkpoint.join("restore.ps1"))
            .unwrap()
            .contains("Copy-Item"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn qorx_codex_shim_keeps_codex_direct_and_memories_enabled_by_default() {
        let body = super::qorx_codex_shim_body(Path::new(r"C:\qorx\qorx.exe"));
        let script = super::qorx_codex_shim_script_body(Path::new(r"C:\qorx\qorx.exe"));

        assert!(!body.contains("set QORX_CODEX_PROXY="));
        assert!(!body.contains("set QORX_CODEX_DISABLE_MEMORIES="));
        assert!(!body.contains("set QORX_CODEX_RAW_MEMORIES=1"));
        assert!(body.contains("qorx-codex.ps1"));
        assert!(script.contains("$env:QORX_GATEWAY = \"http://127.0.0.1:47187\""));
        assert!(script.contains("QORX_CODEX_CONTEXT_OFF"));
        assert!(script.contains("QORX_CODEX_BIN"));
        assert!(script.contains("qorx_user_prompt_submit.py"));
        assert!(script.contains("$wrappedPrompt"));
        assert!(script.contains("& $codexCommand @execArgs"));
        assert!(!body.contains("openai_base_url"));
        assert!(!body.contains("--disable memories"));
        assert!(!script.contains("openai_base_url"));
        assert!(!script.contains("--disable memories"));
        assert!(!body.contains(" run codex -- %*"));
    }

    #[test]
    fn codex_hook_script_emits_qorx_session_additional_context() {
        let script = super::codex_hook_script();

        assert!(script.contains("/capsule/session"));
        assert!(script.contains("/integrations/settings"));
        assert!(script.contains("/context/quetta"));
        assert!(script.contains("/context/nano"));
        assert!(script.contains("/context/inject"));
        assert!(script.contains("/session"));
        assert!(script.contains("QORX_CODEX_CONTEXT_MODE"));
        assert!(script.contains("urllib.parse"));
        assert!(script.contains("QORX_CODEX_CONTEXT_OFF"));
        assert!(script.contains("return not truthy_env(\"QORX_CODEX_CONTEXT_OFF\")"));
        assert!(!script.contains("QORX_CODEX_HOOK\") or truthy_env(\"QORX_CODEX_CONTEXT_ALWAYS"));
        assert!(script.contains("hookSpecificOutput"));
        assert!(script.contains("additionalContext"));
        assert!(script.contains("Qorx is ready for this turn"));
        assert!(script.contains("/context/fault"));
        assert!(script.contains("Qorx Auto is on"));
        assert!(script.contains("Local context stays here"));
        assert!(script.contains("Qorx context is available"));
        assert!(script.contains("Normal chat comes first"));
        assert!(script.contains("Capsule handle"));
        assert!(script.contains("qosm + qshf"));
        assert!(!script.contains(&["KOR", "TEX"].concat()));
    }

    #[test]
    fn platform_slug_matrix_matches_qorx_automcp_autohook_install_surface() {
        for slug in [
            "all",
            "windows",
            "codex",
            "claude",
            "claude-code",
            "opencode",
            "copilot",
            "vscode",
            "vscode-copilot",
            "aider",
            "claw",
            "openclaw",
            "droid",
            "factory-droid",
            "trae",
            "trae-cn",
            "gemini",
            "hermes",
            "kiro",
            "pi",
            "cursor",
            "antigravity",
            "google-antigravity",
        ] {
            assert!(
                super::IntegrationPlatform::from_slug(slug).is_some(),
                "{slug} should parse"
            );
        }
    }

    #[test]
    fn integration_statuses_cover_agent_platform_matrix() {
        let statuses = super::integration_statuses(Path::new("qorx.exe"));
        let names: BTreeSet<String> = statuses.into_iter().map(|status| status.name).collect();

        for expected in [
            "Codex",
            "Claude Code",
            "Gemini CLI",
            "Google Antigravity",
            "OpenCode",
            "GitHub Copilot CLI",
            "VS Code Copilot Chat",
            "Aider",
            "OpenClaw",
            "Factory Droid",
            "Trae",
            "Trae CN",
            "Hermes",
            "Kiro IDE/CLI",
            "Pi coding agent",
            "Cursor",
        ] {
            assert!(names.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn every_agent_platform_has_a_capability_record() {
        let slugs = super::IntegrationPlatform::agent_platforms()
            .iter()
            .map(|platform| platform.slug())
            .collect::<Vec<_>>();
        let capabilities = super::platform_capabilities();
        for slug in slugs {
            let capability = capabilities
                .iter()
                .find(|capability| capability.platform == slug)
                .unwrap_or_else(|| panic!("missing capability record for {slug}"));
            assert!(capability.supports_mcp, "{slug} should expose MCP wiring");
            assert!(
                capability.supports_hooks
                    || capability.hook_mode == "manual-kit"
                    || capability.hook_mode == "mcp-only"
                    || capability.hook_mode == "mcp-pull-only",
                "{slug} should either support hooks directly or declare its hook boundary"
            );
        }
    }

    #[test]
    fn integration_statuses_expose_mcp_and_hook_state_separately() {
        let statuses = super::integration_statuses(Path::new(r"C:\Qorx\qorx.exe"));

        let codex = statuses
            .iter()
            .find(|status| status.platform == "codex")
            .expect("codex status");
        assert!(codex.supports_mcp);
        assert!(codex.supports_hooks);
        assert_eq!(codex.hook_mode, "managed");

        let antigravity = statuses
            .iter()
            .find(|status| status.platform == "antigravity")
            .expect("antigravity status");
        assert!(antigravity.supports_mcp);
        assert!(!antigravity.supports_hooks);
        assert_eq!(antigravity.hook_mode, "mcp-pull-only");
        assert!(antigravity.mechanism.contains("pull-only context tools"));
    }

    #[test]
    fn codex_hook_defaults_to_compact_context_with_deep_context_opt_in() {
        let script = super::codex_hook_script();
        let inject_pos = script.find("/context/inject").expect("inject endpoint");
        let quetta_pos = script.find("/context/quetta").expect("quetta endpoint");
        let nano_pos = script.find("/context/nano").expect("nano endpoint");

        assert!(inject_pos < quetta_pos);
        assert!(inject_pos < nano_pos);
        assert!(script.contains("QORX_CODEX_VERBOSE_CONTEXT"));
        assert!(script.contains("QORX_CODEX_DEEP_CONTEXT"));
        assert!(script.contains("QORX_CODEX_QUETTA"));
        assert!(script.contains("saved_context_mode"));
        assert!(script.contains("normalize_context_mode"));
        assert!(script.contains("DEFAULT_CONTEXT_BUDGET"));
        assert!(script.contains("\"180\""));
        assert!(!script.contains("QORX_CODEX_ULTRA_COMPACT"));
    }

    #[test]
    fn integration_statuses_do_not_duplicate_targets() {
        let statuses = super::integration_statuses(Path::new("qorx.exe"));
        let mut names = BTreeSet::new();

        for status in statuses {
            assert!(
                names.insert(status.name.clone()),
                "duplicate {}",
                status.name
            );
        }
    }

    #[test]
    fn detects_qorx_tray_command_line_for_same_exe_only() {
        let exe = Path::new(r"C:\Qorx\target\release\qorx.exe");

        assert!(super::is_tray_command_line(
            r#""C:\Qorx\target\release\qorx.exe" tray"#,
            exe
        ));
        assert!(!super::is_tray_command_line(
            r#""C:\Qorx\target\release\qorx.exe" daemon"#,
            exe
        ));
        assert!(!super::is_tray_command_line(
            r#""C:\other\qorx.exe" tray"#,
            exe
        ));
    }

    #[test]
    fn detects_qorx_daemon_command_line_for_same_exe_only() {
        let exe = Path::new(r"C:\Qorx\target\release\qorx.exe");

        assert!(super::is_daemon_command_line(
            r#""C:\Qorx\target\release\qorx.exe" daemon"#,
            exe
        ));
        assert!(super::is_daemon_command_line(
            r#""C:\Qorx\target\release\qorx.exe" daemon run"#,
            exe
        ));
        assert!(!super::is_daemon_command_line(
            r#""C:\Qorx\target\release\qorx.exe" daemon stop"#,
            exe
        ));
        assert!(!super::is_daemon_command_line(
            r#""C:\Qorx\target\release\qorx.exe" tray"#,
            exe
        ));
        assert!(!super::is_daemon_command_line(
            r#""C:\other\qorx.exe" daemon"#,
            exe
        ));
    }

    #[test]
    fn autostart_script_launches_daemon_and_tray() {
        let script = super::autostart_script(Path::new(r"C:\Qorx\qorx.exe")).unwrap();

        assert!(script.contains(r#""C:\Qorx\qorx.exe"" daemon start"#));
        assert!(script.contains(r#""C:\Qorx\qorx.exe"" tray"#));
    }
}
