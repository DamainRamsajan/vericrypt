use crate::errors::VeriCryptError; use crate::graph::CryptoGraph;
pub fn generate_cbom(g: &CryptoGraph) -> Result<String, VeriCryptError> {
    let comps: Vec<serde_json::Value> = g.get_all_assets().iter().map(|a| serde_json::json!({
        "type":"cryptographic-asset","name":a.fingerprint,
        "cryptoProperties":{"assetType":format!("{:?}",a.asset_type).to_lowercase(),"algorithmProperties":{"algorithm":a.algorithm.name,"variant":a.algorithm.family,"quantumSecurityLevel":a.nist_quantum_security_level.unwrap_or(0)},"evidence":[{"type":"location","location":a.source_location}]}
    })).collect();
    serde_json::to_string_pretty(&serde_json::json!({
        "bomFormat":"CycloneDX","specVersion":"1.7","serialNumber":format!("urn:uuid:{}",uuid::Uuid::new_v4()),"version":1,
        "metadata":{"component":{"type":"cryptographic-asset-inventory","name":"vericrypt-cbom","version":env!("CARGO_PKG_VERSION")},"timestamp":chrono::Utc::now().to_rfc3339()},
        "components":comps,"dependencies":[]
    })).map_err(|e| VeriCryptError::CbomSerialization(e.to_string()))
}
