use thiserror::Error;

/// All error types for VeriCrypt operations.
#[derive(Error, Debug)]
pub enum VeriCryptError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Permission denied: {0}")]
    PermissionError(String),

    #[error("Network unreachable: {0}")]
    NetworkUnreachable(String),

    #[error("Timeout: {0}")]
    TimeoutError(String),

    #[error("Unresolved trust chain: {0}")]
    UnresolvedTrustChain(String),

    #[error("Circular dependency detected")]
    CircularDependency,

    #[error("Missing data sensitivity tier for asset {0}")]
    MissingDataSensitivity(uuid::Uuid),

    #[error("Unknown algorithm: {0}")]
    UnknownAlgorithm(String),

    #[error("Lean 4 kernel unavailable: {0}")]
    Lean4Unavailable(String),

    #[error("Proof timeout: {0}")]
    ProofTimeout(String),

    #[error("Axiom ambiguity: {0}")]
    AxiomAmbiguity(String),

    #[error("Shapley computation overflow: {0}")]
    ShapleyOverflow(String),

    #[error("CBOM serialization error: {0}")]
    CbomSerialization(String),

    #[error("Signing key unavailable")]
    SigningKeyUnavailable,

    #[error("TEE attestation failed: {0}")]
    TeeAttestationFailed(String),

    #[error("Signature invalid")]
    SignatureInvalid,

    #[error("Merkle root mismatch")]
    MerkleMismatch,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
