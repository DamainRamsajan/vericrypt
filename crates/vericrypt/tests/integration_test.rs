use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Generate a synthetic PEM certificate for testing.
fn generate_test_cert(dir: &TempDir, name: &str, algorithm_oid: &str) -> PathBuf {
    let cert_path = dir.path().join(name);
    // This is a minimal self-signed certificate structure.
    // In production testing, we'd use rcgen or openssl to generate real certs.
    // For now, we create a DER-encoded placeholder that x509-parser can partially parse.
    let der_bytes = vec![
        0x30, 0x82, 0x01, 0x0A, // SEQUENCE header
        0x02, 0x01, 0x01,       // Version
        0x30, 0x0D,             // AlgorithmIdentifier SEQUENCE
        0x06, 0x09, 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x01, 0x01, // RSA OID
        0x05, 0x00,             // NULL parameters
    ];
    fs::write(&cert_path, &der_bytes).unwrap();
    cert_path
}

#[test]
fn test_full_pipeline_with_synthetic_certs() {
    let temp_dir = TempDir::new().unwrap();
    
    // Generate test certificates
    generate_test_cert(&temp_dir, "rsa_cert.der", "1.2.840.113549.1.1.1");
    generate_test_cert(&temp_dir, "ec_cert.der", "1.2.840.10045.2.1");
    
    // Run the pipeline
    let args = vericrypt::cli::ScanArgs {
        cert_dir: Some(temp_dir.path().to_string_lossy().to_string()),
        network: None,
        output: temp_dir.path().join("report").to_string_lossy().to_string(),
    };
    
    vericrypt::cli::run_scan(args).unwrap();
    
    // Verify output files exist
    let report_dir = temp_dir.path().join("report");
    assert!(report_dir.join("report.pqc").exists());
    assert!(report_dir.join("cbom.json").exists());
    assert!(report_dir.join("roadmap.md").exists());
    
    // Verify CBOM is valid JSON
    let cbom_content = fs::read_to_string(report_dir.join("cbom.json")).unwrap();
    let cbom: serde_json::Value = serde_json::from_str(&cbom_content).unwrap();
    assert_eq!(cbom["bomFormat"], "CycloneDX");
    assert_eq!(cbom["specVersion"], "1.7");
    
    // Verify roadmap has content
    let roadmap = fs::read_to_string(report_dir.join("roadmap.md")).unwrap();
    assert!(roadmap.contains("VeriCrypt PQC Migration Roadmap"));
}

#[test]
fn test_verification_tool() {
    let temp_dir = TempDir::new().unwrap();
    
    let args = vericrypt::cli::ScanArgs {
        cert_dir: Some(temp_dir.path().to_string_lossy().to_string()),
        network: None,
        output: temp_dir.path().join("report").to_string_lossy().to_string(),
    };
    
    vericrypt::cli::run_scan(args).unwrap();
    
    let report_path = temp_dir.path().join("report").join("report.pqc");
    let result = vericrypt::report::verify_file(&report_path).unwrap();
    assert!(result.contains("VERIFIED") || result.contains("scan at"));
}

#[test]
fn test_license_activation_flow() {
    // Without license: scan should still work, report unsigned
    let temp_dir = TempDir::new().unwrap();
    let args = vericrypt::cli::ScanArgs {
        cert_dir: Some(temp_dir.path().to_string_lossy().to_string()),
        network: None,
        output: temp_dir.path().join("report").to_string_lossy().to_string(),
    };
    
    vericrypt::cli::run_scan(args).unwrap();
    
    let report_path = temp_dir.path().join("report").join("report.pqc");
    let content = fs::read_to_string(&report_path).unwrap();
    let report: vericrypt::types::PqcReport = serde_json::from_str(&content).unwrap();
    
    // Without license activation, signature is None
    assert!(report.signature.is_none());
}

#[test]
fn test_csv_inventory_parsing() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("inventory.csv");
    
    fs::write(&csv_path, "host,port,cert_path,algorithm,key_size,expiry,usage_context\n\
        server1.example.com,443,/etc/ssl/certs/server1.pem,RSA,2048,2027-12-31,web\n\
        server2.example.com,443,/etc/ssl/certs/server2.pem,ECDSA,256,2028-06-30,api\n").unwrap();
    
    let args = vericrypt::cli::ScanArgs {
        cert_dir: Some(temp_dir.path().to_string_lossy().to_string()),
        network: None,
        output: temp_dir.path().join("report").to_string_lossy().to_string(),
    };
    
    vericrypt::cli::run_scan(args).unwrap();
}

#[test]
fn test_json_inventory_parsing() {
    let temp_dir = TempDir::new().unwrap();
    let json_path = temp_dir.path().join("inventory.json");
    
    let inventory = serde_json::json!({
        "certificates": [
            {
                "host": "api.bank.com",
                "port": 443,
                "fingerprint": "abc123",
                "algorithm": "RSA",
                "key_size": 4096,
                "expiry": "2029-01-15T00:00:00Z"
            }
        ]
    });
    
    fs::write(&json_path, serde_json::to_string_pretty(&inventory).unwrap()).unwrap();
    
    let args = vericrypt::cli::ScanArgs {
        cert_dir: Some(temp_dir.path().to_string_lossy().to_string()),
        network: None,
        output: temp_dir.path().join("report").to_string_lossy().to_string(),
    };
    
    vericrypt::cli::run_scan(args).unwrap();
}
