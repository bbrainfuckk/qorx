mod a2a;
mod adapters;
mod aim;
mod atlas;
mod b2c_quant;
mod cache_plan;
mod capsule;
mod cli;
mod compression;
mod config;
mod context_proto;
mod context_vm;
mod cosmos;
mod cost_stack;
mod crux;
mod demo;
mod drive;
mod graph_view;
mod grounding;
mod hot;
mod impact;
mod index;
mod integrations;
mod judge;
mod kv;
mod lattice;
mod lexicon;
mod mcp;
mod memory;
mod money;
mod orcl;
mod proto_store;
mod proxy;
mod qorx;
mod response_cache;
mod security;
mod session;
mod share;
mod squeeze;
mod stats;
mod text;
#[cfg(windows)]
mod tray;
#[cfg(not(windows))]
mod tray {
    use anyhow::Result;

    use crate::stats::Stats;

    pub fn run_tray(_snapshot: Stats) -> Result<()> {
        anyhow::bail!("qorx tray is Windows-only in this release; use `qorx daemon` and `qorx stats` on this platform")
    }
}
mod truth;
mod version;

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::Serialize;
use std::{
    fs,
    io::{self, Write},
    net::SocketAddr,
    path::PathBuf,
    thread,
    time::Duration,
};

use crate::{
    config::{AppPaths, ProxyConfig},
    stats::StatsStore,
};

#[derive(Debug, Parser)]
#[command(
    name = "qorx",
    version = "0.0.1-ylem",
    about = "Qorx CLI: local context and proof for AI agents",
    long_about = "Qorx CLI runs on this computer. Version 0.0.1-ylem keeps repeated workspace context local, connects supported agents, and returns cited proof only when a task needs it.",
    after_help = "START HERE:
  qorx doctor                     Check the local install
  qorx daemon start               Start the local gateway
  qorx install -p codex           Connect Codex
  qorx integrate status           Show connected agents
  qorx man                        Plain-language manual

WORK WITH A PROJECT:
  qorx index .                    Index this folder
  qorx atlas                      See the local file map
  qorx map \"question\"             Find relevant local files
  qorx strict-answer \"question\"   Answer only from local proof

SHORTCUTS:
  qorx -i                         Same as: qorx install
  qorx -i -p codex                Same as: qorx install --platform codex
  qorx -in -p codex               Same as: qorx integrate activate --platform codex"
)]
struct Args {
    #[command(subcommand)]
    command: Option<CommandKind>,
}

#[derive(Debug, Subcommand)]
enum CommandKind {
    /// First-run setup for the local Qorx runtime.
    Bootstrap {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        path: Option<PathBuf>,
        #[arg(long)]
        no_integrations: bool,
    },
    /// Start, stop, or check the local HTTP gateway.
    Daemon {
        #[command(subcommand)]
        action: Option<DaemonAction>,
    },
    /// Check whether the CLI, daemon, paths, and config look healthy.
    Doctor {
        #[arg(long)]
        json: bool,
    },
    /// Show the 24-hour demo status.
    Demo {
        #[arg(long)]
        json: bool,
    },
    /// Open the Windows tray controller.
    Tray,
    /// Show or reset local counters.
    Stats {
        #[command(subcommand)]
        action: Option<StatsAction>,
    },
    /// Compare a claimed saving against Qorx local accounting.
    Money {
        #[arg(long = "claim-usd")]
        claim_usd: Option<f64>,
    },
    /// Search the local index.
    Search {
        query: String,
        #[arg(short, long, default_value_t = 5)]
        limit: usize,
    },
    /// Print the local file-and-reference map as JSON.
    Graph {
        #[arg(long)]
        query: Option<String>,
        #[arg(short, long, default_value_t = 96)]
        limit: usize,
    },
    /// Show the readable workspace map.
    Atlas {
        #[arg(short, long, default_value_t = 96)]
        limit: usize,
        #[command(subcommand)]
        action: Option<AtlasAction>,
    },
    /// Find a route between two local files.
    GraphPath {
        source: String,
        target: String,
        #[arg(short, long, default_value_t = 128)]
        limit: usize,
    },
    /// Answer from cited local evidence only.
    StrictAnswer {
        question: String,
        #[arg(short, long, default_value_t = 2)]
        limit: usize,
    },
    /// Return a small evidence pack for a question.
    Squeeze {
        query: String,
        #[arg(short = 'b', long, default_value_t = 900)]
        budget_tokens: u64,
        #[arg(short, long, default_value_t = 4)]
        limit: usize,
    },
    /// Check whether answer text is supported by local evidence.
    Judge {
        answer: String,
        #[arg(short, long)]
        query: Option<String>,
    },
    /// Run the proof gate: evidence, answer check, and savings math.
    Ground {
        query: String,
        #[arg(long)]
        answer: Option<String>,
        #[arg(short = 'b', long, default_value_t = 900)]
        budget_tokens: u64,
        #[arg(short, long, default_value_t = 4)]
        limit: usize,
        #[arg(long = "raw-tokens")]
        raw_tokens: Option<u64>,
        #[arg(long = "sent-tokens")]
        sent_tokens: Option<u64>,
        #[arg(long = "input-usd-per-million")]
        input_usd_per_million: Option<f64>,
    },
    /// Explain how stable input could be cached.
    CachePlan { prompt: String },
    #[command(name = "b2c-plan")]
    /// Plan the smallest useful context pack for a task.
    B2cPlan {
        query: String,
        #[arg(short = 'b', long, default_value_t = 900)]
        budget_tokens: u64,
        #[arg(long)]
        diff: Option<String>,
        #[arg(long = "diff-file")]
        diff_file: Option<PathBuf>,
    },
    /// Build a compact task context for an agent.
    Agent {
        objective: String,
        #[arg(short = 'b', long, default_value_t = 900)]
        budget_tokens: u64,
    },
    /// Owner-machine task context shortcut.
    Marvin {
        objective: String,
        #[arg(short = 'b', long, default_value_t = 900)]
        budget_tokens: u64,
    },
    /// Pack local context for a question.
    Pack {
        query: String,
        #[arg(short = 'b', long, default_value_t = 4_000)]
        budget_tokens: u64,
    },
    /// Show what local files a change may affect.
    Impact {
        query: String,
        #[arg(short = 'b', long, default_value_t = 4_000)]
        budget_tokens: u64,
        #[arg(long)]
        diff: Option<String>,
        #[arg(long = "diff-file")]
        diff_file: Option<PathBuf>,
    },
    /// Map a question to local files, symbols, and references.
    Map {
        query: String,
        #[arg(short = 'b', long, default_value_t = 900)]
        budget_tokens: u64,
        #[arg(long)]
        diff: Option<String>,
        #[arg(long = "diff-file")]
        diff_file: Option<PathBuf>,
    },
    /// Return ranked local evidence contracts.
    Orcl {
        query: String,
        #[arg(short = 'b', long, default_value_t = 900)]
        budget_tokens: u64,
        #[arg(long, default_value_t = 2)]
        depth: usize,
        #[arg(short, long, default_value_t = 8)]
        limit: usize,
        #[arg(long)]
        diff: Option<String>,
        #[arg(long = "diff-file")]
        diff_file: Option<PathBuf>,
    },
    /// Run a .qorx task file.
    Qorx { file: PathBuf },
    /// Compile a .qorx file to .qorxb.
    QorxCompile {
        input: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Validate a .qorx file.
    QorxCheck { input: PathBuf },
    /// Inspect a compiled .qorxb file.
    QorxInspect { file: PathBuf },
    /// Render the agent-facing text from a .qorx file.
    QorxPrompt {
        file: PathBuf,
        #[arg(long)]
        block: bool,
    },
    /// Agent-to-agent protocol helpers.
    A2a {
        #[command(subcommand)]
        action: A2aAction,
    },
    /// Compatibility status for older local state.
    Cosmos {
        #[command(subcommand)]
        action: CosmosAction,
    },
    /// Print the plain-language manual.
    Man { topic: Option<String> },
    /// Print the public Qorx glossary.
    Lexicon,
    /// Store, summarize, and clean local memory notes.
    Memory {
        #[command(subcommand)]
        action: MemoryAction,
    },
    /// Formal lattice checks for local memory rules.
    Lattice {
        #[command(subcommand)]
        action: LatticeAction,
    },
    /// Import and export portable Qorx sessions.
    Share {
        #[command(subcommand)]
        action: ShareAction,
    },
    /// Local key-value storage helpers.
    Kv {
        #[command(subcommand)]
        action: KvAction,
    },
    /// Print a local attestation report.
    Attest {
        #[arg(long)]
        formal: bool,
        #[arg(long, default_value_t = 3)]
        level: u8,
    },
    /// Run a local benchmark pack.
    Bench {
        #[arg(short = 'b', long, default_value_t = 4_000)]
        budget_tokens: u64,
        queries: Vec<String>,
    },
    /// List optional adapters.
    Adapters,
    /// Print the scientific boundary and evidence notes.
    Science,
    /// Inspect AIM-compatible local context.
    Aim,
    /// Security attest and verify commands.
    Security {
        #[command(subcommand)]
        action: SecurityAction,
    },
    /// Hot-path local checks.
    Hot {
        #[command(subcommand)]
        action: HotAction,
    },
    /// Create and inspect portable project capsules.
    Capsule {
        #[command(subcommand)]
        action: CapsuleAction,
    },
    /// Create, expand, or verify local context handles.
    Context {
        #[command(subcommand)]
        action: ContextAction,
    },
    /// Print the current local session handle.
    Session {
        #[arg(long)]
        block: bool,
    },
    /// Manage startup registration.
    Startup {
        #[command(subcommand)]
        action: StartupAction,
    },
    /// Build portable runtime artifacts.
    Portable {
        #[command(subcommand)]
        action: PortableAction,
    },
    /// Local drive sync helpers.
    Drive {
        #[command(subcommand)]
        action: DriveAction,
    },
    /// Install Qorx and connect supported agents.
    Install {
        #[arg(short = 'p', long, default_value = "all")]
        platform: String,
    },
    /// Turn agent connectors on, off, or show status.
    Integrate {
        #[command(subcommand)]
        action: IntegrateAction,
    },
    /// Run a production-style local stress pass.
    Crux {
        #[command(subcommand)]
        action: CruxAction,
    },
    /// Run the MCP stdio server.
    Mcp,
    /// Index a folder for local evidence.
    Index { path: PathBuf },
    /// Run an agent provider through Qorx shims.
    Run {
        provider: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Ask an agent provider for a patch.
    Patch {
        provider: String,
        #[arg(long)]
        apply: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum AtlasAction {
    Export {
        #[arg(long, default_value = "qorx-atlas")]
        out: PathBuf,
        #[arg(short, long, default_value_t = 256)]
        limit: usize,
    },
    Query {
        query: String,
        #[arg(short, long, default_value_t = 96)]
        limit: usize,
    },
    Path {
        source: String,
        target: String,
        #[arg(short, long, default_value_t = 128)]
        limit: usize,
    },
    Merge {
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
        #[arg(long)]
        out: PathBuf,
    },
    Global {
        #[command(subcommand)]
        action: AtlasGlobalAction,
    },
    Hook {
        #[arg(long, default_value = "qorx-atlas")]
        out: PathBuf,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum AtlasGlobalAction {
    Add { atlas: PathBuf, name: String },
    List,
    Path,
}

#[derive(Debug, Clone, Subcommand)]
enum StartupAction {
    Enable,
    Disable,
    Status,
}

#[derive(Debug, Clone, Subcommand)]
enum DaemonAction {
    Run,
    Start,
    Stop,
    Status,
}

#[derive(Debug, Clone, Subcommand)]
enum StatsAction {
    Reset,
}

#[derive(Debug, Clone, Subcommand)]
enum A2aAction {
    Card,
    Task { file: PathBuf },
}

#[derive(Debug, Clone, Subcommand)]
enum CosmosAction {
    Status,
}

#[derive(Debug, Clone, Subcommand)]
enum MemoryAction {
    Create {
        kind: String,
        text: String,
    },
    Read {
        query: String,
        #[arg(short, long, default_value_t = 8)]
        limit: usize,
    },
    Update {
        id: String,
        text: String,
    },
    Delete {
        id: String,
    },
    Summarize {
        #[arg(short, long, default_value_t = 8)]
        limit: usize,
    },
    Prune {
        #[arg(long = "max-items", default_value_t = 64)]
        max_items: usize,
    },
    Gc {
        #[arg(long, default_value = "lattice")]
        strategy: String,
        #[arg(long = "max-items", default_value_t = 64)]
        max_items: usize,
    },
    Evolve {
        #[arg(long)]
        task: String,
        #[arg(short = 'b', long, default_value_t = 900)]
        budget_tokens: u64,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum LatticeAction {
    Build {
        #[arg(long)]
        task: String,
        #[arg(short = 'b', long, default_value_t = 900)]
        budget_tokens: u64,
    },
    Status,
    Attest {
        #[arg(long)]
        formal: bool,
    },
    KvHints {
        #[arg(long)]
        task: Option<String>,
    },
    EvolveRules {
        #[arg(long)]
        task: String,
    },
    Rules,
}

#[derive(Debug, Clone, Subcommand)]
enum ShareAction {
    Export {
        #[arg(long)]
        out: PathBuf,
    },
    Capsule {
        #[arg(long)]
        capsule: Option<String>,
        #[arg(long)]
        to: PathBuf,
    },
    Import {
        bundle: PathBuf,
    },
    Session {
        #[arg(long)]
        block: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum KvAction {
    Emit {
        #[arg(long)]
        model: String,
        #[arg(long)]
        task: Option<String>,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum PortableAction {
    Init,
    Status,
}

#[derive(Debug, Clone, Subcommand)]
enum DriveAction {
    Init {
        #[arg(short, long, default_value = "Q")]
        letter: String,
        #[arg(long)]
        ram: bool,
        #[arg(long, default_value = drive::DEFAULT_IMDISK_SIZE)]
        size: String,
    },
    Mount {
        #[arg(short, long, default_value = "Q")]
        letter: String,
        #[arg(long)]
        ram: bool,
        #[arg(long, default_value = drive::DEFAULT_IMDISK_SIZE)]
        size: String,
    },
    Unmount {
        #[arg(short, long, default_value = "Q")]
        letter: String,
    },
    Status {
        #[arg(short, long, default_value = "Q")]
        letter: String,
        #[arg(long)]
        ram: bool,
    },
    InstallStartup {
        #[arg(short, long, default_value = "Q")]
        letter: String,
        #[arg(long)]
        ram: bool,
        #[arg(long, default_value = drive::DEFAULT_IMDISK_SIZE)]
        size: String,
    },
    RemoveStartup {
        #[arg(short, long, default_value = "Q")]
        letter: String,
    },
    InstallImdisk {
        bundle: PathBuf,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum SecurityAction {
    Attest,
    Verify,
}

#[derive(Debug, Clone, Subcommand)]
enum HotAction {
    Status,
    Install {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(short, long, default_value = "Q")]
        letter: String,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum ContextAction {
    Snapshot,
    Verify,
    Vm {
        objective: String,
        #[arg(short = 'b', long, default_value_t = 900)]
        budget_tokens: u64,
        #[arg(short, long, default_value_t = 4)]
        limit: usize,
        #[arg(long)]
        block: bool,
    },
    Fault {
        query: String,
        #[arg(long)]
        handle: Option<String>,
        #[arg(short = 'b', long, default_value_t = 900)]
        budget_tokens: u64,
        #[arg(short, long, default_value_t = 4)]
        limit: usize,
    },
    Inject {
        objective: Option<String>,
        #[arg(short = 'b', long, default_value_t = 900)]
        budget_tokens: u64,
        #[arg(short, long, default_value_t = 4)]
        limit: usize,
        #[arg(long)]
        block: bool,
    },
    Nano {
        objective: Option<String>,
        #[arg(short = 'b', long, default_value_t = 900)]
        budget_tokens: u64,
        #[arg(short, long, default_value_t = 4)]
        limit: usize,
        #[arg(long)]
        block: bool,
    },
    Quetta {
        objective: Option<String>,
        #[arg(short = 'b', long, default_value_t = 900)]
        budget_tokens: u64,
        #[arg(short, long, default_value_t = 4)]
        limit: usize,
        #[arg(long)]
        block: bool,
    },
    Expand {
        carrier: String,
        #[arg(short = 'b', long, default_value_t = 900)]
        budget_tokens: u64,
        #[arg(short, long, default_value_t = 4)]
        limit: usize,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum CapsuleAction {
    Auto {
        #[arg(long)]
        block: bool,
        #[arg(long = "max-files")]
        max_files: Option<usize>,
    },
    Detect,
    Create {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        include_memory: bool,
        #[arg(long)]
        include_aim: bool,
        #[arg(long)]
        include_sensitive: bool,
        #[arg(long = "max-files")]
        max_files: Option<usize>,
        #[arg(long)]
        block: bool,
    },
    Session {
        #[arg(long)]
        block: bool,
    },
    StrictAnswer {
        question: String,
        #[arg(short, long, default_value_t = 2)]
        limit: usize,
    },
}

#[derive(Debug, Serialize)]
struct BootstrapReport {
    schema: String,
    message: String,
    gateway: String,
    dashboard: String,
    dashboard_opened: bool,
    integrations: integrations::IntegrationReport,
    #[serde(rename = "cosmos")]
    brvin: capsule::BrvinReport,
    prompt_block: String,
    next: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DoctorReport {
    schema: String,
    version: String,
    tier: String,
    shared_service_ready: bool,
    gateway_healthy: bool,
    bind: String,
    data_dir: String,
    index_present: bool,
    stats_present: bool,
    response_cache_present: bool,
    provenance_present: bool,
    package_surfaces: Vec<String>,
    production_checks: Vec<String>,
    shared_service_gaps: Vec<String>,
    boundary: String,
}

impl DoctorReport {
    async fn collect() -> Result<Self> {
        let paths = AppPaths::resolve()?;
        let config = ProxyConfig::default();
        let gateway_healthy = gateway_health_for_bind(&config.bind).await;
        Ok(Self {
            schema: "qorx.doctor.v1".to_string(),
            version: crate::version::QORX_VERSION.to_string(),
            tier: if demo::is_demo_mode() {
                "Qorx Void Demo 24-hour runtime".to_string()
            } else {
                format!("{} private runtime", crate::version::product_name())
            },
            shared_service_ready: false,
            gateway_healthy,
            bind: config.bind,
            data_dir: paths.data_dir.display().to_string(),
            index_present: paths.index_file.exists(),
            stats_present: paths.stats_file.exists(),
            response_cache_present: paths.response_cache_file.exists(),
            provenance_present: paths.provenance_file.exists(),
            package_surfaces: vec![
                "GitHub release binaries".to_string(),
                "cargo install --git".to_string(),
                "npm release tarball".to_string(),
                "Python wheel release asset".to_string(),
                "Homebrew/Linuxbrew tap".to_string(),
                "Scoop bucket".to_string(),
                "Dockerfile".to_string(),
                "systemd unit template".to_string(),
            ],
            production_checks: vec![
                "qorx --version".to_string(),
                "qorx index <repo>".to_string(),
                "qorx context verify".to_string(),
                "qorx security attest".to_string(),
                "qorx daemon with /health and /stats".to_string(),
                "scripts/smoke-gateway.ps1 or scripts/smoke-gateway.sh".to_string(),
            ],
            shared_service_gaps: vec![
                "No built-in multi-user authentication or authorization layer".to_string(),
                "No tenant isolation model".to_string(),
                "No published external load-test SLO".to_string(),
                "No managed upgrade or migration controller".to_string(),
            ],
            boundary: "Use Qorx as a local-first runtime, CLI, daemon, and internal service component. Put it behind your own auth, network policy, supervision, and backups before exposing it to multiple users or untrusted networks.".to_string(),
        })
    }

    fn print_human(&self) {
        println!("Qorx {}", self.version);
        println!("tier: {}", self.tier);
        println!("gateway_healthy: {}", self.gateway_healthy);
        println!("bind: {}", self.bind);
        println!("data_dir: {}", self.data_dir);
        println!("index_present: {}", self.index_present);
        println!("stats_present: {}", self.stats_present);
        println!("response_cache_present: {}", self.response_cache_present);
        println!("provenance_present: {}", self.provenance_present);
        println!("shared_service_ready: {}", self.shared_service_ready);
        println!("boundary: {}", self.boundary);
    }
}

async fn gateway_health_for_bind(bind: &str) -> bool {
    let Ok(addr) = bind.parse::<SocketAddr>() else {
        return false;
    };
    let host = if addr.ip().is_unspecified() {
        "127.0.0.1".to_string()
    } else if addr.is_ipv6() {
        format!("[{}]", addr.ip())
    } else {
        addr.ip().to_string()
    };
    reqwest::Client::new()
        .get(format!("http://{host}:{}/health", addr.port()))
        .timeout(Duration::from_millis(700))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

#[derive(Debug, Clone, Subcommand)]
enum IntegrateAction {
    Activate {
        #[arg(short = 'p', long, default_value = "all")]
        platform: String,
    },
    Deactivate,
    Status,
    Settings {
        #[arg(long)]
        automcp: Option<bool>,
        #[arg(long)]
        autohook: Option<bool>,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum CruxAction {
    Run {
        #[arg(long, default_value_t = 1.0)]
        hours: f64,
        #[arg(long, default_value_t = 60)]
        interval_seconds: u64,
        #[arg(long)]
        log: Option<PathBuf>,
    },
    Stop,
    Report,
    Rollback {
        #[arg(long)]
        checkpoint: PathBuf,
    },
    #[command(hide = true)]
    Worker {
        #[arg(long)]
        iterations: u64,
        #[arg(long, default_value_t = 60)]
        interval_seconds: u64,
        #[arg(long)]
        log: PathBuf,
        #[arg(long)]
        summary: PathBuf,
    },
}

fn parse_integration_platform(platform: &str) -> Result<integrations::IntegrationPlatform> {
    integrations::IntegrationPlatform::from_slug(platform).ok_or_else(|| {
        anyhow::anyhow!(
            "unknown platform `{}`; supported: all, windows, codex, claude, opencode, copilot, vscode, aider, claw, droid, trae, trae-cn, gemini, hermes, kiro, pi, cursor, antigravity",
            platform
        )
    })
}

fn local_dashboard_url() -> String {
    format!("{}/monitor", config::local_base())
}

fn should_open_dashboard_after_bootstrap(json: bool, no_integrations: bool) -> bool {
    !json && !no_integrations
}

fn open_local_dashboard() -> bool {
    open::that(local_dashboard_url()).is_ok()
}

fn expand_qorx_shortcuts<I, S>(args: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let raw = args.into_iter().map(Into::into).collect::<Vec<String>>();
    match raw.get(1).map(String::as_str) {
        Some("-i") | Some("-in") => {}
        _ => return raw,
    }

    let mut expanded = Vec::with_capacity(raw.len() + 1);
    for (index, arg) in raw.into_iter().enumerate() {
        if index == 0 {
            expanded.push(arg);
            continue;
        }
        match arg.as_str() {
            "-i" => expanded.push("install".to_string()),
            "-in" => {
                expanded.push("integrate".to_string());
                expanded.push("activate".to_string());
            }
            "-p" => expanded.push("--platform".to_string()),
            _ => expanded.push(arg),
        }
    }
    expanded
}

fn splash_text() -> String {
    format!(
        r#"   ____   ___  ____  __  __
  / __ \ / _ \|  _ \ \ \/ /
 | |  | | | | | |_) | \  /
 | |__| | |_| |  _ <  /  \
  \___\_\\___/|_| \_\/_/\_\

QORX CLI
Version: 0.0.1-ylem
Local context and proof for AI agents.
Runtime: {}.

Start here:
  qorx doctor
  qorx daemon start
  qorx install -p codex
  qorx integrate status

Learn:
  qorx --help
  qorx man
"#,
        crate::version::product_name()
    )
}

fn print_splash() {
    println!("{}", splash_text());
    #[cfg(windows)]
    if std::env::var_os("QORX_NO_SPLASH_PAUSE").is_none() {
        print!("Press Enter to exit...");
        let _ = io::stdout().flush();
        let mut line = String::new();
        let _ = io::stdin().read_line(&mut line);
    }
}

fn manual_text(topic: Option<&str>) -> String {
    match topic.unwrap_or("all").to_ascii_lowercase().as_str() {
        "install" | "integrate" | "automcp" | "autohook" => r#"Qorx CLI Manual - Connect agents
Version: 0.0.1-ylem

What this does:
  Qorx writes the local connector files it owns, starts the local gateway when
  needed, and lets supported agents ask this computer for proof.

Connect everything Qorx knows how to manage:
  qorx install
  qorx -i

Connect one agent:
  qorx install -p codex
  qorx -i -p codex
  qorx integrate activate -p codex
  qorx -in -p codex

Legacy example that still works:
  qorx -in -p antigravity

Check what is connected:
  qorx integrate status

Control the switches:
  qorx integrate settings --automcp true --autohook false
  qorx integrate settings --automcp false --autohook false

Open the monitor:
  http://127.0.0.1:47187/monitor

Plain meaning:
  MCP gives an agent a Qorx tool.
  Hooks prepare the start of a task where the client supports it.
  AutoMCP and AutoHook start on by default after first-run setup.
  turn them off from the local monitor, tray, or the settings command above.

Supported platforms:
  all, windows, codex, claude, opencode, copilot, vscode, aider, claw,
  droid, trae, trae-cn, gemini, hermes, kiro, pi, cursor, antigravity

Boundary:
  Qorx does not copy provider secrets. Each agent keeps its own login and auth.
  Some clients still require a restart or a manual enable step after Qorx writes
  the connector file.
"#
        .to_string(),
        "stats" | "counters" => r#"Qorx CLI Manual - Counters
Version: 0.0.1-ylem

Read the counters:
  qorx stats
  curl http://127.0.0.1:47187/stats

Reset saved context and estimated-cost counters:
  qorx stats reset
  curl -X POST http://127.0.0.1:47187/stats/reset

What the numbers mean:
  Kept here: context Qorx did not send upstream.
  Sent to AI: context that did go to the provider.
  Reduction: local estimate based on kept versus sent context.
  Avoided input cost: local estimate, not a provider invoice.

Boundary:
  Provider billing is decided by the provider. Qorx reports local accounting
  unless routed provider telemetry proves a billable outcome.
"#
        .to_string(),
        "daemon" | "server" => r#"Qorx CLI Manual - Local gateway
Version: 0.0.1-ylem

Run the gateway in the foreground:
  qorx daemon
  qorx daemon run

Workstation controls:
  qorx daemon start
  qorx daemon status
  qorx daemon stop

Default local URLs:
  http://127.0.0.1:47187/health
  http://127.0.0.1:47187/stats
  http://127.0.0.1:47187/monitor

Plain meaning:
  The daemon is the local process behind Qorx tools, the monitor, and the MCP
  server. If it is not running, agents cannot pull local proof.
"#
        .to_string(),
        "atlas" | "map" | "context" | "evidence" => r#"Qorx CLI Manual - Local evidence
Version: 0.0.1-ylem

Index a folder:
  qorx index .

See the workspace map:
  qorx atlas
  qorx atlas query "what should I read first?"
  qorx atlas path src\main.rs src\proxy.rs
  qorx atlas export --out qorx-atlas

Find useful files for a task:
  qorx map "change monitor wording"
  qorx orcl "where is the CLI manual?"

Answer with proof only:
  qorx strict-answer "what version is this repo on?"
  qorx ground "version proof" --answer "Qorx is on 0.0.1-ylem."

Create a local handoff for an agent:
  qorx context inject "fix CLI docs" --block
  qorx context nano "fix CLI docs" --block

Plain meaning:
  Atlas shows what Qorx can see.
  Map and ORCL find the local files worth reading.
  Strict-answer refuses when the local index cannot prove the answer.
  Context commands create small local handles instead of dumping a whole repo.
"#
        .to_string(),
        "crux" => r#"Qorx CLI Manual - Crux
Version: 0.0.1-ylem

Run a production-style local integration stress pass:
  qorx crux run --hours 1

Stop a background Crux run:
  qorx crux stop

Read the latest Crux run state:
  qorx crux report

Rollback configs from an integration checkpoint:
  qorx crux rollback --checkpoint <checkpoint-folder>

Crux checks daemon health, context handoff, MCP line JSON, MCP Content-Length
framing, Codex/Gemini configs, Antigravity configs, and integration
status.
"#
        .to_string(),
        "lexicon" | "terms" => r#"Qorx CLI Manual - Glossary
Version: 0.0.1-ylem

Print the Qorx glossary:
  qorx lexicon

The public glossary uses one hundred 3-character terms. Older wire labels such
as qosm and qshf may still appear in compatibility fields and saved handles.
"#
        .to_string(),
        _ => r#"Qorx CLI Manual
Version: 0.0.1-ylem

What Qorx is:
  Qorx CLI controls the local Qorx runtime. Qorx Void is the runtime. The CLI
  starts it, connects agents, indexes projects, and asks for local proof.

The product line:
  Qorx Ayie       owner-machine build
  Qorx Void       public local runtime
  Qorx Void Demo  24-hour trial

Pick what you want:
  Check Qorx         qorx doctor
  Start Qorx         qorx daemon start
  Connect Codex      qorx install -p codex
  See connectors     qorx integrate status
  Read a project     qorx index .
  See the map         qorx atlas
  Ask with proof      qorx strict-answer "question"
  Turn Qorx off       qorx integrate settings --automcp false --autohook false

One-minute path:
  qorx doctor
  qorx daemon start
  qorx install -p codex
  qorx integrate status
  open http://127.0.0.1:47187/monitor

Work with a project:
  qorx index .
  qorx atlas
  qorx map "what files matter for this change?"
  qorx strict-answer "what can the local repo prove?"

Connect agents:
  qorx install
  qorx install -p codex
  qorx -i -p codex
  qorx integrate activate -p codex
  qorx -in -p codex
  qorx -in -p antigravity
  qorx integrate status

Run the local gateway:
  qorx daemon start
  qorx daemon status
  qorx daemon stop
  qorx tray

Read counters:
  qorx stats
  qorx stats reset

Prove local claims:
  qorx strict-answer "question"
  qorx ground "question" --answer "claim to check"
  qorx orcl "question"
  qorx context nano "objective" --block

Inspect the workspace:
  qorx atlas
  qorx atlas query "what should I read first?"
  qorx atlas export --out qorx-atlas

Advanced language tools:
  qorx qorx <file.qorx>
  qorx qorx-check <file.qorx>
  qorx qorx-compile <file.qorx> --out <file.qorxb>
  qorx lexicon

Manual topics:
  qorx man install
  qorx man daemon
  qorx man stats
  qorx man atlas
  qorx man crux
  qorx man lexicon

Shortcuts:
  -i   install
  -in  integrate activate
  -p   platform

Boundary:
  A Qorx handle is not a file dump. The local runtime must resolve it before an
  agent can see evidence. Qorx does not hide provider billing or make security
  claims that were not verified on this machine.
"#
        .to_string(),
    }
}

pub fn main() -> Result<()> {
    thread::Builder::new()
        .name("qorx-main".to_string())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;
            runtime.block_on(async_main())
        })?
        .join()
        .map_err(|_| anyhow::anyhow!("qorx main thread panicked"))?
}

async fn async_main() -> Result<()> {
    let raw_args = std::env::args().collect::<Vec<_>>();
    let args = Args::parse_from(expand_qorx_shortcuts(raw_args));
    let Some(command) = args.command else {
        print_splash();
        return Ok(());
    };
    match command {
        CommandKind::Bootstrap {
            json,
            path,
            no_integrations,
        } => {
            let paths = AppPaths::resolve()?;
            if !no_integrations {
                let _ = cli::ensure_daemon().await;
            }
            let integrations = if no_integrations {
                integrations::report(&paths)?
            } else {
                integrations::activate_enabled(&paths).or_else(|_| integrations::report(&paths))?
            };
            let brvin = if let Some(path) = path {
                let capsule = capsule::create(
                    &paths,
                    &path,
                    capsule::CapsuleCreateOptions {
                        include_memory: true,
                        include_aim: true,
                        include_sensitive: false,
                        max_files: Some(1_000),
                    },
                )?;
                capsule::BrvinReport {
                    schema: "qorx.cosmos-capsule.v1".to_string(),
                    loaded: true,
                    message: "Qorx cosmos capsule is loaded".to_string(),
                    candidates: Vec::new(),
                    capsule,
                    next: vec![
                        "Use qorx capsule session --block to copy the tiny capsule prompt."
                            .to_string(),
                        "Use qorx capsule strict-answer <question> for evidence-only answers."
                            .to_string(),
                    ],
                    boundary:
                        "Manual bootstrap loads the selected folder as the active Qorx cosmos capsule."
                            .to_string(),
                }
            } else {
                capsule::create_auto(
                    &paths,
                    capsule::CapsuleCreateOptions {
                        include_memory: true,
                        include_aim: true,
                        include_sensitive: false,
                        max_files: Some(1_000),
                    },
                )?
            };
            let report = BootstrapReport {
                schema: "qorx.bootstrap.v1".to_string(),
                message: "Qorx cosmos capsule is loaded".to_string(),
                gateway: config::local_base(),
                dashboard: local_dashboard_url(),
                dashboard_opened: should_open_dashboard_after_bootstrap(json, no_integrations)
                    && open_local_dashboard(),
                prompt_block: brvin.capsule.prompt_block.clone(),
                integrations,
                brvin,
                next: vec![
                    "Paste or route the QORX_CAPSULE block into the active CLI session."
                        .to_string(),
                    "Check Auto-MCP and Auto-hook status with qorx integrate status.".to_string(),
                    "To point Qorx at another project/brain, run qorx bootstrap --path <folder>."
                        .to_string(),
                    "Tray users can choose Capsule: Auto-detect local context or Capsule: Choose folder."
                        .to_string(),
                ],
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("Qorx cosmos capsule is loaded");
                println!("gateway: {}", report.gateway);
                println!("dashboard: {}", report.dashboard);
                println!(
                    "capsule: {} ({} tokens -> {} visible, {:.2}x)",
                    report.brvin.capsule.handle,
                    report.brvin.capsule.indexed_tokens,
                    report.brvin.capsule.visible_tokens,
                    report.brvin.capsule.context_reduction_x
                );
                println!();
                println!("{}", report.prompt_block);
                println!();
                println!("Want to point Qorx at another capsule?");
                println!("qorx bootstrap --path <folder>");
                println!("qorx capsule create <folder> --include-memory --block");
            }
        }
        CommandKind::Daemon { action } => match action.unwrap_or(DaemonAction::Run) {
            DaemonAction::Run => {
                let paths = AppPaths::resolve()?;
                proxy::run_gateway(paths, ProxyConfig::default()).await?;
            }
            DaemonAction::Start => {
                let report = cli::start_daemon().await?;
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
            DaemonAction::Stop => {
                let report = cli::stop_daemon().await?;
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
            DaemonAction::Status => {
                let report = cli::daemon_status().await;
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
        },
        CommandKind::Doctor { json } => {
            let report = DoctorReport::collect().await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                report.print_human();
            }
        }
        CommandKind::Demo { json } => {
            let paths = AppPaths::resolve()?;
            let status = demo::status(&paths)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&status)?);
            } else {
                demo::print_status(&status);
            }
        }
        CommandKind::Tray => {
            let paths = AppPaths::resolve()?;
            cli::ensure_daemon().await?;
            let stats = StatsStore::load(&paths.stats_file)?;
            let snapshot = stats.snapshot().await;
            tray::run_tray(snapshot)?;
        }
        CommandKind::Stats { action } => {
            let paths = AppPaths::resolve()?;
            match action {
                Some(StatsAction::Reset) => cli::reset_stats(&paths).await?,
                None => cli::print_stats(&paths).await?,
            }
        }
        CommandKind::Money { claim_usd } => {
            let paths = AppPaths::resolve()?;
            let legacy = paths.stats_file.with_extension("json");
            let stats: stats::Stats =
                proto_store::load_or_default(&paths.stats_file, &[legacy.as_path()])?;
            let proof = money::build_money_proof(&stats, claim_usd);
            println!("{}", serde_json::to_string_pretty(&proof)?);
        }
        CommandKind::Search { query, limit } => {
            let paths = AppPaths::resolve()?;
            let index = index::load_index_with_live_overlay(&paths.index_file, &query)?;
            let hits = index::search_index(&index, &query, limit);
            println!("{}", serde_json::to_string_pretty(&hits)?);
        }
        CommandKind::Graph { query, limit } => {
            let paths = AppPaths::resolve()?;
            let index = match query.as_deref() {
                Some(query) => index::load_index_with_live_overlay(&paths.index_file, query)?,
                None => index::load_index(&paths.index_file)?,
            };
            let graph = match query.as_deref() {
                Some(query) => graph_view::build_query_graph(&index, query, limit),
                None => graph_view::build_dashboard_graph(&index, limit),
            };
            println!("{}", serde_json::to_string_pretty(&graph)?);
        }
        CommandKind::Atlas { limit, action } => {
            let paths = AppPaths::resolve()?;
            match action {
                None => {
                    let index = index::load_index(&paths.index_file)?;
                    let report = graph_view::build_atlas_report(&index, limit);
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
                Some(AtlasAction::Export { out, limit }) => {
                    let index = index::load_index(&paths.index_file)?;
                    let report = atlas::export_pack(&index, limit, &out)?;
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
                Some(AtlasAction::Query { query, limit }) => {
                    let index = index::load_index_with_live_overlay(&paths.index_file, &query)?;
                    let graph = graph_view::build_query_graph(&index, &query, limit);
                    println!("{}", serde_json::to_string_pretty(&graph)?);
                }
                Some(AtlasAction::Path {
                    source,
                    target,
                    limit,
                }) => {
                    let index = index::load_index(&paths.index_file)?;
                    let trace = graph_view::trace_file_path(&index, &source, &target, limit);
                    println!("{}", serde_json::to_string_pretty(&trace)?);
                }
                Some(AtlasAction::Merge { inputs, out }) => {
                    let report = atlas::merge_packs(&inputs, &out)?;
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
                Some(AtlasAction::Global { action }) => {
                    let report = match action {
                        AtlasGlobalAction::Add { atlas, name } => {
                            serde_json::to_value(atlas::add_global(&paths, &atlas, &name)?)?
                        }
                        AtlasGlobalAction::List => {
                            serde_json::to_value(atlas::load_global(&paths)?)?
                        }
                        AtlasGlobalAction::Path => serde_json::json!({
                            "schema": "qorx.atlas-global-path.v1",
                            "path": atlas::global_path(&paths).to_string_lossy()
                        }),
                    };
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
                Some(AtlasAction::Hook { out }) => {
                    let report = atlas::write_agent_instructions(&out)?;
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
            }
        }
        CommandKind::GraphPath {
            source,
            target,
            limit,
        } => {
            let paths = AppPaths::resolve()?;
            let index = index::load_index(&paths.index_file)?;
            let trace = graph_view::trace_file_path(&index, &source, &target, limit);
            println!("{}", serde_json::to_string_pretty(&trace)?);
        }
        CommandKind::StrictAnswer { question, limit } => {
            let paths = AppPaths::resolve()?;
            let index = index::load_index_with_live_overlay(&paths.index_file, &question)?;
            let answer = truth::strict_answer(&index, &question, limit);
            println!("{}", serde_json::to_string_pretty(&answer)?);
        }
        CommandKind::Squeeze {
            query,
            budget_tokens,
            limit,
        } => {
            let paths = AppPaths::resolve()?;
            let index = index::load_index_with_live_overlay(&paths.index_file, &query)?;
            let report = squeeze::squeeze_context(&index, &query, budget_tokens, limit);
            let _ = stats::record_context_pack(
                &paths.stats_file,
                report.indexed_tokens,
                report.used_tokens,
            );
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        CommandKind::Judge { answer, query } => {
            let paths = AppPaths::resolve()?;
            let index = match query.as_deref() {
                Some(query) => index::load_index_with_live_overlay(&paths.index_file, query)?,
                None => index::load_index(&paths.index_file)?,
            };
            let report = judge::judge_answer(&index, &answer, query.as_deref());
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        CommandKind::Ground {
            query,
            answer,
            budget_tokens,
            limit,
            raw_tokens,
            sent_tokens,
            input_usd_per_million,
        } => {
            let paths = AppPaths::resolve()?;
            let index = index::load_index_with_live_overlay(&paths.index_file, &query)?;
            let report = grounding::grounding_gate(
                &index,
                &query,
                grounding::GroundingOptions {
                    budget_tokens,
                    limit,
                    answer,
                    raw_tokens,
                    sent_tokens,
                    input_usd_per_million,
                },
            );
            let _ = stats::record_context_pack(
                &paths.stats_file,
                report.indexed_tokens,
                report.used_tokens,
            );
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        CommandKind::CachePlan { prompt } => {
            let report = cache_plan::plan_prompt(&prompt);
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        CommandKind::B2cPlan {
            query,
            budget_tokens,
            diff,
            diff_file,
        } => {
            let paths = AppPaths::resolve()?;
            let index = index::load_index_with_live_overlay(&paths.index_file, &query)?;
            let diff_text = match diff_file {
                Some(path) => Some(fs::read_to_string(path)?),
                None => diff,
            };
            let report = b2c_quant::plan_context_with_diff(
                &index,
                &query,
                budget_tokens,
                diff_text.as_deref(),
            );
            let _ = stats::record_context_pack(
                &paths.stats_file,
                report.indexed_tokens,
                report.used_tokens,
            );
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        CommandKind::Agent {
            objective,
            budget_tokens,
        }
        | CommandKind::Marvin {
            objective,
            budget_tokens,
        } => {
            let paths = AppPaths::resolve()?;
            let index = index::load_index_with_live_overlay(&paths.index_file, &objective)?;
            let report = truth::run_agent(&index, &objective, budget_tokens);
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        CommandKind::Pack {
            query,
            budget_tokens,
        } => {
            let paths = AppPaths::resolve()?;
            let index = index::load_index_with_live_overlay(&paths.index_file, &query)?;
            let packed = index::pack_context(&index, &query, budget_tokens);
            let _ = stats::record_context_pack(
                &paths.stats_file,
                packed.indexed_tokens,
                packed.used_tokens,
            );
            println!("{}", serde_json::to_string_pretty(&packed)?);
        }
        CommandKind::Impact {
            query,
            budget_tokens,
            diff,
            diff_file,
        } => {
            let paths = AppPaths::resolve()?;
            let index = index::load_index_with_live_overlay(&paths.index_file, &query)?;
            let diff_text = match diff_file {
                Some(path) => Some(fs::read_to_string(path)?),
                None => diff,
            };
            let packed =
                impact::impact_context(&index, &query, diff_text.as_deref(), budget_tokens);
            let _ = stats::record_context_pack(
                &paths.stats_file,
                packed.indexed_tokens,
                packed.used_tokens,
            );
            println!("{}", serde_json::to_string_pretty(&packed)?);
        }
        CommandKind::Map {
            query,
            budget_tokens,
            diff,
            diff_file,
        } => {
            let paths = AppPaths::resolve()?;
            let index = index::load_index_with_live_overlay(&paths.index_file, &query)?;
            let diff_text = match diff_file {
                Some(path) => Some(fs::read_to_string(path)?),
                None => diff,
            };
            let mapped = impact::map_context(&index, &query, diff_text.as_deref(), budget_tokens);
            let _ = stats::record_context_pack(
                &paths.stats_file,
                mapped.indexed_tokens,
                mapped.used_tokens,
            );
            println!("{}", serde_json::to_string_pretty(&mapped)?);
        }
        CommandKind::Orcl {
            query,
            budget_tokens,
            depth,
            limit,
            diff,
            diff_file,
        } => {
            let paths = AppPaths::resolve()?;
            let index = index::load_index_with_live_overlay(&paths.index_file, &query)?;
            let diff_text = match diff_file {
                Some(path) => Some(fs::read_to_string(path)?),
                None => diff,
            };
            let report = orcl::report(
                &index,
                &query,
                diff_text.as_deref(),
                orcl::OrclOptions {
                    budget_tokens,
                    depth,
                    limit,
                },
            );
            let _ = stats::record_context_pack(
                &paths.stats_file,
                report.indexed_tokens,
                report.used_tokens,
            );
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        CommandKind::Qorx { file } => {
            let paths = AppPaths::resolve()?;
            let index = index::load_index(&paths.index_file)?;
            let report = qorx::run_file(&file, &index)?;
            let cosmos = cosmos::record_run(&paths, "qorx.run", &report)?;
            let mut value = serde_json::to_value(&report)?;
            if let serde_json::Value::Object(map) = &mut value {
                map.insert(
                    "lexicon".to_string(),
                    lexicon::runtime_tags(&report.source_kind),
                );
                map.insert("cosmos".to_string(), serde_json::to_value(cosmos)?);
            }
            println!("{}", serde_json::to_string_pretty(&value)?);
        }
        CommandKind::QorxCompile { input, out } => {
            let report = qorx::compile_file(&input, out.as_deref())?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        CommandKind::QorxCheck { input } => {
            let report = qorx::check_file(&input)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        CommandKind::QorxInspect { file } => {
            let report = qorx::inspect_file(&file)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        CommandKind::QorxPrompt { file, block } => {
            let report = qorx::prompt_file(&file)?;
            if block {
                println!("{}", report.prompt_block);
            } else {
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
        }
        CommandKind::A2a { action } => match action {
            A2aAction::Card => {
                println!("{}", serde_json::to_string_pretty(&a2a::agent_card())?);
            }
            A2aAction::Task { file } => {
                let paths = AppPaths::resolve()?;
                let index = index::load_index(&paths.index_file)?;
                let response = a2a::task_from_file(&file, &index, Some(&paths))?;
                println!("{}", serde_json::to_string_pretty(&response)?);
            }
        },
        CommandKind::Cosmos { action } => match action {
            CosmosAction::Status => {
                let paths = AppPaths::resolve()?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&cosmos::status(&paths)?)?
                );
            }
        },
        CommandKind::Man { topic } => {
            println!("{}", manual_text(topic.as_deref()));
        }
        CommandKind::Lexicon => {
            println!("{}", serde_json::to_string_pretty(&lexicon::report())?);
        }
        CommandKind::Memory { action } => {
            let paths = AppPaths::resolve()?;
            let report = match action {
                MemoryAction::Create { kind, text } => {
                    serde_json::to_value(memory::create(&paths, &kind, &text)?)?
                }
                MemoryAction::Read { query, limit } => {
                    serde_json::to_value(memory::read(&paths, &query, limit)?)?
                }
                MemoryAction::Update { id, text } => {
                    serde_json::to_value(memory::update(&paths, &id, &text)?)?
                }
                MemoryAction::Delete { id } => serde_json::to_value(memory::delete(&paths, &id)?)?,
                MemoryAction::Summarize { limit } => {
                    serde_json::to_value(memory::summarize(&paths, limit)?)?
                }
                MemoryAction::Prune { max_items } => {
                    serde_json::to_value(memory::prune(&paths, max_items)?)?
                }
                MemoryAction::Gc {
                    strategy,
                    max_items,
                } => serde_json::to_value(memory::gc(&paths, &strategy, max_items)?)?,
                MemoryAction::Evolve {
                    task,
                    budget_tokens,
                } => serde_json::to_value(lattice::evolve(&paths, &task, budget_tokens)?)?,
            };
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        CommandKind::Lattice { action } => {
            let paths = AppPaths::resolve()?;
            let report = match action {
                LatticeAction::Build {
                    task,
                    budget_tokens,
                } => {
                    let lattice = lattice::build(&paths, &task, budget_tokens)?;
                    proto_store::save(&lattice::lattice_path(&paths), &lattice)?;
                    serde_json::to_value(lattice)?
                }
                LatticeAction::Status => match lattice::status(&paths) {
                    Ok(report) => serde_json::to_value(report)?,
                    Err(_) => return Err(lattice::missing_lattice_error()),
                },
                LatticeAction::Attest { formal } => {
                    serde_json::to_value(lattice::attest(&paths, formal)?)?
                }
                LatticeAction::KvHints { task } => {
                    serde_json::to_value(lattice::kv_hint_export(&paths, task.as_deref())?)?
                }
                LatticeAction::EvolveRules { task } => {
                    serde_json::to_value(lattice::evolve_rules(&paths, &task)?)?
                }
                LatticeAction::Rules => serde_json::to_value(lattice::load_rules(&paths)?)?,
            };
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        CommandKind::Share { action } => {
            let paths = AppPaths::resolve()?;
            match action {
                ShareAction::Export { out } => {
                    let report = share::export(&paths, &out)?;
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
                ShareAction::Capsule { capsule, to } => {
                    let report = share::export_capsule(&paths, capsule.as_deref(), &to)?;
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
                ShareAction::Import { bundle } => {
                    let report = share::import(&paths, &bundle)?;
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
                ShareAction::Session { block } => {
                    let report = share::session(&paths)?;
                    if block {
                        println!("{}", report.prompt_block);
                    } else {
                        println!("{}", serde_json::to_string_pretty(&report)?);
                    }
                }
            }
        }
        CommandKind::Kv { action } => {
            let paths = AppPaths::resolve()?;
            match action {
                KvAction::Emit { model, task, out } => {
                    let report = kv::emit(&paths, &model, task.as_deref(), out)?;
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
            }
        }
        CommandKind::Attest { formal, level } => {
            let paths = AppPaths::resolve()?;
            let report = lattice::formal_attest(&paths, formal, level)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        CommandKind::Bench {
            budget_tokens,
            queries,
        } => {
            let paths = AppPaths::resolve()?;
            let index = index::load_index(&paths.index_file)?;
            let queries = if queries.is_empty() {
                vec![
                    "provider cached tokens prompt cache".to_string(),
                    "repo quark pack context benchmark".to_string(),
                    "kv cache rotorquant adapter".to_string(),
                ]
            } else {
                queries
            };
            let report = index::benchmark_queries(&index, &queries, budget_tokens);
            for row in &report.rows {
                let _ = stats::record_context_pack(
                    &paths.stats_file,
                    report.indexed_tokens,
                    row.used_tokens,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        CommandKind::Adapters => {
            println!(
                "{}",
                serde_json::to_string_pretty(&adapters::adapter_report())?
            );
        }
        CommandKind::Science => {
            println!(
                "{}",
                serde_json::to_string_pretty(&adapters::science_report())?
            );
        }
        CommandKind::Aim => {
            println!(
                "{}",
                serde_json::to_string_pretty(&aim::inspect_default()?)?
            );
        }
        CommandKind::Security { action } => {
            let paths = AppPaths::resolve()?;
            let report = match action {
                SecurityAction::Attest => security::attest(&paths)?,
                SecurityAction::Verify => security::verify_saved(&paths)?,
            };
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        CommandKind::Hot { action } => {
            let report = match action {
                HotAction::Status => {
                    let paths = AppPaths::resolve()?;
                    serde_json::to_value(hot::status_from_disk(&paths)?)?
                }
                HotAction::Install { path, letter } => {
                    let portable = config::init_portable()?;
                    let paths = AppPaths::resolve()?;
                    let index = index::build_index(&path, &paths.index_file)?;
                    let drive = drive::init(&paths, &letter, &drive::DriveOptions::subst())?;
                    let ram_hot = hot::status_from_disk(&paths)?;
                    serde_json::json!({
                        "portable": portable,
                        "index": {
                            "quarks": index.atoms.len(),
                            "indexed_tokens": index.total_tokens(),
                            "symbols": index.atoms.iter().map(|atom| atom.symbols.len()).sum::<usize>(),
                            "signals": index.atoms.iter().map(|atom| atom.signal_mask.count_ones()).sum::<u32>(),
                            "sparse_vector_terms": index.vector_terms(),
                        },
                        "drive": drive,
                        "ram_hot": ram_hot,
                    })
                }
            };
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        CommandKind::Context { action } => {
            let paths = AppPaths::resolve()?;
            match action {
                ContextAction::Snapshot => {
                    let report = context_proto::snapshot(&paths)?;
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
                ContextAction::Verify => {
                    let report = context_proto::verify(&paths)?;
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
                ContextAction::Vm {
                    objective,
                    budget_tokens,
                    limit,
                    block,
                } => {
                    let index = index::load_index_with_live_overlay(&paths.index_file, &objective)?;
                    let report = context_vm::build_context_vm(
                        &index,
                        &objective,
                        context_vm::ContextVmOptions {
                            budget_tokens,
                            limit,
                        },
                    );
                    let _ = stats::record_context_pack(
                        &paths.stats_file,
                        report.ledger.indexed_tokens,
                        report.ledger.sent_tokens,
                    );
                    if block {
                        println!("{}", report.prompt_block);
                    } else {
                        println!("{}", serde_json::to_string_pretty(&report)?);
                    }
                }
                ContextAction::Fault {
                    query,
                    handle,
                    budget_tokens,
                    limit,
                } => {
                    let index = index::load_index_with_live_overlay(&paths.index_file, &query)?;
                    let session = session::build_session_pointer(&index);
                    let handle = handle.unwrap_or(session.handle);
                    let report = context_vm::resolve_context_fault(
                        &index,
                        &handle,
                        &query,
                        context_vm::ContextVmOptions {
                            budget_tokens,
                            limit,
                        },
                    );
                    let _ = stats::record_context_pack(
                        &paths.stats_file,
                        report.indexed_tokens,
                        report.used_tokens,
                    );
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
                ContextAction::Inject {
                    objective,
                    budget_tokens,
                    limit,
                    block,
                } => {
                    let objective = objective.unwrap_or_else(|| "current agent turn".to_string());
                    let index = index::load_index_with_live_overlay(&paths.index_file, &objective)?;
                    let report = context_vm::build_context_injection(
                        &index,
                        &objective,
                        context_vm::ContextVmOptions {
                            budget_tokens,
                            limit,
                        },
                    );
                    if block {
                        println!("{}", report.additional_context);
                    } else {
                        println!("{}", serde_json::to_string_pretty(&report)?);
                    }
                }
                ContextAction::Nano {
                    objective,
                    budget_tokens,
                    limit,
                    block,
                } => {
                    let objective = objective.unwrap_or_else(|| "current agent turn".to_string());
                    let index = index::load_index_with_live_overlay(&paths.index_file, &objective)?;
                    let report = context_vm::build_context_nano(
                        &index,
                        &objective,
                        context_vm::ContextVmOptions {
                            budget_tokens,
                            limit,
                        },
                    );
                    let _ = stats::record_context_pack(
                        &paths.stats_file,
                        report.indexed_tokens,
                        report.visible_tokens,
                    );
                    if block {
                        println!("{}", report.carrier);
                    } else {
                        println!("{}", serde_json::to_string_pretty(&report)?);
                    }
                }
                ContextAction::Quetta {
                    objective,
                    budget_tokens,
                    limit,
                    block,
                } => {
                    let objective = objective.unwrap_or_else(|| "current agent turn".to_string());
                    let index = index::load_index_with_live_overlay(&paths.index_file, &objective)?;
                    let report = context_vm::build_context_quetta(
                        &index,
                        &objective,
                        context_vm::ContextVmOptions {
                            budget_tokens,
                            limit,
                        },
                    );
                    let _ = stats::record_context_pack(
                        &paths.stats_file,
                        report.local_indexed_tokens,
                        report.visible_tokens,
                    );
                    if block {
                        println!("{}", report.carrier);
                    } else {
                        println!("{}", serde_json::to_string_pretty(&report)?);
                    }
                }
                ContextAction::Expand {
                    carrier,
                    budget_tokens,
                    limit,
                } => {
                    let index = index::load_index_with_live_overlay(&paths.index_file, &carrier)?;
                    let report = context_vm::expand_nano_carrier(
                        &index,
                        &carrier,
                        context_vm::ContextVmOptions {
                            budget_tokens,
                            limit,
                        },
                    );
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
            }
        }
        CommandKind::Capsule { action } => {
            let paths = AppPaths::resolve()?;
            match action {
                CapsuleAction::Auto { block, max_files } => {
                    let report = capsule::create_auto(
                        &paths,
                        capsule::CapsuleCreateOptions {
                            include_memory: true,
                            include_aim: true,
                            include_sensitive: false,
                            max_files,
                        },
                    )?;
                    if block {
                        println!("{}", report.capsule.prompt_block);
                    } else {
                        println!("{}", serde_json::to_string_pretty(&report)?);
                    }
                }
                CapsuleAction::Detect => {
                    let candidates = capsule::detect_brvin_candidates();
                    println!("{}", serde_json::to_string_pretty(&candidates)?);
                }
                CapsuleAction::Create {
                    path,
                    include_memory,
                    include_aim,
                    include_sensitive,
                    max_files,
                    block,
                } => {
                    let report = capsule::create(
                        &paths,
                        &path,
                        capsule::CapsuleCreateOptions {
                            include_memory,
                            include_aim,
                            include_sensitive,
                            max_files,
                        },
                    )?;
                    if block {
                        println!("{}", report.prompt_block);
                    } else {
                        println!("{}", serde_json::to_string_pretty(&report)?);
                    }
                }
                CapsuleAction::Session { block } => {
                    if block {
                        let report = capsule::load_session_pointer(&paths)?;
                        println!("{}", report.prompt_block);
                    } else {
                        let report = capsule::load(&paths)?;
                        println!("{}", serde_json::to_string_pretty(&report)?);
                    }
                }
                CapsuleAction::StrictAnswer { question, limit } => {
                    let report = capsule::strict_answer(&paths, &question, limit)?;
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
            }
        }
        CommandKind::Session { block } => {
            let paths = AppPaths::resolve()?;
            let index = index::load_index(&paths.index_file)?;
            let pointer = session::build_session_pointer(&index);
            if block {
                println!("{}", pointer.prompt_block);
            } else {
                println!("{}", serde_json::to_string_pretty(&pointer)?);
            }
        }
        CommandKind::Startup { action } => {
            let paths = AppPaths::resolve()?;
            let report = match action {
                StartupAction::Enable => {
                    let _ = integrations::install_autostart()?;
                    integrations::report(&paths)?
                }
                StartupAction::Disable => {
                    let _ = integrations::remove_autostart()?;
                    integrations::report(&paths)?
                }
                StartupAction::Status => integrations::report(&paths)?,
            };
            println!("{}", serde_json::to_string_pretty(&report.autostart)?);
        }
        CommandKind::Portable { action } => {
            let report = match action {
                PortableAction::Init => config::init_portable()?,
                PortableAction::Status => {
                    let paths = AppPaths::resolve()?;
                    config::portable_report(&paths)?
                }
            };
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        CommandKind::Drive { action } => {
            let report = match action {
                DriveAction::Init { letter, ram, size } => {
                    let paths = AppPaths::resolve_for_drive()?;
                    serde_json::to_value(drive::init(
                        &paths,
                        &letter,
                        &drive::DriveOptions { ram, size },
                    )?)?
                }
                DriveAction::Mount { letter, ram, size } => {
                    let paths = AppPaths::resolve_for_drive()?;
                    serde_json::to_value(drive::mount(
                        &paths,
                        &letter,
                        &drive::DriveOptions { ram, size },
                    )?)?
                }
                DriveAction::Unmount { letter } => {
                    let paths = AppPaths::resolve_for_drive()?;
                    serde_json::to_value(drive::unmount(&paths, &letter)?)?
                }
                DriveAction::Status { letter, ram } => {
                    let paths = AppPaths::resolve_for_drive()?;
                    serde_json::to_value(drive::status(&paths, &letter, ram)?)?
                }
                DriveAction::InstallStartup { letter, ram, size } => {
                    let paths = AppPaths::resolve_for_drive()?;
                    serde_json::to_value(drive::install_startup(
                        &paths,
                        &letter,
                        &drive::DriveOptions { ram, size },
                    )?)?
                }
                DriveAction::RemoveStartup { letter } => {
                    let paths = AppPaths::resolve_for_drive()?;
                    serde_json::to_value(drive::remove_startup(&paths, &letter)?)?
                }
                DriveAction::InstallImdisk { bundle } => {
                    serde_json::to_value(drive::install_imdisk(&bundle)?)?
                }
            };
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        CommandKind::Install { platform } => {
            let paths = AppPaths::resolve()?;
            let platform = parse_integration_platform(&platform)?;
            let report = integrations::install_platform(&paths, platform)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        CommandKind::Integrate { action } => {
            let paths = AppPaths::resolve()?;
            let report = match action {
                IntegrateAction::Activate { platform } => {
                    let platform = parse_integration_platform(&platform)?;
                    integrations::install_platform(&paths, platform)?
                }
                IntegrateAction::Deactivate => integrations::deactivate_all(&paths)?,
                IntegrateAction::Status => integrations::report(&paths)?,
                IntegrateAction::Settings { automcp, autohook } => {
                    if automcp.is_none() && autohook.is_none() {
                        integrations::report(&paths)?
                    } else {
                        let mut settings = integrations::load_settings(&paths)?;
                        if let Some(enabled) = automcp {
                            settings.automcp_enabled = enabled;
                        }
                        if let Some(enabled) = autohook {
                            settings.autohook_enabled = enabled;
                        }
                        integrations::set_settings(&paths, settings)?
                    }
                }
            };
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        CommandKind::Crux { action } => {
            let paths = AppPaths::resolve()?;
            match action {
                CruxAction::Run {
                    hours,
                    interval_seconds,
                    log,
                } => {
                    let report = crux::start(&paths, hours, interval_seconds, log)?;
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
                CruxAction::Stop => {
                    let report = crux::stop(&paths)?;
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
                CruxAction::Report => {
                    let report = crux::report(&paths)?;
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
                CruxAction::Rollback { checkpoint } => {
                    let report = crux::rollback(&checkpoint)?;
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
                CruxAction::Worker {
                    iterations,
                    interval_seconds,
                    log,
                    summary,
                } => {
                    crux::worker(iterations, interval_seconds, log, summary).await?;
                }
            }
        }
        CommandKind::Mcp => {
            mcp::run_stdio().await?;
        }
        CommandKind::Index { path } => {
            let paths = AppPaths::resolve()?;
            let index = index::build_index(&path, &paths.index_file)?;
            let symbol_count: usize = index.atoms.iter().map(|atom| atom.symbols.len()).sum();
            let signal_count: u32 = index
                .atoms
                .iter()
                .map(|atom| atom.signal_mask.count_ones())
                .sum();
            println!(
                "Indexed {} quarks from {} into {} ({} estimated tokens, {} symbols, {} signals, {} sparse vector terms)",
                index.atoms.len(),
                index.root,
                paths.index_file.display(),
                index.total_tokens(),
                symbol_count,
                signal_count,
                index.vector_terms()
            );
        }
        CommandKind::Run { provider, args } => {
            let code = cli::run_provider(&provider.to_lowercase(), args).await?;
            std::process::exit(code);
        }
        CommandKind::Patch { provider, apply } => {
            cli::patch_provider(&provider.to_lowercase(), apply)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn bootstrap_opens_local_dashboard_only_in_human_integrated_mode() {
        assert!(super::should_open_dashboard_after_bootstrap(false, false));
        assert!(!super::should_open_dashboard_after_bootstrap(true, false));
        assert!(!super::should_open_dashboard_after_bootstrap(false, true));
    }

    #[test]
    fn local_dashboard_url_points_at_monitor() {
        assert_eq!(
            super::local_dashboard_url(),
            format!("{}/monitor", super::config::local_base())
        );
    }

    #[test]
    fn install_shortcut_expands_to_platform_install() {
        assert_eq!(
            super::expand_qorx_shortcuts(vec!["qorx", "-i", "-p", "codex"]),
            vec!["qorx", "install", "--platform", "codex"]
        );
    }

    #[test]
    fn integrate_shortcut_expands_to_platform_activation() {
        assert_eq!(
            super::expand_qorx_shortcuts(vec!["qorx", "-in", "-p", "antigravity"]),
            vec!["qorx", "integrate", "activate", "--platform", "antigravity"]
        );
    }

    #[test]
    fn splash_points_people_at_help_and_manual() {
        let splash = super::splash_text();
        assert!(splash.contains("QORX CLI"));
        assert!(splash.contains("Version: 0.0.1-ylem"));
        assert!(splash.contains("qorx doctor"));
        assert!(splash.contains("qorx --help"));
        assert!(splash.contains("qorx man"));
    }

    #[test]
    fn manual_documents_shortcut_surface() {
        let manual = super::manual_text(None);
        assert!(manual.contains("qorx -i -p codex"));
        assert!(manual.contains("qorx -in -p antigravity"));
    }

    #[test]
    fn install_manual_documents_default_on_integration_protocol() {
        let manual = super::manual_text(Some("install"));

        assert!(manual.contains("AutoMCP and AutoHook start on by default"));
        assert!(manual.contains("turn them off from the local monitor"));
        assert!(!manual.contains("AutoMCP and AutoHook are off until"));
    }

    #[test]
    fn crux_manual_documents_run_stop_report_and_rollback() {
        let manual = super::manual_text(Some("crux"));

        assert!(manual.contains("qorx crux run --hours 1"));
        assert!(manual.contains("qorx crux stop"));
        assert!(manual.contains("qorx crux report"));
        assert!(manual.contains("qorx crux rollback --checkpoint"));
    }
}
