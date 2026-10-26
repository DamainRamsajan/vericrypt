//! Core data model for the VeriCrypt v2 evidence engine.
//! Everything here serialises to plain JSON with sorted keys, integers and strings only
//! (no floats), so reports can be canonicalised, hashed and signed deterministically.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AssetKind {
    Certificate,
    PrivateKey,
    PublicKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Family {
    Rsa,
    Ec,
    EdDsa,
    Dh,
    Dsa,
    MlKem,
    MlDsa,
    SlhDsa,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QuantumStatus {
    Vulnerable,
    Resistant,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn parse(s: &str) -> Option<Severity> {
        match s.to_ascii_lowercase().as_str() {
            "info" => Some(Severity::Info),
            "low" => Some(Severity::Low),
            "medium" => Some(Severity::Medium),
            "high" => Some(Severity::High),
            "critical" => Some(Severity::Critical),
            _ => None,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrustScope {
    /// Operator declared the issuer is publicly trusted (CA/B Forum rules apply).
    Public,
    /// Self-signed or operator-declared private CA (CA/B Forum lifetime caps do not apply).
    Private,
    /// Not declared. Reported as "applies only if publicly trusted".
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub kind: AssetKind,
    pub locations: Vec<String>,
    pub algorithm: String,
    pub family: Family,
    pub oid: String,
    pub key_bits: Option<u32>,
    pub curve: Option<String>,
    pub classical_bits: Option<u32>,
    pub quantum_status: QuantumStatus,
    pub nist_quantum_level: Option<u8>,
    pub signature_algorithm: Option<String>,
    pub signature_oid: Option<String>,
    pub subject: Option<String>,
    pub issuer: Option<String>,
    pub serial: Option<String>,
    pub not_before: Option<String>,
    pub not_after: Option<String>,
    pub validity_days: Option<i64>,
    pub is_ca: bool,
    pub self_signed: bool,
    pub tls_server: bool,
    pub sans: Vec<String>,
    pub sha256: String,
    pub trust_scope: TrustScope,
    pub encrypted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub rule_id: String,
    pub pack: String,
    pub severity: Severity,
    pub asset_id: String,
    pub title: String,
    pub detail: String,
    pub remediation: String,
    pub references: Vec<String>,
    pub evidence: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Params {
    pub as_of: String,
    pub shelf_life_years: u32,
    pub migration_years: u32,
    pub crqc_years: u32,
    pub public_issuers: Vec<String>,
    pub private_issuers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skipped {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsProbe {
    pub target: String,
    pub ok: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Coverage {
    pub files_examined: u64,
    pub items_parsed: u64,
    pub skipped: Vec<Skipped>,
    pub tls_probes: Vec<TlsProbe>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mosca {
    pub shelf_life_years: u32,
    pub migration_years: u32,
    pub crqc_years: u32,
    pub sum_years: u32,
    pub at_risk: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub total_assets: u64,
    pub certificates: u64,
    pub private_keys: u64,
    pub public_keys: u64,
    pub quantum_vulnerable: u64,
    pub quantum_resistant: u64,
    pub unknown_algorithm: u64,
    pub findings_by_severity: BTreeMap<String, u64>,
    pub mosca: Mosca,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase {
    pub effective_from: String,
    pub max_validity_days: u32,
    /// Renewals per certificate per year at the cap, as a string with 2 decimals.
    pub renewals_per_cert_per_year: String,
    /// Total renewals per year across all in-scope certificates, as a string with 1 decimal.
    pub total_renewals_per_year: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertForecast {
    pub asset_id: String,
    pub subject: String,
    pub not_after: String,
    pub days_to_expiry: i64,
    pub trust_scope: TrustScope,
    pub cap_days_at_next_renewal: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast {
    pub in_scope_certificates: u64,
    pub scope_note: String,
    pub schedule: Vec<Phase>,
    pub certificates: Vec<CertForecast>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    pub format: String,
    pub tool: ToolInfo,
    pub generated_at: String,
    pub parameters: Params,
    pub coverage: Coverage,
    pub summary: Summary,
    pub forecast: Forecast,
    pub assumptions: Vec<String>,
    pub limitations: Vec<String>,
    pub assets: Vec<Asset>,
    pub findings: Vec<Finding>,
    pub merkle_root: String,
}
