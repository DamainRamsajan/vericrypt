use std::fs;
use tempfile::TempDir;

fn make_cert(dir: &TempDir, name: &str) -> std::path::PathBuf {
    let p = dir.path().join(name);
    fs::write(
        &p,
        &[
            0x30, 0x82, 0x01, 0x0A, 0x02, 0x01, 0x01, 0x30, 0x0D, 0x06, 0x09, 0x2A, 0x86, 0x48,
            0x86, 0xF7, 0x0D, 0x01, 0x01, 0x01, 0x05, 0x00,
        ],
    )
    .unwrap();
    p
}

#[test]
fn test_full_pipeline() {
    let d = TempDir::new().unwrap();
    make_cert(&d, "r.der");
    let args = vericrypt::cli::ScanArgs {
        cert_dir: Some(d.path().to_string_lossy().to_string()),
        network: None,
        output: d.path().join("o").to_string_lossy().to_string(),
        mode: vericrypt::cli::DeploymentMode::Shadow,
        load_bytecode: None,
        publish_sth: false,
    };
    vericrypt::cli::run_scan(args).unwrap();
    let o = d.path().join("o");
    assert!(o.join("report.pqc").exists());
    assert!(o.join("cbom.json").exists());
    assert!(o.join("roadmap.md").exists());
}

#[test]
fn test_verify() {
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
    let result = vericrypt::report::verify_file(&d.path().join("o").join("report.pqc")).unwrap();
    assert!(result.contains("VERIFIED"));
}

#[test]
fn test_csv() {
    let d = TempDir::new().unwrap();
    fs::write(
        d.path().join("i.csv"),
        "h,p,c,alg,ks,exp,use\ns,443,x,RSA,2048,2027-12-31,w\n",
    )
    .unwrap();
    let args = vericrypt::cli::ScanArgs {
        cert_dir: Some(d.path().to_string_lossy().to_string()),
        network: None,
        output: d.path().join("o").to_string_lossy().to_string(),
        mode: vericrypt::cli::DeploymentMode::Shadow,
        load_bytecode: None,
        publish_sth: false,
    };
    vericrypt::cli::run_scan(args).unwrap();
}
