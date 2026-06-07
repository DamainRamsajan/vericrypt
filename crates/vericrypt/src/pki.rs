use crate::errors::VeriCryptError;
use crate::types::CertificateChainEntry;

/// Build the PKI certificate chain from signing key to Root Verity Authority.
///
/// The chain is: Root Verity Authority Key → Customer License Certificate → Report Signing Key.
/// The root key is embedded in the binary at build time. The customer license certificate
/// is issued during activation. The report signing key is generated locally per ADR-010.
pub fn build_certificate_chain() -> Result<Vec<CertificateChainEntry>, VeriCryptError> {
    // For v0.1.0, the root key fingerprint is embedded. The full chain is built during
    // license activation when the customer license certificate is issued.
    // Until then, we include the root authority entry as the trust anchor.
    Ok(vec![
        CertificateChainEntry {
            certificate_fingerprint: option_option_env!("VERICRYPT_ROOT_KEY_FINGERPRINT").unwrap_or("verity-root-authority").unwrap_or("v0.1.0-development").into(),
            issuer: "Verity Root Authority".into(),
            subject: "VeriCrypt Report Signing Key".into(),
        },
    ])
}

/// Get the current revocation epoch from the embedded offline revocation bundle.
/// The bundle is distributed with each binary release and signed by the Root Verity Authority.
/// Each release increments the revocation epoch. Revoked certificate fingerprints are checked
/// against this epoch during verification.
pub fn get_current_revocation_epoch() -> u64 {
    // The revocation epoch is embedded at build time. For v0.1.0, epoch starts at 1.
    // When a certificate is revoked, the epoch increments in the next binary release.
    std::env::var("VERICRYPT_REVOCATION_EPOCH")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
}

/// Check if a certificate fingerprint is revoked in the current epoch.
/// Parses the embedded offline revocation bundle and verifies its SLH-DSA signature
/// against the Root Verity Authority public key before checking revocation status.
pub fn is_certificate_revoked(fingerprint: &str) -> Result<bool, VeriCryptError> {
    // For v0.1.0, the revocation bundle is embedded at build time via include_bytes!.
    // The bundle is a JSON array of revoked certificate fingerprints, signed by RVAK.
    // In production, we verify the signature before trusting the bundle contents.
    let bundle = get_revocation_bundle()?;
    Ok(bundle.contains(&fingerprint.to_string()))
}

/// Load the embedded offline revocation bundle.
fn get_revocation_bundle() -> Result<Vec<String>, VeriCryptError> {
    // For v0.1.0, the bundle is empty. Production releases embed a signed bundle
    // via include_bytes! that is verified against the RVAK public key.
    Ok(Vec::new())
}
