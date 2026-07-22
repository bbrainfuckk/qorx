use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    config::AppPaths,
    graph_view::{self, AtlasConnection, AtlasHub, AtlasRationale, AtlasReport, GraphView},
    index::RepoIndex,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasPack {
    pub schema: String,
    pub generated_at: String,
    pub report: AtlasReport,
    pub graph: GraphView,
    pub modalities: Vec<AtlasModality>,
    pub research_basis: Vec<AtlasResearchNote>,
    pub boundary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasModality {
    pub kind: String,
    pub file_count: usize,
    pub sample_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasResearchNote {
    pub name: String,
    pub source: String,
    pub use_in_qorx: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasGlobal {
    pub schema: String,
    pub updated_at: String,
    pub projects: Vec<AtlasGlobalProject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasGlobalProject {
    pub name: String,
    pub path: String,
    pub schema: String,
    pub item_count: u64,
    pub link_count: u64,
    pub health: String,
    pub added_at: String,
}

pub fn build_pack(index: &RepoIndex, limit: usize) -> AtlasPack {
    let graph = graph_view::build_dashboard_graph(index, limit);
    let report = graph_view::build_atlas_report(index, limit);
    AtlasPack {
        schema: "qorx.atlas-pack.v1".to_string(),
        generated_at: Utc::now().to_rfc3339(),
        modalities: atlas_modalities(&graph),
        research_basis: research_basis(),
        report,
        graph,
        boundary: "Qorx Atlas Pack is local-first. Code and text evidence are extracted from the local index; multimodal files are inventoried deterministically unless a local extraction adapter is explicitly added.".to_string(),
    }
}

pub fn export_pack(index: &RepoIndex, limit: usize, out_dir: &Path) -> Result<Value> {
    fs::create_dir_all(out_dir)
        .with_context(|| format!("create Atlas output dir {}", out_dir.display()))?;
    let pack = build_pack(index, limit);
    let json_path = out_dir.join("atlas.json");
    let markdown_path = out_dir.join("ATLAS_REPORT.md");
    let html_path = out_dir.join("atlas.html");
    let manifest_path = out_dir.join("manifest.json");
    let agent_path = out_dir.join("AGENTS.atlas.md");

    fs::write(&json_path, serde_json::to_vec_pretty(&pack)?)?;
    fs::write(&markdown_path, render_markdown(&pack))?;
    fs::write(&html_path, render_html(&pack)?)?;
    fs::write(&agent_path, render_agent_instructions())?;

    let manifest = json!({
        "schema": "qorx.atlas-manifest.v1",
        "generated_at": pack.generated_at,
        "files": {
            "json": json_path.to_string_lossy(),
            "markdown": markdown_path.to_string_lossy(),
            "html": html_path.to_string_lossy(),
            "agent_instructions": agent_path.to_string_lossy()
        },
        "boundary": "Commit or share this pack only when local project policy allows it."
    });
    fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;

    Ok(json!({
        "schema": "qorx.atlas-export.v1",
        "files": {
            "json": json_path.to_string_lossy(),
            "markdown": markdown_path.to_string_lossy(),
            "html": html_path.to_string_lossy(),
            "manifest": manifest_path.to_string_lossy(),
            "agent_instructions": agent_path.to_string_lossy()
        },
        "item_count": pack.report.item_count,
        "link_count": pack.report.link_count,
        "local_only": true,
        "provider_calls": 0
    }))
}

pub fn global_path(paths: &AppPaths) -> PathBuf {
    paths.data_dir.join("atlas-global.json")
}

pub fn load_global(paths: &AppPaths) -> Result<AtlasGlobal> {
    let path = global_path(paths);
    if !path.exists() {
        return Ok(empty_global());
    }
    let bytes = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
}

pub fn add_global(paths: &AppPaths, atlas_path: &Path, name: &str) -> Result<AtlasGlobal> {
    let value = read_json(atlas_path)?;
    let report = report_value(&value)?;
    let schema = value
        .get("schema")
        .and_then(Value::as_str)
        .unwrap_or("qorx.atlas-report.v1")
        .to_string();
    let project = AtlasGlobalProject {
        name: name.to_string(),
        path: atlas_path.to_string_lossy().to_string(),
        schema,
        item_count: report
            .get("item_count")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        link_count: report
            .get("link_count")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        health: report
            .get("health")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        added_at: Utc::now().to_rfc3339(),
    };

    let mut global = load_global(paths)?;
    global.projects.retain(|item| item.name != project.name);
    global.projects.push(project);
    global.projects.sort_by(|a, b| a.name.cmp(&b.name));
    global.updated_at = Utc::now().to_rfc3339();
    let path = global_path(paths);
    fs::write(&path, serde_json::to_vec_pretty(&global)?)?;
    Ok(global)
}

pub fn merge_packs(inputs: &[PathBuf], out: &Path) -> Result<Value> {
    if inputs.is_empty() {
        return Err(anyhow!("atlas merge requires at least one input"));
    }
    let mut hubs = BTreeMap::<String, Value>::new();
    let mut connections = BTreeMap::<String, Value>::new();
    let mut rationale = BTreeMap::<String, Value>::new();
    let mut questions = BTreeSet::<String>::new();
    let mut modalities = BTreeMap::<String, usize>::new();

    for input in inputs {
        let value = read_json(input)?;
        let report = report_value(&value)?;
        merge_hubs(report, &mut hubs);
        merge_array(report, "surprising_connections", &mut connections, |item| {
            format!(
                "{}->{}",
                item.get("from_path").and_then(Value::as_str).unwrap_or(""),
                item.get("to_path").and_then(Value::as_str).unwrap_or("")
            )
        });
        merge_array(report, "rationale", &mut rationale, |item| {
            format!(
                "{}:{}:{}",
                item.get("path").and_then(Value::as_str).unwrap_or(""),
                item.get("line").and_then(Value::as_u64).unwrap_or_default(),
                item.get("marker").and_then(Value::as_str).unwrap_or("")
            )
        });
        if let Some(items) = report.get("suggested_questions").and_then(Value::as_array) {
            questions.extend(items.iter().filter_map(Value::as_str).map(str::to_string));
        }
        if let Some(items) = value.get("modalities").and_then(Value::as_array) {
            for item in items {
                let kind = item
                    .get("kind")
                    .and_then(Value::as_str)
                    .unwrap_or("other")
                    .to_string();
                let count = item
                    .get("file_count")
                    .and_then(Value::as_u64)
                    .unwrap_or_default() as usize;
                *modalities.entry(kind).or_default() += count;
            }
        }
    }

    let merged = json!({
        "schema": "qorx.atlas-merged.v1",
        "generated_at": Utc::now().to_rfc3339(),
        "input_count": inputs.len(),
        "hubs": hubs.into_values().collect::<Vec<_>>(),
        "surprising_connections": connections.into_values().collect::<Vec<_>>(),
        "rationale": rationale.into_values().collect::<Vec<_>>(),
        "suggested_questions": questions.into_iter().take(12).collect::<Vec<_>>(),
        "modalities": modalities.into_iter().map(|(kind, file_count)| json!({"kind": kind, "file_count": file_count})).collect::<Vec<_>>(),
        "boundary": "Merged Atlas data is a union of local Atlas packs. It does not invent relationships that were not present in the inputs."
    });
    if let Some(parent) = out.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::write(out, serde_json::to_vec_pretty(&merged)?)?;
    Ok(json!({
        "schema": "qorx.atlas-merge.v1",
        "input_count": inputs.len(),
        "output": out.to_string_lossy(),
        "hubs": merged.get("hubs").and_then(Value::as_array).map_or(0, Vec::len),
        "connections": merged.get("surprising_connections").and_then(Value::as_array).map_or(0, Vec::len)
    }))
}

pub fn write_agent_instructions(out_dir: &Path) -> Result<Value> {
    fs::create_dir_all(out_dir)?;
    let path = out_dir.join("AGENTS.atlas.md");
    fs::write(&path, render_agent_instructions())?;
    Ok(json!({
        "schema": "qorx.atlas-hook-kit.v1",
        "path": path.to_string_lossy(),
        "boundary": "This writes a local instruction kit. It does not overwrite global agent config."
    }))
}

fn atlas_modalities(graph: &GraphView) -> Vec<AtlasModality> {
    let mut groups = BTreeMap::<String, BTreeSet<String>>::new();
    for path in graph
        .nodes
        .iter()
        .filter(|node| node.kind == "file")
        .filter_map(|node| node.path.as_deref())
    {
        groups
            .entry(modality_kind(path).to_string())
            .or_default()
            .insert(path.to_string());
    }
    groups
        .into_iter()
        .map(|(kind, paths)| AtlasModality {
            kind,
            file_count: paths.len(),
            sample_paths: paths.into_iter().take(5).collect(),
        })
        .collect()
}

fn modality_kind(path: &str) -> &'static str {
    match path
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "java" | "c" | "cpp" | "cs" | "rb"
        | "php" | "swift" | "kt" | "lua" | "zig" | "ps1" | "sql" => "code",
        "md" | "mdx" | "txt" | "rst" => "docs",
        "html" | "css" | "scss" => "web",
        "json" | "yaml" | "yml" | "toml" | "xml" | "csv" => "data",
        "pdf" => "pdf",
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "svg" | "ico" => "image",
        "mp4" | "mov" | "mkv" | "webm" => "video",
        "mp3" | "wav" | "m4a" | "flac" => "audio",
        "docx" | "xlsx" | "pptx" => "office",
        _ => "other",
    }
}

fn research_basis() -> Vec<AtlasResearchNote> {
    vec![
        AtlasResearchNote {
            name: "GraphRAG survey".to_string(),
            source: "https://arxiv.org/abs/2501.00309".to_string(),
            use_in_qorx: "Keep query processing, retrieval, organization, generation, and data-source boundaries explicit.".to_string(),
        },
        AtlasResearchNote {
            name: "GraphCoder".to_string(),
            source: "https://arxiv.org/abs/2406.07003".to_string(),
            use_in_qorx: "Prefer structured repository references over raw chunk similarity for code tasks.".to_string(),
        },
        AtlasResearchNote {
            name: "MMGraphRAG".to_string(),
            source: "https://arxiv.org/abs/2507.20804".to_string(),
            use_in_qorx: "Keep cross-modal reasoning paths inspectable when visual/document adapters are added.".to_string(),
        },
        AtlasResearchNote {
            name: "MegaRAG".to_string(),
            source: "https://arxiv.org/abs/2512.20626".to_string(),
            use_in_qorx: "Track multimodal evidence as graph-aware local assets, not just captions.".to_string(),
        },
        AtlasResearchNote {
            name: "M3KG-RAG".to_string(),
            source: "https://arxiv.org/abs/2512.20136".to_string(),
            use_in_qorx: "Use grounded selective pruning before sending context to an agent.".to_string(),
        },
    ]
}

fn render_markdown(pack: &AtlasPack) -> String {
    let mut md = String::new();
    md.push_str("# Qorx Atlas Report\n\n");
    md.push_str(&format!(
        "- Generated: {}\n- Items: {}\n- Links: {}\n- Health: {}\n- Local only: {}\n- Provider calls: {}\n\n",
        pack.generated_at,
        pack.report.item_count,
        pack.report.link_count,
        pack.report.health,
        pack.report.local_only,
        pack.report.provider_calls
    ));
    append_hubs(&mut md, &pack.report.hubs);
    append_connections(&mut md, &pack.report.surprising_connections);
    append_rationale(&mut md, &pack.report.rationale);
    append_questions(&mut md, &pack.report.suggested_questions);
    md.push_str("## Modalities\n\n");
    for item in &pack.modalities {
        md.push_str(&format!("- {}: {} files\n", item.kind, item.file_count));
    }
    md.push_str("\n## Research Basis\n\n");
    for item in &pack.research_basis {
        md.push_str(&format!(
            "- [{}]({}): {}\n",
            item.name, item.source, item.use_in_qorx
        ));
    }
    md.push_str("\n## Boundary\n\n");
    md.push_str(&pack.boundary);
    md.push('\n');
    md
}

fn render_html(pack: &AtlasPack) -> Result<String> {
    let json = serde_json::to_string_pretty(pack)?;
    Ok(format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Qorx Atlas</title>
<style>
body{{font-family:Inter,Segoe UI,Arial,sans-serif;margin:0;background:#0c1014;color:#eef3f7}}
main{{max-width:1100px;margin:0 auto;padding:28px}}
section{{border:1px solid #26313a;border-radius:8px;background:#121820;padding:16px;margin:12px 0}}
h1{{font-size:28px;margin:0 0 8px}} h2{{font-size:16px;margin:0 0 10px}}
input{{width:100%;box-sizing:border-box;border:1px solid #34424d;border-radius:8px;background:#0a0e12;color:#eef3f7;padding:10px}}
pre{{white-space:pre-wrap;word-break:break-word;color:#cbd6df}}
.grid{{display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:10px}}
.pill{{display:inline-block;border:1px solid #34424d;border-radius:999px;padding:5px 8px;color:#9fb4c8;margin:2px}}
</style>
</head>
<body>
<main>
<h1>Qorx Atlas</h1>
<p>Local graph-aware report for this workspace. No provider calls were made to generate this pack.</p>
<input id="filter" placeholder="Filter Atlas JSON">
<section><h2>Summary</h2><div class="grid">
<div class="pill">items: {}</div><div class="pill">links: {}</div><div class="pill">health: {}</div>
</div></section>
<section><h2>Atlas JSON</h2><pre id="json"></pre></section>
</main>
<script>
const atlas = {};
const text = JSON.stringify(atlas, null, 2);
const out = document.getElementById('json');
const filter = document.getElementById('filter');
function render(){{const q=filter.value.toLowerCase();out.textContent=q?text.split('\n').filter(l=>l.toLowerCase().includes(q)).join('\n'):text;}}
filter.addEventListener('input', render); render();
</script>
</body>
</html>"#,
        pack.report.item_count,
        pack.report.link_count,
        html_escape(&pack.report.health),
        json
    ))
}

fn render_agent_instructions() -> String {
    r#"# Qorx Atlas Agent Instructions

Read ATLAS_REPORT.md before broad file reads when this file is present.
Use atlas.json for exact local graph evidence and atlas.html for quick manual inspection.
Prefer extracted Qorx Atlas facts over guessed architecture.
Do not send secrets, credentials, private keys, or unrelated files to providers.
Treat Qorx Atlas confidence labels literally: EXTRACTED is read from local evidence; INFERRED is derived; AMBIGUOUS is not proof.
"#
    .to_string()
}

fn append_hubs(md: &mut String, hubs: &[AtlasHub]) {
    md.push_str("## Central Areas\n\n");
    if hubs.is_empty() {
        md.push_str("- No central areas found in the bounded graph.\n\n");
        return;
    }
    for hub in hubs {
        md.push_str(&format!(
            "- `{}`: {} incoming links ({})\n",
            hub.path, hub.incoming_links, hub.confidence
        ));
    }
    md.push('\n');
}

fn append_connections(md: &mut String, connections: &[AtlasConnection]) {
    md.push_str("## Cross-Area Links\n\n");
    if connections.is_empty() {
        md.push_str("- No cross-area links found in the bounded graph.\n\n");
        return;
    }
    for link in connections {
        md.push_str(&format!(
            "- `{}` -> `{}`: {} ({})\n",
            link.from_path, link.to_path, link.reason, link.confidence
        ));
    }
    md.push('\n');
}

fn append_rationale(md: &mut String, rationale: &[AtlasRationale]) {
    md.push_str("## Rationale Notes\n\n");
    if rationale.is_empty() {
        md.push_str("- No explicit WHY/NOTE/HACK/TODO/FIXME markers found.\n\n");
        return;
    }
    for item in rationale {
        md.push_str(&format!(
            "- `{}`:{} {}: {}\n",
            item.path, item.line, item.marker, item.text
        ));
    }
    md.push('\n');
}

fn append_questions(md: &mut String, questions: &[String]) {
    md.push_str("## Starter Questions\n\n");
    for question in questions {
        md.push_str(&format!("- {}\n", question));
    }
    md.push('\n');
}

fn empty_global() -> AtlasGlobal {
    AtlasGlobal {
        schema: "qorx.atlas-global.v1".to_string(),
        updated_at: Utc::now().to_rfc3339(),
        projects: Vec::new(),
    }
}

fn read_json(path: &Path) -> Result<Value> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse {}", path.display()))
}

fn report_value(value: &Value) -> Result<&Value> {
    if value.get("schema").and_then(Value::as_str) == Some("qorx.atlas-report.v1") {
        Ok(value)
    } else {
        value
            .get("report")
            .ok_or_else(|| anyhow!("Atlas JSON must be an atlas report or atlas pack"))
    }
}

fn merge_hubs(report: &Value, hubs: &mut BTreeMap<String, Value>) {
    let Some(items) = report.get("hubs").and_then(Value::as_array) else {
        return;
    };
    for item in items {
        let Some(path) = item.get("path").and_then(Value::as_str) else {
            continue;
        };
        let incoming = item
            .get("incoming_links")
            .and_then(Value::as_u64)
            .unwrap_or_default();
        hubs.entry(path.to_string())
            .and_modify(|existing| {
                let current = existing
                    .get("incoming_links")
                    .and_then(Value::as_u64)
                    .unwrap_or_default();
                existing["incoming_links"] = json!(current + incoming);
            })
            .or_insert_with(|| item.clone());
    }
}

fn merge_array<F>(report: &Value, field: &str, target: &mut BTreeMap<String, Value>, key_fn: F)
where
    F: Fn(&Value) -> String,
{
    let Some(items) = report.get(field).and_then(Value::as_array) else {
        return;
    };
    for item in items {
        target.entry(key_fn(item)).or_insert_with(|| item.clone());
    }
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
