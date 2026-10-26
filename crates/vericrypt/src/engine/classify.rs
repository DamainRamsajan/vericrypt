//! Algorithm registry: OID -> family, classical strength, quantum status, NIST PQC level,
//! and the NIST IR 8547 (initial public draft, Nov 2024) transition dates.
use super::model::{Family, QuantumStatus};

pub struct KeyInfo {
    pub family: Family,
    pub name: String,
    pub classical_bits: Option<u32>,
    pub quantum: QuantumStatus,
    pub nist_level: Option<u8>,
    pub curve: Option<String>,
    pub key_bits: Option<u32>,
}

/// NIST SP 800-57 style equivalence for integer-factorisation / finite-field keys.
pub fn ifc_security_bits(bits: u32) -> u32 {
    match bits {
        0..=1023 => 64,
        1024..=2047 => 80,
        2048..=3071 => 112,
        3072..=7679 => 128,
        7680..=15359 => 192,
        _ => 256,
    }
}

pub fn curve_name(oid: &str) -> Option<(&'static str, u32, u32)> {
    // (name, key bits, classical security bits)
    Some(match oid {
        "1.2.840.10045.3.1.1" => ("P-192", 192, 96),
        "1.3.132.0.33" => ("P-224", 224, 112),
        "1.2.840.10045.3.1.7" => ("P-256", 256, 128),
        "1.3.132.0.34" => ("P-384", 384, 192),
        "1.3.132.0.35" => ("P-521", 521, 256),
        "1.3.132.0.10" => ("secp256k1", 256, 128),
        "1.3.36.3.3.2.8.1.1.7" => ("brainpoolP256r1", 256, 128),
        "1.3.36.3.3.2.8.1.1.11" => ("brainpoolP384r1", 384, 192),
        "1.3.36.3.3.2.8.1.1.13" => ("brainpoolP512r1", 512, 256),
        _ => return None,
    })
}

/// Curve inferred from a SEC1 private scalar length (used only when parameters are absent).
pub fn curve_from_scalar_len(len: usize) -> Option<(&'static str, u32, u32)> {
    match len {
        24 => Some(("P-192", 192, 96)),
        28 => Some(("P-224", 224, 112)),
        32 => Some(("P-256", 256, 128)),
        48 => Some(("P-384", 384, 192)),
        66 => Some(("P-521", 521, 256)),
        _ => None,
    }
}

fn pqc(oid: &str) -> Option<(Family, &'static str, u8)> {
    Some(match oid {
        "2.16.840.1.101.3.4.3.17" => (Family::MlDsa, "ML-DSA-44", 2),
        "2.16.840.1.101.3.4.3.18" => (Family::MlDsa, "ML-DSA-65", 3),
        "2.16.840.1.101.3.4.3.19" => (Family::MlDsa, "ML-DSA-87", 5),
        "2.16.840.1.101.3.4.4.1" => (Family::MlKem, "ML-KEM-512", 1),
        "2.16.840.1.101.3.4.4.2" => (Family::MlKem, "ML-KEM-768", 3),
        "2.16.840.1.101.3.4.4.3" => (Family::MlKem, "ML-KEM-1024", 5),
        "2.16.840.1.101.3.4.3.20" => (Family::SlhDsa, "SLH-DSA-SHA2-128s", 1),
        "2.16.840.1.101.3.4.3.21" => (Family::SlhDsa, "SLH-DSA-SHA2-128f", 1),
        "2.16.840.1.101.3.4.3.22" => (Family::SlhDsa, "SLH-DSA-SHA2-192s", 3),
        "2.16.840.1.101.3.4.3.23" => (Family::SlhDsa, "SLH-DSA-SHA2-192f", 3),
        "2.16.840.1.101.3.4.3.24" => (Family::SlhDsa, "SLH-DSA-SHA2-256s", 5),
        "2.16.840.1.101.3.4.3.25" => (Family::SlhDsa, "SLH-DSA-SHA2-256f", 5),
        "2.16.840.1.101.3.4.3.26" => (Family::SlhDsa, "SLH-DSA-SHAKE-128s", 1),
        "2.16.840.1.101.3.4.3.27" => (Family::SlhDsa, "SLH-DSA-SHAKE-128f", 1),
        "2.16.840.1.101.3.4.3.28" => (Family::SlhDsa, "SLH-DSA-SHAKE-192s", 3),
        "2.16.840.1.101.3.4.3.29" => (Family::SlhDsa, "SLH-DSA-SHAKE-192f", 3),
        "2.16.840.1.101.3.4.3.30" => (Family::SlhDsa, "SLH-DSA-SHAKE-256s", 5),
        "2.16.840.1.101.3.4.3.31" => (Family::SlhDsa, "SLH-DSA-SHAKE-256f", 5),
        _ => return None,
    })
}

/// Classify a public/private key by its algorithm OID plus whatever size/curve we extracted.
pub fn classify_key(oid: &str, key_bits: Option<u32>, curve_oid: Option<&str>, curve_hint: Option<(&'static str, u32, u32)>) -> KeyInfo {
    if let Some((family, name, level)) = pqc(oid) {
        return KeyInfo {
            family,
            name: name.to_string(),
            classical_bits: None,
            quantum: QuantumStatus::Resistant,
            nist_level: Some(level),
            curve: None,
            key_bits: None,
        };
    }
    match oid {
        // rsaEncryption, RSASSA-PSS, RSAES-OAEP
        "1.2.840.113549.1.1.1" | "1.2.840.113549.1.1.10" | "1.2.840.113549.1.1.7" => {
            let sec = key_bits.map(ifc_security_bits);
            KeyInfo {
                family: Family::Rsa,
                name: match key_bits {
                    Some(b) => format!("RSA-{}", b),
                    None => "RSA".into(),
                },
                classical_bits: sec,
                quantum: QuantumStatus::Vulnerable,
                nist_level: Some(0),
                curve: None,
                key_bits,
            }
        }
        "1.2.840.10045.2.1" => {
            let c = curve_oid.and_then(curve_name).or(curve_hint);
            match c {
                Some((n, bits, sec)) => KeyInfo {
                    family: Family::Ec,
                    name: format!("EC-{}", n),
                    classical_bits: Some(sec),
                    quantum: QuantumStatus::Vulnerable,
                    nist_level: Some(0),
                    curve: Some(n.to_string()),
                    key_bits: Some(bits),
                },
                None => KeyInfo {
                    family: Family::Ec,
                    name: "EC".into(),
                    classical_bits: None,
                    quantum: QuantumStatus::Vulnerable,
                    nist_level: Some(0),
                    curve: curve_oid.map(|s| s.to_string()),
                    key_bits,
                },
            }
        }
        "1.3.101.112" => KeyInfo { family: Family::EdDsa, name: "Ed25519".into(), classical_bits: Some(128), quantum: QuantumStatus::Vulnerable, nist_level: Some(0), curve: Some("Ed25519".into()), key_bits: Some(256) },
        "1.3.101.113" => KeyInfo { family: Family::EdDsa, name: "Ed448".into(), classical_bits: Some(224), quantum: QuantumStatus::Vulnerable, nist_level: Some(0), curve: Some("Ed448".into()), key_bits: Some(448) },
        "1.3.101.110" => KeyInfo { family: Family::Dh, name: "X25519".into(), classical_bits: Some(128), quantum: QuantumStatus::Vulnerable, nist_level: Some(0), curve: Some("Curve25519".into()), key_bits: Some(256) },
        "1.3.101.111" => KeyInfo { family: Family::Dh, name: "X448".into(), classical_bits: Some(224), quantum: QuantumStatus::Vulnerable, nist_level: Some(0), curve: Some("Curve448".into()), key_bits: Some(448) },
        "1.2.840.10040.4.1" => KeyInfo {
            family: Family::Dsa,
            name: match key_bits { Some(b) => format!("DSA-{}", b), None => "DSA".into() },
            classical_bits: key_bits.map(ifc_security_bits),
            quantum: QuantumStatus::Vulnerable,
            nist_level: Some(0),
            curve: None,
            key_bits,
        },
        "1.2.840.113549.1.3.1" | "1.2.840.10046.2.1" => KeyInfo {
            family: Family::Dh,
            name: match key_bits { Some(b) => format!("DH-{}", b), None => "DH".into() },
            classical_bits: key_bits.map(ifc_security_bits),
            quantum: QuantumStatus::Vulnerable,
            nist_level: Some(0),
            curve: None,
            key_bits,
        },
        other => KeyInfo {
            family: Family::Other,
            name: format!("Unknown({})", other),
            classical_bits: None,
            quantum: QuantumStatus::Unknown,
            nist_level: None,
            curve: None,
            key_bits,
        },
    }
}

/// (human name, is_weak_hash)
pub fn signature_algorithm(oid: &str) -> (String, bool) {
    let (n, weak) = match oid {
        "1.2.840.113549.1.1.4" => ("md5WithRSAEncryption", true),
        "1.2.840.113549.1.1.5" => ("sha1WithRSAEncryption", true),
        "1.2.840.113549.1.1.14" => ("sha224WithRSAEncryption", false),
        "1.2.840.113549.1.1.11" => ("sha256WithRSAEncryption", false),
        "1.2.840.113549.1.1.12" => ("sha384WithRSAEncryption", false),
        "1.2.840.113549.1.1.13" => ("sha512WithRSAEncryption", false),
        "1.2.840.113549.1.1.10" => ("RSASSA-PSS", false),
        "1.2.840.10045.4.1" => ("ecdsa-with-SHA1", true),
        "1.2.840.10045.4.3.1" => ("ecdsa-with-SHA224", false),
        "1.2.840.10045.4.3.2" => ("ecdsa-with-SHA256", false),
        "1.2.840.10045.4.3.3" => ("ecdsa-with-SHA384", false),
        "1.2.840.10045.4.3.4" => ("ecdsa-with-SHA512", false),
        "1.2.840.10040.4.3" => ("dsa-with-sha1", true),
        "2.16.840.1.101.3.4.3.2" => ("dsa-with-sha256", false),
        "1.3.101.112" => ("Ed25519", false),
        "1.3.101.113" => ("Ed448", false),
        _ => {
            if let Some((_, name, _)) = pqc(oid) {
                return (name.to_string(), false);
            }
            return (format!("Unknown({})", oid), false);
        }
    };
    (n.to_string(), weak)
}

/// NIST IR 8547 ipd transition years for a quantum-vulnerable algorithm.
/// Returns (deprecated_after, disallowed_after). 112-bit-or-lower strength is deprecated after 2030;
/// every quantum-vulnerable public-key algorithm is disallowed after 2035 (draft).
pub fn ir8547(classical_bits: Option<u32>) -> (Option<i32>, i32) {
    let dep = match classical_bits {
        Some(b) if b <= 112 => Some(2030),
        _ => None,
    };
    (dep, 2035)
}

pub fn replacement(family: Family, kind_is_key_agreement: bool) -> &'static str {
    match family {
        Family::Dh => "ML-KEM-768 (FIPS 203), hybrid with the classical exchange during transition",
        Family::Rsa | Family::Ec | Family::Dsa | Family::EdDsa => {
            if kind_is_key_agreement {
                "ML-KEM-768 (FIPS 203), hybrid during transition"
            } else {
                "ML-DSA-65 (FIPS 204); SLH-DSA (FIPS 205) for long-lived roots; hybrid/composite during transition"
            }
        }
        _ => "No migration needed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rsa_2048_is_112_bits_and_vulnerable() {
        let k = classify_key("1.2.840.113549.1.1.1", Some(2048), None, None);
        assert_eq!(k.family, Family::Rsa);
        assert_eq!(k.classical_bits, Some(112));
        assert_eq!(k.quantum, QuantumStatus::Vulnerable);
        assert_eq!(k.name, "RSA-2048");
    }
    #[test]
    fn ed25519_is_not_missed() {
        let k = classify_key("1.3.101.112", None, None, None);
        assert_eq!(k.quantum, QuantumStatus::Vulnerable);
    }
    #[test]
    fn ml_dsa_is_resistant() {
        let k = classify_key("2.16.840.1.101.3.4.3.18", None, None, None);
        assert_eq!(k.quantum, QuantumStatus::Resistant);
        assert_eq!(k.nist_level, Some(3));
    }
    #[test]
    fn ir8547_dates() {
        assert_eq!(ir8547(Some(112)), (Some(2030), 2035));
        assert_eq!(ir8547(Some(128)), (None, 2035));
    }
}
