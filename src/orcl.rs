use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::{
    compression::estimate_tokens,
    impact,
    index::{search_index, RepoAtom, RepoIndex},
    text::without_string_literals,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrclSymbol {
    pub name: String,
    pub path: String,
    pub quark_id: String,
    pub start_line: usize,
    pub end_line: usize,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrclLink {
    pub from_path: String,
    pub to_path: String,
    pub from_quark_id: String,
    pub to_quark_id: String,
    pub symbol: String,
    pub kind: String,
    pub proof: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrclRank {
    pub symbol: String,
    pub path: String,
    pub quark_id: String,
    pub fan_in: usize,
    pub fan_out: usize,
    pub score: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrclQuark {
    pub id: String,
    pub path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub reason: String,
    pub token_estimate: u64,
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrclReport {
    pub schema: String,
    pub query: String,
    pub changed_paths: Vec<String>,
    pub related_paths: Vec<String>,
    pub depth: usize,
    pub budget_tokens: u64,
    pub indexed_tokens: u64,
    pub used_tokens: u64,
    pub omitted_tokens: u64,
    pub context_reduction_x: f64,
    pub rank: Vec<OrclRank>,
    pub symbols: Vec<OrclSymbol>,
    pub links: Vec<OrclLink>,
    pub quarks: Vec<OrclQuark>,
    pub text: String,
    pub boundary: String,
}

#[derive(Debug, Clone, Copy)]
pub struct OrclOptions {
    pub budget_tokens: u64,
    pub depth: usize,
    pub limit: usize,
}

pub fn report(
    index: &RepoIndex,
    query: &str,
    diff: Option<&str>,
    options: OrclOptions,
) -> OrclReport {
    let budget_tokens = options.budget_tokens.clamp(128, 20_000);
    let depth = options.depth.clamp(1, 6);
    let limit = options.limit.clamp(1, 64);
    let indexed_tokens = index.total_tokens();
    let changed_paths = diff
        .map(impact::changed_paths_from_diff)
        .unwrap_or_default()
        .into_iter()
        .collect::<Vec<_>>();
    let changed_set = changed_paths.iter().cloned().collect::<BTreeSet<_>>();
    let symbols = symbol_contracts(index);
    let links = structural_links(index, &symbols);
    let rank = rank_symbols(&symbols, &links, query, limit);

    let mut seed_paths = changed_set.clone();
    if seed_paths.is_empty() {
        for hit in search_index(index, query, limit.min(8)) {
            seed_paths.insert(hit.path);
        }
    }
    if seed_paths.is_empty() {
        for item in rank.iter().take(3) {
            seed_paths.insert(item.path.clone());
        }
    }

    let path_scope = walk_paths(&seed_paths, &links, depth);
    let related_paths = path_scope
        .iter()
        .filter(|path| !changed_set.contains(*path) && !seed_paths.contains(*path))
        .cloned()
        .collect::<Vec<_>>();
    let selected_links = select_links(&links, &path_scope, limit);
    let selected_symbols = select_symbols(&symbols, &path_scope, query, limit);
    let mut text = render_header(RenderHeader {
        query,
        budget_tokens,
        indexed_tokens,
        depth,
        changed_paths: &changed_paths,
        related_paths: &related_paths,
        rank: &rank,
        symbols: &selected_symbols,
        links: &selected_links,
    });

    let atoms_by_id = index.atom_lookup();
    let mut quarks = Vec::new();
    let mut used_tokens = estimate_tokens(&text);
    let candidates = candidate_quarks(index, &path_scope, query, &changed_set);
    for (atom_id, reason) in candidates {
        let Some(atom) = atoms_by_id.get(atom_id.as_str()).copied() else {
            continue;
        };
        let header = format!(
            "\nQUARK {} {}:{}-{} reason={reason} tokens={}\n",
            atom.id, atom.path, atom.start_line, atom.end_line, atom.token_estimate
        );
        let needed = estimate_tokens(&header) + atom.token_estimate;
        if used_tokens + needed > budget_tokens {
            continue;
        }
        used_tokens += needed;
        text.push_str(&header);
        text.push_str(&atom.text);
        text.push('\n');
        quarks.push(OrclQuark {
            id: atom.id.clone(),
            path: atom.path.clone(),
            start_line: atom.start_line,
            end_line: atom.end_line,
            reason,
            token_estimate: atom.token_estimate,
            symbols: atom.symbols.clone(),
        });
    }

    let omitted_tokens = indexed_tokens.saturating_sub(used_tokens.min(indexed_tokens));
    let context_reduction_x = indexed_tokens.max(1) as f64 / used_tokens.max(1) as f64;
    OrclReport {
        schema: "qorx.orcl.v1".to_string(),
        query: query.to_string(),
        changed_paths,
        related_paths,
        depth,
        budget_tokens,
        indexed_tokens,
        used_tokens,
        omitted_tokens,
        context_reduction_x,
        rank,
        symbols: selected_symbols,
        links: selected_links,
        quarks,
        text,
        boundary: "ORCL is a deterministic Qorx layer over local quarks: compact contracts, ranked symbols, bounded links, and exact evidence only. It has no external parser, Python runtime, UI export, or model call dependency.".to_string(),
    }
}

fn symbol_contracts(index: &RepoIndex) -> Vec<OrclSymbol> {
    let mut symbols = Vec::new();
    let mut seen = BTreeSet::new();
    for atom in &index.atoms {
        if !is_code_path(&atom.path) {
            continue;
        }
        for symbol in &atom.symbols {
            if !is_orcl_symbol(symbol) {
                continue;
            }
            let key = (symbol.clone(), atom.path.clone());
            if !seen.insert(key) {
                continue;
            }
            symbols.push(OrclSymbol {
                name: symbol.clone(),
                path: atom.path.clone(),
                quark_id: atom.id.clone(),
                start_line: atom.start_line,
                end_line: atom.end_line,
                signature: signature_for(atom, symbol),
            });
        }
    }
    symbols.sort_by(|a, b| a.name.cmp(&b.name).then(a.path.cmp(&b.path)));
    symbols
}

fn structural_links(index: &RepoIndex, symbols: &[OrclSymbol]) -> Vec<OrclLink> {
    let mut owners = BTreeMap::<String, Vec<&OrclSymbol>>::new();
    for symbol in symbols {
        owners.entry(symbol.name.clone()).or_default().push(symbol);
    }

    let mut links = BTreeMap::<(String, String, String), OrclLink>::new();
    for atom in &index.atoms {
        if !is_code_path(&atom.path) {
            continue;
        }
        for symbol in referenced_symbols(&atom.text) {
            let Some(targets) = owners.get(&symbol) else {
                continue;
            };
            for target in targets {
                if target.path == atom.path {
                    continue;
                }
                let key = (atom.path.clone(), target.path.clone(), symbol.clone());
                links.entry(key).or_insert_with(|| OrclLink {
                    from_path: atom.path.clone(),
                    to_path: target.path.clone(),
                    from_quark_id: atom.id.clone(),
                    to_quark_id: target.quark_id.clone(),
                    symbol: symbol.clone(),
                    kind: if looks_like_symbol_call(&atom.text, &symbol) {
                        "call".to_string()
                    } else {
                        "use".to_string()
                    },
                    proof: "local_ref".to_string(),
                });
            }
        }
    }
    links.into_values().collect()
}

fn rank_symbols(
    symbols: &[OrclSymbol],
    links: &[OrclLink],
    query: &str,
    limit: usize,
) -> Vec<OrclRank> {
    let query_lower = query.to_ascii_lowercase();
    let mut by_symbol_path = BTreeMap::<(String, String), OrclRank>::new();
    for symbol in symbols {
        by_symbol_path.insert(
            (symbol.name.clone(), symbol.path.clone()),
            OrclRank {
                symbol: symbol.name.clone(),
                path: symbol.path.clone(),
                quark_id: symbol.quark_id.clone(),
                fan_in: 0,
                fan_out: 0,
                score: 0,
            },
        );
    }

    for link in links {
        if let Some(rank) = by_symbol_path.get_mut(&(link.symbol.clone(), link.to_path.clone())) {
            rank.fan_in += 1;
        }
        for rank in by_symbol_path
            .values_mut()
            .filter(|rank| rank.path == link.from_path)
        {
            rank.fan_out += 1;
        }
    }

    let mut ranks = by_symbol_path
        .into_values()
        .map(|mut rank| {
            rank.score = (rank.fan_in as u64 * 100)
                + (rank.fan_out as u64 * 20)
                + if query_lower.contains(&rank.symbol.to_ascii_lowercase()) {
                    50
                } else {
                    0
                };
            rank
        })
        .filter(|rank| rank.score > 0)
        .collect::<Vec<_>>();
    ranks.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then(a.symbol.cmp(&b.symbol))
            .then(a.path.cmp(&b.path))
    });
    ranks.truncate(limit);
    ranks
}

fn walk_paths(seed_paths: &BTreeSet<String>, links: &[OrclLink], depth: usize) -> BTreeSet<String> {
    let mut seen = seed_paths.clone();
    let mut queue = seed_paths
        .iter()
        .cloned()
        .map(|path| (path, 0usize))
        .collect::<VecDeque<_>>();
    while let Some((path, distance)) = queue.pop_front() {
        if distance >= depth {
            continue;
        }
        for link in links {
            let next = if link.from_path == path {
                Some(link.to_path.clone())
            } else if link.to_path == path {
                Some(link.from_path.clone())
            } else {
                None
            };
            if let Some(next) = next {
                if seen.insert(next.clone()) {
                    queue.push_back((next, distance + 1));
                }
            }
        }
    }
    seen
}

fn select_links(links: &[OrclLink], path_scope: &BTreeSet<String>, limit: usize) -> Vec<OrclLink> {
    let mut selected = links
        .iter()
        .filter(|link| path_scope.contains(&link.from_path) || path_scope.contains(&link.to_path))
        .cloned()
        .collect::<Vec<_>>();
    selected.sort_by(|a, b| {
        a.from_path
            .cmp(&b.from_path)
            .then(a.to_path.cmp(&b.to_path))
            .then(a.symbol.cmp(&b.symbol))
    });
    selected.truncate(limit.saturating_mul(4).max(1));
    selected
}

fn select_symbols(
    symbols: &[OrclSymbol],
    path_scope: &BTreeSet<String>,
    query: &str,
    limit: usize,
) -> Vec<OrclSymbol> {
    let query_lower = query.to_ascii_lowercase();
    let mut selected = symbols
        .iter()
        .filter(|symbol| {
            path_scope.contains(&symbol.path)
                || query_lower.contains(&symbol.name.to_ascii_lowercase())
        })
        .cloned()
        .collect::<Vec<_>>();
    selected.sort_by(|a, b| a.name.cmp(&b.name).then(a.path.cmp(&b.path)));
    selected.truncate(limit.saturating_mul(3).max(1));
    selected
}

fn candidate_quarks(
    index: &RepoIndex,
    path_scope: &BTreeSet<String>,
    query: &str,
    changed_set: &BTreeSet<String>,
) -> Vec<(String, String)> {
    let mut reasons = BTreeMap::<String, BTreeSet<String>>::new();
    for atom in &index.atoms {
        if changed_set.contains(&atom.path) {
            reasons
                .entry(atom.id.clone())
                .or_default()
                .insert("changed".to_string());
        }
        if path_scope.contains(&atom.path) {
            reasons
                .entry(atom.id.clone())
                .or_default()
                .insert("orcl_scope".to_string());
        }
    }
    for hit in search_index(index, query, 32) {
        reasons
            .entry(hit.id)
            .or_default()
            .insert(format!("search:{}", hit.score));
    }

    let atoms = index.atom_lookup();
    let mut rows = reasons.into_iter().collect::<Vec<_>>();
    rows.sort_by(|(a_id, a_reason), (b_id, b_reason)| {
        let a_changed = a_reason.contains("changed");
        let b_changed = b_reason.contains("changed");
        let a_tokens = atoms
            .get(a_id.as_str())
            .map_or(u64::MAX, |atom| atom.token_estimate);
        let b_tokens = atoms
            .get(b_id.as_str())
            .map_or(u64::MAX, |atom| atom.token_estimate);
        b_changed
            .cmp(&a_changed)
            .then(a_tokens.cmp(&b_tokens))
            .then(a_id.cmp(b_id))
    });
    rows.into_iter()
        .map(|(id, reason)| (id, reason.into_iter().collect::<Vec<_>>().join(",")))
        .collect()
}

struct RenderHeader<'a> {
    query: &'a str,
    budget_tokens: u64,
    indexed_tokens: u64,
    depth: usize,
    changed_paths: &'a [String],
    related_paths: &'a [String],
    rank: &'a [OrclRank],
    symbols: &'a [OrclSymbol],
    links: &'a [OrclLink],
}

fn render_header(input: RenderHeader<'_>) -> String {
    let mut text = format!(
        "QORX_ORCL\nquery: {}\nbudget_tokens: {}\nindexed_tokens: {}\ndepth: {}\n",
        input.query, input.budget_tokens, input.indexed_tokens, input.depth
    );
    append_list(
        &mut text,
        "changed_paths",
        input.changed_paths,
        input.budget_tokens,
    );
    append_list(
        &mut text,
        "related_paths",
        input.related_paths,
        input.budget_tokens,
    );
    push_budgeted(&mut text, "rank:\n", input.budget_tokens);
    for item in input.rank {
        if !push_budgeted(
            &mut text,
            &format!(
                "- {} {} fan_in={} fan_out={} score={}\n",
                item.symbol, item.path, item.fan_in, item.fan_out, item.score
            ),
            input.budget_tokens,
        ) {
            break;
        }
    }
    push_budgeted(&mut text, "contracts:\n", input.budget_tokens);
    for symbol in input.symbols {
        if !push_budgeted(
            &mut text,
            &format!(
                "- {} {}:{}-{} id={} sig={}\n",
                symbol.name,
                symbol.path,
                symbol.start_line,
                symbol.end_line,
                symbol.quark_id,
                symbol.signature
            ),
            input.budget_tokens,
        ) {
            break;
        }
    }
    push_budgeted(&mut text, "links:\n", input.budget_tokens);
    for link in input.links {
        if !push_budgeted(
            &mut text,
            &format!(
                "- {} -> {} via {} kind={} proof={}\n",
                link.from_path, link.to_path, link.symbol, link.kind, link.proof
            ),
            input.budget_tokens,
        ) {
            break;
        }
    }
    text
}

fn append_list(text: &mut String, label: &str, values: &[String], budget_tokens: u64) {
    if values.is_empty() {
        let line = format!("{label}: []\n");
        push_budgeted(text, &line, budget_tokens);
        return;
    }
    let header = format!("{label}:\n");
    if !push_budgeted(text, &header, budget_tokens) {
        return;
    }
    for value in values {
        if !push_budgeted(text, &format!("- {value}\n"), budget_tokens) {
            break;
        }
    }
}

fn push_budgeted(text: &mut String, line: &str, budget_tokens: u64) -> bool {
    if estimate_tokens(text) + estimate_tokens(line) > budget_tokens {
        return false;
    }
    text.push_str(line);
    true
}

fn referenced_symbols(text: &str) -> BTreeSet<String> {
    let mut symbols = BTreeSet::new();
    let code_text = without_string_literals(text);
    for token in identifier_tokens(&code_text) {
        if is_orcl_symbol(&token) {
            symbols.insert(token);
        }
    }
    symbols
}

fn identifier_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() || ch == '_' || ch == '$' {
            current.push(ch);
        } else {
            push_identifier(&mut tokens, &mut current);
        }
    }
    push_identifier(&mut tokens, &mut current);
    tokens
}

fn push_identifier(tokens: &mut Vec<String>, current: &mut String) {
    if current.is_empty() {
        return;
    }
    if current.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        current.clear();
        return;
    }
    tokens.push(std::mem::take(current));
}

fn signature_for(atom: &RepoAtom, symbol: &str) -> String {
    atom.text
        .lines()
        .map(str::trim)
        .find(|line| line.contains(symbol) && looks_like_contract_line(line))
        .or_else(|| {
            atom.text
                .lines()
                .map(str::trim)
                .find(|line| line.contains(symbol))
        })
        .unwrap_or(symbol)
        .chars()
        .take(180)
        .collect()
}

fn looks_like_contract_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.starts_with("pub fn ")
        || lower.starts_with("fn ")
        || lower.starts_with("async fn ")
        || lower.starts_with("pub struct ")
        || lower.starts_with("struct ")
        || lower.starts_with("pub enum ")
        || lower.starts_with("enum ")
        || lower.starts_with("pub trait ")
        || lower.starts_with("trait ")
        || lower.starts_with("def ")
        || lower.starts_with("class ")
        || lower.starts_with("function ")
        || lower.starts_with("export function ")
        || lower.starts_with("export class ")
        || lower.starts_with("const ")
        || lower.starts_with("interface ")
        || lower.starts_with("type ")
        || lower.starts_with("func ")
}

fn looks_like_symbol_call(text: &str, symbol: &str) -> bool {
    let code_text = without_string_literals(text);
    let needle = format!("{symbol}(");
    code_text.contains(&needle)
}

fn is_code_path(path: &str) -> bool {
    let extension = path.rsplit('.').next().unwrap_or_default();
    matches!(
        extension,
        "rs" | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "mjs"
            | "cjs"
            | "py"
            | "go"
            | "java"
            | "kt"
            | "swift"
            | "c"
            | "cc"
            | "cpp"
            | "h"
            | "hpp"
            | "cs"
            | "php"
            | "rb"
    )
}

fn is_orcl_symbol(symbol: &str) -> bool {
    if symbol.len() <= 2 || is_reference_noise(symbol) {
        return false;
    }
    symbol.contains('_') || symbol.chars().any(|ch| ch.is_uppercase())
}

fn is_reference_noise(token: &str) -> bool {
    matches!(
        token,
        "Err"
            | "Json"
            | "None"
            | "Ok"
            | "Option"
            | "Result"
            | "Some"
            | "String"
            | "Value"
            | "Vec"
            | "async"
            | "await"
            | "class"
            | "const"
            | "def"
            | "enum"
            | "export"
            | "false"
            | "fn"
            | "function"
            | "impl"
            | "import"
            | "interface"
            | "let"
            | "mod"
            | "mut"
            | "new"
            | "null"
            | "pub"
            | "return"
            | "self"
            | "struct"
            | "trait"
            | "true"
            | "type"
    )
}
