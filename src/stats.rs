use std::{
    env, fs,
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, RwLock,
    },
};

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

const DEFAULT_INPUT_USD_PER_MILLION: f64 = 2.50;
const DEFAULT_CACHED_INPUT_USD_PER_MILLION: f64 = 0.25;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Stats {
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub requests: u64,
    pub raw_prompt_tokens: u64,
    pub compressed_prompt_tokens: u64,
    pub saved_prompt_tokens: u64,
    pub upstream_errors: u64,
    #[serde(alias = "atoms_created")]
    pub quarks_created: u64,
    pub cache_lookups: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_saved_prompt_tokens: u64,
    pub provider_cached_prompt_tokens: u64,
    pub provider_cache_write_tokens: u64,
    pub context_pack_requests: u64,
    pub context_indexed_tokens: u64,
    pub context_sent_tokens: u64,
    pub context_omitted_tokens: u64,
    pub last_provider: Option<String>,
}

impl Default for Stats {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            started_at: now,
            updated_at: now,
            requests: 0,
            raw_prompt_tokens: 0,
            compressed_prompt_tokens: 0,
            saved_prompt_tokens: 0,
            upstream_errors: 0,
            quarks_created: 0,
            cache_lookups: 0,
            cache_hits: 0,
            cache_misses: 0,
            cache_saved_prompt_tokens: 0,
            provider_cached_prompt_tokens: 0,
            provider_cache_write_tokens: 0,
            context_pack_requests: 0,
            context_indexed_tokens: 0,
            context_sent_tokens: 0,
            context_omitted_tokens: 0,
            last_provider: None,
        }
    }
}

impl Stats {
    pub fn savings_percent(&self) -> f64 {
        if self.raw_prompt_tokens == 0 {
            0.0
        } else {
            (self.saved_prompt_tokens as f64 / self.raw_prompt_tokens as f64) * 100.0
        }
    }

    pub fn atomic_ratio(&self) -> f64 {
        if self.compressed_prompt_tokens == 0 {
            1.0
        } else {
            self.raw_prompt_tokens.max(1) as f64 / self.compressed_prompt_tokens.max(1) as f64
        }
    }

    pub fn quark_ratio(&self) -> f64 {
        self.atomic_ratio()
    }

    pub fn context_reduction_x(&self) -> f64 {
        if self.context_sent_tokens == 0 {
            1.0
        } else {
            self.context_indexed_tokens.max(1) as f64 / self.context_sent_tokens.max(1) as f64
        }
    }

    pub fn pricing(&self) -> Pricing {
        Pricing::from_env()
    }

    pub fn context_usd_saved(&self) -> f64 {
        self.pricing().input_usd(self.context_omitted_tokens)
    }

    pub fn proxy_usd_saved(&self) -> f64 {
        self.pricing().input_usd(self.saved_prompt_tokens)
    }

    pub fn provider_cache_usd_saved(&self) -> f64 {
        self.pricing()
            .cached_discount_usd(self.provider_cached_prompt_tokens)
    }

    pub fn total_estimated_usd_saved(&self) -> f64 {
        self.context_usd_saved() + self.proxy_usd_saved() + self.provider_cache_usd_saved()
    }

    pub fn cache_hit_rate_percent(&self) -> f64 {
        if self.requests == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / self.requests as f64) * 100.0
        }
    }

    pub fn cache_lookup_hit_rate_percent(&self) -> f64 {
        if self.cache_lookups == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / self.cache_lookups as f64) * 100.0
        }
    }

    pub fn delta_since(&self, baseline: &Stats, started_at: DateTime<Utc>) -> Stats {
        Stats {
            started_at,
            updated_at: self.updated_at,
            requests: self.requests.saturating_sub(baseline.requests),
            raw_prompt_tokens: self
                .raw_prompt_tokens
                .saturating_sub(baseline.raw_prompt_tokens),
            compressed_prompt_tokens: self
                .compressed_prompt_tokens
                .saturating_sub(baseline.compressed_prompt_tokens),
            saved_prompt_tokens: self
                .saved_prompt_tokens
                .saturating_sub(baseline.saved_prompt_tokens),
            upstream_errors: self
                .upstream_errors
                .saturating_sub(baseline.upstream_errors),
            quarks_created: self.quarks_created.saturating_sub(baseline.quarks_created),
            cache_lookups: self.cache_lookups.saturating_sub(baseline.cache_lookups),
            cache_hits: self.cache_hits.saturating_sub(baseline.cache_hits),
            cache_misses: self.cache_misses.saturating_sub(baseline.cache_misses),
            cache_saved_prompt_tokens: self
                .cache_saved_prompt_tokens
                .saturating_sub(baseline.cache_saved_prompt_tokens),
            provider_cached_prompt_tokens: self
                .provider_cached_prompt_tokens
                .saturating_sub(baseline.provider_cached_prompt_tokens),
            provider_cache_write_tokens: self
                .provider_cache_write_tokens
                .saturating_sub(baseline.provider_cache_write_tokens),
            context_pack_requests: self
                .context_pack_requests
                .saturating_sub(baseline.context_pack_requests),
            context_indexed_tokens: self
                .context_indexed_tokens
                .saturating_sub(baseline.context_indexed_tokens),
            context_sent_tokens: self
                .context_sent_tokens
                .saturating_sub(baseline.context_sent_tokens),
            context_omitted_tokens: self
                .context_omitted_tokens
                .saturating_sub(baseline.context_omitted_tokens),
            last_provider: self.last_provider.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pricing {
    pub input_usd_per_million_tokens: f64,
    pub cached_input_usd_per_million_tokens: f64,
    pub source: String,
    pub assumption: String,
}

impl Pricing {
    pub fn from_env() -> Self {
        let input_env = env::var("QORX_USD_PER_M_INPUT_TOKENS").ok();
        let cached_env = env::var("QORX_USD_PER_M_CACHED_INPUT_TOKENS").ok();
        let input = input_env
            .as_deref()
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|value| *value >= 0.0)
            .unwrap_or(DEFAULT_INPUT_USD_PER_MILLION);
        let cached = cached_env
            .as_deref()
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|value| *value >= 0.0)
            .unwrap_or(DEFAULT_CACHED_INPUT_USD_PER_MILLION);
        let source = if input_env.is_some() || cached_env.is_some() {
            "env_override".to_string()
        } else {
            "default_example_rates_2026_04_28".to_string()
        };

        Self {
            input_usd_per_million_tokens: input,
            cached_input_usd_per_million_tokens: cached,
            source,
            assumption: "Dollar savings are estimates from configured input-token prices; set QORX_USD_PER_M_INPUT_TOKENS and QORX_USD_PER_M_CACHED_INPUT_TOKENS for the actual model/account.".to_string(),
        }
    }

    pub fn input_usd(&self, tokens: u64) -> f64 {
        (tokens as f64 / 1_000_000.0) * self.input_usd_per_million_tokens
    }

    pub fn cached_discount_usd(&self, tokens: u64) -> f64 {
        let discount =
            (self.input_usd_per_million_tokens - self.cached_input_usd_per_million_tokens).max(0.0);
        (tokens as f64 / 1_000_000.0) * discount
    }
}

pub fn record_context_pack(
    path: impl AsRef<Path>,
    indexed_tokens: u64,
    sent_tokens: u64,
) -> Result<()> {
    let path = path.as_ref();
    let legacy = path.with_extension("json");
    let mut stats: Stats = crate::proto_store::load_or_default(path, &[legacy.as_path()])?;

    stats.updated_at = Utc::now();
    stats.context_pack_requests += 1;
    stats.context_indexed_tokens += indexed_tokens;
    stats.context_sent_tokens += sent_tokens;
    stats.context_omitted_tokens += indexed_tokens.saturating_sub(sent_tokens);
    crate::proto_store::save(path, &stats)?;
    Ok(())
}

pub fn reset(path: impl AsRef<Path>) -> Result<Stats> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let stats = Stats::default();
    crate::proto_store::save(path, &stats)?;
    Ok(stats)
}

#[derive(Debug, Clone)]
pub struct RequestStats<'a> {
    pub provider: &'a str,
    pub raw_prompt_tokens: u64,
    pub compressed_prompt_tokens: u64,
    pub quarks_created: u64,
    pub upstream_error: bool,
    pub cache_lookup: bool,
    pub cache_hit: bool,
    pub provider_cached_prompt_tokens: u64,
    pub provider_cache_write_tokens: u64,
}

#[derive(Clone)]
pub struct StatsStore {
    path: Arc<std::path::PathBuf>,
    inner: Arc<Mutex<Stats>>,
    live: Arc<LiveCounters>,
}

#[derive(Debug)]
struct LiveCounters {
    session_window: RwLock<SessionWindow>,
    current_processing_requests: AtomicU64,
    current_processing_tokens: AtomicU64,
    last_raw_prompt_tokens: AtomicU64,
    last_compressed_prompt_tokens: AtomicU64,
    last_saved_prompt_tokens: AtomicU64,
}

#[derive(Debug, Clone)]
struct SessionWindow {
    started_at: DateTime<Utc>,
    baseline: Stats,
}

impl LiveCounters {
    fn new(session_baseline: Stats) -> Self {
        Self {
            session_window: RwLock::new(SessionWindow {
                started_at: Utc::now(),
                baseline: session_baseline,
            }),
            current_processing_requests: AtomicU64::new(0),
            current_processing_tokens: AtomicU64::new(0),
            last_raw_prompt_tokens: AtomicU64::new(0),
            last_compressed_prompt_tokens: AtomicU64::new(0),
            last_saved_prompt_tokens: AtomicU64::new(0),
        }
    }

    fn session_window(&self) -> SessionWindow {
        self.session_window
            .read()
            .map(|window| window.clone())
            .unwrap_or_else(|_| SessionWindow {
                started_at: Utc::now(),
                baseline: Stats::default(),
            })
    }

    fn reset_session_baseline(&self, baseline: Stats) {
        if let Ok(mut window) = self.session_window.write() {
            *window = SessionWindow {
                started_at: Utc::now(),
                baseline,
            };
        }
    }

    fn record_last(&self, raw_prompt_tokens: u64, compressed_prompt_tokens: u64) {
        self.last_raw_prompt_tokens
            .store(raw_prompt_tokens, Ordering::Relaxed);
        self.last_compressed_prompt_tokens
            .store(compressed_prompt_tokens, Ordering::Relaxed);
        self.last_saved_prompt_tokens.store(
            raw_prompt_tokens.saturating_sub(compressed_prompt_tokens),
            Ordering::Relaxed,
        );
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LiveStats {
    pub session_started_at: DateTime<Utc>,
    pub metric_mode: String,
    pub current_processing_requests: u64,
    pub current_processing_tokens: u64,
    pub last_raw_prompt_tokens: u64,
    pub last_compressed_prompt_tokens: u64,
    pub last_saved_prompt_tokens: u64,
    pub last_reduction_x: f64,
    pub last_conversion_label: String,
}

#[derive(Debug)]
pub struct ProcessingGuard {
    live: Arc<LiveCounters>,
    tokens: u64,
    active: bool,
}

impl StatsStore {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let legacy = path.with_extension("json");
        let stats: Stats = crate::proto_store::load_or_default(&path, &[legacy.as_path()])?;

        Ok(Self {
            path: Arc::new(path),
            inner: Arc::new(Mutex::new(stats.clone())),
            live: Arc::new(LiveCounters::new(stats)),
        })
    }

    pub async fn snapshot(&self) -> Stats {
        self.inner.lock().await.clone()
    }

    pub fn session_snapshot_from(&self, lifetime: &Stats) -> Stats {
        let window = self.live.session_window();
        lifetime.delta_since(&window.baseline, window.started_at)
    }

    pub fn live_snapshot(&self) -> LiveStats {
        let window = self.live.session_window();
        let raw = self.live.last_raw_prompt_tokens.load(Ordering::Relaxed);
        let compressed = self
            .live
            .last_compressed_prompt_tokens
            .load(Ordering::Relaxed);
        LiveStats {
            session_started_at: window.started_at,
            metric_mode:
                "ledger persists across restarts; session counters start when this daemon starts"
                    .to_string(),
            current_processing_requests: self
                .live
                .current_processing_requests
                .load(Ordering::Relaxed),
            current_processing_tokens: self.live.current_processing_tokens.load(Ordering::Relaxed),
            last_raw_prompt_tokens: raw,
            last_compressed_prompt_tokens: compressed,
            last_saved_prompt_tokens: self.live.last_saved_prompt_tokens.load(Ordering::Relaxed),
            last_reduction_x: if compressed == 0 {
                raw.max(1) as f64
            } else {
                raw.max(1) as f64 / compressed.max(1) as f64
            },
            last_conversion_label: if raw == 0 {
                "waiting for first proxied request".to_string()
            } else if compressed == 0 {
                format!("{raw} -> 0 upstream tokens")
            } else {
                format!("{raw} -> {compressed} tokens")
            },
        }
    }

    pub fn begin_processing(
        &self,
        raw_prompt_tokens: u64,
        compressed_prompt_tokens: u64,
    ) -> ProcessingGuard {
        self.live
            .current_processing_requests
            .fetch_add(1, Ordering::Relaxed);
        self.live
            .current_processing_tokens
            .fetch_add(raw_prompt_tokens, Ordering::Relaxed);
        self.live
            .record_last(raw_prompt_tokens, compressed_prompt_tokens);
        ProcessingGuard {
            live: Arc::clone(&self.live),
            tokens: raw_prompt_tokens,
            active: true,
        }
    }

    pub async fn record_request(&self, request: RequestStats<'_>) -> Result<()> {
        self.live
            .record_last(request.raw_prompt_tokens, request.compressed_prompt_tokens);
        let mut stats = self.inner.lock().await;
        stats.updated_at = Utc::now();
        stats.requests += 1;
        stats.raw_prompt_tokens += request.raw_prompt_tokens;
        stats.compressed_prompt_tokens += request.compressed_prompt_tokens;
        stats.saved_prompt_tokens += request
            .raw_prompt_tokens
            .saturating_sub(request.compressed_prompt_tokens);
        stats.quarks_created += request.quarks_created;
        if request.cache_lookup {
            stats.cache_lookups += 1;
        }
        stats.provider_cached_prompt_tokens += request.provider_cached_prompt_tokens;
        stats.provider_cache_write_tokens += request.provider_cache_write_tokens;
        stats.last_provider = Some(request.provider.to_string());
        if request.cache_hit {
            stats.cache_hits += 1;
            stats.cache_saved_prompt_tokens += request.raw_prompt_tokens;
        } else if request.cache_lookup {
            stats.cache_misses += 1;
        }
        if request.upstream_error {
            stats.upstream_errors += 1;
        }
        crate::proto_store::save(&self.path, &*stats)?;
        Ok(())
    }

    pub async fn reset(&self) -> Result<Stats> {
        let mut stats = self.inner.lock().await;
        *stats = Stats::default();
        crate::proto_store::save(&self.path, &*stats)?;
        self.live.reset_session_baseline(stats.clone());
        Ok(stats.clone())
    }
}

impl Drop for ProcessingGuard {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        self.live
            .current_processing_requests
            .fetch_sub(1, Ordering::Relaxed);
        self.live
            .current_processing_tokens
            .fetch_sub(self.tokens, Ordering::Relaxed);
        self.active = false;
    }
}
