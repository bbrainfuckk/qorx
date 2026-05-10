use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize)]
pub struct LexiconReport {
    pub schema: String,
    pub language: String,
    pub format: String,
    pub vocabulary: Value,
    pub aliases: Value,
    pub terms: &'static [LexiconTerm],
    pub layers: Value,
    pub boundary: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct LexiconTerm {
    pub name: &'static str,
    pub kind: &'static str,
    pub meaning: &'static str,
}

pub const TERMS: &[LexiconTerm] = &[
    term(
        "qvd",
        "runtime",
        "void; empty resolver state before an index, session, or hook exists",
    ),
    term(
        "qcm",
        "data",
        "cosmos; raw local corpus before Qorx indexes and bounds it",
    ),
    term(
        "qst",
        "architecture",
        "strata; source, index, handle, evidence, and provider layers",
    ),
    term(
        "qay",
        "runtime",
        "Void; local Qorx gateway, CLI, MCP server, and hook surface",
    ),
    term(
        "qbs",
        "storage",
        "abyss; cold local archive for inactive capsules and memories",
    ),
    term(
        "qnd",
        "compression",
        "nadir; smallest safe carrier state before meaning is lost",
    ),
    term(
        "qsg",
        "handle",
        "singularity; one compact qorx:// handle pointing at local state",
    ),
    term(
        "qnt",
        "cleanup",
        "entropy; duplicate, stale, or noisy context marked for culling",
    ),
    term(
        "qqt",
        "data",
        "quanta; smallest scored evidence unit used by the index",
    ),
    term(
        "qsm",
        "boundary",
        "schism; separation between local data and model-visible text",
    ),
    term(
        "qfs",
        "planner",
        "fission; splitting a large task into bounded evidence packs",
    ),
    term(
        "qml",
        "compat",
        "monolith; heavy legacy prompt block that should be indexed locally",
    ),
    term(
        "qnr",
        "cleanup",
        "inertia; low-signal payload weight removed from the prompt",
    ),
    term(
        "qgn",
        "bootstrap",
        "genesis; first index, capsule, or session created for a workspace",
    ),
    term(
        "qct",
        "runtime",
        "catalyst; trigger that expands a handle into cited local evidence",
    ),
    term(
        "qxs",
        "surface",
        "nexus; local routing hub for CLI, MCP, hooks, and HTTP",
    ),
    term(
        "qpl",
        "runtime",
        "plasma; active compression state while a request is being packed",
    ),
    term(
        "qeh",
        "boundary",
        "event horizon; point where only local Qorx can expand the carrier",
    ),
    term(
        "qhl",
        "metadata",
        "halo; minimum metadata wrapper around a compact carrier",
    ),
    term(
        "qpx",
        "analysis",
        "parallax; view shift between raw prompt and compact evidence pack",
    ),
    term(
        "qko",
        "evidence",
        "echo; structural trace of the original source in a compact pack",
    ),
    term(
        "qzr",
        "runtime",
        "zero; clean start state after counters and live state are reset",
    ),
    term(
        "qfx",
        "stream",
        "flux; incoming context stream before indexing or compression",
    ),
    term(
        "qvr",
        "buffer",
        "vortex; intake buffer that batches raw text before scoring",
    ),
    term(
        "qmc",
        "output",
        "macro; expanded human-readable answer or proof page",
    ),
    term(
        "qmi",
        "data",
        "micro; local token-level or symbol-level manipulation",
    ),
    term(
        "qnv",
        "runtime",
        "nova; deliberate high-compute expansion from handle to context",
    ),
    term(
        "qpr",
        "scheduler",
        "pulsar; periodic emission of hook, tray, or daemon status",
    ),
    term(
        "qdm",
        "index",
        "dark matter; latent sparse-vector weight kept out of the prompt",
    ),
    term(
        "qgl",
        "index",
        "gluon; binding signal that keeps related quanta grouped",
    ),
    term(
        "qbr",
        "data",
        "baryon; heavier local cluster made from related quanta",
    ),
    term(
        "qtc",
        "cache",
        "tachyon; precomputed cache path that skips repeated packing work",
    ),
    term(
        "qkn",
        "runtime",
        "kinetic; active execution phase for a compact prompt",
    ),
    term(
        "qsc",
        "storage",
        "static; read-only indexed context that should not mutate",
    ),
    term(
        "qfr",
        "compiler",
        "forge; local compiler or builder for Qorx artifacts",
    ),
    term(
        "qax",
        "policy",
        "axiom; non-negotiable local policy embedded in a carrier",
    ),
    term(
        "qhr",
        "boundary",
        "horizon; model-visible budget limit before Qorx must pack",
    ),
    term(
        "qom",
        "prefetch",
        "omen; predicted context block prepared before the agent asks",
    ),
    term(
        "qrg",
        "safety",
        "rogue; orphaned process, token, or hook outside the nexus",
    ),
    term(
        "qcl",
        "cleanup",
        "cull; remove low-value data from a prompt or cache",
    ),
    term(
        "qzn",
        "metric",
        "zenith; best observed local efficiency for a pack or session",
    ),
    term(
        "qap",
        "quality",
        "apex; highest structural integrity score for a compact carrier",
    ),
    term(
        "qph",
        "runtime",
        "phantom; background task with no terminal output",
    ),
    term(
        "qsh",
        "data",
        "shard; partial context block waiting for reassembly",
    ),
    term(
        "qcr",
        "runtime",
        "core; guarded local runtime and protobuf store",
    ),
    term(
        "qmt",
        "index",
        "matrix; relation grid for active quanta and symbols",
    ),
    term(
        "qwr",
        "adapter",
        "warp; provider-facing shape change without losing local citations",
    ),
    term(
        "qfl",
        "signal",
        "flare; sudden token spike that should be packed immediately",
    ),
    term(
        "qcd",
        "runtime",
        "cascade; ordered expansion of a handle chain",
    ),
    term(
        "qvl",
        "metric",
        "velocity; local token throughput over time",
    ),
    term(
        "qdr",
        "quality",
        "drift; semantic loss detected after over-compression",
    ),
    term(
        "qal",
        "policy",
        "alignment; formatting raw prompts into a strict Qorx contract",
    ),
    term(
        "qch",
        "merge",
        "chimera; fused context from conflicting sources that needs review",
    ),
    term(
        "qen",
        "security",
        "enigma; opaque encrypted or signed carrier",
    ),
    term(
        "qor",
        "analysis",
        "oracle; query that extracts the core intent of a carrier",
    ),
    term(
        "qsl",
        "output",
        "silencer; compact mode that suppresses non-essential output",
    ),
    term(
        "qmg",
        "index",
        "magnetar; high-affinity node that attracts related fragments",
    ),
    term(
        "qib",
        "metric",
        "isobar; region of equal semantic density inside a pack",
    ),
    term(
        "qob",
        "scheduler",
        "orbit; repeating background maintenance path",
    ),
    term(
        "qtd",
        "memory",
        "tide; expected rise and fall of local memory use",
    ),
    term(
        "qif",
        "cleanup",
        "inferno; aggressive cache clear for stale local bloat",
    ),
    term(
        "qcg",
        "security",
        "cipher; key material or verification handle for enigma data",
    ),
    term(
        "qrd",
        "daemon",
        "radar; lightweight scanner for active local Qorx tasks",
    ),
    term(
        "qlx",
        "index",
        "lattice; graph that holds quanta, symbols, and edges",
    ),
    term(
        "qsy",
        "language",
        "syntax; grammar and structure of .qorx source",
    ),
    term(
        "qpm",
        "planner",
        "prism; split one objective into parallel agent-ready packs",
    ),
    term(
        "qum",
        "storage",
        "umbra; coldest layer of the local abyss archive",
    ),
    term(
        "qpn",
        "cache",
        "penumbra; semi-active cache above cold storage",
    ),
    term(
        "qlu",
        "debug",
        "lumen; transparent view into a compact carrier",
    ),
    term(
        "qig",
        "startup",
        "ignition; local boot path for daemon, tray, MCP, and hooks",
    ),
    term(
        "qex",
        "safety",
        "exile; quarantine corrupted or untrusted context",
    ),
    term(
        "qvt",
        "intent",
        "vector; directional intent carried by a request",
    ),
    term(
        "qtn",
        "index",
        "tensor; multi-axis mapping of a compressed context block",
    ),
    term(
        "qam",
        "merge",
        "amalgam; mixed source bundle requiring stricter evidence checks",
    ),
    term(
        "qfc",
        "cleanup",
        "fractus; jagged or malformed context before smoothing",
    ),
    term(
        "qbl",
        "stability",
        "ballast; required context added to keep an answer grounded",
    ),
    term(
        "qsw",
        "planner",
        "swarm; many small carriers dispatched across tool surfaces",
    ),
    term(
        "qfd",
        "compiler",
        "foundry; first casting ground for raw tokens before refinement",
    ),
    term(
        "qhx",
        "diagnostic",
        "hex; unoptimizable block flagged for manual review",
    ),
    term(
        "qav",
        "staging",
        "anvil; staging surface where heavy prompts are inspected",
    ),
    term(
        "qhm",
        "compressor",
        "hammer; local packer that turns staged prompt into carrier",
    ),
    term(
        "qcy",
        "cleanup",
        "scythe; trims dangling or redundant tokens",
    ),
    term(
        "qsk",
        "retrieval",
        "skewer; direct extraction of one fact from deep context",
    ),
    term(
        "qtg",
        "handle",
        "tesseract; self-referential handle that folds through sessions",
    ),
    term(
        "qnm",
        "adapter",
        "nomad; carrier that moves between agent clients safely",
    ),
    term(
        "qmz",
        "routing",
        "maze; internal routing paths through the nexus",
    ),
    term(
        "qag",
        "security",
        "aegis; protective wrapper around core runtime state",
    ),
    term(
        "qoq",
        "output",
        "omega; final answer token or completed agent result",
    ),
    term(
        "qab",
        "truth",
        "absolute; locally verified claim with cited evidence",
    ),
    term(
        "qkd",
        "memory",
        "k-drift; movement of live local context across memory windows",
    ),
    term(
        "qsa",
        "runtime",
        "stasis; freeze expansion and keep the current carrier stable",
    ),
    term(
        "qvw",
        "policy",
        "vow; immutable execution constraint for a carrier",
    ),
    term(
        "qrp",
        "ingest",
        "rupture; open a heavy payload before packing",
    ),
    term(
        "qlz",
        "scheduler",
        "laz; lazy-evaluation state for a dormant agent task",
    ),
    term(
        "qmu",
        "data",
        "muon; short-lived carrier created for a quick task",
    ),
    term(
        "qis",
        "versioning",
        "isotope; variant of a saved carrier with small changes",
    ),
    term(
        "qpd",
        "sandbox",
        "pod; isolated runtime for an untrusted task",
    ),
    term(
        "qmn",
        "recall",
        "mnemonic; minimum recall key for local abyss retrieval",
    ),
    term(
        "qrn",
        "reset",
        "ruin; scorched reset of active processes, counters, and caches",
    ),
    term(
        "qlw",
        "creation",
        "likha; create the local stack from nothing and execute it",
    ),
];

const fn term(name: &'static str, kind: &'static str, meaning: &'static str) -> LexiconTerm {
    LexiconTerm {
        name,
        kind,
        meaning,
    }
}

pub fn report() -> LexiconReport {
    LexiconReport {
        schema: "qorx.lexicon.v1".to_string(),
        language: "qorx".to_string(),
        format: "protobuf-envelope".to_string(),
        vocabulary: vocabulary(),
        aliases: aliases(),
        terms: TERMS,
        layers: json!({
            "ai_language": "Qorx is an AI language and local context runtime.",
            "product": "Qorx Void is the local gateway, CLI, MCP server, and hook surface.",
            "encoding": "Qorx state and bytecode use protobuf-envelope storage.",
            "terms": "Primary Qorx glossary terms are exactly 3 characters. Longer wire labels remain compatibility vocabulary, not the public glossary.",
            "qay": "Qorx Void local runtime and integration surface",
            "qcm": "raw local corpus before indexing",
            "qsg": "compact qorx:// handle that points back to local Qorx state",
            "qsm": "explicit boundary between local evidence and provider-visible tokens",
            "qct": "local trigger that expands tiny handles into exact indexed evidence",
            "qlx": "graph of quanta, symbols, and relations",
            "mass": "deterministic local token estimate, not provider tokenizer billing truth",
            "qzn": "best observed local efficiency during packing",
            "cost_transform": "measured local compaction backed by local counters"
        }),
        boundary: "Qorx uses compact Qorx vocabulary, but it is not a physics engine and these are not physics claims. It does not claim physical compression. provider billing is not bypassed; outside cost depends on the actual upstream request sent.".to_string(),
    }
}

pub fn vocabulary() -> Value {
    json!({
        ".qorx": "qwav_source",
        ".qorxb": "qfal_bytecode",
        "Qorx Void": "qay_runtime",
        "qorx://s": "qses_handle",
        "qorx://c": "qcap_handle",
        "qorx://l": "qlat_handle",
        "qorx://f": "qfed_handle",
        "qorx://u": "qsng_handle",
        "qstk": "forth_like_stack_tape",
        "qosm": "local_resolver_ledger",
        "qshf": "baseline_to_compact_accounting",
        "qv0d": "resolver_miss_or_empty_evidence",
        "cosmos_store": "qorx_data_dir_or_portable_store",
        "qorx_data_dir": "qosm_storage",
        "qorx-cosmos.pb": "qpb_qosm_ledger",
        "prompt_block": "phot_carrier",
        "visible_tokens": "phot_mass",
        "indexed_tokens": "qosm_mass",
        "b2c": "qshf_accounting",
        "cost_transform": "qshf_compaction_transform",
        "context_reduction_x": "qshf_factor",
        "provider_calls": "external_observation_count"
    })
}

pub fn aliases() -> Value {
    json!({
        "ayie": "qay",
        "edge": "qay",
        "quark": "qqt",
        "cosmos": "qcm",
        "wavefunction": "qwav",
        "collapse": "qfal",
        "photon": "qsg",
        "event_horizon": "qeh",
        "redshift": "qshf",
        "rshift": "qshf",
        "qshift": "qshf",
        "qvoid": "qv0d",
        "void": "qv0d",
        "proof": "qprf",
        "capsule": "qcap",
        "session": "qses"
    })
}

pub fn runtime_tags(source_kind: &str) -> Value {
    json!({
        "source": if source_kind == "qorxb" { "qfal" } else { "qwav" },
        "model_visible_carrier": "phot",
        "local_runtime": "qosm",
        "storage": "q_drive_or_qorx_data_dir",
        "handle": "qsng",
        "cost_transform": "qshf",
        "boundary": "hzon"
    })
}
