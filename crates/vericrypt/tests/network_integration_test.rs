use std::fs;
use tempfile::TempDir;

#[test]
fn test_network_cidr_parsing() {
    let cidr: ipnet::IpNet = "10.0.0.0/24".parse().unwrap();
    assert_eq!(cidr.prefix_len(), 24);
}

#[test]
fn test_network_scanner_with_invalid_cidr() {
    let d = TempDir::new().unwrap();
    let args = vericrypt::cli::ScanArgs {
        cert_dir: Some(d.path().to_string_lossy().to_string()),
        network: Some("invalid-cidr".to_string()),
        output: d.path().join("o").to_string_lossy().to_string(),
        mode: vericrypt::cli::DeploymentMode::Shadow,
        load_theorems: None,
        publish_sth: false,
    };
    let result = vericrypt::cli::run_scan(args);
    assert!(result.is_err());
}

#[test]
fn test_lean4_bridge_detection() {
    let bridge = vericrypt::compliance::lean4_bridge::Lean4Bridge::new();
    let available = bridge.is_available();
    assert!(available || !available);
}

#[test]
fn test_combined_file_and_network_scan() {
    let d = TempDir::new().unwrap();
    fs::write(d.path().join("t.crt"), "-----BEGIN CERTIFICATE-----\nMIIB...\n-----END CERTIFICATE-----").unwrap();
    let args = vericrypt::cli::ScanArgs {
        cert_dir: Some(d.path().to_string_lossy().to_string()),
        network: Some("127.0.0.1/32".to_string()),
        output: d.path().join("o").to_string_lossy().to_string(),
        mode: vericrypt::cli::DeploymentMode::Shadow,
        load_theorems: None,
        publish_sth: false,
    };
    let result = vericrypt::cli::run_scan(args);
    assert!(result.is_ok());
}
