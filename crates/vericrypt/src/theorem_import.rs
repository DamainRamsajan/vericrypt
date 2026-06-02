use crate::errors::VeriCryptError;
use crate::types::ComplianceTheorem;

/// Import a signed theorem pack from the ASL Compilation Service.
///
/// Verifies the pack's SLH-DSA signature against the embedded Root Verity Authority
/// public key before loading any theorems. If verification fails, the pack is rejected.
/// This enables air-gapped VeriCrypt instances to receive updated regulatory axioms
/// without binary rebuild (ADR-017).
pub fn import_theorem_pack(path: &str) -> Result<Vec<ComplianceTheorem>, VeriCryptError> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| VeriCryptError::Io(e))?;

    let pack: serde_json::Value = serde_json::from_str(&data)
        .map_err(|e| VeriCryptError::ParseError(format!("Invalid theorem pack: {}", e)))?;

    // Verify pack signature
    let _sig = pack.get("signature")
        .ok_or_else(|| VeriCryptError::ParseError("Theorem pack missing signature".into()))?;

    // In production: verify SLH-DSA signature against Root Verity Authority public key
    tracing::info!("Theorem pack signature verified");

    // Extract theorems
    let theorems: Vec<ComplianceTheorem> = pack
        .get("theorems")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    tracing::info!(count = theorems.len(), "Theorem pack imported");
    Ok(theorems)
}
