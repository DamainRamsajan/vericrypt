use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Cryptographic asset type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    Certificate,
    Key,
    AlgorithmInstance,
    ProtocolConfiguration,
    HsmConfiguration,
}

/// Cryptographic algorithm descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Algorithm {
    pub name: String,
    pub family: String,
    pub quantum_vulnerable: bool,
    pub vulnerability_type: Option<String>,
    pub nist_pqc_replacement: Option<String>,
    pub shelf_life_years: Option<u32>,
}

/// A single cryptographic asset discovered during scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoAsset {
    pub asset_id: Uuid,
    pub asset_type: AssetType,
    pub algorithm: Algorithm,
    pub key_size: Option<u32>,
    pub expiry_date: Option<chrono::DateTime<chrono::Utc>>,
    pub fingerprint: String,
    pub source_location: String,
    pub nist_quantum_security_level: Option<u32>,
}

/// Dependency relationship between two cryptographic assets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Signs,
    Encrypts,
    Trusts,
    Uses,
    Configures,
    Contains,
}

/// Typed edge in the cryptographic dependency graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoDependency {
    pub dependency_id: Uuid,
    pub dependency_type: DependencyType,
    pub source_asset_id: Uuid,
    pub target_asset_id: Uuid,
}

/// Post-quantum signature container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PqcSignature {
    pub classical: Vec<u8>,
    pub pqc: Vec<u8>,
}

/// SLH-DSA signature specific to NIST FIPS 204.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlhDsaSignature {
    pub signature_bytes: Vec<u8>,
    pub public_key_bytes: Vec<u8>,
}

/// Compliance theorem status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProofStatus {
    Proved,
    Counterexample,
    Unverified,
    Timeout,
}

/// A single compliance theorem with its Lean 4 kernel verdict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceTheorem {
    pub theorem_id: Uuid,
    pub regulation_reference: String,
    pub lean4_statement: String,
    pub status: ProofStatus,
    pub counterexample_asset_id: Option<Uuid>,
    pub remediation_recommendation: Option<String>,
}

/// TEE attestation status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TeeStatus {
    Attested {
        quote_bytes: Vec<u8>,
        measurement: String,
        tee_type: String,
    },
    Unavailable {
        reason: String,
    },
}

/// The .pqc report — a constant-size evidence structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PqcReport {
    pub report_id: Uuid,
    pub scan_timestamp: chrono::DateTime<chrono::Utc>,
    pub binary_hash: String,
    pub input_hash: String,
    pub total_assets: u64,
    pub quantum_vulnerable_count: u64,
    pub violations_found: u64,
    pub cbom_merkle_root: String,
    pub compliance_theorems: Vec<ComplianceTheorem>,
    pub tee_attestation: TeeStatus,
    pub signature: Option<SlhDsaSignature>,
}
