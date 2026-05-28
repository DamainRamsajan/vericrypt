use crate::errors::VeriCryptError;
use crate::graph::CryptoGraph;

/// Generate a CycloneDX 1.7 CBOM from the cryptographic graph.
pub fn generate_cbom(graph: &CryptoGraph) -> Result<String, VeriCryptError> {
    let components: Vec<serde_json::Value> = graph
        .get_all_assets()
        .iter()
        .map(|asset| {
            serde_json::json!({
                "type": "cryptographic-asset",
                "name": asset.fingerprint,
                "cryptoProperties": {
                    "assetType": format!("{:?}", asset.asset_type).to_lowercase(),
                    "algorithmProperties": {
                        "algorithm": asset.algorithm.name,
                        "variant": asset.algorithm.family,
                        "quantumSecurityLevel": asset.nist_quantum_security_level.unwrap_or(0),
                        "vulnerabilityStatus": if asset.algorithm.quantum_vulnerable { "vulnerable" } else { "secure" }
                    },
                    "relatedCryptoMaterial": [],
                    "evidence": [
                        {
                            "type": "location",
                            "location": asset.source_location
                        }
                    ]
                }
            })
        })
        .collect();

    let cbom = serde_json::json!({
        "bomFormat": "CycloneDX",
        "specVersion": "1.7",
        "serialNumber": format!("urn:uuid:{}", uuid::Uuid::new_v4()),
        "version": 1,
        "metadata": {
            "component": {
                "type": "cryptographic-asset-inventory",
                "name": "vericrypt-cbom",
                "version": env!("CARGO_PKG_VERSION")
            },
            "timestamp": chrono::Utc::now().to_rfc3339()
        },
        "components": components,
        "dependencies": []
    });

    serde_json::to_string_pretty(&cbom)
        .map_err(|e| VeriCryptError::CbomSerialization(e.to_string()))
}
