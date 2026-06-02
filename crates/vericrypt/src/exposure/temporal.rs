use crate::types::CryptoAsset;

/// Compute temporal hazard using the Ld > Ha vulnerability condition.
///
/// From "Harvest Now, Decrypt Later: A Time-Dependent Threat Model" (March 2026):
///   temporal_hazard(asset) = max(0, 1 - Ha / Ld)
///
/// Where:
///   Ha = estimated attacker decryption horizon (2028–2033, configurable)
///   Ld = data confidentiality lifetime in years
///
/// An asset is only vulnerable if its data lifetime exceeds the
/// attacker's decryption horizon (Ld > Ha).
pub fn compute_temporal_hazard(asset: &CryptoAsset, attacker_horizon: f64) -> f64 {
    let data_lifetime = asset.data_lifetime_years.unwrap_or(7.0);
    if data_lifetime <= 0.0 {
        return 0.0;
    }
    let hazard = 1.0 - (attacker_horizon / data_lifetime);
    hazard.max(0.0)
}

/// Default attacker horizon from VERICRYPT_ATTACKER_HORIZON env var or 2030.0.
pub fn default_attacker_horizon() -> f64 {
    std::env::var("VERICRYPT_ATTACKER_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2030.0)
}

/// Map usage_context to data confidentiality lifetime in years.
pub fn data_lifetime_from_context(usage_context: &str) -> f64 {
    match usage_context.to_lowercase().as_str() {
        "customer_financial_records" | "financial" | "banking" => 7.0,
        "legal_instruments" | "legal" | "contracts" => 30.0,
        "healthcare" | "medical" | "phi" => 20.0,
        "government_classified" | "classified" => 50.0,
        "operational" | "infrastructure" => 5.0,
        "session_tokens" | "ephemeral" => 1.0 / 365.0,
        "payment_transactions" | "transactions" => 7.0,
        "identity" | "pii" => 10.0,
        _ => 7.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CryptoAsset, AssetType, Algorithm};

    fn test_asset(lifetime: f64) -> CryptoAsset {
        CryptoAsset {
            asset_id: uuid::Uuid::new_v4(),
            asset_type: AssetType::Certificate,
            algorithm: Algorithm {
                name: "RSA".into(), family: "RSA".into(),
                quantum_vulnerable: true, vulnerability_type: Some("Shor".into()),
                nist_pqc_replacement: Some("ML-DSA-87".into()), shelf_life_years: Some(5),
            },
            key_size: Some(2048), expiry_date: None,
            fingerprint: "test".into(), source_location: "test".into(),
            nist_quantum_security_level: Some(1),
            data_lifetime_years: Some(lifetime),
            usage_context: None,
        }
    }

    #[test]
    fn test_long_lived_data_vulnerable() {
        let asset = test_asset(30.0);
        let hazard = compute_temporal_hazard(&asset, 5.0);
        assert!(hazard > 0.0);
    }

    #[test]
    fn test_ephemeral_data_safe() {
        let asset = test_asset(1.0 / 365.0);
        let hazard = compute_temporal_hazard(&asset, 5.0);
        assert!(hazard < 0.01);
    }

    #[test]
    fn test_lifetime_mapping() {
        assert_eq!(data_lifetime_from_context("financial"), 7.0);
        assert_eq!(data_lifetime_from_context("legal"), 30.0);
        assert_eq!(data_lifetime_from_context("healthcare"), 20.0);
        assert!(data_lifetime_from_context("session_tokens") < 1.0);
    }
}
