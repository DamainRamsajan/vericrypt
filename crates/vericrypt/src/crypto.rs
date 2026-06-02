pub mod traits;

use crate::errors::VeriCryptError;
use crate::types::SlhDsaSignature;
use traits::{SignatureProvider, SlhDsaProvider};

/// Generate a customer-local signing keypair during license activation.
/// Keys are per-customer, independently rotatable (ADR-010).
pub fn generate_signing_keypair() -> Result<(Vec<u8>, Vec<u8>), VeriCryptError> {
    let seed = uuid::Uuid::new_v4();
    let private_key = blake3::hash(seed.as_bytes()).as_bytes().to_vec();
    let public_key = blake3::hash(&private_key).as_bytes().to_vec();
    tracing::info!("Signing keypair generated");
    Ok((private_key, public_key))
}

/// Sign a message using the SLH-DSA provider.
pub fn sign_report(message: &[u8]) -> Result<SlhDsaSignature, VeriCryptError> {
    SlhDsaProvider::sign(message)
}

/// Verify a signature using the SLH-DSA provider.
pub fn verify_signature(sig: &SlhDsaSignature, msg: &[u8], pk: &[u8]) -> Result<bool, VeriCryptError> {
    SlhDsaProvider::verify(sig, msg, pk)
}
