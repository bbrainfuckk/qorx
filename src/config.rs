use std::{
    env, fs,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

pub const APP_NAME: &str = "Qorx";
pub const DEFAULT_BIND: &str = "127.0.0.1:47187";
pub const LOCAL_BASE: &str = "http://127.0.0.1:47187";
pub const PORTABLE_MARKER: &str = "qorx.portable";
pub const PORTABLE_DATA_DIR: &str = "qorx-data";
pub const PORTABLE_EXE_MAX_BYTES: u64 = 5 * 1024 * 1024;
pub const DRIVE_HOME_CONFIG: &str = "qorx-drive.pb";
const LEGACY_DRIVE_HOME_CONFIG: &str = "qorx-drive.json";

pub fn runtime_bind() -> String {
    env::var("QORX_BIND").unwrap_or_else(|_| DEFAULT_BIND.to_string())
}

pub fn local_base() -> String {
    if let Ok(value) = env::var("QORX_GATEWAY") {
        let trimmed = value.trim().trim_end_matches('/');
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    gateway_base_from_bind(&runtime_bind()).unwrap_or_else(|| LOCAL_BASE.to_string())
}

pub fn gateway_base_from_bind(bind: &str) -> Option<String> {
    let addr = bind.parse::<SocketAddr>().ok()?;
    let host = if addr.ip().is_unspecified() {
        "127.0.0.1".to_string()
    } else if addr.is_ipv6() {
        format!("[{}]", addr.ip())
    } else {
        addr.ip().to_string()
    };
    Some(format!("http://{host}:{}", addr.port()))
}

#[derive(Clone, Debug)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub portable: bool,
    pub stats_file: PathBuf,
    pub atom_file: PathBuf,
    pub index_file: PathBuf,
    pub context_protobuf_file: PathBuf,
    pub response_cache_file: PathBuf,
    pub integration_report_file: PathBuf,
    pub provenance_file: PathBuf,
    pub security_keys_file: PathBuf,
    pub shim_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveHomeConfig {
    pub backend: String,
    pub letter: String,
    pub size: String,
    pub home_dir: String,
    pub backing_dir: String,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    fn qorx_home_wins_over_portable_marker() {
        let selected = super::choose_data_dir(
            PathBuf::from(r"C:\portable"),
            Some(PathBuf::from(r"D:\QorxHome")),
            true,
            true,
            PathBuf::from(r"C:\Users\Example\AppData\Local\qorx"),
        );

        assert_eq!(selected, PathBuf::from(r"D:\QorxHome"));
    }

    #[test]
    fn qorx_home_wins_over_saved_drive_home() {
        let drive_home = super::DriveHomeConfig {
            backend: "imdisk-vm".to_string(),
            letter: "Q".to_string(),
            size: "512M".to_string(),
            home_dir: r"Q:\qorx-data".to_string(),
            backing_dir: r"C:\Users\Example\AppData\Local\qorx".to_string(),
        };
        let selected = super::select_data_dir(
            PathBuf::from(r"C:\portable"),
            Some(PathBuf::from(r"D:\QorxHome")),
            Some(&drive_home),
            false,
            false,
            PathBuf::from(r"C:\Users\Example\AppData\Local\qorx"),
        );

        assert_eq!(selected, PathBuf::from(r"D:\QorxHome"));
    }

    #[test]
    fn portable_marker_keeps_data_next_to_exe() {
        let selected = super::choose_data_dir(
            PathBuf::from(r"C:\Tools\Qorx"),
            None,
            false,
            true,
            PathBuf::from(r"C:\Users\Example\AppData\Local\qorx"),
        );

        assert_eq!(selected, PathBuf::from(r"C:\Tools\Qorx\qorx-data"));
    }

    #[test]
    fn normal_mode_uses_platform_data_dir() {
        let selected = super::choose_data_dir(
            PathBuf::from(r"C:\Tools\Qorx"),
            None,
            false,
            false,
            PathBuf::from(r"C:\Users\Example\AppData\Local\qorx"),
        );

        assert_eq!(
            selected,
            PathBuf::from(r"C:\Users\Example\AppData\Local\qorx")
        );
    }

    #[test]
    fn gateway_base_uses_runtime_bind_without_stale_static_ports() {
        assert_eq!(
            super::gateway_base_from_bind("127.0.0.1:8765").unwrap(),
            "http://127.0.0.1:8765"
        );
        assert_eq!(
            super::gateway_base_from_bind("0.0.0.0:47187").unwrap(),
            "http://127.0.0.1:47187"
        );
        assert_eq!(
            super::gateway_base_from_bind("[::1]:47187").unwrap(),
            "http://[::1]:47187"
        );
    }
}

impl AppPaths {
    pub fn resolve() -> Result<Self> {
        resolve_paths(true)
    }

    pub fn resolve_for_drive() -> Result<Self> {
        resolve_paths(false)
    }
}

fn resolve_paths(create_dirs: bool) -> Result<AppPaths> {
    let normal_data_dir = ProjectDirs::from("ai", "qorx", APP_NAME)
        .ok_or_else(|| anyhow!("could not resolve local app data directory"))?
        .data_local_dir()
        .to_path_buf();
    let exe_dir = current_exe_dir()?;
    let qorx_home = env::var_os("QORX_HOME").map(PathBuf::from);
    let portable_env = truthy_env("QORX_PORTABLE");
    let marker_exists = exe_dir.join(PORTABLE_MARKER).exists();
    let drive_home = load_drive_home_config()?;
    let portable = qorx_home.is_none() && (portable_env || marker_exists || drive_home.is_some());
    let data_dir = select_data_dir(
        exe_dir,
        qorx_home,
        drive_home.as_ref(),
        portable_env,
        marker_exists,
        normal_data_dir,
    );
    if create_dirs {
        fs::create_dir_all(&data_dir)?;
    }
    let shim_dir = data_dir.join("shims");
    if create_dirs {
        fs::create_dir_all(&shim_dir)?;
    }
    Ok(AppPaths {
        data_dir: data_dir.clone(),
        portable,
        stats_file: data_dir.join("stats.pb"),
        atom_file: data_dir.join("quarks.pb"),
        index_file: data_dir.join("repo_index.pb"),
        context_protobuf_file: data_dir.join("qorx-context.pb"),
        response_cache_file: data_dir.join("response_cache.pb"),
        integration_report_file: data_dir.join("integrations.pb"),
        provenance_file: data_dir.join("qorx-provenance.pb"),
        security_keys_file: data_dir.join("qorx-security-keys.pb"),
        shim_dir,
    })
}

fn select_drive_home(config: &DriveHomeConfig) -> PathBuf {
    let home_dir = PathBuf::from(&config.home_dir);
    if drive_root_from_path(&home_dir).exists() {
        home_dir
    } else {
        PathBuf::from(&config.backing_dir)
    }
}

fn select_data_dir(
    exe_dir: PathBuf,
    qorx_home: Option<PathBuf>,
    drive_home: Option<&DriveHomeConfig>,
    portable_env: bool,
    marker_exists: bool,
    normal_data_dir: PathBuf,
) -> PathBuf {
    if let Some(path) = qorx_home {
        return path;
    }
    if let Some(drive_home) = drive_home {
        return select_drive_home(drive_home);
    }
    choose_data_dir(exe_dir, None, portable_env, marker_exists, normal_data_dir)
}

pub fn load_drive_home_config() -> Result<Option<DriveHomeConfig>> {
    let path = current_exe_dir()?.join(DRIVE_HOME_CONFIG);
    let legacy_path = current_exe_dir()?.join(LEGACY_DRIVE_HOME_CONFIG);
    if !path.exists() && !legacy_path.exists() {
        return Ok(None);
    }
    Ok(Some(crate::proto_store::load_required(
        &path,
        &[legacy_path.as_path()],
    )?))
}

pub fn save_drive_home_config(config: &DriveHomeConfig) -> Result<()> {
    let exe_dir = current_exe_dir()?;
    fs::create_dir_all(&exe_dir)?;
    crate::proto_store::save(&exe_dir.join(DRIVE_HOME_CONFIG), config)?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortableReport {
    pub portable: bool,
    pub exe_path: String,
    pub exe_size_bytes: u64,
    pub max_portable_exe_bytes: u64,
    pub exe_within_size_target: bool,
    pub data_dir: String,
    pub marker: String,
    pub marker_exists: bool,
    pub env_home: Option<String>,
    pub env_portable: bool,
    pub q_drive_hint: String,
    pub boundary: String,
}

pub fn portable_report(paths: &AppPaths) -> Result<PortableReport> {
    let exe_dir = current_exe_dir()?;
    let exe_path = env::current_exe()?;
    let exe_size_bytes = fs::metadata(&exe_path).map(|meta| meta.len()).unwrap_or(0);
    let marker = exe_dir.join(PORTABLE_MARKER);
    Ok(PortableReport {
        portable: paths.portable,
        exe_path: exe_path.display().to_string(),
        exe_size_bytes,
        max_portable_exe_bytes: PORTABLE_EXE_MAX_BYTES,
        exe_within_size_target: exe_size_bytes > 0 && exe_size_bytes <= PORTABLE_EXE_MAX_BYTES,
        data_dir: paths.data_dir.display().to_string(),
        marker: marker.display().to_string(),
        marker_exists: marker.exists(),
        env_home: env::var("QORX_HOME").ok(),
        env_portable: truthy_env("QORX_PORTABLE"),
        q_drive_hint: format!(
            "Windows optional persistent drive letter: qorx drive init --letter Q maps \"{}\"",
            paths.data_dir.display()
        ),
        boundary: "The portable exe contains the Qorx controller, proxy, AIM reader, cache, indexer, and provenance logic. A true RAM-backed Q: drive still requires a separate Windows RAM-disk runtime such as ImDisk today or a future signed qorxram.sys driver; the exe will not fake RAM with subst.".to_string(),
    })
}

pub fn init_portable() -> Result<PortableReport> {
    let exe_dir = current_exe_dir()?;
    fs::create_dir_all(&exe_dir)?;
    fs::write(
        exe_dir.join(PORTABLE_MARKER),
        "Qorx portable mode: keep index, quarks, cache, stats, and shims beside qorx.exe.\n",
    )?;
    let data_dir = exe_dir.join(PORTABLE_DATA_DIR);
    fs::create_dir_all(&data_dir)?;
    let shim_dir = data_dir.join("shims");
    fs::create_dir_all(&shim_dir)?;
    let paths = AppPaths {
        data_dir: data_dir.clone(),
        portable: true,
        stats_file: data_dir.join("stats.pb"),
        atom_file: data_dir.join("quarks.pb"),
        index_file: data_dir.join("repo_index.pb"),
        context_protobuf_file: data_dir.join("qorx-context.pb"),
        response_cache_file: data_dir.join("response_cache.pb"),
        integration_report_file: data_dir.join("integrations.pb"),
        provenance_file: data_dir.join("qorx-provenance.pb"),
        security_keys_file: data_dir.join("qorx-security-keys.pb"),
        shim_dir,
    };
    portable_report(&paths)
}

fn current_exe_dir() -> Result<PathBuf> {
    env::current_exe()?
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow!("could not resolve qorx executable directory"))
}

fn drive_root_from_path(path: &Path) -> PathBuf {
    let text = path.display().to_string();
    if text.len() >= 2 && text.as_bytes()[1] == b':' {
        PathBuf::from(format!("{}\\", &text[..2]))
    } else {
        path.to_path_buf()
    }
}

fn truthy_env(key: &str) -> bool {
    env::var(key)
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on" | "portable"
            )
        })
        .unwrap_or(false)
}

fn choose_data_dir(
    exe_dir: PathBuf,
    qorx_home: Option<PathBuf>,
    portable_env: bool,
    marker_exists: bool,
    normal_data_dir: PathBuf,
) -> PathBuf {
    if let Some(path) = qorx_home {
        return path;
    }
    if portable_env || marker_exists {
        return exe_dir.join(PORTABLE_DATA_DIR);
    }
    normal_data_dir
}

#[derive(Clone, Debug)]
pub struct ProxyConfig {
    pub bind: String,
    pub openai_upstream: String,
    pub anthropic_upstream: String,
    pub gemini_upstream: String,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            bind: env::var("QORX_BIND").unwrap_or_else(|_| DEFAULT_BIND.to_string()),
            openai_upstream: env::var("QORX_OPENAI_UPSTREAM")
                .unwrap_or_else(|_| "https://api.openai.com".to_string()),
            anthropic_upstream: env::var("QORX_ANTHROPIC_UPSTREAM")
                .unwrap_or_else(|_| "https://api.anthropic.com".to_string()),
            gemini_upstream: env::var("QORX_GEMINI_UPSTREAM")
                .unwrap_or_else(|_| "https://generativelanguage.googleapis.com".to_string()),
        }
    }
}
