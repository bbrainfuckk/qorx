use std::{
    env,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};

use crate::config::AppPaths;

pub const VOID_DEMO_LIMIT_HOURS: i64 = 24;
const VOID_DEMO_STATE_FILE: &str = "void-demo.pb";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct DemoState {
    schema: String,
    first_started_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    stopped_at: Option<DateTime<Utc>>,
}

impl Default for DemoState {
    fn default() -> Self {
        let now = Utc::now();
        Self::new(now)
    }
}

impl DemoState {
    fn new(now: DateTime<Utc>) -> Self {
        Self {
            schema: "qorx.void-demo.v1".to_string(),
            first_started_at: now,
            expires_at: now + ChronoDuration::hours(VOID_DEMO_LIMIT_HOURS),
            stopped_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DemoStatus {
    pub product: String,
    pub edition: String,
    pub demo: bool,
    pub limit_hours: Option<i64>,
    pub first_started_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub seconds_remaining: u64,
    pub expired: bool,
    pub state_file: Option<String>,
    pub boundary: String,
}

pub fn is_demo_mode() -> bool {
    truthy_env("QORX_VOID_DEMO")
        || env::var("QORX_EDITION")
            .map(|value| value.trim().eq_ignore_ascii_case("demo"))
            .unwrap_or(false)
        || current_exe_name_contains_demo()
}

pub fn status(paths: &AppPaths) -> Result<DemoStatus> {
    let path = demo_state_path(paths);
    if !is_demo_mode() {
        return Ok(status_from_state(&path, None, Utc::now(), false));
    }

    let state = load_or_create_state(&path, Utc::now())?;
    Ok(status_from_state(&path, Some(&state), Utc::now(), true))
}

pub fn ensure_runtime(paths: &AppPaths) -> Result<DemoStatus> {
    let status = status(paths)?;
    if status.demo && status.expired {
        mark_stopped(paths)?;
        return Err(anyhow!(
            "Qorx Void Demo expired after {VOID_DEMO_LIMIT_HOURS} hours. Install Qorx Void to keep the local gateway running."
        ));
    }
    Ok(status)
}

pub fn shutdown_duration(status: &DemoStatus) -> Option<Duration> {
    if !status.demo {
        return None;
    }
    Some(Duration::from_secs(status.seconds_remaining.max(1)))
}

pub fn mark_stopped(paths: &AppPaths) -> Result<()> {
    if !is_demo_mode() {
        return Ok(());
    }
    let path = demo_state_path(paths);
    if !path.exists() {
        return Ok(());
    }
    let legacy = path.with_extension("json");
    let mut state: DemoState = crate::proto_store::load_required(&path, &[legacy.as_path()])?;
    if state.stopped_at.is_none() {
        state.stopped_at = Some(Utc::now());
        crate::proto_store::save(&path, &state)?;
    }
    Ok(())
}

pub fn print_status(status: &DemoStatus) {
    println!("product: {}", status.product);
    println!("edition: {}", status.edition);
    if status.demo {
        println!(
            "limit_hours: {}",
            status.limit_hours.unwrap_or(VOID_DEMO_LIMIT_HOURS)
        );
        println!(
            "first_started_at: {}",
            status
                .first_started_at
                .map(|value| value.to_rfc3339())
                .unwrap_or_else(|| "unknown".to_string())
        );
        println!(
            "expires_at: {}",
            status
                .expires_at
                .map(|value| value.to_rfc3339())
                .unwrap_or_else(|| "unknown".to_string())
        );
        println!("seconds_remaining: {}", status.seconds_remaining);
        println!("expired: {}", status.expired);
    } else {
        println!("limit_hours: none");
        println!("expired: false");
    }
    println!("boundary: {}", status.boundary);
}

fn load_or_create_state(path: &Path, now: DateTime<Utc>) -> Result<DemoState> {
    if path.exists() {
        let legacy = path.with_extension("json");
        return crate::proto_store::load_required(path, &[legacy.as_path()]);
    }
    let state = DemoState::new(now);
    crate::proto_store::save(path, &state)?;
    Ok(state)
}

fn status_from_state(
    path: &Path,
    state: Option<&DemoState>,
    now: DateTime<Utc>,
    demo_enabled: bool,
) -> DemoStatus {
    let Some(state) = state else {
        let product = crate::version::product_name().to_string();
        let edition = crate::version::runtime_edition().to_string();
        return DemoStatus {
            product: product.clone(),
            edition,
            demo: false,
            limit_hours: None,
            first_started_at: None,
            expires_at: None,
            seconds_remaining: 0,
            expired: false,
            state_file: None,
            boundary: format!(
                "{product} has no built-in demo timer; normal licensing and distribution policy apply."
            ),
        };
    };
    let remaining = state.expires_at.signed_duration_since(now).num_seconds();
    let expired = demo_enabled && remaining <= 0;
    DemoStatus {
        product: "Qorx Void Demo".to_string(),
        edition: "demo".to_string(),
        demo: demo_enabled,
        limit_hours: Some(VOID_DEMO_LIMIT_HOURS),
        first_started_at: Some(state.first_started_at),
        expires_at: Some(state.expires_at),
        seconds_remaining: remaining.max(0) as u64,
        expired,
        state_file: Some(path.display().to_string()),
        boundary: "Qorx Void Demo runs the full local Void gateway for 24 hours from first launch, then the daemon refuses to start and any running gateway shuts down.".to_string(),
    }
}

fn demo_state_path(paths: &AppPaths) -> PathBuf {
    paths.data_dir.join(VOID_DEMO_STATE_FILE)
}

fn truthy_env(key: &str) -> bool {
    env::var(key)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on" | "demo"
            )
        })
        .unwrap_or(false)
}

fn current_exe_name_contains_demo() -> bool {
    env::current_exe()
        .ok()
        .and_then(|path| {
            path.file_stem()
                .map(|name| name.to_string_lossy().to_string())
        })
        .map(|name| name.to_ascii_lowercase().contains("demo"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_status_has_no_demo_limit() {
        let now = Utc::now();
        let path = PathBuf::from("void-demo.pb");
        let status = status_from_state(&path, None, now, false);

        assert!(!status.demo);
        assert!(!status.expired);
        assert_eq!(status.limit_hours, None);
        assert_eq!(status.product, crate::version::QORX_PRODUCT);
        assert_eq!(status.edition, "void");
    }

    #[test]
    fn demo_status_counts_down_from_first_start() {
        let now = Utc::now();
        let start = now - ChronoDuration::hours(1);
        let state = DemoState::new(start);
        let path = PathBuf::from("void-demo.pb");
        let status = status_from_state(&path, Some(&state), now, true);

        assert!(status.demo);
        assert!(!status.expired);
        assert!(status.seconds_remaining <= 23 * 60 * 60);
        assert!(status.seconds_remaining > 22 * 60 * 60);
    }

    #[test]
    fn demo_status_expires_after_twenty_four_hours() {
        let now = Utc::now();
        let start = now - ChronoDuration::hours(25);
        let state = DemoState::new(start);
        let path = PathBuf::from("void-demo.pb");
        let status = status_from_state(&path, Some(&state), now, true);

        assert!(status.demo);
        assert!(status.expired);
        assert_eq!(status.seconds_remaining, 0);
    }
}
