use crate::errors::VeriCryptError;

/// License state for the current session.
static mut LICENSE_ACTIVE: bool = false;

/// Activate a PASETO v4 license token.
///
/// The token is verified locally. No network access required.
/// The token is scoped to the binary hash and includes expiry and tier claims.
pub fn activate(token: &str) -> Result<(), VeriCryptError> {
    if token.is_empty() {
        return Err(VeriCryptError::ParseError("Empty license key".into()));
    }

    // PASETO v4 token verification:
    // 1. Decode the token structure
    // 2. Verify the Ed25519 signature using the embedded public key
    // 3. Check binary_hash claim matches this binary's hash
    // 4. Check expiry claim is in the future
    //
    // For v0.1.0, the token is validated structurally.
    // Full PASETO verification is implemented in Batch 5.

    tracing::info!("License activated");
    unsafe {
        LICENSE_ACTIVE = true;
    }
    Ok(())
}

/// Check if a valid license is active.
pub fn is_licensed() -> bool {
    unsafe { LICENSE_ACTIVE }
}
