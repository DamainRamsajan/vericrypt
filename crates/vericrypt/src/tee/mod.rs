use crate::types::TeeStatus;

/// TEE type detected at runtime.
#[derive(Debug, Clone, PartialEq)]
pub enum TeeType {
    IntelTdx,
    AmdSevSnp,
    None,
}

/// Detect available TEE hardware.
pub fn detect_tee() -> TeeType {
    if std::path::Path::new("/dev/tdx_guest").exists() {
        return TeeType::IntelTdx;
    }
    if std::path::Path::new("/dev/sev-guest").exists() {
        return TeeType::AmdSevSnp;
    }
    TeeType::None
}

/// Collect TEE attestation evidence.
///
/// Attempts hardware-signed attestation from Intel TDX or AMD SEV-SNP.
/// Gracefully degrades to Unavailable if no TEE is present (ADR-008).
pub fn collect_attestation() -> TeeStatus {
    match detect_tee() {
        TeeType::IntelTdx => collect_tdx_attestation(),
        TeeType::AmdSevSnp => collect_sev_attestation(),
        TeeType::None => TeeStatus::Unavailable {
            reason: "No TEE device files detected (/dev/tdx_guest or /dev/sev-guest)".into(),
        },
    }
}

fn collect_tdx_attestation() -> TeeStatus {
    match std::fs::read("/dev/tdx_guest") {
        Ok(quote_bytes) => {
            let measurement = hex::encode(&quote_bytes[..32.min(quote_bytes.len())]);
            TeeStatus::Attested {
                quote_bytes,
                measurement,
                tee_type: "Intel TDX".into(),
            }
        }
        Err(e) => TeeStatus::Unavailable {
            reason: format!("Cannot read /dev/tdx_guest: {}", e),
        },
    }
}

fn collect_sev_attestation() -> TeeStatus {
    match std::fs::read("/dev/sev-guest") {
        Ok(quote_bytes) => {
            let measurement = hex::encode(&quote_bytes[..32.min(quote_bytes.len())]);
            TeeStatus::Attested {
                quote_bytes,
                measurement,
                tee_type: "AMD SEV-SNP".into(),
            }
        }
        Err(e) => TeeStatus::Unavailable {
            reason: format!("Cannot read /dev/sev-guest: {}", e),
        },
    }
}

/// Check if TEE attestation is available.
pub fn is_tee_available() -> bool {
    matches!(collect_attestation(), TeeStatus::Attested { .. })
}
