use std::fs;
use tempfile::TempDir;

#[test]
fn test_theorem_pack_import() {
    let d = TempDir::new().unwrap();
    let pack = serde_json::json!({
        "signature": "test-signature",
        "theorems": [
            {
                "theorem_id": "00000000-0000-0000-0000-000000000001",
                "regulation_reference": "TEST",
                "lean4_statement": "test",
                "status": "Unverified",
                "counterexample_asset_id": null,
                "remediation_recommendation": "test remediation"
            }
        ]
    });
    let pack_path = d.path().join("theorems.pack");
    fs::write(&pack_path, serde_json::to_string_pretty(&pack).unwrap()).unwrap();
    let theorems = vericrypt::theorem_import::import_theorem_pack(
        pack_path.to_str().unwrap()
    ).unwrap();
    assert_eq!(theorems.len(), 1);
}

#[test]
fn test_sth_generation_and_export() {
    let root = b"test-root-hash-32-bytes-xxxxxxxxx".to_vec();
    let sth = vericrypt::report::verichain::SignedTreeHead::new(root.clone(), 0);
    assert_eq!(sth.sequence_number, 0);
    assert_eq!(sth.root_hash, root);
    let exported = sth.export_for_anchoring();
    let parsed: serde_json::Value = serde_json::from_str(&exported).unwrap();
    assert_eq!(parsed["sequence_number"], 0);
    assert!(parsed["root_hash"].as_str().unwrap().len() > 0);
}

#[test]
fn test_sth_consistency_verification() {
    let root1 = b"root-hash-number-one-32-bytes-".to_vec();
    let root2 = b"root-hash-number-two-32-bytes-".to_vec();
    let sth1 = vericrypt::report::verichain::SignedTreeHead::new(root1.clone(), 0);
    let sth2 = vericrypt::report::verichain::SignedTreeHead::new(root2.clone(), 1);
    let valid = vericrypt::report::verichain::SignedTreeHead::verify_consistency(
        &sth1, &sth2, &[],
    ).unwrap();
    assert!(valid);
}

#[test]
fn test_sth_non_equivocation_detection() {
    let root_a = b"root-hash-aaaa-32-bytes-xxxxxx".to_vec();
    let root_b = b"root-hash-bbbb-32-bytes-xxxxxx".to_vec();
    let sth_a = vericrypt::report::verichain::SignedTreeHead::new(root_a, 5);
    let sth_b = vericrypt::report::verichain::SignedTreeHead::new(root_b, 5);
    let result = vericrypt::report::verichain::SignedTreeHead::verify_non_equivocation(
        &sth_a, &sth_b,
    ).unwrap();
    assert!(!result);
}

#[test]
fn test_crypto_agility_traits() {
    use vericrypt::crypto::traits::SignatureProvider;
    let sig = vericrypt::crypto::traits::SlhDsaProvider::sign(b"test").unwrap();
    let valid = vericrypt::crypto::traits::SlhDsaProvider::verify(&sig, b"test", &[]).unwrap();
    assert!(valid);
    assert_eq!(
        vericrypt::crypto::traits::SlhDsaProvider::algorithm_name(),
        "SLH-DSA-SHAKE-256s"
    );
    assert_eq!(
        vericrypt::crypto::traits::SlhDsaProvider::nist_security_level(),
        5
    );
}

#[test]
fn test_deployment_mode_flags() {
    let d = TempDir::new().unwrap();
    let args = vericrypt::cli::ScanArgs {
        cert_dir: Some(d.path().to_string_lossy().to_string()),
        network: None,
        output: d.path().join("o").to_string_lossy().to_string(),
        mode: vericrypt::cli::DeploymentMode::Shadow,
        load_theorems: None,
        publish_sth: false,
    };
    vericrypt::cli::run_scan(args).unwrap();
}

#[test]
fn test_sth_export_flag() {
    let d = TempDir::new().unwrap();
    let cert_path = d.path().join("t.der");
    fs::write(&cert_path, &[0x30, 0x82, 0x01, 0x0A, 0x02, 0x01, 0x01, 0x30, 0x0D, 0x06, 0x09, 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x01, 0x01, 0x05, 0x00]).unwrap();
    let args = vericrypt::cli::ScanArgs {
        cert_dir: Some(d.path().to_string_lossy().to_string()),
        network: None,
        output: d.path().join("o").to_string_lossy().to_string(),
        mode: vericrypt::cli::DeploymentMode::Primary,
        load_theorems: None,
        publish_sth: true,
    };
    vericrypt::cli::run_scan(args).unwrap();
    assert!(d.path().join("o").join("sth.json").exists());
}
