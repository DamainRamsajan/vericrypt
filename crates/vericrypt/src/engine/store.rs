//! The observation store (v4 architecture, section 5.2).
//!
//!   <store>/obs/YYYY/MM/<batch-hash>.jsonl   immutable batch of observations, one JSON object per line
//!   <store>/scans/<scan-id>.json             scan record: targets, timing, coverage, batch hashes
//!
//! A batch is named by the SHA-256 of its bytes. Reading a scan re-hashes every batch and refuses
//! any that changed.
use super::observe::{Observation, ObservationSet, OBSERVATIONS_FORMAT};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Component, Path, PathBuf};

const SCAN_FORMAT: &str = "vericrypt-scan/1";

fn safe_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

fn safe_relative(p: &str) -> bool {
    Path::new(p).components().all(|c| matches!(c, Component::Normal(_))) && p.starts_with("obs/")
}

/// Save a scan. Returns the scan id.
pub fn write_scan(store: &str, set: &ObservationSet) -> Result<String, String> {
    if !safe_id(&set.scan_id) {
        return Err(format!("unsafe scan id '{}'", set.scan_id));
    }
    let mut batches: Vec<Value> = Vec::new();
    if !set.observations.is_empty() {
        let mut bytes: Vec<u8> = Vec::new();
        for o in &set.observations {
            let v = serde_json::to_value(o).map_err(|e| e.to_string())?;
            bytes.extend(super::canonical(&v));
            bytes.push(b'\n');
        }
        let hash = hex::encode(Sha256::digest(&bytes));
        let (year, month) = (&set.started_at[0..4], &set.started_at[5..7]);
        let rel = format!("obs/{}/{}/{}.jsonl", year, month, hash);
        let path = PathBuf::from(store).join(&rel);
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::write(&path, &bytes).map_err(|e| format!("{}: {}", path.display(), e))?;
        batches.push(json!({"path": rel, "sha256": hash, "observations": set.observations.len()}));
    }
    let record = json!({
        "format": SCAN_FORMAT,
        "scan_id": set.scan_id,
        "tool": {"name": "vericrypt", "version": env!("CARGO_PKG_VERSION")},
        "started_at": set.started_at,
        "targets": set.targets,
        "batches": batches,
        "coverage": set.coverage,
    });
    let dir = PathBuf::from(store).join("scans");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{}.json", set.scan_id));
    std::fs::write(&path, serde_json::to_string_pretty(&record).unwrap()).map_err(|e| format!("{}: {}", path.display(), e))?;
    Ok(set.scan_id.clone())
}

/// Load a saved scan, checking every batch against its recorded hash.
pub fn read_scan(store: &str, scan_id: &str) -> Result<ObservationSet, String> {
    if !safe_id(scan_id) {
        return Err(format!("'{}' is not a valid scan id", scan_id));
    }
    let rec_path = PathBuf::from(store).join("scans").join(format!("{}.json", scan_id));
    let text = std::fs::read_to_string(&rec_path).map_err(|e| format!("cannot read scan record {}: {}", rec_path.display(), e))?;
    let rec: Value = serde_json::from_str(&text).map_err(|e| format!("scan record is not valid JSON: {}", e))?;
    if rec["format"] != SCAN_FORMAT {
        return Err(format!("unsupported scan record format {}", rec["format"]));
    }
    if rec["scan_id"] != scan_id {
        return Err("scan record does not match the requested scan id".into());
    }
    let mut observations: Vec<Observation> = Vec::new();
    for b in rec["batches"].as_array().ok_or("scan record has no batch list")? {
        let rel = b["path"].as_str().ok_or("batch entry has no path")?;
        let want = b["sha256"].as_str().ok_or("batch entry has no hash")?;
        if !safe_relative(rel) {
            return Err(format!("batch path '{}' is not allowed", rel));
        }
        let bytes = std::fs::read(PathBuf::from(store).join(rel)).map_err(|e| format!("cannot read batch {}: {}", rel, e))?;
        if hex::encode(Sha256::digest(&bytes)) != want {
            return Err(format!("batch {} has been changed: its hash no longer matches the scan record", rel));
        }
        let before = observations.len();
        for line in String::from_utf8(bytes).map_err(|_| "batch is not UTF-8".to_string())?.lines() {
            let o: Observation = serde_json::from_str(line).map_err(|e| format!("bad observation line: {}", e))?;
            if o.scan_id != scan_id {
                return Err("an observation belongs to a different scan".into());
            }
            observations.push(o);
        }
        if let Some(n) = b["observations"].as_u64() {
            if n as usize != observations.len() - before {
                return Err(format!("batch {} holds a different number of observations than the record says", rel));
            }
        }
    }
    Ok(ObservationSet {
        format: OBSERVATIONS_FORMAT.to_string(),
        scan_id: scan_id.to_string(),
        started_at: rec["started_at"].as_str().unwrap_or_default().to_string(),
        targets: serde_json::from_value(rec["targets"].clone()).map_err(|e| format!("bad targets: {}", e))?,
        observations,
        coverage: serde_json::from_value(rec["coverage"].clone()).map_err(|e| format!("bad coverage: {}", e))?,
    })
}
