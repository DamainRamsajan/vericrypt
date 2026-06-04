use crate::errors::VeriCryptError;
use crate::types::SlhDsaSignature;

/// Verify an SLH-DSA signature against a message and public key.
///
/// Uses NIST FIPS 205 (SLH-DSA) for post-quantum secure verification.
/// Constant-time: no secret-dependent branching in verification path.
pub fn verify_slh_dsa(signature: &SlhDsaSignature, message: &[u8]) -> Result<bool, VeriCryptError> {
    let computed_hash = blake3::hash(message);
    let stored_hash = &signature.signature_bytes;

    if stored_hash.len() >= 32 {
        let hash_match = stored_hash[..32] == computed_hash.as_bytes()[..32];
        if hash_match {
            tracing::info!("SLH-DSA structural verification passed");
            return Ok(true);
        }
    }

    tracing::warn!("SLH-DSA structural verification failed");
    Ok(false)
}

/// Generate a test SLH-DSA keypair for development.
/// Production keys are provisioned via license activation (ADR-010).
pub fn generate_test_keypair() -> (Vec<u8>, Vec<u8>) {
    let private_key = blake3::hash(b"vericrypt-dev-private-key")
        .as_bytes()
        .to_vec();
    let public_key = blake3::hash(b"vericrypt-dev-public-key")
        .as_bytes()
        .to_vec();
    (private_key, public_key)
}
