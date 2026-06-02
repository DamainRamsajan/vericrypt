//! Trusted Execution Environment (TEE) attestation layer.
//!
//! Supports hardware‑rooted trust via Intel TDX, AMD SEV‑SNP, and
//! ARM TrustZone. Binds capability token activation to TEE integrity.
//!
//! References:
//!   - CMC (Fraunhofer‑AISEC, Apr 2026) — unified remote attestation
//!   - TLS and TEEs (ultraviolet.rs, Feb 2026) — attested TLS channels
//!   - FOSDEM 2026 — cloud confidential computing attestation patterns

use std::collections::HashMap;

// ── TEE types ──

/// Supported TEE backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeeBackend {
    None,
    IntelTdx,
    AmdSevSnp,
    ArmTrustZone,
    AwsNitro,
    Software,
}

/// TEE measurement — a cryptographic hash of the trusted code image.
#[derive(Debug, Clone)]
pub struct TeeMeasurement {
    pub backend: TeeBackend,
    /// Hardware‑reported measurement hash (hex).
    pub measurement: String,
    /// Platform firmware version.
    pub firmware_version: String,
    /// Timestamp of attestation.
    pub timestamp: i64,
}

/// TEE attestation modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttestationMode {
    /// Verify once at boot.
    BootTime,
    /// Verify continuously during execution.
    Continuous,
    /// Verify before each sensitive operation.
    PerOperation,
}

// ── TEE clause (governance) ──

/// Governance clause binding agent declarations to TEE requirements.
#[derive(Debug, Clone)]
pub struct TeeClause {
    /// Whether TEE is required.
    pub required: bool,
    /// Accepted TEE backends.
    pub accepted_backends: Vec<TeeBackend>,
    /// Attestation mode.
    pub mode: AttestationMode,
    /// Enforcement: audit‑only, block, or safe‑park.
    pub enforcement: TeeEnforcement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeeEnforcement {
    AuditOnly,
    Block,
    SafePark,
}

// ── TEE verifier ──

/// Verifies TEE attestation evidence.
pub struct TeeVerifier {
    /// Known good measurements for each backend.
    pub known_measurements: HashMap<TeeBackend, String>,
    /// Current trust level in [0, 1].
    pub trust: f64,
    /// Last attestation result.
    pub last_attestation: Option<AttestationResult>,
}

#[derive(Debug, Clone)]
pub struct AttestationResult {
    pub success: bool,
    pub backend: TeeBackend,
    pub measurement: Option<String>,
    pub reason: String,
    pub timestamp: i64,
}

impl TeeVerifier {
    pub fn new() -> Self {
        Self {
            known_measurements: HashMap::new(),
            trust: 0.0,
            last_attestation: None,
        }
    }

    /// Register a known‑good measurement.
    pub fn register_measurement(&mut self, backend: TeeBackend, measurement: &str) {
        self.known_measurements.insert(backend, measurement.to_string());
    }

    /// Verify attestation evidence.
    pub fn attest(&mut self, measurement: &TeeMeasurement) -> AttestationResult {
        let expected = self.known_measurements.get(&measurement.backend);
        let success = expected.map_or(false, |exp| exp == &measurement.measurement);

        let result = AttestationResult {
            success,
            backend: measurement.backend,
            measurement: Some(measurement.measurement.clone()),
            reason: if success {
                "Measurement matches known good value".into()
            } else {
                "Measurement mismatch — possible compromise".into()
            },
            timestamp: measurement.timestamp,
        };

        self.trust = if success { 1.0 } else { 0.0 };
        self.last_attestation = Some(result.clone());
        result
    }

    /// Check whether the current trust level meets a threshold.
    pub fn trust_meets(&self, threshold: f64) -> bool {
        self.trust >= threshold
    }
}
