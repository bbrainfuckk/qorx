use std::{net::SocketAddr, sync::Arc};

use std::{fs, path::PathBuf};

use anyhow::Result;
use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{response::Builder, HeaderMap, HeaderValue, Method, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{any, get, post},
    Json, Router,
};
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{
    b2c_quant, cache_plan, capsule,
    compression::{compress_json_body, estimate_tokens, AtomStore},
    config::{AppPaths, ProxyConfig},
    context_vm, cost_stack, demo, grounding,
    hot::{self, RamHotState},
    index::load_index,
    integrations, judge, memory, money,
    response_cache::{self, ExactResponseCache},
    session::build_session_pointer,
    squeeze,
    stats::{RequestStats, Stats, StatsStore},
    truth,
};

const MONITOR_HTML: &str = include_str!("monitor.html");

#[cfg(test)]
mod monitor_tests {
    #[test]
    fn monitor_opens_with_one_consumer_story() {
        let html = super::MONITOR_HTML;
        assert!(html.contains("<title>Qorx Void Monitor</title>"));
        assert!(html.contains("id=\"runtimeTitle\">Qorx Void"));
        assert!(html.contains("id=\"voidSwitch\""));
        assert!(html.contains("class=\"switch-orb\""));
        assert!(html.contains("aria-pressed=\"false\""));
        assert!(html.contains("id=\"switchState\""));
        assert!(html.contains("Qorx is off"));
        assert!(html.contains("tokens on this computer and sent"));
        let switch = html.find("id=\"voidSwitch\"").expect("switch");
        let kept = html.find("id=\"keptTokens\"").expect("kept counter");
        let prefs = html.find("id=\"preferencesPanel\"").expect("preferences");
        assert!(switch < kept);
        assert!(kept < prefs);
        assert!(!html.contains("sprite-board"));
        assert!(!html.contains("pixel-character"));
        assert!(!html.contains("RAM quarks"));
    }

    #[test]
    fn monitor_uses_the_website_font_system() {
        let html = super::MONITOR_HTML;
        assert!(html.contains("--bg:#f4f3ee"));
        assert!(html.contains("--paper:#fbfaf5"));
        assert!(html.contains("--logo:#050505"));
        assert!(html.contains("--logo:#f7f3ea"));
        assert!(html.contains("fonts.googleapis.com/css2?family=Geist"));
        assert!(html.contains("family=Instrument+Serif"));
        assert!(html.contains("\"Geist\""));
        assert!(html.contains("\"Geist Mono\""));
        assert!(html.contains("--accent-font:\"Instrument Serif\""));
        assert!(html.contains("\"SF Pro Display\""));
        assert!(html.contains("--ui:-apple-system,BlinkMacSystemFont"));
        assert!(html.contains("\"SF Pro Text\""));
        assert!(html.contains("--num:var(--display)"));
        assert!(html.contains("color:var(--logo)"));
        assert!(html.contains("font-family:var(--ui)"));
        assert!(html.contains("font-family:var(--display)"));
        assert!(html.contains("font-family:var(--accent-font)"));
        assert!(html.contains("font-family:var(--mono)"));
        assert!(html.contains("font-style:italic"));
        assert!(html.contains("font-style:normal"));
        assert!(html.contains("Q<sup>x</sup>"));
        assert!(!html.contains("\"New York\""));
        assert!(!html.contains("linear-gradient"));
        assert!(!html.contains("radial-gradient"));
        assert!(!html.contains("repeating-linear-gradient"));
    }

    #[test]
    fn monitor_surfaces_only_the_numbers_a_customer_needs_first() {
        let html = super::MONITOR_HTML;
        assert!(html.contains("Kept here"));
        assert!(html.contains("Sent to AI"));
        assert!(html.contains("Reduction"));
        assert!(html.contains("Avoided input cost"));
        assert!(html.contains("id=\"keptTokens\""));
        assert!(html.contains("id=\"sentTokens\""));
        assert!(html.contains("id=\"reduction\""));
        assert!(html.contains("id=\"savedUsd\""));
        assert!(html.contains("id=\"plainStatus\""));
        assert!(html.contains("s.context_omitted_tokens"));
        assert!(html.contains("s.context_sent_tokens"));
        assert!(html.contains("s.context_reduction_x"));
        assert!(html.contains("s.context_usd_saved"));
        let kept = html.find("id=\"keptTokens\"").expect("kept counter");
        let sent = html.find("id=\"sentTokens\"").expect("sent counter");
        let reduction = html.find("id=\"reduction\"").expect("reduction counter");
        let savings = html.find("id=\"savedUsd\"").expect("saved counter");
        let preferences = html
            .find("id=\"preferencesPanel\"")
            .expect("preferences panel");
        assert!(kept < sent);
        assert!(sent < reduction);
        assert!(reduction < savings);
        assert!(savings < preferences);
        assert!(!html.contains("aria-label=\"Context movement\""));
        assert!(!html.contains("aria-label=\"Session numbers\""));
    }

    #[test]
    fn monitor_rolls_counter_changes_like_an_odometer() {
        let html = super::MONITOR_HTML;
        assert!(html.contains("@keyframes counter-roll"));
        assert!(html.contains("prefers-reduced-motion:reduce"));
        assert!(html.contains("function animateCounter"));
        assert!(html.contains("requestAnimationFrame(frame)"));
        assert!(html.contains("is-rolling"));
        assert!(html.contains("animateCounter(\"keptTokens\""));
        assert!(html.contains("animateCounter(\"sentTokens\""));
        assert!(html.contains("animateCounter(\"reduction\""));
        assert!(html.contains("animateCounter(\"savedUsd\""));
    }

    #[test]
    fn monitor_keeps_power_tools_available_but_not_first() {
        let html = super::MONITOR_HTML;
        assert!(html.contains("<details class=\"details-block\" id=\"advancedDetails\">"));
        assert!(html.contains("Workspace Map"));
        assert!(html.contains("id=\"atlasCanvas\""));
        assert!(html.contains("function renderAtlasScene"));
        assert!(html.contains("Advanced data"));
        assert!(html.contains("The live map above is the simple view"));
        assert!(html.contains("Connected areas"));
        assert!(html.contains("Find a path"));
        assert!(html.contains("File tree"));
        assert!(html.contains("id=\"atlasZoomIn\""));
        assert!(html.contains("id=\"atlasZoomOut\""));
        assert!(html.contains("function setAtlasZoom"));
        assert!(html.contains("atlas-overlay"));
        assert!(html.contains("pointerdown"));
        assert!(html.contains("panX"));
        assert!(!html.contains("id=\"atlasFocus\""));
        assert!(!html.contains("Suggested checks"));
        assert!(html.contains("/graph?limit=96"));
        assert!(html.contains("/atlas?limit=96"));
        assert!(html.contains("graphQuery"));
        assert!(html.contains("graphPathSource"));
        assert!(html.contains("graphPathTarget"));
    }

    #[test]
    fn monitor_focus_points_at_local_workspaces() {
        let html = super::MONITOR_HTML;
        assert!(html.contains("id=\"workspaceShelf\""));
        assert!(html.contains("workspaceTargets"));
        assert!(html.contains("function workspaceKey"));
        assert!(html.contains("function renderWorkspaceShelf"));
        assert!(html.contains("function focusWorkspace"));
        assert!(html.contains("Filter map by folder or file"));
        assert!(html.contains("Other files are still searchable"));
        assert!(html.contains("return openWorkspacePicker()"));
        assert!(html.contains("$(\"graphQuery\").addEventListener(\"keydown\""));
    }

    #[test]
    fn monitor_focus_can_pick_a_workspace_folder() {
        let html = super::MONITOR_HTML;
        assert!(html.contains("id=\"workspaceDirectoryInput\""));
        assert!(html.contains("webkitdirectory"));
        assert!(html.contains("function openWorkspacePicker"));
        assert!(html.contains("showDirectoryPicker"));
        assert!(html.contains("focusWorkspaceFromFolderName"));
    }

    #[test]
    fn monitor_preserves_integrations_and_live_endpoint_controls() {
        let html = super::MONITOR_HTML;
        assert!(html.contains("/health"));
        assert!(html.contains("/stats"));
        assert!(html.contains("/integrations"));
        assert!(html.contains("/integrations/settings"));
        assert!(html.contains("/integrations/activate"));
        assert!(html.contains("/account/status"));
        assert!(html.contains("/account/connect"));
        assert!(html.contains("/account/disconnect"));
        assert!(html.contains("qorx_account"));
        assert!(html.contains("automcpSwitch"));
        assert!(html.contains("autohookSwitch"));
        assert!(html.contains("Agent context"));
        assert!(html.contains("data-context-mode=\"auto\""));
        assert!(html.contains("data-context-mode=\"readable\""));
        assert!(html.contains("data-context-mode=\"deep\""));
        assert!(html.contains("data-context-mode=\"off\""));
        assert!(!html.contains("Prompt injection"));
        assert!(!html.contains(&["private", "internals"].join(" ")));
        assert!(html.contains("codex_context_mode"));
        assert!(html.contains("Docs"));
        assert!(html.contains("What Qorx does"));
        assert!(html.contains("Proof boundary"));
        assert!(html.contains("qorx man"));
        assert!(html.contains("qorx integrate status"));
        assert!(html.contains("qorx atlas export"));
        assert!(html.contains("Qorx CLI, version 0.0.1-ylem"));
        assert!(html.contains("platformTiles"));
        assert!(html.contains("https://qorx.orin.work/dashboard#account"));
        assert!(html.contains("Google Login"));
        assert!(html.contains("No cloud account connected."));
        assert!(html.contains("Turn on MCP + hooks"));
        assert!(html.contains("Turn off MCP + hooks"));
        assert!(html.contains("id=\"preferencesPanel\" hidden"));
        assert!(html.contains("id=\"settingsButton\""));
        assert!(html.contains("Preferences"));
    }

    #[test]
    fn monitor_documents_default_on_integration_protocol() {
        let html = super::MONITOR_HTML;

        assert!(html.contains("MCP + hooks are on by default."));
        assert!(html.contains("MCP + hooks are off from this device."));
        assert!(!html.contains("MCP + hooks are off until enabled."));
    }

    #[test]
    fn monitor_defaults_to_public_void_and_can_reflect_runtime_product() {
        let html = super::MONITOR_HTML;
        assert!(html.contains("const product=s.product||\"Qorx Void\";"));
        assert!(html.contains("document.title=`${product} Monitor`;"));
        assert!(html.contains("setText(\"runtimeTitle\",product);"));
        assert!(html.contains("document.body.dataset.product"));
        assert!(html.contains("setTheme(localStorage.qorxTheme||\"light\")"));
    }
}

#[derive(Debug, Clone, Deserialize)]
struct IntegrationActionParams {
    platform: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct IntegrationSettingsParams {
    automcp_enabled: Option<bool>,
    autohook_enabled: Option<bool>,
    codex_context_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct LocalAccountBridge {
    connected: bool,
    tenant: Option<String>,
    plan: Option<String>,
    api_key_prefix: Option<String>,
    email: Option<String>,
    display_name: Option<String>,
    avatar_url: Option<String>,
    monthly_call_limit: Option<u64>,
    email_verified: Option<bool>,
    connected_from: Option<String>,
    connected_at_utc: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccountConnectParams {
    tenant: Option<String>,
    plan: Option<String>,
    api_key_prefix: Option<String>,
    email: Option<String>,
    display_name: Option<String>,
    avatar_url: Option<String>,
    monthly_call_limit: Option<u64>,
    email_verified: Option<bool>,
    connected_from: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct StrictAnswerParams {
    question: String,
    limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct SqueezeParams {
    query: String,
    budget_tokens: Option<u64>,
    limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct JudgeParams {
    answer: String,
    query: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct GroundParams {
    query: String,
    answer: Option<String>,
    budget_tokens: Option<u64>,
    limit: Option<usize>,
    raw_tokens: Option<u64>,
    sent_tokens: Option<u64>,
    input_usd_per_million: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct CachePlanParams {
    prompt: String,
}

#[derive(Debug, Clone, Deserialize)]
struct B2cPlanParams {
    query: String,
    budget_tokens: Option<u64>,
    diff: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct AgentParams {
    objective: String,
    budget_tokens: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
struct MapParams {
    query: String,
    budget_tokens: Option<u64>,
    diff: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct GraphParams {
    query: Option<String>,
    limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct GraphPathParams {
    source: String,
    target: String,
    limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct OrclParams {
    query: String,
    budget_tokens: Option<u64>,
    depth: Option<usize>,
    limit: Option<usize>,
    diff: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ContextVmParams {
    objective: String,
    budget_tokens: Option<u64>,
    limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct ContextFaultParams {
    query: String,
    handle: Option<String>,
    budget_tokens: Option<u64>,
    limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct ContextInjectParams {
    objective: Option<String>,
    cwd: Option<String>,
    budget_tokens: Option<u64>,
    limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct ContextNanoParams {
    objective: Option<String>,
    budget_tokens: Option<u64>,
    limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct ContextQuettaParams {
    objective: Option<String>,
    budget_tokens: Option<u64>,
    limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct ContextExpandParams {
    carrier: String,
    budget_tokens: Option<u64>,
    limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct MemoryParams {
    action: String,
    kind: Option<String>,
    text: Option<String>,
    query: Option<String>,
    id: Option<String>,
    limit: Option<usize>,
    max_items: Option<usize>,
}

#[derive(Clone)]
pub struct ProxyState {
    pub config: ProxyConfig,
    pub demo: demo::DemoStatus,
    pub stats: StatsStore,
    pub paths: AppPaths,
    pub atoms: Arc<Mutex<AtomStore>>,
    pub response_cache: Arc<Mutex<ExactResponseCache>>,
    pub hot: Arc<RamHotState>,
    pub client: Client,
}

pub async fn run_gateway(paths: AppPaths, config: ProxyConfig) -> Result<()> {
    let demo_status = demo::ensure_runtime(&paths)?;
    let demo_shutdown = demo::shutdown_duration(&demo_status);
    let stop_paths = paths.clone();
    let stats = StatsStore::load(&paths.stats_file)?;
    let atoms = AtomStore::load(&paths.atom_file)?;
    let response_cache = ExactResponseCache::load(&paths.response_cache_file)?;
    let hot = hot::load(&paths, &atoms, &response_cache)?;
    let state = ProxyState {
        config: config.clone(),
        demo: demo_status,
        stats,
        paths,
        atoms: Arc::new(Mutex::new(atoms)),
        response_cache: Arc::new(Mutex::new(response_cache)),
        hot: Arc::new(hot),
        client: Client::new(),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/monitor", get(monitor))
        .route("/account/status", get(get_account_status))
        .route("/account/connect", post(connect_account))
        .route("/account/disconnect", post(disconnect_account))
        .route("/integrations", get(get_integrations))
        .route(
            "/integrations/settings",
            get(get_integration_settings).post(set_integration_settings),
        )
        .route("/integrations/activate", post(activate_integrations))
        .route("/integrations/deactivate", post(deactivate_integrations))
        .route("/hot", get(get_hot))
        .route("/demo", get(get_demo))
        .route("/stats", get(get_stats))
        .route("/stats/reset", post(reset_stats))
        .route("/money", get(get_money))
        .route("/session", get(get_session))
        .route("/capsule/session", get(get_capsule_session))
        .route(
            "/strict-answer",
            get(get_strict_answer).post(post_strict_answer),
        )
        .route("/squeeze", get(get_squeeze).post(post_squeeze))
        .route("/judge", get(get_judge).post(post_judge))
        .route("/ground", get(get_ground).post(post_ground))
        .route("/cache-plan", get(get_cache_plan).post(post_cache_plan))
        .route("/b2c-plan", get(get_b2c_plan).post(post_b2c_plan))
        .route("/agent", get(get_agent).post(post_agent))
        .route("/marvin", get(get_agent).post(post_agent))
        .route("/map", get(get_map).post(post_map))
        .route("/graph", get(get_graph))
        .route("/atlas", get(get_atlas))
        .route("/graph/path", get(get_graph_path))
        .route("/orcl", get(get_orcl).post(post_orcl))
        .route("/context/vm", get(get_context_vm).post(post_context_vm))
        .route(
            "/context/fault",
            get(get_context_fault).post(post_context_fault),
        )
        .route(
            "/context/inject",
            get(get_context_inject).post(post_context_inject),
        )
        .route(
            "/context/nano",
            get(get_context_nano).post(post_context_nano),
        )
        .route(
            "/context/quetta",
            get(get_context_quetta).post(post_context_quetta),
        )
        .route(
            "/context/expand",
            get(get_context_expand).post(post_context_expand),
        )
        .route("/vm", get(get_context_vm).post(post_context_vm))
        .route("/memory", get(get_memory).post(post_memory))
        .route("/anthropic/*path", any(proxy_anthropic))
        .route("/gemini/*path", any(proxy_gemini))
        .route("/*path", any(proxy_openai))
        .with_state(state);

    let addr: SocketAddr = config.bind.parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    if let Some(duration) = demo_shutdown {
        tokio::select! {
            result = axum::serve(listener, app) => {
                result?;
            }
            _ = tokio::time::sleep(duration) => {
                demo::mark_stopped(&stop_paths)?;
                anyhow::bail!("Qorx Void Demo expired after {} hours", demo::VOID_DEMO_LIMIT_HOURS);
            }
        }
    } else {
        axum::serve(listener, app).await?;
    }
    Ok(())
}

async fn health(State(state): State<ProxyState>) -> impl IntoResponse {
    let demo_status = demo::status(&state.paths).unwrap_or_else(|_| state.demo.clone());
    let product = demo_status.product.clone();
    let edition = demo_status.edition.clone();
    Json(serde_json::json!({
        "ok": true,
        "name": "qorx",
        "product": product,
        "version": crate::version::QORX_VERSION,
        "edition": edition,
        "demo": demo_status,
        "bind": state.config.bind,
        "cost_stack": cost_stack::policy(),
        "ram_hot": state.hot.report,
    }))
}

async fn monitor() -> Html<&'static str> {
    Html(MONITOR_HTML)
}

async fn get_account_status(State(state): State<ProxyState>) -> Response {
    Json(read_local_account(&state.paths)).into_response()
}

async fn connect_account(
    State(state): State<ProxyState>,
    Json(params): Json<AccountConnectParams>,
) -> Response {
    let paths = state.paths.clone();
    match tokio::task::spawn_blocking(move || {
        let account = account_from_connect(params);
        write_local_account(&paths, &account)?;
        Ok::<_, anyhow::Error>(account)
    })
    .await
    {
        Ok(Ok(account)) => Json(account).into_response(),
        Ok(Err(err)) => account_error(StatusCode::INTERNAL_SERVER_ERROR, err),
        Err(err) => account_error(StatusCode::INTERNAL_SERVER_ERROR, anyhow::anyhow!(err)),
    }
}

async fn disconnect_account(State(state): State<ProxyState>) -> Response {
    let paths = state.paths.clone();
    match tokio::task::spawn_blocking(move || {
        let path = account_bridge_file(&paths);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok::<_, anyhow::Error>(LocalAccountBridge::default())
    })
    .await
    {
        Ok(Ok(account)) => Json(account).into_response(),
        Ok(Err(err)) => account_error(StatusCode::INTERNAL_SERVER_ERROR, err),
        Err(err) => account_error(StatusCode::INTERNAL_SERVER_ERROR, anyhow::anyhow!(err)),
    }
}

fn account_from_connect(params: AccountConnectParams) -> LocalAccountBridge {
    LocalAccountBridge {
        connected: true,
        tenant: clean_account_text(params.tenant, 96),
        plan: clean_account_text(params.plan, 64),
        api_key_prefix: clean_account_text(params.api_key_prefix, 64),
        email: clean_account_text(params.email, 160),
        display_name: clean_account_text(params.display_name, 120),
        avatar_url: clean_account_text(params.avatar_url, 512),
        monthly_call_limit: params.monthly_call_limit,
        email_verified: params.email_verified,
        connected_from: clean_account_text(params.connected_from, 160)
            .or_else(|| Some("qorx.orin.work".to_string())),
        connected_at_utc: Some(
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        ),
    }
}

fn clean_account_text(value: Option<String>, limit: usize) -> Option<String> {
    let clean = value?
        .chars()
        .filter(|ch| !ch.is_control())
        .take(limit)
        .collect::<String>()
        .trim()
        .to_string();
    if clean.is_empty() {
        None
    } else {
        Some(clean)
    }
}

fn account_bridge_file(paths: &AppPaths) -> PathBuf {
    paths.data_dir.join("account-bridge.json")
}

fn read_local_account(paths: &AppPaths) -> LocalAccountBridge {
    let path = account_bridge_file(paths);
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<LocalAccountBridge>(&text).ok())
        .unwrap_or_default()
}

fn write_local_account(paths: &AppPaths, account: &LocalAccountBridge) -> Result<()> {
    fs::create_dir_all(&paths.data_dir)?;
    fs::write(
        account_bridge_file(paths),
        serde_json::to_string_pretty(account)?,
    )?;
    Ok(())
}

fn account_error(status: StatusCode, err: anyhow::Error) -> Response {
    (
        status,
        Json(serde_json::json!({
            "error": "qorx_account_bridge_failed",
            "message": err.to_string(),
        })),
    )
        .into_response()
}

async fn get_integrations(State(state): State<ProxyState>) -> Response {
    match integrations::report(&state.paths) {
        Ok(report) => Json(report).into_response(),
        Err(err) => integration_error(StatusCode::INTERNAL_SERVER_ERROR, err),
    }
}

async fn get_integration_settings(State(state): State<ProxyState>) -> Response {
    match integrations::load_settings(&state.paths) {
        Ok(settings) => Json(settings).into_response(),
        Err(err) => integration_error(StatusCode::INTERNAL_SERVER_ERROR, err),
    }
}

async fn set_integration_settings(
    State(state): State<ProxyState>,
    Json(params): Json<IntegrationSettingsParams>,
) -> Response {
    let paths = state.paths.clone();
    match tokio::task::spawn_blocking(move || {
        let mut settings = integrations::load_settings(&paths)?;
        if let Some(enabled) = params.automcp_enabled {
            settings.automcp_enabled = enabled;
        }
        if let Some(enabled) = params.autohook_enabled {
            settings.autohook_enabled = enabled;
        }
        if let Some(mode) = params.codex_context_mode {
            settings.codex_context_mode = integrations::normalize_codex_context_mode(&mode);
        }
        integrations::set_settings(&paths, settings)
    })
    .await
    {
        Ok(Ok(report)) => Json(report).into_response(),
        Ok(Err(err)) => integration_error(StatusCode::INTERNAL_SERVER_ERROR, err),
        Err(err) => integration_error(StatusCode::INTERNAL_SERVER_ERROR, anyhow::anyhow!(err)),
    }
}

async fn activate_integrations(
    State(state): State<ProxyState>,
    Json(params): Json<IntegrationActionParams>,
) -> Response {
    let platform = match integration_platform_param(params.platform.as_deref()) {
        Ok(platform) => platform,
        Err(err) => return integration_error(StatusCode::BAD_REQUEST, err),
    };
    let paths = state.paths.clone();
    match tokio::task::spawn_blocking(move || integrations::install_platform(&paths, platform))
        .await
    {
        Ok(Ok(report)) => Json(report).into_response(),
        Ok(Err(err)) => integration_error(StatusCode::INTERNAL_SERVER_ERROR, err),
        Err(err) => integration_error(StatusCode::INTERNAL_SERVER_ERROR, anyhow::anyhow!(err)),
    }
}

async fn deactivate_integrations(State(state): State<ProxyState>) -> Response {
    let paths = state.paths.clone();
    match tokio::task::spawn_blocking(move || integrations::deactivate_all(&paths)).await {
        Ok(Ok(report)) => Json(report).into_response(),
        Ok(Err(err)) => integration_error(StatusCode::INTERNAL_SERVER_ERROR, err),
        Err(err) => integration_error(StatusCode::INTERNAL_SERVER_ERROR, anyhow::anyhow!(err)),
    }
}

fn integration_platform_param(
    raw: Option<&str>,
) -> anyhow::Result<integrations::IntegrationPlatform> {
    let platform = raw.unwrap_or("all").trim();
    integrations::IntegrationPlatform::from_slug(platform)
        .ok_or_else(|| anyhow::anyhow!("unknown Qorx platform `{platform}`"))
}

fn integration_error(status: StatusCode, err: anyhow::Error) -> Response {
    (
        status,
        Json(serde_json::json!({
            "error": "qorx_integration_failed",
            "message": err.to_string(),
        })),
    )
        .into_response()
}

async fn get_hot(State(state): State<ProxyState>) -> impl IntoResponse {
    Json(hot::state_json(&state.hot))
}

async fn get_demo(State(state): State<ProxyState>) -> impl IntoResponse {
    Json(demo::status(&state.paths).unwrap_or_else(|_| state.demo.clone()))
}

async fn get_stats(State(state): State<ProxyState>) -> impl IntoResponse {
    let stats = fresh_stats(&state).await;
    let session = state.stats.session_snapshot_from(&stats);
    let live = state.stats.live_snapshot();
    let pricing = stats.pricing();
    let demo_status = demo::status(&state.paths).unwrap_or_else(|_| state.demo.clone());
    let product = demo_status.product.clone();
    let edition = demo_status.edition.clone();
    let session_json = serde_json::json!({
        "started_at": session.started_at,
        "updated_at": session.updated_at,
        "requests": session.requests,
        "raw_prompt_tokens": session.raw_prompt_tokens,
        "compressed_prompt_tokens": session.compressed_prompt_tokens,
        "saved_prompt_tokens": session.saved_prompt_tokens,
        "savings_percent": session.savings_percent(),
        "quark_ratio": session.quark_ratio(),
        "quarks_created": session.quarks_created,
        "cache_lookups": session.cache_lookups,
        "cache_hits": session.cache_hits,
        "cache_misses": session.cache_misses,
        "cache_hit_rate_percent": session.cache_hit_rate_percent(),
        "cache_lookup_hit_rate_percent": session.cache_lookup_hit_rate_percent(),
        "cache_saved_prompt_tokens": session.cache_saved_prompt_tokens,
        "provider_cached_prompt_tokens": session.provider_cached_prompt_tokens,
        "provider_cache_write_tokens": session.provider_cache_write_tokens,
        "context_pack_requests": session.context_pack_requests,
        "context_indexed_tokens": session.context_indexed_tokens,
        "context_sent_tokens": session.context_sent_tokens,
        "context_omitted_tokens": session.context_omitted_tokens,
        "context_reduction_x": session.context_reduction_x(),
        "context_usd_saved": session.context_usd_saved(),
        "proxy_usd_saved": session.proxy_usd_saved(),
        "provider_cache_usd_saved": session.provider_cache_usd_saved(),
        "total_estimated_usd_saved": session.total_estimated_usd_saved(),
        "upstream_errors": session.upstream_errors,
        "last_provider": session.last_provider,
    });
    Json(serde_json::json!({
        "product": product,
        "version": crate::version::QORX_VERSION,
        "edition": edition,
        "demo": demo_status,
        "metric_mode": live.metric_mode.clone(),
        "requests": stats.requests,
        "raw_prompt_tokens": stats.raw_prompt_tokens,
        "compressed_prompt_tokens": stats.compressed_prompt_tokens,
        "saved_prompt_tokens": stats.saved_prompt_tokens,
        "savings_percent": stats.savings_percent(),
        "quark_ratio": stats.quark_ratio(),
        "quarks_created": stats.quarks_created,
        "cache_lookups": stats.cache_lookups,
        "cache_hits": stats.cache_hits,
        "cache_misses": stats.cache_misses,
        "cache_hit_rate_percent": stats.cache_hit_rate_percent(),
        "cache_lookup_hit_rate_percent": stats.cache_lookup_hit_rate_percent(),
        "cache_saved_prompt_tokens": stats.cache_saved_prompt_tokens,
        "provider_cached_prompt_tokens": stats.provider_cached_prompt_tokens,
        "provider_cache_write_tokens": stats.provider_cache_write_tokens,
        "context_pack_requests": stats.context_pack_requests,
        "context_indexed_tokens": stats.context_indexed_tokens,
        "context_sent_tokens": stats.context_sent_tokens,
        "context_omitted_tokens": stats.context_omitted_tokens,
        "context_reduction_x": stats.context_reduction_x(),
        "pricing": pricing,
        "cost_stack": cost_stack::policy(),
        "context_usd_saved": stats.context_usd_saved(),
        "proxy_usd_saved": stats.proxy_usd_saved(),
        "provider_cache_usd_saved": stats.provider_cache_usd_saved(),
        "total_estimated_usd_saved": stats.total_estimated_usd_saved(),
        "session": session_json,
        "live": live.clone(),
        "current_processing_requests": live.current_processing_requests,
        "current_processing_tokens": live.current_processing_tokens,
        "last_conversion_label": live.last_conversion_label.clone(),
        "last_reduction_x": live.last_reduction_x,
        "upstream_errors": stats.upstream_errors,
        "last_provider": stats.last_provider,
        "updated_at": stats.updated_at,
    }))
}

async fn reset_stats(State(state): State<ProxyState>) -> Response {
    match state.stats.reset().await {
        Ok(stats) => Json(stats).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "qorx_stats_reset_failed",
                "message": err.to_string(),
            })),
        )
            .into_response(),
    }
}

async fn get_money(State(state): State<ProxyState>) -> impl IntoResponse {
    let stats = fresh_stats(&state).await;
    Json(money::build_money_proof(&stats, None))
}

async fn fresh_stats(state: &ProxyState) -> Stats {
    let legacy = state.paths.stats_file.with_extension("json");
    if let Ok(stats) =
        crate::proto_store::load_or_default(&state.paths.stats_file, &[legacy.as_path()])
    {
        return stats;
    }
    state.stats.snapshot().await
}

async fn get_strict_answer(
    State(state): State<ProxyState>,
    Query(params): Query<StrictAnswerParams>,
) -> Response {
    strict_answer_response(state, params)
}

async fn post_strict_answer(
    State(state): State<ProxyState>,
    Json(params): Json<StrictAnswerParams>,
) -> Response {
    strict_answer_response(state, params)
}

fn strict_answer_response(state: ProxyState, params: StrictAnswerParams) -> Response {
    match state_index_for_query(&state, &params.question) {
        Ok(index) => {
            let answer = truth::strict_answer(&index, &params.question, params.limit.unwrap_or(2));
            Json(answer).into_response()
        }
        Err(err) => index_unavailable_response(err),
    }
}

async fn get_squeeze(
    State(state): State<ProxyState>,
    Query(params): Query<SqueezeParams>,
) -> Response {
    squeeze_response(state, params)
}

async fn post_squeeze(
    State(state): State<ProxyState>,
    Json(params): Json<SqueezeParams>,
) -> Response {
    squeeze_response(state, params)
}

fn squeeze_response(state: ProxyState, params: SqueezeParams) -> Response {
    match state_index_for_query(&state, &params.query) {
        Ok(index) => {
            let report = squeeze::squeeze_context(
                &index,
                &params.query,
                params.budget_tokens.unwrap_or(900),
                params.limit.unwrap_or(4),
            );
            Json(report).into_response()
        }
        Err(err) => index_unavailable_response(err),
    }
}

async fn get_judge(State(state): State<ProxyState>, Query(params): Query<JudgeParams>) -> Response {
    judge_response(state, params)
}

async fn post_judge(State(state): State<ProxyState>, Json(params): Json<JudgeParams>) -> Response {
    judge_response(state, params)
}

fn judge_response(state: ProxyState, params: JudgeParams) -> Response {
    match state_index(&state) {
        Ok(index) => {
            let report = judge::judge_answer(&index, &params.answer, params.query.as_deref());
            Json(report).into_response()
        }
        Err(err) => index_unavailable_response(err),
    }
}

async fn get_ground(
    State(state): State<ProxyState>,
    Query(params): Query<GroundParams>,
) -> Response {
    ground_response(state, params)
}

async fn post_ground(
    State(state): State<ProxyState>,
    Json(params): Json<GroundParams>,
) -> Response {
    ground_response(state, params)
}

fn ground_response(state: ProxyState, params: GroundParams) -> Response {
    match state_index_for_query(&state, &params.query) {
        Ok(index) => {
            let report = grounding::grounding_gate(
                &index,
                &params.query,
                grounding::GroundingOptions {
                    budget_tokens: params.budget_tokens.unwrap_or(900),
                    limit: params.limit.unwrap_or(4),
                    answer: params.answer,
                    raw_tokens: params.raw_tokens,
                    sent_tokens: params.sent_tokens,
                    input_usd_per_million: params.input_usd_per_million,
                },
            );
            Json(report).into_response()
        }
        Err(err) => index_unavailable_response(err),
    }
}

async fn get_cache_plan(Query(params): Query<CachePlanParams>) -> impl IntoResponse {
    Json(cache_plan::plan_prompt(&params.prompt))
}

async fn post_cache_plan(Json(params): Json<CachePlanParams>) -> impl IntoResponse {
    Json(cache_plan::plan_prompt(&params.prompt))
}

async fn get_b2c_plan(
    State(state): State<ProxyState>,
    Query(params): Query<B2cPlanParams>,
) -> Response {
    b2c_plan_response(state, params)
}

async fn post_b2c_plan(
    State(state): State<ProxyState>,
    Json(params): Json<B2cPlanParams>,
) -> Response {
    b2c_plan_response(state, params)
}

fn b2c_plan_response(state: ProxyState, params: B2cPlanParams) -> Response {
    match state_index_for_query(&state, &params.query) {
        Ok(index) => Json(b2c_quant::plan_context_with_diff(
            &index,
            &params.query,
            params.budget_tokens.unwrap_or(900),
            params.diff.as_deref(),
        ))
        .into_response(),
        Err(err) => index_unavailable_response(err),
    }
}

async fn get_agent(State(state): State<ProxyState>, Query(params): Query<AgentParams>) -> Response {
    agent_response(state, params)
}

async fn post_agent(State(state): State<ProxyState>, Json(params): Json<AgentParams>) -> Response {
    agent_response(state, params)
}

fn agent_response(state: ProxyState, params: AgentParams) -> Response {
    match state_index_for_query(&state, &params.objective) {
        Ok(index) => {
            let report = truth::run_agent(
                &index,
                &params.objective,
                params.budget_tokens.unwrap_or(900),
            );
            Json(report).into_response()
        }
        Err(err) => index_unavailable_response(err),
    }
}

async fn get_map(State(state): State<ProxyState>, Query(params): Query<MapParams>) -> Response {
    map_response(state, params)
}

async fn post_map(State(state): State<ProxyState>, Json(params): Json<MapParams>) -> Response {
    map_response(state, params)
}

fn map_response(state: ProxyState, params: MapParams) -> Response {
    match state_index_for_query(&state, &params.query) {
        Ok(index) => {
            let report = crate::impact::map_context(
                &index,
                &params.query,
                params.diff.as_deref(),
                params.budget_tokens.unwrap_or(900),
            );
            Json(report).into_response()
        }
        Err(err) => index_unavailable_response(err),
    }
}

async fn get_graph(State(state): State<ProxyState>, Query(params): Query<GraphParams>) -> Response {
    let index = match params.query.as_deref() {
        Some(query) => state_index_for_query(&state, query),
        None => state_index(&state),
    };
    match index {
        Ok(index) => {
            let graph = match params.query.as_deref() {
                Some(query) => {
                    crate::graph_view::build_query_graph(&index, query, params.limit.unwrap_or(96))
                }
                None => {
                    crate::graph_view::build_dashboard_graph(&index, params.limit.unwrap_or(96))
                }
            };
            Json(graph).into_response()
        }
        Err(err) => index_unavailable_response(err),
    }
}

async fn get_atlas(State(state): State<ProxyState>, Query(params): Query<GraphParams>) -> Response {
    match state_index(&state) {
        Ok(index) => Json(crate::graph_view::build_atlas_report(
            &index,
            params.limit.unwrap_or(96),
        ))
        .into_response(),
        Err(err) => index_unavailable_response(err),
    }
}

async fn get_graph_path(
    State(state): State<ProxyState>,
    Query(params): Query<GraphPathParams>,
) -> Response {
    match state_index(&state) {
        Ok(index) => Json(crate::graph_view::trace_file_path(
            &index,
            &params.source,
            &params.target,
            params.limit.unwrap_or(128),
        ))
        .into_response(),
        Err(err) => index_unavailable_response(err),
    }
}

async fn get_orcl(State(state): State<ProxyState>, Query(params): Query<OrclParams>) -> Response {
    orcl_response(state, params)
}

async fn post_orcl(State(state): State<ProxyState>, Json(params): Json<OrclParams>) -> Response {
    orcl_response(state, params)
}

fn orcl_response(state: ProxyState, params: OrclParams) -> Response {
    match state_index_for_query(&state, &params.query) {
        Ok(index) => {
            let report = crate::orcl::report(
                &index,
                &params.query,
                params.diff.as_deref(),
                crate::orcl::OrclOptions {
                    budget_tokens: params.budget_tokens.unwrap_or(900),
                    depth: params.depth.unwrap_or(2),
                    limit: params.limit.unwrap_or(8),
                },
            );
            Json(report).into_response()
        }
        Err(err) => index_unavailable_response(err),
    }
}

async fn get_context_vm(
    State(state): State<ProxyState>,
    Query(params): Query<ContextVmParams>,
) -> Response {
    context_vm_response(state, params)
}

async fn post_context_vm(
    State(state): State<ProxyState>,
    Json(params): Json<ContextVmParams>,
) -> Response {
    context_vm_response(state, params)
}

fn context_vm_response(state: ProxyState, params: ContextVmParams) -> Response {
    match state_index(&state) {
        Ok(index) => {
            let report = context_vm::build_context_vm(
                &index,
                &params.objective,
                context_vm::ContextVmOptions {
                    budget_tokens: params.budget_tokens.unwrap_or(900),
                    limit: params.limit.unwrap_or(4),
                },
            );
            Json(report).into_response()
        }
        Err(err) => index_unavailable_response(err),
    }
}

async fn get_context_fault(
    State(state): State<ProxyState>,
    Query(params): Query<ContextFaultParams>,
) -> Response {
    context_fault_response(state, params)
}

async fn post_context_fault(
    State(state): State<ProxyState>,
    Json(params): Json<ContextFaultParams>,
) -> Response {
    context_fault_response(state, params)
}

fn context_fault_response(state: ProxyState, params: ContextFaultParams) -> Response {
    match state_index(&state) {
        Ok(index) => {
            let evidence_index = crate::index::with_live_overlay(&index, &params.query, 2_048, 128);
            let handle = params
                .handle
                .unwrap_or_else(|| build_session_pointer(&index).handle);
            let report = context_vm::resolve_context_fault_with_auth_index(
                &index,
                &evidence_index,
                &handle,
                &params.query,
                context_vm::ContextVmOptions {
                    budget_tokens: params.budget_tokens.unwrap_or(900),
                    limit: params.limit.unwrap_or(4),
                },
            );
            Json(report).into_response()
        }
        Err(err) => index_unavailable_response(err),
    }
}

async fn get_context_inject(
    State(state): State<ProxyState>,
    Query(params): Query<ContextInjectParams>,
) -> Response {
    context_inject_response(state, params)
}

async fn post_context_inject(
    State(state): State<ProxyState>,
    Json(params): Json<ContextInjectParams>,
) -> Response {
    context_inject_response(state, params)
}

fn context_inject_response(state: ProxyState, params: ContextInjectParams) -> Response {
    match state_index(&state) {
        Ok(index) => {
            let mut objective = params
                .objective
                .unwrap_or_else(|| "current agent turn".to_string());
            if let Some(cwd) = params.cwd.filter(|cwd| !cwd.trim().is_empty()) {
                objective = format!("caller_cwd: {}\nobjective: {}", cwd.trim(), objective);
            }
            let report = context_vm::build_context_injection(
                &index,
                &objective,
                context_vm::ContextVmOptions {
                    budget_tokens: params.budget_tokens.unwrap_or(900),
                    limit: params.limit.unwrap_or(4),
                },
            );
            record_context_pack_from_http(
                &state,
                index.total_tokens(),
                estimate_tokens(&report.additional_context).max(1),
            );
            Json(report).into_response()
        }
        Err(err) => index_unavailable_response(err),
    }
}

async fn get_context_nano(
    State(state): State<ProxyState>,
    Query(params): Query<ContextNanoParams>,
) -> Response {
    context_nano_response(state, params)
}

async fn post_context_nano(
    State(state): State<ProxyState>,
    Json(params): Json<ContextNanoParams>,
) -> Response {
    context_nano_response(state, params)
}

fn context_nano_response(state: ProxyState, params: ContextNanoParams) -> Response {
    match state_index(&state) {
        Ok(index) => {
            let objective = params
                .objective
                .unwrap_or_else(|| "current agent turn".to_string());
            let report = context_vm::build_context_nano(
                &index,
                &objective,
                context_vm::ContextVmOptions {
                    budget_tokens: params.budget_tokens.unwrap_or(900),
                    limit: params.limit.unwrap_or(4),
                },
            );
            record_context_pack_from_http(&state, report.indexed_tokens, report.visible_tokens);
            Json(report).into_response()
        }
        Err(err) => index_unavailable_response(err),
    }
}

async fn get_context_quetta(
    State(state): State<ProxyState>,
    Query(params): Query<ContextQuettaParams>,
) -> Response {
    context_quetta_response(state, params)
}

async fn post_context_quetta(
    State(state): State<ProxyState>,
    Json(params): Json<ContextQuettaParams>,
) -> Response {
    context_quetta_response(state, params)
}

fn context_quetta_response(state: ProxyState, params: ContextQuettaParams) -> Response {
    match state_index(&state) {
        Ok(index) => {
            let objective = params
                .objective
                .unwrap_or_else(|| "current agent turn".to_string());
            let report = context_vm::build_context_quetta(
                &index,
                &objective,
                context_vm::ContextVmOptions {
                    budget_tokens: params.budget_tokens.unwrap_or(900),
                    limit: params.limit.unwrap_or(4),
                },
            );
            record_context_pack_from_http(
                &state,
                report.local_indexed_tokens,
                report.visible_tokens,
            );
            Json(report).into_response()
        }
        Err(err) => index_unavailable_response(err),
    }
}

async fn get_context_expand(
    State(state): State<ProxyState>,
    Query(params): Query<ContextExpandParams>,
) -> Response {
    context_expand_response(state, params)
}

async fn post_context_expand(
    State(state): State<ProxyState>,
    Json(params): Json<ContextExpandParams>,
) -> Response {
    context_expand_response(state, params)
}

fn context_expand_response(state: ProxyState, params: ContextExpandParams) -> Response {
    match state_index(&state) {
        Ok(index) => {
            let report = context_vm::expand_nano_carrier(
                &index,
                &params.carrier,
                context_vm::ContextVmOptions {
                    budget_tokens: params.budget_tokens.unwrap_or(900),
                    limit: params.limit.unwrap_or(4),
                },
            );
            Json(report).into_response()
        }
        Err(err) => index_unavailable_response(err),
    }
}

fn record_context_pack_from_http(state: &ProxyState, indexed_tokens: u64, sent_tokens: u64) {
    let _ = crate::stats::record_context_pack(&state.paths.stats_file, indexed_tokens, sent_tokens);
}

async fn get_memory(
    State(state): State<ProxyState>,
    Query(params): Query<MemoryParams>,
) -> Response {
    memory_response(state, params)
}

async fn post_memory(
    State(state): State<ProxyState>,
    Json(params): Json<MemoryParams>,
) -> Response {
    memory_response(state, params)
}

fn memory_response(state: ProxyState, params: MemoryParams) -> Response {
    let result = match params.action.as_str() {
        "create" => memory::create(
            &state.paths,
            params.kind.as_deref().unwrap_or("note"),
            params.text.as_deref().unwrap_or_default(),
        ),
        "read" => memory::read(
            &state.paths,
            params.query.as_deref().unwrap_or_default(),
            params.limit.unwrap_or(8),
        ),
        "update" => memory::update(
            &state.paths,
            params.id.as_deref().unwrap_or_default(),
            params.text.as_deref().unwrap_or_default(),
        ),
        "delete" => memory::delete(&state.paths, params.id.as_deref().unwrap_or_default()),
        "summarize" => memory::summarize(&state.paths, params.limit.unwrap_or(8)),
        "prune" => memory::prune(&state.paths, params.max_items.unwrap_or(64)),
        _ => Err(anyhow::anyhow!("unknown memory action: {}", params.action)),
    };
    match result {
        Ok(report) => Json(report).into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "qorx_memory_error",
                "message": err.to_string(),
            })),
        )
            .into_response(),
    }
}

fn state_index(state: &ProxyState) -> anyhow::Result<crate::index::RepoIndex> {
    load_index(&state.paths.index_file).or_else(|_| {
        state
            .hot
            .index
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Qorx index is not loaded"))
    })
}

fn state_index_for_query(
    state: &ProxyState,
    query: &str,
) -> anyhow::Result<crate::index::RepoIndex> {
    state_index(state).map(|index| crate::index::with_live_overlay(&index, query, 2_048, 128))
}

fn index_unavailable_response(err: anyhow::Error) -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({
            "error": "qorx_index_unavailable",
            "message": err.to_string(),
        })),
    )
        .into_response()
}

async fn get_session(State(state): State<ProxyState>) -> Response {
    let index = state_index(&state);
    match index {
        Ok(index) => {
            let pointer = build_session_pointer(&index);
            Json(pointer).into_response()
        }
        Err(err) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "qorx_session_unavailable",
                "message": err.to_string(),
            })),
        )
            .into_response(),
    }
}

async fn get_capsule_session(State(state): State<ProxyState>) -> Response {
    match capsule::load_session_pointer(&state.paths) {
        Ok(pointer) => Json(pointer).into_response(),
        Err(err) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": "qorx_capsule_unavailable",
                "message": err.to_string(),
            })),
        )
            .into_response(),
    }
}

async fn proxy_openai(
    State(state): State<ProxyState>,
    Path(path): Path<String>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    proxy_to_provider(state, "openai", None, path, method, headers, body).await
}

async fn proxy_anthropic(
    State(state): State<ProxyState>,
    Path(path): Path<String>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    proxy_to_provider(
        state,
        "anthropic",
        Some("anthropic"),
        path,
        method,
        headers,
        body,
    )
    .await
}

async fn proxy_gemini(
    State(state): State<ProxyState>,
    Path(path): Path<String>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    proxy_to_provider(state, "gemini", Some("gemini"), path, method, headers, body).await
}

async fn proxy_to_provider(
    state: ProxyState,
    provider: &str,
    _prefix: Option<&str>,
    path: String,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let upstream = match provider {
        "anthropic" => &state.config.anthropic_upstream,
        "gemini" => &state.config.gemini_upstream,
        _ => &state.config.openai_upstream,
    };

    let target = format!("{}/{}", upstream.trim_end_matches('/'), path);
    let prompt_plan = cache_plan::plan_prompt(&request_prompt_surface(&body));
    let mut atoms = state.atoms.lock().await;
    let (compressed_body, report) = compress_json_body(&body, &mut atoms);
    let processing_guard = state
        .stats
        .begin_processing(report.raw_tokens, report.compressed_tokens);
    let atom_save = atoms.save(&state.paths.atom_file);
    drop(atoms);

    let cache_key = response_cache::request_key(provider, &method, &path, &compressed_body);
    let cache_lookup = cache_key.is_some();
    if let Some(key) = cache_key.as_deref() {
        let mut cache = state.response_cache.lock().await;
        if let Some(hit) = cache.get(key) {
            let cache_save = cache.save(&state.paths.response_cache_file);
            drop(cache);
            let _ = state
                .stats
                .record_request(RequestStats {
                    provider,
                    raw_prompt_tokens: report.raw_tokens,
                    compressed_prompt_tokens: 0,
                    quarks_created: report.quarks_created,
                    upstream_error: cache_save.is_err() || atom_save.is_err(),
                    cache_lookup: true,
                    cache_hit: true,
                    provider_cached_prompt_tokens: 0,
                    provider_cache_write_tokens: 0,
                })
                .await;
            let mut response = response_cache::response_from_cached(hit);
            add_qorx_headers(
                response.headers_mut(),
                provider,
                "hit",
                report.raw_tokens,
                0,
            );
            add_cache_plan_headers(response.headers_mut(), &prompt_plan);
            return response;
        }
    }

    let mut req = state.client.request(method.clone(), target);
    for (name, value) in headers.iter() {
        if !should_forward_request_header(name.as_str()) {
            continue;
        }
        req = req.header(name, value);
    }

    let result = req.body(compressed_body.clone()).send().await;
    let upstream_error = result.is_err() || atom_save.is_err();

    match result {
        Ok(resp) => {
            let status =
                StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            if cache_lookup
                && status.is_success()
                && !resp
                    .headers()
                    .get("content-type")
                    .and_then(|value| value.to_str().ok())
                    .is_some_and(|value| value.contains("text/event-stream"))
            {
                let content_type = resp
                    .headers()
                    .get("content-type")
                    .and_then(|value| value.to_str().ok())
                    .map(ToOwned::to_owned);
                let body = match resp.bytes().await {
                    Ok(body) => body,
                    Err(err) => {
                        let _ = state
                            .stats
                            .record_request(RequestStats {
                                provider,
                                raw_prompt_tokens: report.raw_tokens,
                                compressed_prompt_tokens: report.compressed_tokens,
                                quarks_created: report.quarks_created,
                                upstream_error: true,
                                cache_lookup,
                                cache_hit: false,
                                provider_cached_prompt_tokens: 0,
                                provider_cache_write_tokens: 0,
                            })
                            .await;
                        return (
                            StatusCode::BAD_GATEWAY,
                            Json(serde_json::json!({
                                "error": "qorx_upstream_error",
                                "message": err.to_string(),
                            })),
                        )
                            .into_response();
                    }
                };
                let (provider_cached_tokens, provider_cache_write_tokens) =
                    provider_cache_tokens(&body);
                if let Some(key) = cache_key {
                    let mut cache = state.response_cache.lock().await;
                    cache.insert(key, status, content_type.clone(), &body);
                    let _ = cache.save(&state.paths.response_cache_file);
                }
                let _ = state
                    .stats
                    .record_request(RequestStats {
                        provider,
                        raw_prompt_tokens: report.raw_tokens,
                        compressed_prompt_tokens: report.compressed_tokens,
                        quarks_created: report.quarks_created,
                        upstream_error,
                        cache_lookup: true,
                        cache_hit: false,
                        provider_cached_prompt_tokens: provider_cached_tokens,
                        provider_cache_write_tokens,
                    })
                    .await;
                let mut builder = with_qorx_headers(
                    Response::builder().status(status),
                    provider,
                    "miss",
                    report.raw_tokens,
                    report.compressed_tokens,
                );
                builder = with_cache_plan_headers(builder, &prompt_plan);
                if let Some(content_type) = content_type {
                    builder = builder.header("content-type", content_type);
                }
                return builder.body(axum::body::Body::from(body)).unwrap();
            }

            let (provider_cached_tokens, provider_cache_write_tokens) =
                provider_cache_tokens_from_headers(resp.headers());
            let _ = state
                .stats
                .record_request(RequestStats {
                    provider,
                    raw_prompt_tokens: report.raw_tokens,
                    compressed_prompt_tokens: report.compressed_tokens,
                    quarks_created: report.quarks_created,
                    upstream_error,
                    cache_lookup: false,
                    cache_hit: false,
                    provider_cached_prompt_tokens: provider_cached_tokens,
                    provider_cache_write_tokens,
                })
                .await;
            let mut builder = with_qorx_headers(
                Response::builder().status(status),
                provider,
                "stream",
                report.raw_tokens,
                report.compressed_tokens,
            );
            builder = with_cache_plan_headers(builder, &prompt_plan);
            for (name, value) in resp.headers().iter() {
                let name_text = name.as_str();
                if name_text.eq_ignore_ascii_case("content-length")
                    || name_text.eq_ignore_ascii_case("transfer-encoding")
                    || name_text.eq_ignore_ascii_case("content-encoding")
                {
                    continue;
                }
                builder = builder.header(name, value);
            }
            builder
                .body(axum::body::Body::from_stream(resp.bytes_stream().map(
                    move |chunk| {
                        let _keep_processing_guard_alive = &processing_guard;
                        chunk
                    },
                )))
                .unwrap()
        }
        Err(err) => {
            let _ = state
                .stats
                .record_request(RequestStats {
                    provider,
                    raw_prompt_tokens: report.raw_tokens,
                    compressed_prompt_tokens: report.compressed_tokens,
                    quarks_created: report.quarks_created,
                    upstream_error: true,
                    cache_lookup,
                    cache_hit: false,
                    provider_cached_prompt_tokens: 0,
                    provider_cache_write_tokens: 0,
                })
                .await;
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({
                    "error": "qorx_upstream_error",
                    "message": err.to_string(),
                })),
            )
                .into_response()
        }
    }
}

fn with_qorx_headers(
    builder: Builder,
    provider: &str,
    cache: &str,
    raw_tokens: u64,
    compressed_tokens: u64,
) -> Builder {
    builder
        .header("x-qorx-proxy", "1")
        .header("x-qorx-provider", provider)
        .header("x-qorx-cache", cache)
        .header("x-qorx-raw-prompt-tokens", raw_tokens.to_string())
        .header(
            "x-qorx-compressed-prompt-tokens",
            compressed_tokens.to_string(),
        )
        .header(
            "x-qorx-saved-prompt-tokens",
            raw_tokens.saturating_sub(compressed_tokens).to_string(),
        )
        .header("x-qorx-cost-stack", cost_stack::RUN_HEADER_VALUE)
        .header("x-qorx-cost-stages", cost_stack::HEADER_STAGES)
}

fn with_cache_plan_headers(builder: Builder, plan: &cache_plan::CachePlan) -> Builder {
    builder
        .header("x-qorx-cache-plan", "background")
        .header("x-qorx-cache-plan-marker", plan.marker.as_str())
        .header(
            "x-qorx-cacheable-prefix-tokens",
            plan.estimated_cacheable_tokens.to_string(),
        )
        .header(
            "x-qorx-dynamic-tail-tokens",
            plan.dynamic_tail_tokens.to_string(),
        )
        .header(
            "x-qorx-provider-cache-floor-met",
            plan.provider_cache_floor_met.to_string(),
        )
}

fn add_qorx_headers(
    headers: &mut HeaderMap,
    provider: &str,
    cache: &str,
    raw_tokens: u64,
    compressed_tokens: u64,
) {
    for (name, value) in [
        ("x-qorx-proxy", "1".to_string()),
        ("x-qorx-provider", provider.to_string()),
        ("x-qorx-cache", cache.to_string()),
        ("x-qorx-raw-prompt-tokens", raw_tokens.to_string()),
        (
            "x-qorx-compressed-prompt-tokens",
            compressed_tokens.to_string(),
        ),
        (
            "x-qorx-saved-prompt-tokens",
            raw_tokens.saturating_sub(compressed_tokens).to_string(),
        ),
        (
            "x-qorx-cost-stack",
            cost_stack::RUN_HEADER_VALUE.to_string(),
        ),
        ("x-qorx-cost-stages", cost_stack::HEADER_STAGES.to_string()),
    ] {
        if let Ok(value) = HeaderValue::from_str(&value) {
            headers.insert(name, value);
        }
    }
}

fn add_cache_plan_headers(headers: &mut HeaderMap, plan: &cache_plan::CachePlan) {
    for (name, value) in [
        ("x-qorx-cache-plan", "background".to_string()),
        ("x-qorx-cache-plan-marker", plan.marker.clone()),
        (
            "x-qorx-cacheable-prefix-tokens",
            plan.estimated_cacheable_tokens.to_string(),
        ),
        (
            "x-qorx-dynamic-tail-tokens",
            plan.dynamic_tail_tokens.to_string(),
        ),
        (
            "x-qorx-provider-cache-floor-met",
            plan.provider_cache_floor_met.to_string(),
        ),
    ] {
        if let Ok(value) = HeaderValue::from_str(&value) {
            headers.insert(name, value);
        }
    }
}

fn should_forward_request_header(name: &str) -> bool {
    !matches!(
        name.to_ascii_lowercase().as_str(),
        "host" | "content-length" | "accept-encoding"
    )
}

fn request_prompt_surface(body: &[u8]) -> String {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return String::from_utf8_lossy(body).into_owned();
    };
    let mut parts = Vec::new();
    collect_prompt_strings(&value, &mut parts);
    if parts.is_empty() {
        serde_json::to_string(&value).unwrap_or_else(|_| String::from_utf8_lossy(body).into_owned())
    } else {
        parts.join("\n\n")
    }
}

fn collect_prompt_strings(value: &serde_json::Value, parts: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                if let Some(text) = child.as_str() {
                    if is_prompt_text_key(key) {
                        parts.push(text.to_string());
                    }
                } else {
                    collect_prompt_strings(child, parts);
                }
            }
        }
        serde_json::Value::Array(items) => {
            for child in items {
                collect_prompt_strings(child, parts);
            }
        }
        _ => {}
    }
}

fn is_prompt_text_key(key: &str) -> bool {
    matches!(
        key,
        "content" | "text" | "input" | "prompt" | "system" | "developer" | "instructions"
    )
}

fn provider_cache_tokens(body: &[u8]) -> (u64, u64) {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return (0, 0);
    };

    let read = sum_numeric_keys(
        &value,
        &[
            "cached_tokens",
            "cache_read_input_tokens",
            "cached_content_token_count",
            "cachedcontenttokencount",
        ],
    );
    let direct_write = sum_numeric_keys(&value, &["cache_creation_input_tokens"]);
    let ephemeral_write = sum_numeric_keys(
        &value,
        &["ephemeral_5m_input_tokens", "ephemeral_1h_input_tokens"],
    );
    let write = if direct_write > 0 {
        direct_write
    } else {
        ephemeral_write
    };
    (read, write)
}

fn provider_cache_tokens_from_headers(headers: &reqwest::header::HeaderMap) -> (u64, u64) {
    let read = sum_header_values(
        headers,
        &[
            "x-provider-cached-tokens",
            "x-provider-cache-read-input-tokens",
            "x-openai-cached-tokens",
            "anthropic-cache-read-input-tokens",
            "cache-read-input-tokens",
        ],
    );
    let write = sum_header_values(
        headers,
        &[
            "x-provider-cache-write-input-tokens",
            "anthropic-cache-creation-input-tokens",
            "cache-creation-input-tokens",
        ],
    );
    (read, write)
}

fn sum_header_values(headers: &reqwest::header::HeaderMap, names: &[&str]) -> u64 {
    names
        .iter()
        .filter_map(|name| headers.get(*name))
        .filter_map(|value| value.to_str().ok())
        .filter_map(|value| value.trim().parse::<u64>().ok())
        .sum()
}

fn sum_numeric_keys(value: &serde_json::Value, wanted: &[&str]) -> u64 {
    match value {
        serde_json::Value::Object(map) => map
            .iter()
            .map(|(key, child)| {
                let normalized = normalize_metric_key(key);
                let own = if wanted.iter().any(|wanted| *wanted == normalized) {
                    child.as_u64().unwrap_or(0)
                } else {
                    0
                };
                own + sum_numeric_keys(child, wanted)
            })
            .sum(),
        serde_json::Value::Array(items) => items
            .iter()
            .map(|child| sum_numeric_keys(child, wanted))
            .sum(),
        _ => 0,
    }
}

fn normalize_metric_key(key: &str) -> String {
    key.chars()
        .filter(|ch| ch.is_alphanumeric() || *ch == '_')
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        time::{SystemTime, UNIX_EPOCH},
    };
    use tokio::time::{sleep, Duration};

    #[test]
    fn parses_provider_cached_token_fields() {
        let openai = br#"{"usage":{"prompt_tokens_details":{"cached_tokens":1920}}}"#;
        assert_eq!(provider_cache_tokens(openai), (1920, 0));

        let anthropic = br#"{"usage":{"cache_read_input_tokens":1800,"cache_creation_input_tokens":248,"cache_creation":{"ephemeral_5m_input_tokens":456}}}"#;
        assert_eq!(provider_cache_tokens(anthropic), (1800, 248));

        let gemini = br#"{"usageMetadata":{"cachedContentTokenCount":100000}}"#;
        assert_eq!(provider_cache_tokens(gemini), (100000, 0));
    }

    #[test]
    fn parses_provider_cached_token_headers_for_streaming_routes() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("x-provider-cached-tokens", "123".parse().unwrap());
        headers.insert(
            "anthropic-cache-creation-input-tokens",
            "45".parse().unwrap(),
        );

        assert_eq!(provider_cache_tokens_from_headers(&headers), (123, 45));
    }

    #[test]
    fn qorx_headers_mark_routed_provider_savings() {
        let mut response = Response::builder()
            .status(StatusCode::OK)
            .body(axum::body::Body::empty())
            .unwrap();

        add_qorx_headers(response.headers_mut(), "openai", "hit", 1000, 125);
        let plan = cache_plan::plan_prompt("stable system prefix\n--- QORX_DYNAMIC ---\nuser turn");
        add_cache_plan_headers(response.headers_mut(), &plan);

        assert_eq!(response.headers()["x-qorx-proxy"], "1");
        assert_eq!(response.headers()["x-qorx-provider"], "openai");
        assert_eq!(response.headers()["x-qorx-cache"], "hit");
        assert_eq!(response.headers()["x-qorx-raw-prompt-tokens"], "1000");
        assert_eq!(response.headers()["x-qorx-compressed-prompt-tokens"], "125");
        assert_eq!(response.headers()["x-qorx-saved-prompt-tokens"], "875");
        assert_eq!(response.headers()["x-qorx-cost-stack"], "qosm=core");
        assert_eq!(response.headers()["x-qorx-cache-plan"], "background");
        assert_eq!(
            response.headers()["x-qorx-cache-plan-marker"],
            "--- QORX_DYNAMIC ---"
        );
        assert_eq!(
            response.headers()["x-qorx-provider-cache-floor-met"],
            "false"
        );
        let stages = response.headers()["x-qorx-cost-stages"]
            .to_str()
            .expect("cost stages header");
        assert!(stages.contains("b2c_quant_allocator"));
        assert!(stages.contains("quark_compress"));
        assert!(stages.contains("exact_replay_cache"));
        assert!(stages.contains("provider_cache_accounting"));
    }

    #[test]
    fn drops_compression_request_headers_before_upstream() {
        assert!(!should_forward_request_header("host"));
        assert!(!should_forward_request_header("content-length"));
        assert!(!should_forward_request_header("accept-encoding"));
        assert!(!should_forward_request_header("Accept-Encoding"));
        assert!(should_forward_request_header("authorization"));
        assert!(should_forward_request_header("content-type"));
    }

    #[tokio::test]
    async fn canonical_response_cache_hits_every_warmed_equivalent_request() {
        let root = unique_temp_dir("qorx-proxy-cache");
        fs::create_dir_all(&root).expect("create temp root");
        let paths = test_paths(&root.join("qorx-home"));
        fs::create_dir_all(&paths.data_dir).expect("create qorx home");
        fs::create_dir_all(&paths.shim_dir).expect("create shim dir");

        let upstream_calls = Arc::new(AtomicUsize::new(0));
        let (upstream_url, upstream_task) = spawn_fake_upstream(upstream_calls.clone()).await;
        let qorx_bind = free_bind().await;
        let qorx_url = format!("http://{qorx_bind}");
        let qorx_task = tokio::spawn(run_gateway(
            paths.clone(),
            ProxyConfig {
                bind: qorx_bind,
                openai_upstream: upstream_url,
                anthropic_upstream: "http://127.0.0.1:9".to_string(),
                gemini_upstream: "http://127.0.0.1:9".to_string(),
            },
        ));
        wait_for_health(&qorx_url).await;

        let client = Client::new();
        let body_a = r#"{"stream":false,"model":"qorx-test","messages":[{"role":"user","content":"cache me"}]}"#;
        let body_b = r#"{
          "messages": [{"content": "cache me", "role": "user"}],
          "model": "qorx-test",
          "stream": false
        }"#;

        let first = post_json_text(&client, &qorx_url, body_a).await;
        let second = post_json_text(&client, &qorx_url, body_b).await;
        let third = post_json_text(&client, &qorx_url, body_a).await;
        let stats: serde_json::Value = client
            .get(format!("{qorx_url}/stats"))
            .send()
            .await
            .expect("stats response")
            .json()
            .await
            .expect("stats json");

        assert_eq!(first, "miss");
        assert_eq!(second, "hit");
        assert_eq!(third, "hit");
        assert_eq!(
            upstream_calls.load(Ordering::SeqCst),
            1,
            "warmed equivalent requests must not call upstream again"
        );
        assert_eq!(stats["requests"], 3);
        assert_eq!(stats["cache_lookups"], 3);
        assert_eq!(stats["cache_hits"], 2);
        assert_eq!(stats["cache_misses"], 1);
        assert!(stats["cache_lookup_hit_rate_percent"].as_f64().unwrap() > 66.0);

        qorx_task.abort();
        upstream_task.abort();
        let _ = fs::remove_dir_all(root);
    }

    async fn post_json_text(client: &Client, base: &str, body: &str) -> String {
        let response = client
            .post(format!("{base}/v1/chat/completions"))
            .header("content-type", "application/json")
            .body(body.to_string())
            .send()
            .await
            .expect("proxy response");
        assert!(response.status().is_success());
        response
            .headers()
            .get("x-qorx-cache")
            .expect("cache header")
            .to_str()
            .expect("cache header text")
            .to_string()
    }

    async fn spawn_fake_upstream(calls: Arc<AtomicUsize>) -> (String, tokio::task::JoinHandle<()>) {
        async fn handler(
            State(calls): State<Arc<AtomicUsize>>,
        ) -> ([(String, String); 1], Json<serde_json::Value>) {
            let call = calls.fetch_add(1, Ordering::SeqCst) + 1;
            (
                [("content-type".to_string(), "application/json".to_string())],
                Json(serde_json::json!({
                    "id": format!("fake-{call}"),
                    "choices": [{"message": {"role": "assistant", "content": "cached upstream response"}}],
                    "usage": {"prompt_tokens": 100, "completion_tokens": 5}
                })),
            )
        }

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind fake upstream");
        let addr = listener.local_addr().expect("fake upstream addr");
        let app = Router::new()
            .route("/*path", any(handler))
            .with_state(calls);
        let task = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("fake upstream serve");
        });
        (format!("http://{addr}"), task)
    }

    async fn wait_for_health(base: &str) {
        let client = Client::new();
        for _ in 0..40 {
            if client
                .get(format!("{base}/health"))
                .send()
                .await
                .is_ok_and(|response| response.status().is_success())
            {
                return;
            }
            sleep(Duration::from_millis(50)).await;
        }
        panic!("qorx gateway did not become healthy at {base}");
    }

    async fn free_bind() -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind free port");
        let addr = listener.local_addr().expect("free addr");
        drop(listener);
        addr.to_string()
    }

    fn test_paths(data_dir: &Path) -> AppPaths {
        AppPaths {
            data_dir: data_dir.to_path_buf(),
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

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{}-{suffix}", std::process::id()))
    }
}
