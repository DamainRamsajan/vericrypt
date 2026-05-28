use crate::types::TeeStatus;

/// Collect TEE attestation evidence.
///
/// Attempts to collect hardware-signed attestation from Intel TDX or AMD SEV-SNP.
/// Gracefully degrades to Unavailable if no TEE is present.
pub fn collect_attestation() -> TeeStatus {
    // Check for Intel TDX
    if std::path::Path::new("/dev/tdx_guest").exists() {
        return collect_tdx_attestation();
    }

    // Check for AMD SEV-SNP
    if std::path::Path::new("/dev/sev-guest").exists() {
        return collect_sev_attestation();
    }

    TeeStatus::Unavailable {
        reason: "No TEE device files detected (/dev/tdx_guest or /dev/sev-guest)".into(),
    }
}

fn collect_tdx_attestation() -> TeeStatus {
    // Full TDX attestation via ioctl to /dev/tdx_guest
    // Returns a TDX quote with MRTD and RTMRs
    TeeStatus::Unavailable {
        reason: "TDX attestation collection — full implementation in Batch 3".into(),
    }
}

fn collect_sev_attestation() -> TeeStatus {
    // Full SEV-SNP attestation via /dev/sev-guest
    // Returns an SNP quote with launch measurement
    TeeStatus::Unavailable {
        reason: "SEV-SNP attestation collection — full implementation in Batch 3".into(),
    }
}
