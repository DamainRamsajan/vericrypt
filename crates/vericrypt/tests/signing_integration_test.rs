use std::fs;
use tempfile::TempDir;

#[test]
fn test_slh_dsa_structural_verification() {
    let sig = vericrypt::types::SlhDsaSignature {
        signature_bytes: blake3::hash(b"test-message").as_bytes().to_vec(),
        public_key_bytes: vec![],
    };
    let result = vericrypt::report::slh_dsa::verify_slh_dsa(&sig, b"test-message").unwrap();
    assert!(result);
}

#[test]
fn test_slh_dsa_tampered_message_fails() {
    let sig = vericrypt::types::SlhDsaSignature {
        signature_bytes: blake3::hash(b"original").as_bytes().to_vec(),
        public_key_bytes: vec![],
    };
    let result = vericrypt::report::slh_dsa::verify_slh_dsa(&sig, b"tampered").unwrap();
    assert!(!result);
}

#[test]
fn test_report_unsigned_without_license() {
    let d = TempDir::new().unwrap();
    let args = vericrypt::cli::ScanArgs {
        cert_dir: Some(d.path().to_string_lossy().to_string()),
        network: None,
        output: d.path().join("o").to_string_lossy().to_string(),
        mode: vericrypt::cli::DeploymentMode::Shadow,
        load_bytecode: None,
        publish_sth: false,
    };
    vericrypt::cli::run_scan(args).unwrap();
    let content = fs::read_to_string(d.path().join("o").join("report.pqc")).unwrap();
    let report: vericrypt::types::PqcReport = serde_json::from_str(&content).unwrap();
    assert!(report.signature.is_none());
}

#[test]
fn test_tee_detection_does_not_panic() {
    let tee_type = vericrypt::tee::detect_tee();
    assert!(matches!(tee_type, vericrypt::tee::TeeType::IntelTdx | vericrypt::tee::TeeType::AmdSevSnp | vericrypt::tee::TeeType::None));
}

#[test]
fn test_tee_attestation_collection() {
    let status = vericrypt::tee::collect_attestation();
    match status {
        vericrypt::types::TeeStatus::Attested { .. } => {},
        vericrypt::types::TeeStatus::Unavailable { .. } => {},
    }
}

#[test]
fn test_offline_verifier_rejects_invalid_file() {
    let d = TempDir::new().unwrap();
    let bad_path = d.path().join("nonexistent.pqc");
    let result = vericrypt::report::verify_file(&bad_path);
    assert!(result.is_err());
}
