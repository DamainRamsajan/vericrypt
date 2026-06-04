#[test]
fn test_asl_runtime_initialization() {
    let runtime = vericrypt::compliance::asl_runtime::AslRuntime::new();
    let frameworks = runtime.available_frameworks();
    assert!(!frameworks.is_empty(), "No regulatory frameworks loaded");
}

#[test]
fn test_dora_framework_available() {
    let runtime = vericrypt::compliance::asl_runtime::AslRuntime::new();
    assert!(runtime.has_framework("DORA"), "DORA framework not found");
}

#[test]
fn test_pqfif_framework_available() {
    let runtime = vericrypt::compliance::asl_runtime::AslRuntime::new();
    assert!(runtime.has_framework("PQFIF"), "PQFIF framework not found");
}

#[test]
fn test_ncsc_framework_available() {
    let runtime = vericrypt::compliance::asl_runtime::AslRuntime::new();
    assert!(runtime.has_framework("NCSC"), "NCSC framework not found");
}

#[test]
fn test_nist_framework_available() {
    let runtime = vericrypt::compliance::asl_runtime::AslRuntime::new();
    assert!(runtime.has_framework("NIST"), "NIST framework not found");
}

#[test]
fn test_asl_execution_with_inventory_hash() {
    let runtime = vericrypt::compliance::asl_runtime::AslRuntime::new();
    let inventory_hash = blake3::hash(b"test-inventory").as_bytes().to_vec();
    let result = runtime.execute_framework("DORA", &inventory_hash);
    assert!(
        result.is_ok(),
        "ASL VM execution failed: {:?}",
        result.err()
    );
}

#[test]
fn test_asl_execution_produces_vm_state() {
    let runtime = vericrypt::compliance::asl_runtime::AslRuntime::new();
    let inventory_hash = blake3::hash(b"test-inventory").as_bytes().to_vec();
    let (vm_state, theorem) = runtime.execute_framework("DORA", &inventory_hash).unwrap();
    assert!(
        vm_state.schedule_trace.len() > 0,
        "Schedule trace should not be empty"
    );
    assert!(
        theorem.asl_statement.contains("ASL VM"),
        "Theorem should reference ASL VM"
    );
}

#[test]
fn test_asl_deterministic_execution() {
    let runtime = vericrypt::compliance::asl_runtime::AslRuntime::new();
    let inventory_hash = blake3::hash(b"deterministic-test").as_bytes().to_vec();

    let (vm_state1, _) = runtime.execute_framework("DORA", &inventory_hash).unwrap();
    let (vm_state2, _) = runtime.execute_framework("DORA", &inventory_hash).unwrap();

    assert_eq!(
        vm_state1.schedule_trace.len(),
        vm_state2.schedule_trace.len(),
        "Same input should produce identical trace length"
    );
}

#[test]
fn test_bytecode_loading_flag() {
    use std::fs;
    use tempfile::TempDir;

    let d = TempDir::new().unwrap();
    let cert_path = d.path().join("t.der");
    fs::write(
        &cert_path,
        &[
            0x30, 0x82, 0x01, 0x0A, 0x02, 0x01, 0x01, 0x30, 0x0D, 0x06, 0x09, 0x2A, 0x86, 0x48,
            0x86, 0xF7, 0x0D, 0x01, 0x01, 0x01, 0x05, 0x00,
        ],
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
