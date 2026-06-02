use std::fs;
use tempfile::TempDir;

#[test]
fn test_custody_chain_built() {
    let status = vericrypt::tee::collect_attestation();
    let custody = vericrypt::evidence::build_custody_chain("test_merkle_root", &status);
    assert!(!custody.custody_root.is_empty());
    assert!(custody.scan_timestamp <= chrono::Utc::now());
}

#[test]
fn test_compliance_confidence_computed() {
    let theorems = vec![
        vericrypt::types::ComplianceTheorem {
            theorem_id: uuid::Uuid::new_v4(),
            regulation_reference: "DORA Art. 12.3".into(),
            lean4_statement: "test".into(),
            status: vericrypt::types::ProofStatus::Proved,
            counterexample_asset_id: None,
            remediation_recommendation: None,
        },
    ];
    let inventory = vericrypt::confidence::compute_inventory_confidence(100, 0, &[], 0);
    let conf = vericrypt::confidence::compute_compliance_confidence(&theorems, &inventory);
    assert_eq!(conf.proof_confidence, 1.0);
    assert!(conf.composite_confidence > 0.0);
}

#[test]
fn test_pki_chain_built() {
    let chain = vericrypt::pki::build_certificate_chain().unwrap();
    assert!(!chain.is_empty());
    assert_eq!(chain[0].issuer, "Verity Root Authority");
}

#[test]
fn test_violations_written_when_counterexamples_exist() {
    let d = TempDir::new().unwrap();
    let theorems = vec![
        vericrypt::types::ComplianceTheorem {
            theorem_id: uuid::Uuid::new_v4(),
            regulation_reference: "TEST".into(),
            lean4_statement: "test".into(),
            status: vericrypt::types::ProofStatus::Counterexample,
            counterexample_asset_id: Some(uuid::Uuid::new_v4()),
            remediation_recommendation: Some("Fix it".into()),
        },
    ];
    vericrypt::violations::write_violations(&d.path().to_path_buf(), &theorems).unwrap();
    assert!(d.path().join("violations.txt").exists());
}

#[test]
fn test_verification_script_generated() {
    let d = TempDir::new().unwrap();
    vericrypt::verify_script::write_verification_script(&d.path().to_path_buf()).unwrap();
    let script = fs::read_to_string(d.path().join("verify.sh")).unwrap();
    assert!(script.contains("vericrypt-verify"));
}

#[test]
fn test_full_pipeline_with_hardening() {
    let d = TempDir::new().unwrap();
    let cert_path = d.path().join("t.der");
    fs::write(&cert_path, &[0x30, 0x82, 0x01, 0x0A, 0x02, 0x01, 0x01, 0x30, 0x0D, 0x06, 0x09, 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x01, 0x01, 0x05, 0x00]).unwrap();
    
    let args = vericrypt::cli::ScanArgs {
        cert_dir: Some(d.path().to_string_lossy().to_string()),
        network: None,
        output: d.path().join("o").to_string_lossy().to_string(),
        mode: vericrypt::cli::DeploymentMode::Shadow,
        load_theorems: None,
        publish_sth: false,
    };
    vericrypt::cli::run_scan(args).unwrap();
    
    let o = d.path().join("o");
    assert!(o.join("report.pqc").exists());
    assert!(o.join("cbom.json").exists());
    assert!(o.join("roadmap.md").exists());
    assert!(o.join("verify.sh").exists());
    
    let report_content = fs::read_to_string(o.join("report.pqc")).unwrap();
    let report: vericrypt::types::PqcReport = serde_json::from_str(&report_content).unwrap();
    assert!(report.total_assets > 0);
}
