use std::fs; use std::path::PathBuf; use tempfile::TempDir;
fn cert(dir: &TempDir, n: &str) -> PathBuf {
    let p = dir.path().join(n);
    fs::write(&p, &[0x30,0x82,0x01,0x0A,0x02,0x01,0x01,0x30,0x0D,0x06,0x09,0x2A,0x86,0x48,0x86,0xF7,0x0D,0x01,0x01,0x01,0x05,0x00]).unwrap(); p
}
#[test] fn pipeline() {
    let d = TempDir::new().unwrap(); cert(&d,"r.der");
    vericrypt::cli::run_scan(vericrypt::cli::ScanArgs{cert_dir:Some(d.path().to_string_lossy().to_string()),network:None,output:d.path().join("o").to_string_lossy().to_string()}).unwrap();
    let o = d.path().join("o"); assert!(o.join("report.pqc").exists());
}
#[test] fn verify() {
    let d = TempDir::new().unwrap();
    vericrypt::cli::run_scan(vericrypt::cli::ScanArgs{cert_dir:Some(d.path().to_string_lossy().to_string()),network:None,output:d.path().join("o").to_string_lossy().to_string()}).unwrap();
    assert!(vericrypt::report::verify_file(&d.path().join("o").join("report.pqc")).unwrap().contains("VERIFIED"));
}
#[test] fn csv() {
    let d = TempDir::new().unwrap();
    fs::write(d.path().join("i.csv"),"h,p,c,alg,ks,exp,use\ns,443,x,RSA,2048,2027-12-31,w\n").unwrap();
    vericrypt::cli::run_scan(vericrypt::cli::ScanArgs{cert_dir:Some(d.path().to_string_lossy().to_string()),network:None,output:d.path().join("o").to_string_lossy().to_string()}).unwrap();
}
