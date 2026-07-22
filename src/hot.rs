use std::{fs, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    aim::AimReport,
    compression::AtomStore,
    config::AppPaths,
    index::{load_index, RepoIndex},
    response_cache::ExactResponseCache,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamHotReport {
    pub mode: String,
    pub active: bool,
    pub portable: bool,
    pub data_dir: String,
    pub total_hot_bytes: u64,
    pub index_hot: bool,
    pub index_quarks: usize,
    pub indexed_tokens: u64,
    #[serde(alias = "aim_hot")]
    pub sidecar_hot: bool,
    #[serde(alias = "aim_bytes")]
    pub sidecar_bytes: u64,
    pub quark_store_entries: usize,
    pub response_cache_entries: usize,
    pub boundary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotBytesReport {
    pub stats_bytes: u64,
    pub provenance_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct RamHotState {
    pub aim: AimReport,
    pub index: Option<RepoIndex>,
    pub stats_bytes: Option<Vec<u8>>,
    pub provenance_bytes: Option<Vec<u8>>,
    pub report: RamHotReport,
}

pub fn load(
    paths: &AppPaths,
    atoms: &AtomStore,
    cache: &ExactResponseCache,
) -> Result<RamHotState> {
    let index = load_index(&paths.index_file).ok();
    let aim = crate::aim::inspect_default()?;
    let stats_bytes = read_optional(&paths.stats_file)?;
    let provenance_bytes = read_optional(&paths.provenance_file)?;
    let index_bytes = file_len(&paths.index_file);
    let quark_bytes = file_len(&paths.atom_file).max(serde_json::to_vec(atoms)?.len() as u64);
    let cache_bytes =
        file_len(&paths.response_cache_file).max(serde_json::to_vec(cache)?.len() as u64);
    let stats_len = stats_bytes
        .as_ref()
        .map(|bytes| bytes.len() as u64)
        .unwrap_or(0);
    let provenance_len = provenance_bytes
        .as_ref()
        .map(|bytes| bytes.len() as u64)
        .unwrap_or(0);
    let indexed_tokens = index.as_ref().map(RepoIndex::total_tokens).unwrap_or(0);
    let index_quarks = index.as_ref().map(|index| index.atoms.len()).unwrap_or(0);
    let total_hot_bytes =
        index_bytes + quark_bytes + cache_bytes + stats_len + provenance_len + aim.bytes;
    let index_hot = index.is_some();

    Ok(RamHotState {
        aim: aim.clone(),
        index,
        stats_bytes,
        provenance_bytes,
        report: RamHotReport {
            mode: "resident-heap-cache".to_string(),
            active: index_hot,
            portable: paths.portable,
            data_dir: paths.data_dir.display().to_string(),
            total_hot_bytes,
            index_hot,
            index_quarks,
            indexed_tokens,
            sidecar_hot: aim.found,
            sidecar_bytes: aim.bytes,
            quark_store_entries: atoms.atoms.len(),
            response_cache_entries: cache.entries.len(),
            boundary: "This endpoint reports Qorx state already loaded in the process heap. It is not a RAM disk and does not provide block-cache semantics. RAM-backed Cosmos storage requires a native RAM-disk backend and is reported separately.".to_string(),
        },
    })
}

pub fn status_from_disk(paths: &AppPaths) -> Result<RamHotReport> {
    let atoms = AtomStore::load(&paths.atom_file)?;
    let cache = ExactResponseCache::load(&paths.response_cache_file)?;
    Ok(load(paths, &atoms, &cache)?.report)
}

pub fn state_json(state: &RamHotState) -> Value {
    json!({
        "report": state.report,
        "sidecar": state.aim,
        "resident_bytes": HotBytesReport {
            stats_bytes: state.stats_bytes.as_ref().map(|bytes| bytes.len() as u64).unwrap_or(0),
            provenance_bytes: state.provenance_bytes.as_ref().map(|bytes| bytes.len() as u64).unwrap_or(0),
        }
    })
}

fn read_optional(path: &Path) -> Result<Option<Vec<u8>>> {
    if path.exists() {
        Ok(Some(fs::read(path)?))
    } else {
        Ok(None)
    }
}

fn file_len(path: &Path) -> u64 {
    fs::metadata(path).map(|meta| meta.len()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use std::fs;

    use super::*;

    #[test]
    fn ram_hot_report_counts_loaded_files() {
        let tmp = std::env::temp_dir().join(format!(
            "qorx-hot-test-{}",
            Utc::now().timestamp_nanos_opt().unwrap()
        ));
        fs::create_dir_all(&tmp).unwrap();
        let paths = AppPaths {
            data_dir: tmp.clone(),
            portable: true,
            stats_file: tmp.join("stats.pb"),
            atom_file: tmp.join("quarks.pb"),
            index_file: tmp.join("repo_index.pb"),
            context_protobuf_file: tmp.join("qorx-context.pb"),
            response_cache_file: tmp.join("response_cache.pb"),
            integration_report_file: tmp.join("integrations.pb"),
            provenance_file: tmp.join("qorx-provenance.pb"),
            security_keys_file: tmp.join("qorx-security-keys.pb"),
            shim_dir: tmp.join("shims"),
        };
        crate::proto_store::save(
            &paths.index_file,
            &serde_json::json!({
                "root": "C:/repo",
                "updated_at": Utc::now(),
                "quarks": [{
                    "id": "qva_hot",
                    "path": "src/lib.rs",
                    "start_line": 1,
                    "end_line": 1,
                    "hash": "abc",
                    "token_estimate": 42,
                    "symbols": ["hot"],
                    "signal_mask": 0,
                    "vector": [1],
                    "text": "fn hot() {}"
                }]
            }),
        )
        .unwrap();
        crate::proto_store::save(&paths.stats_file, &serde_json::json!({})).unwrap();
        crate::proto_store::save(
            &paths.provenance_file,
            &serde_json::json!({"verified": true}),
        )
        .unwrap();
        let atoms = AtomStore {
            atoms: [("qvk_hot".to_string(), "payload".to_string())].into(),
        };
        let cache = ExactResponseCache::default();

        let state = load(&paths, &atoms, &cache).unwrap();

        assert!(state.report.active);
        assert!(state.report.index_hot);
        assert_eq!(state.report.index_quarks, 1);
        assert_eq!(state.report.indexed_tokens, 42);
        assert_eq!(state.report.quark_store_entries, 1);
        assert!(state.report.total_hot_bytes > 0);

        let _ = fs::remove_dir_all(tmp);
    }
}
