use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::config::{
    load_drive_home_config, save_drive_home_config, AppPaths, DriveHomeConfig, PORTABLE_DATA_DIR,
};

pub const DEFAULT_IMDISK_SIZE: &str = "2G";

#[derive(Debug, Clone)]
pub struct DriveOptions {
    pub ram: bool,
    pub size: String,
}

impl DriveOptions {
    pub fn subst() -> Self {
        Self {
            ram: false,
            size: DEFAULT_IMDISK_SIZE.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveReport {
    pub letter: String,
    pub target: String,
    pub mounted: bool,
    pub mapped_to: Option<String>,
    pub persistent: bool,
    pub startup_file: String,
    pub mode: String,
    pub backend: String,
    pub ram_requested: bool,
    pub ram_available: bool,
    pub ram_driver: RamDriverReport,
    pub imdisk: ImDiskReport,
    pub home_on_drive: bool,
    pub home_backing: Option<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamDriverReport {
    pub service_name: String,
    pub installed: bool,
    pub running: bool,
    pub can_mount: bool,
    pub expected_driver_file: String,
    pub requirement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImDiskReport {
    pub service_name: String,
    pub installed: bool,
    pub running: bool,
    pub can_mount: bool,
    pub cli_path: Option<String>,
    pub requirement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImDiskInstallReport {
    pub bundle: String,
    pub install_script: String,
    pub launched: bool,
    pub note: String,
}

pub fn init(paths: &AppPaths, letter: &str, options: &DriveOptions) -> Result<DriveReport> {
    mount(paths, letter, options)?;
    install_startup(paths, letter, options)
}

pub fn mount(paths: &AppPaths, letter: &str, options: &DriveOptions) -> Result<DriveReport> {
    let letter = normalize_letter(letter)?;
    if options.ram {
        return mount_imdisk(paths, &letter, &options.size);
    }

    mount_subst(paths, &letter)
}

pub fn unmount(paths: &AppPaths, letter: &str) -> Result<DriveReport> {
    let letter = normalize_letter(letter)?;

    if current_imdisk_mount(&letter)? {
        sync_drive_home_to_backing(&letter)?;
        let detach_status =
            Command::new(imdisk_cli_path().ok_or_else(|| anyhow!("ImDisk is not installed"))?)
                .args(["-D", "-m", &letter])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()?;
        if !detach_status.success() {
            return Err(anyhow!("failed to unmount {letter} with ImDisk"));
        }
        return status(paths, &letter, false);
    }

    if current_subst_mapping(&letter)?.is_some() {
        let status = Command::new("cmd")
            .args(subst_unmount_args(&letter))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        if !status.success() {
            return Err(anyhow!("failed to unmount {letter}"));
        }
    }
    status(paths, &letter, false)
}

pub fn install_startup(
    paths: &AppPaths,
    letter: &str,
    options: &DriveOptions,
) -> Result<DriveReport> {
    let letter = normalize_letter(letter)?;
    if options.ram {
        ensure_imdisk_backend()?;
    }
    let startup = startup_file(&letter)?;
    if let Some(parent) = startup.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&startup, startup_script(&current_exe()?, &letter, options))?;
    status(paths, &letter, options.ram)
}

pub fn remove_startup(paths: &AppPaths, letter: &str) -> Result<DriveReport> {
    let letter = normalize_letter(letter)?;
    let startup = startup_file(&letter)?;
    if startup.exists() {
        fs::remove_file(&startup)?;
    }
    status(paths, &letter, false)
}

pub fn status(paths: &AppPaths, letter: &str, ram_requested: bool) -> Result<DriveReport> {
    let letter = normalize_letter(letter)?;
    let mapped_subst = current_subst_mapping(&letter)?;
    let imdisk_mounted = current_imdisk_mount(&letter)?;
    let startup = startup_file(&letter)?;
    let persistent = startup.exists() || tray_startup_mounts_drive(&letter);
    let ram_driver = native_ram_driver_report();
    let imdisk = imdisk_report();
    let drive_home = load_drive_home_config()?;
    let ram_target = ram_disk_data_dir(&letter);
    let home_config = drive_home
        .as_ref()
        .filter(|config| normalize_letter(&config.letter).ok().as_deref() == Some(letter.as_str()));
    let home_on_drive = home_config.is_some() && imdisk_mounted;
    let home_backing = home_config.map(|config| config.backing_dir.clone());
    let mapped_to = if imdisk_mounted {
        Some(ram_target.display().to_string())
    } else {
        mapped_subst.as_ref().map(|path| path.display().to_string())
    };
    let target = home_config
        .map(|config| config.home_dir.clone())
        .unwrap_or_else(|| paths.data_dir.display().to_string());
    let mounted = imdisk_mounted || mapped_subst.is_some();
    let backend = if imdisk_mounted {
        "imdisk-vm"
    } else if mapped_subst.is_some() {
        "subst"
    } else {
        "unmounted"
    }
    .to_string();
    let mode = if imdisk_mounted {
        "ram".to_string()
    } else if mapped_subst.is_some() {
        "subst".to_string()
    } else if ram_requested {
        if imdisk.can_mount {
            "ram-ready".to_string()
        } else {
            "ram-unavailable".to_string()
        }
    } else {
        "unmounted".to_string()
    };
    let note = if imdisk_mounted {
        "Qorx drive letter is mounted on an ImDisk RAM disk; provider context still uses qorx://s session pointers".to_string()
    } else if ram_requested && !imdisk.can_mount && !ram_driver.can_mount {
        "RAM-backed Q: requires either a working ImDisk install or a signed qorxram.sys Windows kernel driver".to_string()
    } else if mapped_subst.is_some() {
        "Qorx drive letter is mounted with subst; provider context still uses qorx://s session pointers".to_string()
    } else {
        "Qorx drive letter is not mounted; run qorx drive init --letter Q or qorx drive init --letter Q --ram".to_string()
    };

    Ok(DriveReport {
        letter,
        target,
        mounted,
        mapped_to,
        persistent,
        startup_file: startup.display().to_string(),
        mode,
        backend,
        ram_requested,
        ram_available: imdisk.can_mount || ram_driver.can_mount,
        ram_driver,
        imdisk,
        home_on_drive,
        home_backing,
        note,
    })
}

pub fn install_imdisk(bundle: &Path) -> Result<ImDiskInstallReport> {
    let script = bundle.join("install.cmd");
    if !script.exists() {
        return Err(anyhow!(
            "ImDisk bundle is missing install.cmd at {}",
            script.display()
        ));
    }

    let escaped_bundle = bundle.display().to_string().replace('\'', "''");
    let escaped_script = script.display().to_string().replace('\'', "''");
    let ps = format!(
        "Start-Process -FilePath cmd.exe -WorkingDirectory '{}' -ArgumentList '/c','{}' -Verb RunAs -Wait",
        escaped_bundle, escaped_script
    );
    let status = Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if !status.success() {
        return Err(anyhow!(
            "ImDisk installer did not complete successfully; accept the UAC prompt and try again"
        ));
    }

    Ok(ImDiskInstallReport {
        bundle: bundle.display().to_string(),
        install_script: script.display().to_string(),
        launched: true,
        note: "ImDisk install command was launched elevated through the bundled install.cmd"
            .to_string(),
    })
}

fn mount_subst(paths: &AppPaths, letter: &str) -> Result<DriveReport> {
    fs::create_dir_all(&paths.data_dir)?;
    let target = paths
        .data_dir
        .canonicalize()
        .unwrap_or_else(|_| paths.data_dir.clone());
    if let Some(existing) = current_subst_mapping(letter)? {
        if same_path(&existing, &target) {
            return status(paths, letter, false);
        }
        return Err(anyhow!(
            "{letter} is already mapped to {}; refusing to overwrite it",
            existing.display()
        ));
    }
    if drive_root_exists(letter) {
        return Err(anyhow!(
            "{letter} already exists and is not a Qorx subst mapping; refusing to overwrite it"
        ));
    }

    let cmd_status = Command::new("cmd")
        .args(subst_mount_args(letter, &target))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if !cmd_status.success() {
        return Err(anyhow!("failed to mount {letter} with subst"));
    }
    status(paths, letter, false)
}

fn mount_imdisk(paths: &AppPaths, letter: &str, size: &str) -> Result<DriveReport> {
    ensure_imdisk_backend()?;
    if let Some(existing) = current_subst_mapping(letter)? {
        let status = Command::new("cmd")
            .args(subst_unmount_args(letter))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        if !status.success() {
            return Err(anyhow!(
                "failed to unmount existing subst mapping {letter} => {}",
                existing.display()
            ));
        }
    } else if drive_root_exists(letter) && !current_imdisk_mount(letter)? {
        return Err(anyhow!(
            "{letter} already exists and is not managed by Qorx or ImDisk; refusing to overwrite it"
        ));
    }

    let backing_dir = current_backing_dir(paths, letter);
    fs::create_dir_all(&backing_dir)?;
    if paths.data_dir.exists() && !same_path(&paths.data_dir, &backing_dir) {
        copy_dir_contents(&paths.data_dir, &backing_dir)?;
    }

    if !current_imdisk_mount(letter)? {
        let cli = imdisk_cli_path().ok_or_else(|| anyhow!("ImDisk is not installed"))?;
        let mount_args = imdisk_mount_args(letter, size);
        let output = Command::new(cli)
            .args(&mount_args)
            .stdin(Stdio::null())
            .output()?;
        if !output.status.success() {
            return Err(anyhow!(
                "failed to mount {letter} with ImDisk: {}",
                command_output_text(&output)
            ));
        }
        if wait_for_drive_root(letter).is_err() {
            let elevated = elevated_start_process_command(
                &imdisk_cli_path().ok_or_else(|| anyhow!("ImDisk is not installed"))?,
                &mount_args,
            );
            let status = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-WindowStyle",
                    "Hidden",
                    "-Command",
                    &elevated,
                ])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()?;
            if !status.success() {
                return Err(anyhow!(
                    "failed to mount {letter} with ImDisk; accept the UAC prompt and try again"
                ));
            }
            wait_for_drive_root(letter)?;
        }
    }

    let ram_home = ram_disk_data_dir(letter);
    fs::create_dir_all(&ram_home)?;
    copy_dir_contents(&backing_dir, &ram_home)?;
    save_drive_home_config(&DriveHomeConfig {
        backend: "imdisk".to_string(),
        letter: letter.to_string(),
        size: size.to_string(),
        home_dir: ram_home.display().to_string(),
        backing_dir: backing_dir.display().to_string(),
    })?;

    status(paths, letter, true)
}

fn current_backing_dir(paths: &AppPaths, letter: &str) -> PathBuf {
    load_drive_home_config()
        .ok()
        .flatten()
        .filter(|config| normalize_letter(&config.letter).ok().as_deref() == Some(letter))
        .map(|config| PathBuf::from(config.backing_dir))
        .unwrap_or_else(|| ram_backing_dir(&paths.data_dir))
}

fn normalize_letter(letter: &str) -> Result<String> {
    let trimmed = letter.trim().trim_end_matches('\\').trim_end_matches('/');
    let trimmed = trimmed.strip_suffix(':').unwrap_or(trimmed);
    let mut chars = trimmed.chars();
    let Some(ch) = chars.next() else {
        return Err(anyhow!("drive letter is empty"));
    };
    if chars.next().is_some() || !ch.is_ascii_alphabetic() {
        return Err(anyhow!("invalid drive letter: {letter}"));
    }
    Ok(format!("{}:", ch.to_ascii_uppercase()))
}

fn subst_mount_args(letter: &str, target: &Path) -> Vec<String> {
    vec![
        "/C".to_string(),
        "subst".to_string(),
        letter.to_string(),
        target.display().to_string(),
    ]
}

fn subst_unmount_args(letter: &str) -> Vec<String> {
    vec![
        "/C".to_string(),
        "subst".to_string(),
        letter.to_string(),
        "/D".to_string(),
    ]
}

fn imdisk_mount_args(letter: &str, size: &str) -> Vec<String> {
    vec![
        "-a".to_string(),
        "-t".to_string(),
        "vm".to_string(),
        "-s".to_string(),
        size.to_string(),
        "-m".to_string(),
        letter.to_string(),
        "-p".to_string(),
        "/fs:ntfs /q /y".to_string(),
    ]
}

fn current_subst_mapping(letter: &str) -> Result<Option<PathBuf>> {
    let output = Command::new("cmd")
        .args(["/C", "subst"])
        .stdin(Stdio::null())
        .output()?;
    let text = command_output_text(&output);
    Ok(parse_subst_mapping(&text, letter))
}

fn current_imdisk_mount(letter: &str) -> Result<bool> {
    let Some(cli) = imdisk_cli_path() else {
        return Ok(false);
    };
    let output = Command::new(cli)
        .args(["-l", "-m", letter])
        .stdin(Stdio::null())
        .output()?;
    let text = command_output_text(&output);
    let upper = text.to_ascii_uppercase();
    Ok(output.status.success()
        && !upper.contains("NO VIRTUAL DISKS")
        && !upper.contains("CANNOT FIND THE FILE SPECIFIED"))
}

fn parse_subst_mapping(output: &str, letter: &str) -> Option<PathBuf> {
    let prefix = format!("{}\\: =>", letter.to_ascii_uppercase());
    output.lines().find_map(|line| {
        let line = line.trim();
        line.to_ascii_uppercase()
            .strip_prefix(&prefix)
            .map(|_| PathBuf::from(line[prefix.len()..].trim()))
    })
}

fn startup_script(exe: &Path, letter: &str, options: &DriveOptions) -> String {
    let ram_arg = if options.ram { " --ram" } else { "" };
    let size_arg = if options.ram {
        format!(" --size {}", options.size)
    } else {
        String::new()
    };
    format!(
        "CreateObject(\"WScript.Shell\").Run \"\"\"{}\"\" drive mount --letter {}{}{}\", 0, False\r\n",
        exe.display(),
        letter,
        ram_arg,
        size_arg
    )
}

fn startup_file(letter: &str) -> Result<PathBuf> {
    let appdata = env::var("APPDATA")?;
    let normalized = normalize_letter(letter)?.trim_end_matches(':').to_string();
    Ok(PathBuf::from(appdata)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Startup")
        .join(format!("Qorx Drive {normalized}.vbs")))
}

fn tray_startup_file() -> Result<PathBuf> {
    let appdata = env::var("APPDATA")?;
    Ok(PathBuf::from(appdata)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Startup")
        .join("Qorx Tray.vbs"))
}

fn tray_startup_mounts_drive(letter: &str) -> bool {
    let Ok(startup) = tray_startup_file() else {
        return false;
    };
    let Ok(script) = fs::read_to_string(startup) else {
        return false;
    };
    let needle = format!("drive mount --letter {}", letter).to_ascii_lowercase();
    script.to_ascii_lowercase().contains(&needle)
}

fn current_exe() -> Result<PathBuf> {
    env::current_exe().map_err(Into::into)
}

fn drive_root_exists(letter: &str) -> bool {
    Path::new(&format!("{letter}\\")).exists()
}

fn same_path(left: &Path, right: &Path) -> bool {
    let left = fs::canonicalize(left).unwrap_or_else(|_| left.to_path_buf());
    let right = fs::canonicalize(right).unwrap_or_else(|_| right.to_path_buf());
    left == right
}

fn ensure_imdisk_backend() -> Result<()> {
    if imdisk_report().can_mount {
        Ok(())
    } else {
        Err(anyhow!(
            "RAM-backed drive mode requires ImDisk installed and running, or a future signed qorxram.sys path"
        ))
    }
}

fn native_ram_driver_report() -> RamDriverReport {
    let (installed, running) = qorxram_service_state();
    RamDriverReport {
        service_name: "qorxram".to_string(),
        installed,
        running,
        can_mount: false,
        expected_driver_file: expected_qorxram_driver_file().display().to_string(),
        requirement: "Qorx native RAM drive requires a clean-room qorxram.sys Windows driver built with WDK, installed with admin rights, and signed per Windows kernel-mode driver policy.".to_string(),
    }
}

fn imdisk_report() -> ImDiskReport {
    let cli_path = imdisk_cli_path();
    let (installed, running) = service_state("imdisk");
    ImDiskReport {
        service_name: "imdisk".to_string(),
        installed: cli_path.is_some() || installed,
        running,
        can_mount: cli_path.is_some() && installed && running,
        cli_path: cli_path.map(|path| path.display().to_string()),
        requirement: "ImDisk RAM mode requires ImDisk Virtual Disk Driver installed with admin rights; Qorx can then create RAM-backed drive letters without its own signed driver.".to_string(),
    }
}

fn qorxram_service_state() -> (bool, bool) {
    service_state("qorxram")
}

fn service_state(name: &str) -> (bool, bool) {
    if !cfg!(windows) {
        return (false, false);
    }
    let Ok(output) = Command::new("sc")
        .args(["query", name])
        .stdin(Stdio::null())
        .output()
    else {
        return (false, false);
    };
    parse_sc_query_state(&command_output_text(&output), name)
}

fn parse_sc_query_state(output: &str, service_name: &str) -> (bool, bool) {
    let upper = output.to_ascii_uppercase();
    if upper.contains("FAILED 1060") || upper.contains("DOES NOT EXIST") {
        return (false, false);
    }
    let marker = format!("SERVICE_NAME: {}", service_name.to_ascii_uppercase());
    let installed = upper.contains(&marker);
    let running = installed
        && upper
            .lines()
            .any(|line| line.contains("STATE") && line.contains("RUNNING"));
    (installed, running)
}

fn expected_qorxram_driver_file() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("qorxram.sys")
}

fn imdisk_cli_path() -> Option<PathBuf> {
    let root = env::var("SystemRoot").ok()?;
    let path = PathBuf::from(root).join("System32").join("imdisk.exe");
    path.exists().then_some(path)
}

fn ram_disk_data_dir(letter: &str) -> PathBuf {
    PathBuf::from(format!("{letter}\\{PORTABLE_DATA_DIR}"))
}

fn ram_backing_dir(data_dir: &Path) -> PathBuf {
    let name = data_dir
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| PORTABLE_DATA_DIR.to_string());
    data_dir
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{name}-backing"))
}

fn sync_drive_home_to_backing(letter: &str) -> Result<()> {
    let Some(config) = load_drive_home_config()? else {
        return Ok(());
    };
    if normalize_letter(&config.letter)? != letter {
        return Ok(());
    }
    let home_dir = PathBuf::from(config.home_dir);
    let backing_dir = PathBuf::from(config.backing_dir);
    if home_dir.exists() {
        fs::create_dir_all(&backing_dir)?;
        copy_dir_contents(&home_dir, &backing_dir)?;
    }
    Ok(())
}

fn copy_dir_contents(source: &Path, destination: &Path) -> Result<()> {
    if !source.exists() {
        return Ok(());
    }
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir_contents(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&source_path, &destination_path)?;
        }
    }
    Ok(())
}

fn wait_for_drive_root(letter: &str) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(10);
    let probe = PathBuf::from(format!("{letter}\\.__qorx_ready"));
    while Instant::now() < deadline {
        if drive_root_exists(letter) && fs::write(&probe, b"qorx").is_ok() {
            thread::sleep(Duration::from_millis(300));
            if probe.exists() {
                let _ = fs::remove_file(&probe);
                return Ok(());
            }
        }
        thread::sleep(Duration::from_millis(200));
    }
    Err(anyhow!(
        "timed out waiting for {letter} to become available"
    ))
}

fn elevated_start_process_command(executable: &Path, args: &[String]) -> String {
    let escaped_executable = executable.display().to_string().replace('\'', "''");
    let args = args
        .iter()
        .map(|arg| format!("'{}'", arg.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "Start-Process -FilePath '{}' -ArgumentList {} -Verb RunAs -Wait",
        escaped_executable, args
    )
}

fn command_output_text(output: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    #[test]
    fn normalizes_drive_letters() {
        assert_eq!(super::normalize_letter("q").unwrap(), "Q:");
        assert_eq!(super::normalize_letter("Q:").unwrap(), "Q:");
        assert!(super::normalize_letter("QQ").is_err());
    }

    #[test]
    fn builds_subst_mount_arguments() {
        let args = super::subst_mount_args("Q:", &PathBuf::from(r"C:\qorx-data"));

        assert_eq!(args, vec!["/C", "subst", "Q:", r"C:\qorx-data"]);
    }

    #[test]
    fn builds_imdisk_mount_arguments() {
        let args = super::imdisk_mount_args("Q:", "2G");

        assert_eq!(
            args,
            vec![
                "-a",
                "-t",
                "vm",
                "-s",
                "2G",
                "-m",
                "Q:",
                "-p",
                "/fs:ntfs /q /y"
            ]
        );
    }

    #[test]
    fn parses_existing_subst_mapping() {
        let output = "Q:\\: => C:\\Users\\Example\\AppData\\Local\\qorx\\Qorx\\data\r\n";

        let mapped = super::parse_subst_mapping(output, "Q:");

        assert_eq!(
            mapped,
            Some(PathBuf::from(
                r"C:\Users\Example\AppData\Local\qorx\Qorx\data"
            ))
        );
    }

    #[test]
    fn startup_script_remounts_drive_hidden() {
        let script = super::startup_script(
            &PathBuf::from(r"C:\Qorx\qorx.exe"),
            "Q:",
            &super::DriveOptions {
                ram: false,
                size: "2G".to_string(),
            },
        );

        assert!(script.contains("qorx.exe"));
        assert!(script.contains("drive mount"));
        assert!(script.contains("--letter Q:"));
        assert!(!script.contains("--ram"));
    }

    #[test]
    fn startup_script_preserves_ram_mode_flag() {
        let script = super::startup_script(
            &PathBuf::from(r"C:\Qorx\qorx.exe"),
            "Q:",
            &super::DriveOptions {
                ram: true,
                size: "2G".to_string(),
            },
        );

        assert!(script.contains("--ram"));
    }

    #[test]
    fn startup_script_preserves_imdisk_ram_mode_and_size() {
        let script = super::startup_script(
            &PathBuf::from(r"C:\Qorx\qorx.exe"),
            "Q:",
            &super::DriveOptions {
                ram: true,
                size: "3G".to_string(),
            },
        );

        assert!(script.contains("--ram"));
        assert!(script.contains("--size 3G"));
    }

    #[test]
    fn ram_disk_data_dir_uses_drive_root() {
        assert_eq!(
            super::ram_disk_data_dir("Q:"),
            PathBuf::from(r"Q:\qorx-data")
        );
    }

    #[test]
    fn backing_dir_lives_next_to_existing_data_dir() {
        assert_eq!(
            super::ram_backing_dir(Path::new(r"C:\Qorx\qorx-data")),
            PathBuf::from(r"C:\Qorx\qorx-data-backing")
        );
    }

    #[test]
    fn builds_elevated_imdisk_mount_command() {
        let command = super::elevated_start_process_command(
            Path::new(r"C:\Windows\System32\imdisk.exe"),
            &super::imdisk_mount_args("Q:", "2G"),
        );

        assert!(command.contains("Start-Process"));
        assert!(command.contains("-Verb RunAs"));
        assert!(command.contains("imdisk.exe"));
        assert!(command.contains("Q:"));
        assert!(command.contains("2G"));
    }

    #[test]
    fn parses_qorxram_service_state() {
        let running = "SERVICE_NAME: qorxram\r\n        STATE              : 4  RUNNING\r\n";
        let missing = "[SC] EnumQueryServicesStatus:OpenService FAILED 1060:\r\nThe specified service does not exist as an installed service.\r\n";

        assert_eq!(
            super::parse_sc_query_state(running, "qorxram"),
            (true, true)
        );
        assert_eq!(
            super::parse_sc_query_state(missing, "qorxram"),
            (false, false)
        );
    }
}
