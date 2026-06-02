use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =============================================================================
// VeriCrypt Domain Model — Arc42 Section 2.2
// =============================================================================

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
    pub data_lifetime_years: Option<f64>,
    pub usage_context: Option<String>,
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

/// SLH-DSA signature (NIST FIPS 205).
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

/// A single compliance theorem with its ASL VM kernel verdict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceTheorem {
    pub theorem_id: Uuid,
    pub regulation_reference: String,
    pub ASL VM_statement: String,
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

// =============================================================================
// Batch 2+ types — Exposure, Prioritization, Evidence
// =============================================================================

/// Result from the Quantum Exposure Analyzer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureResult {
    pub total_hndl_exposure: f64,
    pub per_asset_exposure: std::collections::HashMap<Uuid, f64>,
    pub shapley_values: std::collections::HashMap<Uuid, f64>,
    pub breakdown: ExposureBreakdown,
    pub shapley_metadata: Option<ShapleyApproximationMetadata>,
}

/// Factorized HNDL exposure breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureBreakdown {
    pub temporal_hazard: f64,
    pub crypto_vulnerability: f64,
    pub operational_exposure: f64,
    pub defense_attack_ratio: f64,
}

/// Metadata for Monte Carlo Shapley value approximation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapleyApproximationMetadata {
    pub samples: u64,
    pub convergence_error: f64,
    pub confidence_interval: f64,
    pub converged: bool,
    pub convergence_threshold: f64,
}

/// A single entry in the prioritized migration roadmap.
#[derive(Debug, Clone, Serialize)]
pub struct MigrationPhase {
    pub phase: u32,
    pub asset_id: Uuid,
    pub current_algorithm: String,
    pub recommended_replacement: String,
    pub regulatory_reference: String,
    pub estimated_complexity: String,
}

/// Inventory confidence model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryConfidence {
    pub visibility_score: f64,
    pub unreachable_assets: u64,
    pub unsupported_formats: Vec<String>,
    pub encrypted_uninspectable: u64,
    pub inferred_dependencies: u64,
    pub confidence_level: ConfidenceLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    Complete,
    High,
    Partial,
    Low,
    Unknown,
}

/// Evidence chain of custody.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceCustody {
    pub scan_timestamp: chrono::DateTime<chrono::Utc>,
    pub binary_hash: String,
    pub operator_identity: Option<String>,
    pub environment_identity: Option<String>,
    pub custody_root: String,
}

/// Compliance confidence model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfidence {
    pub proof_confidence: f64,
    pub inventory_confidence: f64,
    pub regulatory_axiom_confidence: f64,
    pub composite_confidence: f64,
}

/// PKI certificate chain entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateChainEntry {
    pub certificate_fingerprint: String,
    pub issuer: String,
    pub subject: String,
}

/// Performance stage timing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageTiming {
    pub stage_name: String,
    pub elapsed_ms: u64,
    pub complexity: String,
    pub item_count: u64,
}
