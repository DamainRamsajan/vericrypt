//! Observations: the raw facts a scan saw, kept apart from any judgement about them.
//!
//! The record follows the v4 architecture (section 8.1). A scan produces an `ObservationSet`;
//! `engine::store` saves it; `engine::evaluate` judges it later. `evaluate` reads only an
//! `ObservationSet`, the parameters and an as-of time. It never touches files or the clock, so a
//! saved scan can be re-evaluated at another date or under other parameters.
use super::model::{Asset, Coverage};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const OBSERVATIONS_FORMAT: &str = "vericrypt-observations/1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorInfo {
    /// "files" or "tls-leaf"
    pub id: String,
    pub version: String,
    /// "file" or "network"
    pub kind: String,
}

/// One thing a sensor saw at one place, at one time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// "obs-" plus the first 16 hex characters of the SHA-256 of this record (with this field empty).
    pub obs_id: String,
    pub sensor: SensorInfo,
    /// Where the sensor looked from: "host" (files) or "network" (TLS). More vantages are planned.
    pub vantage: String,
    /// "file-parse" or "tls-leaf-probe"
    pub method: String,
    /// The path, or `tls://host:port`.
    pub target: String,
    /// RFC 3339 time at which the scan ran. Not used by evaluation.
    pub observed_at: String,
    /// The id of the asset that was seen (a hash of its content).
    pub subject: String,
    /// What was parsed, before any scope (public/private issuer) has been applied.
    pub attributes: Asset,
    /// 0 to 100. Always 100 today: current sensors either parse exactly or record a skip.
    /// Corroboration scoring across vantages is designed (v4 section 8.4) but not built.
    pub confidence: u32,
    /// A reference to the raw item by hash. The raw bytes are not stored yet.
    pub raw_ref: Option<String>,
    pub scan_id: String,
}

impl Observation {
    pub fn compute_id(&self) -> String {
        let mut c = self.clone();
        c.obs_id = String::new();
        let v = serde_json::to_value(&c).expect("observation serialises");
        format!("obs-{}", &hex::encode(Sha256::digest(super::canonical(&v)))[..16])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanTargets {
    pub dirs: Vec<String>,
    pub tls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationSet {
    pub format: String,
    pub scan_id: String,
    pub started_at: String,
    pub targets: ScanTargets,
    pub observations: Vec<Observation>,
    pub coverage: Coverage,
}

pub fn scan_id_for(started_at: &str, dirs: &[String], tls: &[String], tool_version: &str) -> String {
    let v = serde_json::json!({"started_at": started_at, "dirs": dirs, "tls": tls, "tool_version": tool_version});
    format!("scan-{}", &hex::encode(Sha256::digest(super::canonical(&v)))[..16])
}
