use crate::errors::VeriCryptError;
use crate::types::{EvidenceCustody, TeeStatus};

/// Build a complete evidence chain of custody for a scan.
///
/// Computes custody_root = BLAKE3(operator || binary_hash || merkle_root ||
///                                 timestamp || attestation_hash || environment)
/// as specified in Addendum 3 §4.
pub fn build_custody_chain(merkle_root: &str, tee_attestation: &TeeStatus) -> EvidenceCustody {
    let now = chrono::Utc::now();
    let binary_hash = env!("CARGO_PKG_VERSION").to_string();
    let operator = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .ok();
    let hostname = hostname::get().ok().and_then(|h| h.into_string().ok());
    let attestation_hash = match tee_attestation {
        TeeStatus::Attested { measurement, .. } => measurement.clone(),
        TeeStatus::Unavailable { .. } => "none".to_string(),
    };

    // Compute custody root
    let mut hasher = blake3::Hasher::new();
    hasher.update(operator.as_deref().unwrap_or("unknown").as_bytes());
    hasher.update(binary_hash.as_bytes());
    hasher.update(merkle_root.as_bytes());
    hasher.update(now.to_rfc3339().as_bytes());
    hasher.update(attestation_hash.as_bytes());
    hasher.update(hostname.as_deref().unwrap_or("unknown").as_bytes());
    let custody_root = hex::encode(hasher.finalize().as_bytes());

    EvidenceCustody {
        scan_timestamp: now,
        binary_hash,
        operator_identity: operator,
        environment_identity: hostname,
        custody_root,
    }
}
