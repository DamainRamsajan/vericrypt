//! CycloneDX 1.7 Cryptographic Bill of Materials built to (and tested against) the official schema:
//! algorithm components, related-crypto-material (keys) and certificate components,
//! cross-linked with bom-refs and a dependency graph.
use super::classify;
use super::model::*;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

fn primitive_for(f: Family) -> &'static str {
    match f {
        Family::EdDsa | Family::Dsa | Family::MlDsa | Family::SlhDsa => "signature",
        Family::Dh => "key-agree",
        Family::MlKem => "kem",
        // RSA and EC keys can sign or encrypt; usage is not visible from the key alone.
        _ => "unknown",
    }
}

pub fn build(body: &Body) -> Value {
    let mut algs: BTreeMap<String, Value> = BTreeMap::new();
    let mut components: Vec<Value> = Vec::new();
    let mut deps: Vec<Value> = Vec::new();

    let mut add_key_alg = |a: &Asset, algs: &mut BTreeMap<String, Value>| -> String {
        let r = format!("alg:{}", a.algorithm);
        algs.entry(r.clone()).or_insert_with(|| {
            let mut ap = Map::new();
            ap.insert("primitive".into(), json!(primitive_for(a.family)));
            if let Some(b) = a.key_bits {
                if a.family == Family::Rsa || a.family == Family::Dsa || a.family == Family::Dh {
                    ap.insert("parameterSetIdentifier".into(), json!(b.to_string()));
                }
            }
            if let Some(c) = &a.curve {
                ap.insert("curve".into(), json!(c));
            }
            if let Some(c) = a.classical_bits {
                ap.insert("classicalSecurityLevel".into(), json!(c));
            }
            if let Some(l) = a.nist_quantum_level {
                ap.insert("nistQuantumSecurityLevel".into(), json!(l));
            }
            let mut cp = json!({"assetType": "algorithm", "algorithmProperties": Value::Object(ap)});
            if !a.oid.is_empty() {
                cp["oid"] = json!(a.oid);
            }
            json!({
                "type": "cryptographic-asset",
                "bom-ref": r,
                "name": a.algorithm,
                "cryptoProperties": cp,
                "properties": [{"name": "vericrypt:quantum-status", "value": format!("{:?}", a.quantum_status).to_lowercase()}]
            })
        });
        r
    };

    for a in &body.assets {
        let alg_ref = add_key_alg(a, &mut algs);
        let key_ref = format!("key:{}", a.id);
        let ktype = if a.kind == AssetKind::PrivateKey { "private-key" } else { "public-key" };
        let mut rm = json!({"type": ktype, "algorithmRef": alg_ref});
        if let Some(b) = a.key_bits {
            rm["size"] = json!(b);
        }
        components.push(json!({
            "type": "cryptographic-asset",
            "bom-ref": key_ref,
            "name": if a.kind == AssetKind::Certificate { format!("public key of {}", a.id) } else { format!("{} {}", ktype, a.id) },
            "cryptoProperties": {"assetType": "related-crypto-material", "relatedCryptoMaterialProperties": rm},
            "properties": [{"name": "vericrypt:locations", "value": a.locations.join("; ")}]
        }));
        deps.push(json!({"ref": key_ref, "dependsOn": [alg_ref.clone()]}));

        if a.kind == AssetKind::Certificate {
            let mut cert_deps = vec![json!(key_ref)];
            let mut cprops = Map::new();
            if let Some(s) = &a.subject {
                cprops.insert("subjectName".into(), json!(s));
            }
            if let Some(s) = &a.issuer {
                cprops.insert("issuerName".into(), json!(s));
            }
            if let Some(s) = &a.not_before {
                cprops.insert("notValidBefore".into(), json!(s));
            }
            if let Some(s) = &a.not_after {
                cprops.insert("notValidAfter".into(), json!(s));
            }
            if let (Some(name), Some(oid)) = (&a.signature_algorithm, &a.signature_oid) {
                let sref = format!("alg:{}", name);
                algs.entry(sref.clone()).or_insert_with(|| {
                    let ki = classify::classify_key(oid, None, None, None);
                    let mut ap = json!({"primitive": "signature"});
                    if let Some(l) = ki.nist_level {
                        if ki.quantum == QuantumStatus::Resistant {
                            ap["nistQuantumSecurityLevel"] = json!(l);
                        }
                    }
                    let mut cp = json!({"assetType": "algorithm", "algorithmProperties": ap});
                    cp["oid"] = json!(oid);
                    json!({"type": "cryptographic-asset", "bom-ref": sref, "name": name, "cryptoProperties": cp})
                });
                cprops.insert("signatureAlgorithmRef".into(), json!(sref));
                cert_deps.push(json!(sref));
            }
            cprops.insert("subjectPublicKeyRef".into(), json!(key_ref));
            cprops.insert("certificateFormat".into(), json!("X.509"));
            cprops.insert("fingerprint".into(), json!({"alg": "SHA-256", "content": a.sha256}));
            let cref = format!("cert:{}", a.id);
            components.push(json!({
                "type": "cryptographic-asset",
                "bom-ref": cref,
                "name": a.subject.clone().unwrap_or_else(|| a.id.clone()),
                "cryptoProperties": {"assetType": "certificate", "certificateProperties": Value::Object(cprops)},
                "properties": [
                    {"name": "vericrypt:locations", "value": a.locations.join("; ")},
                    {"name": "vericrypt:trust-scope", "value": format!("{:?}", a.trust_scope).to_lowercase()}
                ]
            }));
            deps.push(json!({"ref": cref, "dependsOn": cert_deps}));
        }
    }
    let mut all: Vec<Value> = algs.into_values().collect();
    all.extend(components);

    let root_bytes = hex::decode(&body.merkle_root).unwrap_or_else(|_| vec![0u8; 32]);
    let mut rb = [0u8; 16];
    for (i, b) in root_bytes.iter().take(16).enumerate() {
        rb[i] = *b;
    }
    let serial = uuid::Builder::from_random_bytes(rb).into_uuid();
    json!({
        "bomFormat": "CycloneDX",
        "specVersion": "1.7",
        "serialNumber": format!("urn:uuid:{}", serial),
        "version": 1,
        "metadata": {
            "timestamp": body.generated_at,
            "tools": {"components": [{"type": "application", "name": body.tool.name, "version": body.tool.version}]},
            "properties": [{"name": "vericrypt:merkle-root", "value": body.merkle_root}]
        },
        "components": all,
        "dependencies": deps
    })
}
