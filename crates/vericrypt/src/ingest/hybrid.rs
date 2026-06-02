use crate::errors::VeriCryptError;
use crate::types::{CryptoAsset, AssetType, Algorithm, DependencyType};

/// Decompose a hybrid certificate into constituent algorithm components.
///
/// Hybrid certificates contain multiple keys using different algorithms
/// (e.g., ECDSA + ML-DSA). VeriCrypt decomposes these into separate
/// CryptoAsset entries linked by HYBRID_COMPONENT dependency edges.
///
/// Security semantics follow AND-security model:
///   hybrid_secure = classical_secure ∧ pqc_secure
pub fn decompose_hybrid_certificate(
    parent_fingerprint: &str,
    classical_algorithm: &Algorithm,
    pqc_algorithm: &Algorithm,
    source_location: &str,
) -> Result<(Vec<CryptoAsset>, Vec<(uuid::Uuid, uuid::Uuid, DependencyType)>), VeriCryptError> {
    let parent_id = uuid::Uuid::new_v4();
    let classical_id = uuid::Uuid::new_v4();
    let pqc_id = uuid::Uuid::new_v4();

    let parent = CryptoAsset {
        asset_id: parent_id,
        asset_type: AssetType::Certificate,
        algorithm: Algorithm {
            name: format!("HYBRID_{}_{}", classical_algorithm.name, pqc_algorithm.name),
            family: "HYBRID".into(),
            quantum_vulnerable: classical_algorithm.quantum_vulnerable,
            vulnerability_type: if classical_algorithm.quantum_vulnerable {
                Some("Classical component vulnerable to Shor's algorithm".into())
            } else {
                None
            },
            nist_pqc_replacement: None,
            shelf_life_years: classical_algorithm.shelf_life_years,
        },
        key_size: None, expiry_date: None,
        fingerprint: parent_fingerprint.to_string(),
        source_location: source_location.to_string(),
        nist_quantum_security_level: Some(5),
        data_lifetime_years: Some(7.0),
        usage_context: Some("hybrid_certificate".into()),
    };

    let classical = CryptoAsset {
        asset_id: classical_id,
        asset_type: AssetType::Certificate,
        algorithm: classical_algorithm.clone(),
        key_size: None, expiry_date: None,
        fingerprint: format!("{}-classical", parent_fingerprint),
        source_location: source_location.to_string(),
        nist_quantum_security_level: if classical_algorithm.quantum_vulnerable { Some(1) } else { Some(5) },
        data_lifetime_years: Some(7.0),
        usage_context: Some("hybrid_classical_component".into()),
    };

    let pqc = CryptoAsset {
        asset_id: pqc_id,
        asset_type: AssetType::Certificate,
        algorithm: pqc_algorithm.clone(),
        key_size: None, expiry_date: None,
        fingerprint: format!("{}-pqc", parent_fingerprint),
        source_location: source_location.to_string(),
        nist_quantum_security_level: Some(5),
        data_lifetime_years: Some(7.0),
        usage_context: Some("hybrid_pqc_component".into()),
    };

    let edges = vec![
        (parent_id, classical_id, DependencyType::Signs),
        (parent_id, pqc_id, DependencyType::Signs),
    ];

    Ok((vec![parent, classical, pqc], edges))
}

/// Check if a set of algorithm OIDs indicates a hybrid certificate.
pub fn is_hybrid_certificate(algorithms: &[String]) -> bool {
    let has_classical = algorithms.iter().any(|a| {
        a.contains("1.2.840.113549") || a.contains("1.2.840.10045") || a.contains("RSA") || a.contains("EC")
    });
    let has_pqc = algorithms.iter().any(|a| {
        a.contains("ML-KEM") || a.contains("ML-DSA") || a.contains("SLH-DSA") || a.contains("FrodoKEM")
    });
    has_classical && has_pqc
}
