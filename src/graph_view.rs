use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::{index::RepoIndex, text::without_string_literals};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphView {
    pub schema: String,
    pub indexed_tokens: u64,
    pub node_count: usize,
    pub edge_count: usize,
    pub metrics: GraphMetrics,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub tree: TreeNode,
    pub boundary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub path: Option<String>,
    pub token_estimate: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetrics {
    pub file_nodes: usize,
    pub symbol_nodes: usize,
    pub definition_edges: usize,
    pub reference_edges: usize,
    pub isolated_files: usize,
    pub component_count: usize,
    pub largest_component_files: usize,
    pub connected_file_ratio: f64,
    pub components: Vec<GraphComponent>,
    pub density: f64,
    pub health: String,
    pub top_referenced_files: Vec<GraphHotspot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphComponent {
    pub id: usize,
    pub file_count: usize,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphHotspot {
    pub path: String,
    pub incoming_references: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub name: String,
    pub kind: String,
    pub path: Option<String>,
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphTrace {
    pub schema: String,
    pub source: String,
    pub target: String,
    pub found: bool,
    pub hops: usize,
    pub path: Vec<GraphTraceStep>,
    pub boundary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphTraceStep {
    pub node_id: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasReport {
    pub schema: String,
    pub mode: String,
    pub local_only: bool,
    pub provider_calls: u64,
    pub indexed_tokens: u64,
    pub item_count: usize,
    pub link_count: usize,
    pub health: String,
    pub hubs: Vec<AtlasHub>,
    pub surprising_connections: Vec<AtlasConnection>,
    pub rationale: Vec<AtlasRationale>,
    pub suggested_questions: Vec<String>,
    pub confidence: BTreeMap<String, String>,
    pub boundary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasHub {
    pub path: String,
    pub incoming_links: usize,
    pub confidence: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasConnection {
    pub from_path: String,
    pub to_path: String,
    pub relation: String,
    pub confidence: String,
    pub score: usize,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasRationale {
    pub path: String,
    pub line: usize,
    pub marker: String,
    pub text: String,
    pub confidence: String,
}

pub fn build_dashboard_graph(index: &RepoIndex, limit: usize) -> GraphView {
    build_graph(index, limit, None)
}

pub fn build_query_graph(index: &RepoIndex, query: &str, limit: usize) -> GraphView {
    let mut paths = crate::index::search_index(index, query, limit.max(8))
        .into_iter()
        .map(|hit| hit.path)
        .collect::<BTreeSet<_>>();
    expand_paths_with_direct_references(index, &mut paths);
    build_graph(index, limit, Some(&paths))
}

pub fn build_atlas_report(index: &RepoIndex, limit: usize) -> AtlasReport {
    let graph = build_dashboard_graph(index, limit);
    let hubs = atlas_hubs(&graph);
    let surprising_connections = atlas_surprising_connections(&graph);
    let rationale = atlas_rationale(index);
    let suggested_questions = atlas_questions(&hubs, &surprising_connections, &rationale, &graph);

    AtlasReport {
        schema: "qorx.atlas-report.v1".to_string(),
        mode: "local_atlas_report".to_string(),
        local_only: true,
        provider_calls: 0,
        indexed_tokens: graph.indexed_tokens,
        item_count: graph.node_count,
        link_count: graph.metrics.reference_edges,
        health: graph.metrics.health.clone(),
        hubs,
        surprising_connections,
        rationale,
        suggested_questions,
        confidence: atlas_confidence_policy(),
        boundary: "Qorx Atlas is a deterministic local report over indexed quarks. It highlights central files, extracted cross-area references, rationale comments, and useful questions without sending project content to a provider.".to_string(),
    }
}

fn build_graph(
    index: &RepoIndex,
    limit: usize,
    allowed_paths: Option<&BTreeSet<String>>,
) -> GraphView {
    let limit = limit.clamp(8, 512);
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut seen_nodes = BTreeSet::new();
    let mut seen_edges = BTreeSet::new();
    let mut included_files = BTreeSet::new();
    let mut symbol_owners = BTreeMap::<String, BTreeSet<String>>::new();
    let mut root = TreeNode {
        name: "workspace".to_string(),
        kind: "root".to_string(),
        path: None,
        children: Vec::new(),
    };

    for atom in sorted_atoms(index)
        .into_iter()
        .filter(|atom| path_allowed(atom, allowed_paths))
    {
        for symbol in &atom.symbols {
            if is_graph_symbol(symbol) {
                symbol_owners
                    .entry(symbol.clone())
                    .or_default()
                    .insert(atom.path.clone());
            }
        }
    }

    for atom in sorted_atoms(index)
        .into_iter()
        .filter(|atom| path_allowed(atom, allowed_paths))
    {
        let file_id = file_node_id(&atom.path);
        if seen_nodes.insert(file_id.clone()) && nodes.len() < limit {
            insert_tree_path(&mut root, &atom.path);
            included_files.insert(atom.path.clone());
            nodes.push(GraphNode {
                id: file_id.clone(),
                label: atom.path.clone(),
                kind: "file".to_string(),
                path: Some(atom.path.clone()),
                token_estimate: atom.token_estimate,
            });
        }

        for symbol in atom.symbols.iter().filter(|symbol| is_graph_symbol(symbol)) {
            if nodes.len() >= limit {
                continue;
            }
            let symbol_id = symbol_node_id(symbol);
            if seen_nodes.insert(symbol_id.clone()) {
                nodes.push(GraphNode {
                    id: symbol_id.clone(),
                    label: symbol.clone(),
                    kind: "symbol".to_string(),
                    path: Some(atom.path.clone()),
                    token_estimate: 0,
                });
            }
            add_edge(
                &mut edges,
                &mut seen_edges,
                &file_id,
                &symbol_id,
                "defines",
                "EXTRACTED",
            );
        }
    }

    for atom in sorted_atoms(index)
        .into_iter()
        .filter(|atom| path_allowed(atom, allowed_paths))
    {
        if !included_files.contains(&atom.path) {
            continue;
        }
        let source_id = file_node_id(&atom.path);
        for symbol in referenced_symbols(&atom.text) {
            let Some(paths) = symbol_owners.get(&symbol) else {
                continue;
            };
            for target_path in paths {
                if target_path == &atom.path || !included_files.contains(target_path) {
                    continue;
                }
                add_edge(
                    &mut edges,
                    &mut seen_edges,
                    &source_id,
                    &file_node_id(target_path),
                    "references",
                    "EXTRACTED",
                );
            }
        }
    }

    sort_tree(&mut root);
    nodes.sort_by(|a, b| a.kind.cmp(&b.kind).then(a.label.cmp(&b.label)));
    edges.sort_by(|a, b| {
        a.source
            .cmp(&b.source)
            .then(a.target.cmp(&b.target))
            .then(a.relation.cmp(&b.relation))
    });

    GraphView {
        schema: "qorx.graph-view.v1".to_string(),
        indexed_tokens: index.total_tokens(),
        node_count: nodes.len(),
        edge_count: edges.len(),
        metrics: build_graph_metrics(&nodes, &edges),
        nodes,
        edges,
        tree: root,
        boundary: "Qorx graph view is a bounded local projection over indexed quarks. It emits files, symbols, extracted references, and a folder tree for the local dashboard without external graph runtimes.".to_string(),
    }
}

fn expand_paths_with_direct_references(index: &RepoIndex, paths: &mut BTreeSet<String>) {
    let mut symbol_owners = BTreeMap::<String, BTreeSet<String>>::new();
    for atom in sorted_atoms(index) {
        for symbol in atom.symbols.iter().filter(|symbol| is_graph_symbol(symbol)) {
            symbol_owners
                .entry(symbol.clone())
                .or_default()
                .insert(atom.path.clone());
        }
    }

    let mut extra = BTreeSet::new();
    for atom in sorted_atoms(index) {
        if !paths.contains(&atom.path) {
            continue;
        }
        for symbol in referenced_symbols(&atom.text) {
            if let Some(owners) = symbol_owners.get(&symbol) {
                extra.extend(owners.iter().cloned());
            }
        }
    }
    paths.extend(extra);
}

fn path_allowed(atom: &&crate::index::RepoAtom, allowed_paths: Option<&BTreeSet<String>>) -> bool {
    allowed_paths.is_none_or(|paths| paths.contains(&atom.path))
}

pub fn trace_file_path(index: &RepoIndex, source: &str, target: &str, limit: usize) -> GraphTrace {
    let graph = build_dashboard_graph(index, limit);
    let source_id = resolve_file_node(&graph, source);
    let target_id = resolve_file_node(&graph, target);
    let mut trace = GraphTrace {
        schema: "qorx.graph-trace.v1".to_string(),
        source: source.to_string(),
        target: target.to_string(),
        found: false,
        hops: 0,
        path: Vec::new(),
        boundary: "Qorx graph trace follows extracted file reference edges only. Missing paths mean the bounded graph or symbol extraction did not prove a connection.".to_string(),
    };
    let (Some(source_id), Some(target_id)) = (source_id, target_id) else {
        return trace;
    };

    let adjacency = reference_adjacency(&graph.edges);
    let mut queue = VecDeque::from([source_id.clone()]);
    let mut seen = BTreeSet::from([source_id.clone()]);
    let mut previous = BTreeMap::<String, String>::new();

    while let Some(current) = queue.pop_front() {
        if current == target_id {
            trace.found = true;
            break;
        }
        for next in adjacency.get(&current).into_iter().flatten() {
            if seen.insert(next.clone()) {
                previous.insert(next.clone(), current.clone());
                queue.push_back(next.clone());
            }
        }
    }

    if !trace.found {
        return trace;
    }

    let mut ids = vec![target_id.clone()];
    let mut cursor = target_id;
    while cursor != source_id {
        let Some(prev) = previous.get(&cursor).cloned() else {
            trace.found = false;
            trace.path.clear();
            return trace;
        };
        ids.push(prev.clone());
        cursor = prev;
    }
    ids.reverse();
    trace.hops = ids.len().saturating_sub(1);
    trace.path = ids
        .into_iter()
        .map(|id| GraphTraceStep {
            path: id.trim_start_matches("file:").to_string(),
            node_id: id,
        })
        .collect();
    trace
}

fn build_graph_metrics(nodes: &[GraphNode], edges: &[GraphEdge]) -> GraphMetrics {
    let file_nodes = nodes.iter().filter(|node| node.kind == "file").count();
    let symbol_nodes = nodes.iter().filter(|node| node.kind == "symbol").count();
    let definition_edges = edges
        .iter()
        .filter(|edge| edge.relation == "defines")
        .count();
    let reference_edges = edges
        .iter()
        .filter(|edge| edge.relation == "references")
        .count();
    let density = if nodes.len() > 1 {
        edges.len() as f64 / (nodes.len() * (nodes.len() - 1)) as f64
    } else {
        0.0
    };

    let file_ids = nodes
        .iter()
        .filter(|node| node.kind == "file")
        .map(|node| node.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut file_reference_degree = BTreeMap::<String, usize>::new();
    let mut incoming_references = BTreeMap::<String, usize>::new();
    let mut neighbors = file_ids
        .iter()
        .map(|id| ((*id).to_string(), BTreeSet::<String>::new()))
        .collect::<BTreeMap<_, _>>();

    for edge in edges.iter().filter(|edge| edge.relation == "references") {
        if file_ids.contains(edge.source.as_str()) {
            *file_reference_degree
                .entry(edge.source.clone())
                .or_default() += 1;
        }
        if file_ids.contains(edge.target.as_str()) {
            *file_reference_degree
                .entry(edge.target.clone())
                .or_default() += 1;
            *incoming_references.entry(edge.target.clone()).or_default() += 1;
        }
        if file_ids.contains(edge.source.as_str()) && file_ids.contains(edge.target.as_str()) {
            neighbors
                .entry(edge.source.clone())
                .or_default()
                .insert(edge.target.clone());
            neighbors
                .entry(edge.target.clone())
                .or_default()
                .insert(edge.source.clone());
        }
    }

    let isolated_files = file_ids
        .iter()
        .filter(|id| !file_reference_degree.contains_key(**id))
        .count();
    let components = graph_components(&neighbors);
    let largest_component_files = components
        .iter()
        .map(|component| component.file_count)
        .max()
        .unwrap_or(0);
    let connected_file_ratio = if file_nodes == 0 {
        0.0
    } else {
        (file_nodes.saturating_sub(isolated_files)) as f64 / file_nodes as f64
    };
    let mut top_referenced_files = incoming_references
        .into_iter()
        .filter_map(|(id, incoming_references)| {
            id.strip_prefix("file:").map(|path| GraphHotspot {
                path: path.to_string(),
                incoming_references,
            })
        })
        .collect::<Vec<_>>();
    top_referenced_files.sort_by(|a, b| {
        b.incoming_references
            .cmp(&a.incoming_references)
            .then(a.path.cmp(&b.path))
    });
    top_referenced_files.truncate(8);

    GraphMetrics {
        file_nodes,
        symbol_nodes,
        definition_edges,
        reference_edges,
        isolated_files,
        component_count: components.len(),
        largest_component_files,
        connected_file_ratio,
        components,
        density,
        health: graph_health(file_nodes, reference_edges, isolated_files),
        top_referenced_files,
    }
}

fn atlas_hubs(graph: &GraphView) -> Vec<AtlasHub> {
    graph
        .metrics
        .top_referenced_files
        .iter()
        .map(|hotspot| AtlasHub {
            path: hotspot.path.clone(),
            incoming_links: hotspot.incoming_references,
            confidence: "EXTRACTED".to_string(),
            reason: "Other indexed files reference this path.".to_string(),
        })
        .collect()
}

fn atlas_surprising_connections(graph: &GraphView) -> Vec<AtlasConnection> {
    let paths_by_id = graph
        .nodes
        .iter()
        .filter(|node| node.kind == "file")
        .filter_map(|node| {
            node.path
                .as_ref()
                .map(|path| (node.id.clone(), path.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    let mut connections = graph
        .edges
        .iter()
        .filter(|edge| edge.relation == "references")
        .filter_map(|edge| {
            let from_path = paths_by_id.get(&edge.source)?;
            let to_path = paths_by_id.get(&edge.target)?;
            let score = atlas_surprise_score(from_path, to_path);
            Some(AtlasConnection {
                from_path: from_path.clone(),
                to_path: to_path.clone(),
                relation: edge.relation.clone(),
                confidence: edge.confidence.clone(),
                score,
                reason: atlas_surprise_reason(from_path, to_path),
            })
        })
        .collect::<Vec<_>>();

    connections.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then(a.from_path.cmp(&b.from_path))
            .then(a.to_path.cmp(&b.to_path))
    });
    connections.truncate(8);
    connections
}

fn atlas_rationale(index: &RepoIndex) -> Vec<AtlasRationale> {
    let mut rationale = Vec::new();
    for atom in sorted_atoms(index) {
        for (offset, line) in atom.text.lines().enumerate() {
            let Some((marker, text)) = rationale_marker(line) else {
                continue;
            };
            rationale.push(AtlasRationale {
                path: atom.path.clone(),
                line: atom.start_line + offset,
                marker,
                text,
                confidence: "EXTRACTED".to_string(),
            });
            if rationale.len() >= 12 {
                return rationale;
            }
        }
    }
    rationale
}

fn atlas_questions(
    hubs: &[AtlasHub],
    connections: &[AtlasConnection],
    rationale: &[AtlasRationale],
    graph: &GraphView,
) -> Vec<String> {
    let mut questions = Vec::new();
    if let Some(hub) = hubs.first() {
        questions.push(format!(
            "Why does {} receive the most local references?",
            short_path(&hub.path)
        ));
    }
    if let Some(connection) = connections.first() {
        questions.push(format!(
            "What work depends on the route from {} to {}?",
            short_path(&connection.from_path),
            short_path(&connection.to_path)
        ));
    }
    if let Some(item) = rationale.first() {
        questions.push(format!(
            "Which decision is documented near {}:{}?",
            short_path(&item.path),
            item.line
        ));
    }
    if graph.metrics.isolated_files > 0 {
        questions.push("Which quiet files should be connected to the main work?".to_string());
    }
    questions.push("Which local areas should Qorx read before changing this project?".to_string());
    questions.push("Where are the important paths for the next operator task?".to_string());

    let mut seen = BTreeSet::new();
    questions
        .into_iter()
        .filter(|question| seen.insert(question.clone()))
        .take(5)
        .collect()
}

fn atlas_confidence_policy() -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "AMBIGUOUS".to_string(),
            "Possible but not proven by the current local index.".to_string(),
        ),
        (
            "EXTRACTED".to_string(),
            "Qorx read directly from indexed local files or extracted references.".to_string(),
        ),
        (
            "INFERRED".to_string(),
            "Derived from local structure and marked separately from extracted facts.".to_string(),
        ),
    ])
}

fn atlas_surprise_score(from_path: &str, to_path: &str) -> usize {
    let mut score = 1;
    if top_area(from_path) != top_area(to_path) {
        score += 3;
    }
    if parent_path(from_path) != parent_path(to_path) {
        score += 2;
    }
    if extension(from_path) != extension(to_path) {
        score += 1;
    }
    score
}

fn atlas_surprise_reason(from_path: &str, to_path: &str) -> String {
    if top_area(from_path) != top_area(to_path) {
        "Different top-level areas are linked by an extracted reference.".to_string()
    } else if parent_path(from_path) != parent_path(to_path) {
        "Different folders are linked by an extracted reference.".to_string()
    } else {
        "Nearby files are linked by an extracted reference.".to_string()
    }
}

fn rationale_marker(line: &str) -> Option<(String, String)> {
    let upper = line.to_ascii_uppercase();
    for marker in ["WHY", "NOTE", "HACK", "TODO", "FIXME"] {
        let needle = format!("{marker}:");
        if let Some(start) = upper.find(&needle) {
            let before = &line[..start];
            let before_trimmed = before.trim();
            let marker_text = &line[start..start + marker.len()];
            let has_comment_prefix = before_trimmed.ends_with("//")
                || before_trimmed.ends_with('#')
                || before_trimmed.ends_with('*')
                || before_trimmed.ends_with("/*")
                || before_trimmed.ends_with("<!--");
            if !has_comment_prefix && marker_text != marker {
                continue;
            }
            let text = line[start + needle.len()..]
                .trim()
                .trim_start_matches(['/', '#', '*', '-', ' '])
                .trim()
                .to_string();
            if !text.is_empty() {
                return Some((marker.to_string(), text));
            }
        }
    }
    None
}

fn top_area(path: &str) -> &str {
    path.split('/').next().unwrap_or(path)
}

fn parent_path(path: &str) -> &str {
    path.rsplit_once('/')
        .map(|(parent, _)| parent)
        .unwrap_or("")
}

fn extension(path: &str) -> &str {
    path.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("")
}

fn short_path(path: &str) -> String {
    let parts = path.split('/').collect::<Vec<_>>();
    if parts.len() <= 2 {
        path.to_string()
    } else {
        parts[parts.len() - 2..].join("/")
    }
}

fn resolve_file_node(graph: &GraphView, query: &str) -> Option<String> {
    let query = query.trim();
    let mut candidates = graph
        .nodes
        .iter()
        .filter(|node| node.kind == "file")
        .filter(|node| {
            node.path
                .as_deref()
                .is_some_and(|path| path == query || path.ends_with(query) || path.contains(query))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|a, b| {
        let a_path = a.path.as_deref().unwrap_or_default();
        let b_path = b.path.as_deref().unwrap_or_default();
        exact_path_rank(a_path, query)
            .cmp(&exact_path_rank(b_path, query))
            .then(a_path.len().cmp(&b_path.len()))
            .then(a_path.cmp(b_path))
    });
    candidates.first().map(|node| node.id.clone())
}

fn exact_path_rank(path: &str, query: &str) -> u8 {
    if path == query {
        0
    } else if path.ends_with(query) {
        1
    } else {
        2
    }
}

fn reference_adjacency(edges: &[GraphEdge]) -> BTreeMap<String, BTreeSet<String>> {
    let mut adjacency = BTreeMap::<String, BTreeSet<String>>::new();
    for edge in edges.iter().filter(|edge| edge.relation == "references") {
        adjacency
            .entry(edge.source.clone())
            .or_default()
            .insert(edge.target.clone());
    }
    adjacency
}

fn graph_components(neighbors: &BTreeMap<String, BTreeSet<String>>) -> Vec<GraphComponent> {
    let mut seen = BTreeSet::<String>::new();
    let mut components = Vec::new();

    for id in neighbors.keys() {
        if seen.contains(id) {
            continue;
        }
        let mut stack = vec![id.clone()];
        let mut paths = Vec::new();
        seen.insert(id.clone());

        while let Some(current) = stack.pop() {
            if let Some(path) = current.strip_prefix("file:") {
                paths.push(path.to_string());
            }
            if let Some(next) = neighbors.get(&current) {
                for neighbor in next {
                    if seen.insert(neighbor.clone()) {
                        stack.push(neighbor.clone());
                    }
                }
            }
        }

        paths.sort();
        components.push(GraphComponent {
            id: components.len() + 1,
            file_count: paths.len(),
            paths,
        });
    }

    components.sort_by(|a, b| b.file_count.cmp(&a.file_count).then(a.paths.cmp(&b.paths)));
    for (idx, component) in components.iter_mut().enumerate() {
        component.id = idx + 1;
    }
    components.truncate(12);
    components
}

fn graph_health(file_nodes: usize, reference_edges: usize, isolated_files: usize) -> String {
    if file_nodes == 0 {
        "empty".to_string()
    } else if reference_edges == 0 || isolated_files > 0 {
        "needs_attention".to_string()
    } else {
        "connected".to_string()
    }
}

fn sorted_atoms(index: &RepoIndex) -> Vec<&crate::index::RepoAtom> {
    let mut atoms = index.atoms.iter().collect::<Vec<_>>();
    atoms.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.start_line.cmp(&b.start_line))
            .then(a.id.cmp(&b.id))
    });
    atoms
}

fn file_node_id(path: &str) -> String {
    format!("file:{path}")
}

fn symbol_node_id(symbol: &str) -> String {
    format!("symbol:{symbol}")
}

fn add_edge(
    edges: &mut Vec<GraphEdge>,
    seen_edges: &mut BTreeSet<(String, String, String)>,
    source: &str,
    target: &str,
    relation: &str,
    confidence: &str,
) {
    if seen_edges.insert((source.to_string(), target.to_string(), relation.to_string())) {
        edges.push(GraphEdge {
            source: source.to_string(),
            target: target.to_string(),
            relation: relation.to_string(),
            confidence: confidence.to_string(),
        });
    }
}

fn referenced_symbols(text: &str) -> BTreeSet<String> {
    let mut symbols = BTreeSet::new();
    let code = without_string_literals(text);
    let mut current = String::new();
    for ch in code.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            current.push(ch);
        } else {
            push_symbol_token(&mut symbols, &mut current);
        }
    }
    push_symbol_token(&mut symbols, &mut current);
    symbols
}

fn push_symbol_token(symbols: &mut BTreeSet<String>, current: &mut String) {
    if current
        .chars()
        .next()
        .is_some_and(|ch| !ch.is_ascii_digit())
        && is_graph_symbol(current)
    {
        symbols.insert(std::mem::take(current));
    }
    current.clear();
}

fn is_graph_symbol(symbol: &str) -> bool {
    symbol.len() > 2
        && !matches!(
            symbol,
            "const" | "else" | "false" | "fn" | "let" | "mod" | "pub" | "return" | "true" | "use"
        )
}

fn insert_tree_path(root: &mut TreeNode, path: &str) {
    let mut cursor = root;
    let parts = path
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    for (idx, part) in parts.iter().enumerate() {
        let is_file = idx + 1 == parts.len();
        let node_path = parts[..=idx].join("/");
        let existing = cursor.children.iter().position(|child| child.name == *part);
        let child_idx = match existing {
            Some(existing) => existing,
            None => {
                cursor.children.push(TreeNode {
                    name: (*part).to_string(),
                    kind: if is_file { "file" } else { "directory" }.to_string(),
                    path: is_file.then_some(node_path),
                    children: Vec::new(),
                });
                cursor.children.len() - 1
            }
        };
        cursor = &mut cursor.children[child_idx];
    }
}

fn sort_tree(node: &mut TreeNode) {
    node.children
        .sort_by(|a, b| a.kind.cmp(&b.kind).then(a.name.cmp(&b.name)));
    for child in &mut node.children {
        sort_tree(child);
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::index::RepoAtom;

    use super::*;

    fn atom(id: &str, path: &str, symbols: &[&str], text: &str) -> RepoAtom {
        RepoAtom {
            id: id.to_string(),
            path: path.to_string(),
            start_line: 1,
            end_line: text.lines().count().max(1),
            hash: id.to_string(),
            token_estimate: 16,
            symbols: symbols.iter().map(|symbol| (*symbol).to_string()).collect(),
            signal_mask: 0,
            vector: Vec::new(),
            text: text.to_string(),
        }
    }

    #[test]
    fn graph_view_exposes_files_symbols_and_reference_edges() {
        let index = RepoIndex {
            root: "test".to_string(),
            updated_at: Utc::now(),
            atoms: vec![
                atom(
                    "api",
                    "src/api.rs",
                    &["handle_login"],
                    "fn handle_login() {}",
                ),
                atom(
                    "router",
                    "src/router.rs",
                    &["mount_routes"],
                    "fn mount_routes() { handle_login(); }",
                ),
            ],
        };

        let graph = build_dashboard_graph(&index, 128);

        assert!(graph
            .nodes
            .iter()
            .any(|node| node.id == "file:src/api.rs" && node.kind == "file"));
        assert!(graph
            .nodes
            .iter()
            .any(|node| node.id == "symbol:handle_login" && node.kind == "symbol"));
        assert!(graph.edges.iter().any(|edge| {
            edge.source == "file:src/router.rs"
                && edge.target == "file:src/api.rs"
                && edge.relation == "references"
                && edge.confidence == "EXTRACTED"
        }));
    }

    #[test]
    fn graph_view_builds_a_workspace_tree_for_the_dashboard() {
        let index = RepoIndex {
            root: "test".to_string(),
            updated_at: Utc::now(),
            atoms: vec![
                atom(
                    "api",
                    "src/api.rs",
                    &["handle_login"],
                    "fn handle_login() {}",
                ),
                atom("doc", "docs/ops.md", &[], "ops runbook"),
            ],
        };

        let graph = build_dashboard_graph(&index, 128);
        let src = graph
            .tree
            .children
            .iter()
            .find(|node| node.name == "src")
            .expect("src directory");
        assert!(src
            .children
            .iter()
            .any(|node| node.name == "api.rs" && node.path.as_deref() == Some("src/api.rs")));
    }

    #[test]
    fn graph_view_honors_the_node_limit_for_minimal_footprint() {
        let index = RepoIndex {
            root: "test".to_string(),
            updated_at: Utc::now(),
            atoms: (0..50)
                .map(|idx| {
                    atom(
                        &format!("a{idx}"),
                        &format!("src/file_{idx}.rs"),
                        &[&format!("Symbol{idx}")],
                        &format!("fn Symbol{idx}() {{}}"),
                    )
                })
                .collect(),
        };

        let graph = build_dashboard_graph(&index, 24);

        assert!(graph.nodes.len() <= 24);
        assert_eq!(graph.node_count, graph.nodes.len());
        assert_eq!(graph.edge_count, graph.edges.len());
    }

    #[test]
    fn graph_view_reports_production_graph_health_metrics() {
        let index = RepoIndex {
            root: "test".to_string(),
            updated_at: Utc::now(),
            atoms: vec![
                atom("auth", "src/auth.rs", &["login_user"], "fn login_user() {}"),
                atom(
                    "api",
                    "src/api.rs",
                    &["handle_api"],
                    "fn handle_api() { login_user(); }",
                ),
                atom("lonely", "docs/runbook.md", &[], "ops runbook"),
            ],
        };

        let graph = build_dashboard_graph(&index, 128);

        assert_eq!(graph.metrics.file_nodes, 3);
        assert_eq!(graph.metrics.symbol_nodes, 2);
        assert_eq!(graph.metrics.definition_edges, 2);
        assert_eq!(graph.metrics.reference_edges, 1);
        assert_eq!(graph.metrics.isolated_files, 1);
        assert!(graph.metrics.density > 0.0);
        assert_eq!(graph.metrics.health, "needs_attention");
        assert!(graph
            .metrics
            .top_referenced_files
            .iter()
            .any(|hotspot| { hotspot.path == "src/auth.rs" && hotspot.incoming_references == 1 }));
    }

    #[test]
    fn graph_view_reports_file_components_for_architecture_islands() {
        let index = RepoIndex {
            root: "test".to_string(),
            updated_at: Utc::now(),
            atoms: vec![
                atom("a", "src/a.rs", &["alpha"], "fn alpha() { beta(); }"),
                atom("b", "src/b.rs", &["beta"], "fn beta() {}"),
                atom("c", "src/c.rs", &["gamma"], "fn gamma() { delta(); }"),
                atom("d", "src/d.rs", &["delta"], "fn delta() {}"),
                atom("e", "docs/alone.md", &[], "no code references"),
            ],
        };

        let graph = build_dashboard_graph(&index, 128);

        assert_eq!(graph.metrics.component_count, 3);
        assert_eq!(graph.metrics.largest_component_files, 2);
        assert_eq!(graph.metrics.components[0].file_count, 2);
        assert!(graph
            .metrics
            .components
            .iter()
            .any(|component| component.paths == vec!["docs/alone.md"]));
        assert!(graph.metrics.connected_file_ratio > 0.0);
        assert!(graph.metrics.connected_file_ratio < 1.0);
    }

    #[test]
    fn graph_trace_finds_multi_hop_file_reference_path() {
        let index = RepoIndex {
            root: "test".to_string(),
            updated_at: Utc::now(),
            atoms: vec![
                atom(
                    "routes",
                    "src/routes.rs",
                    &["handle_checkout"],
                    "fn handle_checkout() { charge_customer(); }",
                ),
                atom(
                    "billing",
                    "src/billing.rs",
                    &["charge_customer"],
                    "fn charge_customer() { write_ledger(); }",
                ),
                atom(
                    "ledger",
                    "src/ledger.rs",
                    &["write_ledger"],
                    "fn write_ledger() {}",
                ),
            ],
        };

        let trace = trace_file_path(&index, "routes.rs", "ledger.rs", 128);

        assert!(trace.found);
        assert_eq!(trace.hops, 2);
        assert_eq!(
            trace
                .path
                .iter()
                .map(|step| step.path.as_str())
                .collect::<Vec<_>>(),
            vec!["src/routes.rs", "src/billing.rs", "src/ledger.rs"]
        );
    }

    #[test]
    fn atlas_rationale_ignores_lowercase_struct_fields() {
        assert!(rationale_marker("    note: String,").is_none());
        assert_eq!(
            rationale_marker("  // WHY: keep the local proof chain")
                .expect("comment marker")
                .1,
            "keep the local proof chain"
        );
        assert_eq!(
            rationale_marker("NOTE: architecture decision")
                .expect("uppercase marker")
                .1,
            "architecture decision"
        );
    }
}
