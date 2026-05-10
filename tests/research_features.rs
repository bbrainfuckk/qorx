use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

fn unique_temp_dir() -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    let sequence = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    env::temp_dir().join(format!(
        "qorx-research-features-{}-{suffix}-{sequence}",
        std::process::id()
    ))
}

fn seed_research_index(dir: &Path) {
    fs::create_dir_all(dir).expect("create qorx home");
    fs::write(
        dir.join("repo_index.json"),
        r#"{
  "root": "C:/repo",
  "updated_at": "2026-04-29T00:00:00Z",
  "quarks": [
    {
      "id": "qva_money",
      "path": "src/money.rs",
      "start_line": 81,
      "end_line": 90,
      "hash": "abc",
      "token_estimate": 120,
      "symbols": ["production_gate_passed", "routed_provider_requests"],
      "signal_mask": 0,
      "vector": [11, 12, 13],
      "text": "production gate requires routed provider savings evidence before money claims are allowed\nfiller alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu\nfiller unrelated screen copy should not survive query aware squeezing\nrouted provider requests must be observed before B2C money claims pass"
    },
    {
      "id": "qva_auth_route",
      "path": "src/routes/auth.ts",
      "start_line": 1,
      "end_line": 8,
      "hash": "def",
      "token_estimate": 40,
      "symbols": ["loginRoute"],
      "signal_mask": 66,
      "vector": [21, 22, 23],
      "text": "export function loginRoute(req) {\n  // WHY: session route proves local context flow\n  const session = issueSession(req.user);\n  logAudit(session.id);\n  return session;\n}"
    },
    {
      "id": "qva_session_service",
      "path": "src/services/session.ts",
      "start_line": 1,
      "end_line": 5,
      "hash": "ghi",
      "token_estimate": 32,
      "symbols": ["issueSession"],
      "signal_mask": 64,
      "vector": [31, 32, 33],
      "text": "export function issueSession(user) {\n  return { id: user.id, expires: Date.now() + 3600 };\n}"
    },
    {
      "id": "qva_audit_service",
      "path": "src/services/audit.ts",
      "start_line": 1,
      "end_line": 5,
      "hash": "jkl",
      "token_estimate": 28,
      "symbols": ["logAudit"],
      "signal_mask": 64,
      "vector": [41, 42, 43],
      "text": "export function logAudit(sessionId) {\n  return `audit:${sessionId}`;\n}"
    },
    {
      "id": "qva_unrelated",
      "path": "src/billing.ts",
      "start_line": 1,
      "end_line": 3,
      "hash": "mno",
      "token_estimate": 20,
      "symbols": ["billCustomer"],
      "signal_mask": 0,
      "vector": [51, 52, 53],
      "text": "export function billCustomer(customer) { return customer.plan; }"
    }
  ]
}"#,
    )
    .expect("write index");
    fs::write(dir.join("stats.json"), r#"{"requests":0}"#).expect("write stats");
}

fn qorx(args: &[&str], qorx_home: &Path) -> serde_json::Value {
    let output = Command::new(env!("CARGO_BIN_EXE_qorx"))
        .args(args)
        .env("QORX_HOME", qorx_home)
        .output()
        .unwrap_or_else(|err| panic!("run qorx {args:?}: {err}"));
    assert!(
        output.status.success(),
        "qorx {args:?} failed: status={:?} stderr={} stdout={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
        panic!(
            "parse qorx {args:?} JSON: {err}\nstdout={}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

#[test]
fn squeeze_returns_query_aware_extracts_under_budget() {
    let qorx_home = unique_temp_dir();
    seed_research_index(&qorx_home);

    let report = qorx(
        &[
            "squeeze",
            "production gate routed provider evidence",
            "--budget-tokens",
            "180",
        ],
        &qorx_home,
    );

    assert_eq!(report["schema"], "qorx.squeeze.v1");
    assert_eq!(report["mode"], "extractive_query_squeeze");
    assert_eq!(report["local_only"], true);
    assert_eq!(report["provider_calls"], 0);
    assert!(report["used_tokens"].as_u64().unwrap() <= 180);
    assert!(
        report["squeezed_tokens"].as_u64().unwrap() < report["source_tokens"].as_u64().unwrap()
    );
    assert!(report["text"]
        .as_str()
        .unwrap()
        .contains("production gate requires routed provider savings evidence"));
    assert_eq!(report["quarks_used"], 1);
    assert!(!report["text"]
        .as_str()
        .unwrap()
        .contains("filler unrelated screen copy"));
    assert_eq!(report["evidence"][0]["id"], "qva_money");

    let _ = fs::remove_dir_all(&qorx_home);
}

#[test]
fn graph_cli_exports_dashboard_metrics_for_offline_audits() {
    let qorx_home = unique_temp_dir();
    seed_research_index(&qorx_home);

    let report = qorx(&["graph", "--limit", "64"], &qorx_home);

    assert_eq!(report["schema"], "qorx.graph-view.v1");
    assert_eq!(report["metrics"]["file_nodes"], 5);
    assert_eq!(report["metrics"]["reference_edges"], 2);
    assert_eq!(report["metrics"]["health"], "needs_attention");
    assert!(report["metrics"]["top_referenced_files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["path"] == "src/services/session.ts" && item["incoming_references"] == 1));

    let _ = fs::remove_dir_all(&qorx_home);
}

#[test]
fn graph_cli_can_scope_dashboard_graph_to_a_query() {
    let qorx_home = unique_temp_dir();
    seed_research_index(&qorx_home);

    let report = qorx(
        &[
            "graph",
            "--query",
            "login route session audit",
            "--limit",
            "64",
        ],
        &qorx_home,
    );

    let file_paths = report["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|node| node["kind"] == "file")
        .map(|node| node["path"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();

    assert!(file_paths.contains(&"src/routes/auth.ts".to_string()));
    assert!(file_paths.contains(&"src/services/session.ts".to_string()));
    assert!(file_paths.contains(&"src/services/audit.ts".to_string()));
    assert!(!file_paths.contains(&"src/billing.ts".to_string()));

    let _ = fs::remove_dir_all(&qorx_home);
}

#[test]
fn graph_path_cli_traces_file_to_file_references() {
    let qorx_home = unique_temp_dir();
    seed_research_index(&qorx_home);

    let report = qorx(
        &[
            "graph-path",
            "routes/auth.ts",
            "services/audit.ts",
            "--limit",
            "64",
        ],
        &qorx_home,
    );

    assert_eq!(report["schema"], "qorx.graph-trace.v1");
    assert_eq!(report["found"], true);
    assert_eq!(report["hops"], 1);
    assert_eq!(report["path"][0]["path"], "src/routes/auth.ts");
    assert_eq!(report["path"][1]["path"], "src/services/audit.ts");

    let _ = fs::remove_dir_all(&qorx_home);
}

#[test]
fn atlas_cli_summarizes_local_connections_without_external_branding() {
    let qorx_home = unique_temp_dir();
    seed_research_index(&qorx_home);

    let report = qorx(&["atlas", "--limit", "64"], &qorx_home);

    assert_eq!(report["schema"], "qorx.atlas-report.v1");
    assert_eq!(report["mode"], "local_atlas_report");
    assert_eq!(report["local_only"], true);
    assert_eq!(report["provider_calls"], 0);
    assert_eq!(report["item_count"], 11);
    assert_eq!(report["link_count"], 2);
    assert!(report["hubs"].as_array().unwrap().iter().any(|hub| {
        hub["path"] == "src/services/session.ts"
            && hub["incoming_links"] == 1
            && hub["confidence"] == "EXTRACTED"
    }));
    assert!(report["surprising_connections"]
        .as_array()
        .unwrap()
        .iter()
        .any(|connection| {
            connection["from_path"] == "src/routes/auth.ts"
                && connection["to_path"] == "src/services/session.ts"
                && connection["confidence"] == "EXTRACTED"
        }));
    assert!(report["rationale"].as_array().unwrap().iter().any(|item| {
        item["path"] == "src/routes/auth.ts"
            && item["marker"] == "WHY"
            && item["text"]
                .as_str()
                .unwrap()
                .contains("session route proves local context flow")
    }));
    assert!(report["suggested_questions"].as_array().unwrap().len() >= 4);
    assert!(report["confidence"]["EXTRACTED"]
        .as_str()
        .unwrap()
        .contains("read directly"));

    let rendered = serde_json::to_string(&report).unwrap();
    let external_graph_term = ["Gra", "phify"].concat();
    let external_graph_term_lower = external_graph_term.to_ascii_lowercase();
    for banned in [
        external_graph_term.as_str(),
        external_graph_term_lower.as_str(),
        "god",
    ] {
        assert!(
            !rendered.contains(banned),
            "atlas surface leaked external term {banned}: {rendered}"
        );
    }

    let _ = fs::remove_dir_all(&qorx_home);
}

#[test]
fn atlas_export_writes_qorx_pack_files_for_agents() {
    let qorx_home = unique_temp_dir();
    let out_dir = unique_temp_dir();
    seed_research_index(&qorx_home);

    let report = qorx(
        &[
            "atlas",
            "export",
            "--out",
            out_dir.to_str().unwrap(),
            "--limit",
            "64",
        ],
        &qorx_home,
    );

    assert_eq!(report["schema"], "qorx.atlas-export.v1");
    assert_eq!(
        report["files"]["json"].as_str().unwrap(),
        out_dir.join("atlas.json").to_string_lossy()
    );
    assert_eq!(
        report["files"]["markdown"].as_str().unwrap(),
        out_dir.join("ATLAS_REPORT.md").to_string_lossy()
    );
    assert_eq!(
        report["files"]["html"].as_str().unwrap(),
        out_dir.join("atlas.html").to_string_lossy()
    );
    assert!(out_dir.join("atlas.json").exists());
    assert!(out_dir.join("ATLAS_REPORT.md").exists());
    assert!(out_dir.join("atlas.html").exists());
    assert!(out_dir.join("manifest.json").exists());
    assert!(out_dir.join("AGENTS.atlas.md").exists());

    let markdown = fs::read_to_string(out_dir.join("ATLAS_REPORT.md")).unwrap();
    assert!(markdown.contains("# Qorx Atlas Report"));
    assert!(markdown.contains("src/services/session.ts"));
    assert!(markdown.contains("session route proves local context flow"));
    let agent_note = fs::read_to_string(out_dir.join("AGENTS.atlas.md")).unwrap();
    assert!(agent_note.contains("Read ATLAS_REPORT.md before broad file reads"));

    let pack: serde_json::Value =
        serde_json::from_slice(&fs::read(out_dir.join("atlas.json")).unwrap()).unwrap();
    assert_eq!(pack["schema"], "qorx.atlas-pack.v1");
    assert_eq!(pack["report"]["schema"], "qorx.atlas-report.v1");
    assert!(pack["modalities"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| { item["kind"] == "code" && item["file_count"].as_u64().unwrap() >= 4 }));
    assert!(pack["research_basis"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| {
            item["name"] == "GraphCoder" && item["source"] == "https://arxiv.org/abs/2406.07003"
        }));

    let rendered = serde_json::to_string(&pack).unwrap();
    let external_graph_term = ["Gra", "phify"].concat();
    assert!(!rendered.contains(&external_graph_term));

    let _ = fs::remove_dir_all(&qorx_home);
    let _ = fs::remove_dir_all(&out_dir);
}

#[test]
fn atlas_global_registry_adds_lists_and_reports_path() {
    let qorx_home = unique_temp_dir();
    let out_dir = unique_temp_dir();
    seed_research_index(&qorx_home);

    let _ = qorx(
        &[
            "atlas",
            "export",
            "--out",
            out_dir.to_str().unwrap(),
            "--limit",
            "64",
        ],
        &qorx_home,
    );

    let added = qorx(
        &[
            "atlas",
            "global",
            "add",
            out_dir.join("atlas.json").to_str().unwrap(),
            "fixture",
        ],
        &qorx_home,
    );
    assert_eq!(added["schema"], "qorx.atlas-global.v1");
    assert_eq!(added["projects"][0]["name"], "fixture");

    let listed = qorx(&["atlas", "global", "list"], &qorx_home);
    assert_eq!(listed["projects"][0]["name"], "fixture");
    assert_eq!(listed["projects"][0]["schema"], "qorx.atlas-pack.v1");

    let path = qorx(&["atlas", "global", "path"], &qorx_home);
    assert_eq!(
        path["path"].as_str().unwrap(),
        qorx_home.join("atlas-global.json").to_string_lossy()
    );

    let _ = fs::remove_dir_all(&qorx_home);
    let _ = fs::remove_dir_all(&out_dir);
}

#[test]
fn atlas_merge_combines_exported_packs() {
    let qorx_home = unique_temp_dir();
    let out_a = unique_temp_dir();
    let out_b = unique_temp_dir();
    let merged = unique_temp_dir().join("merged-atlas.json");
    seed_research_index(&qorx_home);

    let _ = qorx(
        &[
            "atlas",
            "export",
            "--out",
            out_a.to_str().unwrap(),
            "--limit",
            "64",
        ],
        &qorx_home,
    );
    let _ = qorx(
        &[
            "atlas",
            "export",
            "--out",
            out_b.to_str().unwrap(),
            "--limit",
            "64",
        ],
        &qorx_home,
    );

    let report = qorx(
        &[
            "atlas",
            "merge",
            out_a.join("atlas.json").to_str().unwrap(),
            out_b.join("atlas.json").to_str().unwrap(),
            "--out",
            merged.to_str().unwrap(),
        ],
        &qorx_home,
    );

    assert_eq!(report["schema"], "qorx.atlas-merge.v1");
    assert_eq!(report["input_count"], 2);
    assert_eq!(report["output"].as_str().unwrap(), merged.to_string_lossy());
    assert!(merged.exists());

    let merged_json: serde_json::Value =
        serde_json::from_slice(&fs::read(&merged).unwrap()).unwrap();
    assert_eq!(merged_json["schema"], "qorx.atlas-merged.v1");
    assert!(merged_json["hubs"].as_array().unwrap().iter().any(|hub| {
        hub["path"] == "src/services/session.ts" && hub["incoming_links"].as_u64().unwrap() >= 1
    }));

    let _ = fs::remove_dir_all(&qorx_home);
    let _ = fs::remove_dir_all(&out_a);
    let _ = fs::remove_dir_all(&out_b);
    if let Some(parent) = merged.parent() {
        let _ = fs::remove_dir_all(parent);
    }
}

#[test]
fn atlas_query_and_path_reuse_local_graph_surfaces() {
    let qorx_home = unique_temp_dir();
    seed_research_index(&qorx_home);

    let query = qorx(
        &[
            "atlas",
            "query",
            "login route session audit",
            "--limit",
            "64",
        ],
        &qorx_home,
    );
    assert_eq!(query["schema"], "qorx.graph-view.v1");
    assert!(query["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|node| { node["path"] == "src/routes/auth.ts" && node["kind"] == "file" }));

    let path = qorx(
        &[
            "atlas",
            "path",
            "routes/auth.ts",
            "services/audit.ts",
            "--limit",
            "64",
        ],
        &qorx_home,
    );
    assert_eq!(path["schema"], "qorx.graph-trace.v1");
    assert_eq!(path["found"], true);

    let _ = fs::remove_dir_all(&qorx_home);
}

#[test]
fn judge_marks_supported_and_unsupported_claims() {
    let qorx_home = unique_temp_dir();
    seed_research_index(&qorx_home);

    let report = qorx(
        &[
            "judge",
            "production gate requires routed provider savings evidence. warp drive cooking schedule is approved.",
        ],
        &qorx_home,
    );

    assert_eq!(report["schema"], "qorx.judge.v1");
    assert_eq!(report["unsupported_claims"], 1);
    assert_eq!(report["claims"][0]["verdict"], "supported");
    assert_eq!(report["claims"][1]["verdict"], "unsupported");
    assert!(report["boundary"]
        .as_str()
        .unwrap()
        .contains("indexed local evidence"));

    let _ = fs::remove_dir_all(&qorx_home);
}

#[test]
fn ground_gate_blocks_unsupported_answer_and_simulates_large_context_savings() {
    let qorx_home = unique_temp_dir();
    seed_research_index(&qorx_home);

    let report = qorx(
        &[
            "ground",
            "production gate routed provider evidence",
            "--answer",
            "production gate requires routed provider savings evidence. warp drive cooking schedule is approved.",
            "--budget-tokens",
            "220",
            "--raw-tokens",
            "5000000000",
            "--sent-tokens",
            "1000",
            "--input-usd-per-million",
            "2.5",
        ],
        &qorx_home,
    );

    assert_eq!(report["schema"], "qorx.grounding-gate.v1");
    assert_eq!(report["local_only"], true);
    assert_eq!(report["provider_calls"], 0);
    assert_eq!(report["hallucination_gate_passed"], false);
    assert_eq!(report["verdict"], "blocked_unsupported_claims");
    assert_eq!(report["answer_judgement"]["unsupported_claims"], 1);
    assert!(report["claim_policy"]
        .as_str()
        .unwrap()
        .contains("no 100 percent hallucination claim"));
    assert_eq!(
        report["savings_simulation"]["raw_input_tokens"],
        5_000_000_000u64
    );
    assert_eq!(report["savings_simulation"]["sent_input_tokens"], 1_000u64);
    assert_eq!(report["savings_simulation"]["raw_input_cost_usd"], 12_500.0);
    assert_eq!(
        report["savings_simulation"]["compact_input_cost_usd"],
        0.0025
    );
    assert!(report["retrieval_plan"]["stages"]
        .as_array()
        .unwrap()
        .iter()
        .any(|stage| stage["name"] == "strict-answer"));
    assert!(report["retrieval_plan"]["stages"]
        .as_array()
        .unwrap()
        .iter()
        .any(|stage| stage["name"] == "squeeze"));

    let _ = fs::remove_dir_all(&qorx_home);
}

#[test]
fn ground_gate_passes_supported_answer_with_proof_per_token_metrics() {
    let qorx_home = unique_temp_dir();
    seed_research_index(&qorx_home);

    let report = qorx(
        &[
            "ground",
            "production gate routed provider evidence",
            "--answer",
            "production gate requires routed provider savings evidence.",
            "--budget-tokens",
            "220",
        ],
        &qorx_home,
    );

    assert_eq!(report["schema"], "qorx.grounding-gate.v1");
    assert_eq!(report["hallucination_gate_passed"], true);
    assert_eq!(report["verdict"], "grounded");
    assert_eq!(report["answer_judgement"]["unsupported_claims"], 0);
    assert_eq!(report["answer_judgement"]["supported_claims"], 1);
    assert!(report["proof_per_token"]["support_rate"].as_f64().unwrap() >= 1.0);
    assert!(
        report["proof_per_token"]["evidence_items"]
            .as_u64()
            .unwrap()
            >= 1
    );
    assert!(report["prompt_contract"]
        .as_str()
        .unwrap()
        .contains("Use only cited Qorx evidence"));

    let _ = fs::remove_dir_all(&qorx_home);
}

#[test]
fn cache_plan_splits_stable_prefix_from_dynamic_tail() {
    let qorx_home = unique_temp_dir();
    fs::create_dir_all(&qorx_home).expect("create home");
    let prompt = "system: use qorx session pointer\npolicy: stable cache prefix\n--- QORX_DYNAMIC ---\nuser asks live question";

    let report = qorx(&["cache-plan", prompt], &qorx_home);

    assert_eq!(report["schema"], "qorx.cache-plan.v1");
    assert_eq!(report["marker"], "--- QORX_DYNAMIC ---");
    assert_eq!(report["can_cache_prefix"], true);
    assert!(report["stable_prefix_tokens"].as_u64().unwrap() > 0);
    assert!(report["dynamic_tail_tokens"].as_u64().unwrap() > 0);
    assert!(report["recommendations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item.as_str().unwrap().contains("stable prefix first")));

    let _ = fs::remove_dir_all(&qorx_home);
}

#[test]
fn b2c_plan_runs_quant_allocator_over_indexed_quarks() {
    let qorx_home = unique_temp_dir();
    seed_research_index(&qorx_home);

    let report = qorx(
        &[
            "b2c-plan",
            "login route session audit",
            "--budget-tokens",
            "220",
        ],
        &qorx_home,
    );

    assert_eq!(report["schema"], "qorx.b2c-plan.v1");
    assert_eq!(report["local_only"], true);
    assert_eq!(report["provider_calls"], 0);
    assert!(report["used_tokens"].as_u64().unwrap() <= 220);
    assert!(report["selected_quarks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["path"] == "src/routes/auth.ts"));
    assert!(report["parallel_lanes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|lane| lane["name"] == "portfolio"));
    assert_eq!(report["math"]["budget_model"], "bounded_knapsack");
    assert!(report["boundary"]
        .as_str()
        .unwrap()
        .contains("deterministic local math"));

    let _ = fs::remove_dir_all(&qorx_home);
}

#[test]
fn b2c_plan_uses_orcl_scope_to_pull_linked_quarks_under_budget() {
    let qorx_home = unique_temp_dir();
    seed_research_index(&qorx_home);
    let diff_file = qorx_home.join("auth.diff");
    fs::write(
        &diff_file,
        "diff --git a/src/routes/auth.ts b/src/routes/auth.ts\n+++ b/src/routes/auth.ts\n@@\n+  logAudit(session.id);\n",
    )
    .expect("write diff");

    let diff_file_text = diff_file.display().to_string();
    let report = qorx(
        &[
            "b2c-plan",
            "login route",
            "--diff-file",
            &diff_file_text,
            "--budget-tokens",
            "220",
        ],
        &qorx_home,
    );

    assert_eq!(report["schema"], "qorx.b2c-plan.v1");
    assert!(report["used_tokens"].as_u64().unwrap() <= 220);
    let selected = report["selected_quarks"].as_array().unwrap();
    assert!(selected
        .iter()
        .any(|item| item["id"] == "qva_auth_route" && item["orcl_score"].as_f64().unwrap() > 0.0));
    assert!(selected.iter().any(|item| {
        item["id"] == "qva_session_service" && item["orcl_score"].as_f64().unwrap() > 0.0
    }));
    assert!(selected.iter().any(|item| {
        item["id"] == "qva_audit_service" && item["orcl_score"].as_f64().unwrap() > 0.0
    }));
    assert!(!selected.iter().any(|item| item["id"] == "qva_unrelated"));
    assert!(report["text"].as_str().unwrap().contains("orcl_scope=true"));

    let _ = fs::remove_dir_all(&qorx_home);
}

#[test]
fn pack_carries_b2c_allocator_proof_in_the_hot_context() {
    let qorx_home = unique_temp_dir();
    seed_research_index(&qorx_home);

    let report = qorx(
        &[
            "pack",
            "login route session audit",
            "--budget-tokens",
            "220",
        ],
        &qorx_home,
    );

    assert_eq!(report["query"], "login route session audit");
    assert!(report["text"]
        .as_str()
        .unwrap()
        .contains("# Qorx B2C packed context"));
    assert!(report["text"]
        .as_str()
        .unwrap()
        .contains("b2c_parallel_lanes=retrieval,portfolio,risk,cache,carrier"));
    assert!(report["text"]
        .as_str()
        .unwrap()
        .contains("b2c_math=bounded_knapsack"));
    assert!(report["quarks_used"].as_u64().unwrap() >= 1);

    let _ = fs::remove_dir_all(&qorx_home);
}

#[test]
fn map_reports_changed_paths_symbols_and_related_edges() {
    let qorx_home = unique_temp_dir();
    seed_research_index(&qorx_home);
    let diff_file = qorx_home.join("auth.diff");
    fs::write(
        &diff_file,
        "diff --git a/src/routes/auth.ts b/src/routes/auth.ts\n+++ b/src/routes/auth.ts\n@@\n+  logAudit(session.id);\n",
    )
    .expect("write diff");

    let diff_file_text = diff_file.display().to_string();
    let report = qorx(
        &[
            "map",
            "login route session audit",
            "--diff-file",
            &diff_file_text,
            "--budget-tokens",
            "320",
        ],
        &qorx_home,
    );

    assert_eq!(report["schema"], "qorx.map.v1");
    assert_eq!(report["changed_paths"][0], "src/routes/auth.ts");
    assert!(report["symbols"]
        .as_array()
        .unwrap()
        .iter()
        .any(|symbol| symbol["name"] == "issueSession"));
    assert!(report["graph_edges"]
        .as_array()
        .unwrap()
        .iter()
        .any(|edge| {
            edge["from_path"] == "src/routes/auth.ts"
                && edge["to_path"] == "src/services/session.ts"
        }));
    assert!(!report["text"].as_str().unwrap().contains("billCustomer"));

    let _ = fs::remove_dir_all(&qorx_home);
}

#[test]
fn orcl_reports_ranked_contracts_and_links_without_external_terms() {
    let qorx_home = unique_temp_dir();
    seed_research_index(&qorx_home);
    let diff_file = qorx_home.join("auth.diff");
    fs::write(
        &diff_file,
        "diff --git a/src/routes/auth.ts b/src/routes/auth.ts\n+++ b/src/routes/auth.ts\n@@\n+  logAudit(session.id);\n",
    )
    .expect("write diff");

    let diff_file_text = diff_file.display().to_string();
    let report = qorx(
        &[
            "orcl",
            "login route session audit",
            "--diff-file",
            &diff_file_text,
            "--budget-tokens",
            "420",
            "--depth",
            "2",
        ],
        &qorx_home,
    );

    assert_eq!(report["schema"], "qorx.orcl.v1");
    assert_eq!(report["changed_paths"][0], "src/routes/auth.ts");
    assert!(report["symbols"].as_array().unwrap().iter().any(|symbol| {
        symbol["name"] == "issueSession"
            && symbol["signature"] == "export function issueSession(user) {"
    }));
    assert!(report["links"].as_array().unwrap().iter().any(|link| {
        link["from_path"] == "src/routes/auth.ts"
            && link["to_path"] == "src/services/session.ts"
            && link["symbol"] == "issueSession"
    }));
    assert!(report["rank"]
        .as_array()
        .unwrap()
        .iter()
        .any(|rank| rank["symbol"] == "issueSession" && rank["fan_in"].as_u64().unwrap() > 0));

    let rendered = serde_json::to_string(&report).unwrap();
    let external_graph_term = ["Gra", "phify"].concat();
    let external_graph_term_lower = external_graph_term.to_ascii_lowercase();
    for banned in [
        external_graph_term.as_str(),
        external_graph_term_lower.as_str(),
        "god",
        "node",
        "edge",
        "graph_edges",
    ] {
        assert!(
            !rendered.contains(banned),
            "orcl surface leaked external term {banned}: {rendered}"
        );
    }

    let _ = fs::remove_dir_all(&qorx_home);
}

#[test]
fn memory_crud_summarize_and_prune_are_local() {
    let qorx_home = unique_temp_dir();
    fs::create_dir_all(&qorx_home).expect("create home");

    let created = qorx(
        &[
            "memory",
            "create",
            "decision",
            "provider traffic routes through Qorx before money claims",
        ],
        &qorx_home,
    );
    assert_eq!(created["schema"], "qorx.memory.v1");
    assert_eq!(created["action"], "create");
    assert_eq!(created["local_only"], true);
    let id = created["item"]["id"].as_str().unwrap().to_string();

    let read = qorx(&["memory", "read", "provider traffic"], &qorx_home);
    assert_eq!(read["items"].as_array().unwrap().len(), 1);
    assert_eq!(read["items"][0]["id"], id);

    let updated = qorx(
        &[
            "memory",
            "update",
            &id,
            "provider traffic routes through Qorx and records Baseline-to-Compact proof",
        ],
        &qorx_home,
    );
    assert_eq!(updated["action"], "update");
    assert!(updated["item"]["text"]
        .as_str()
        .unwrap()
        .contains("Baseline-to-Compact proof"));

    let summary = qorx(&["memory", "summarize"], &qorx_home);
    assert_eq!(summary["action"], "summarize");
    assert!(summary["summary"]
        .as_str()
        .unwrap()
        .contains("Baseline-to-Compact proof"));

    let pruned = qorx(&["memory", "prune", "--max-items", "1"], &qorx_home);
    assert_eq!(pruned["action"], "prune");
    assert_eq!(pruned["items_kept"], 1);

    let deleted = qorx(&["memory", "delete", &id], &qorx_home);
    assert_eq!(deleted["action"], "delete");
    assert_eq!(deleted["deleted"], true);

    let empty = qorx(&["memory", "read", "provider traffic"], &qorx_home);
    assert!(empty["items"].as_array().unwrap().is_empty());

    let _ = fs::remove_dir_all(&qorx_home);
}
