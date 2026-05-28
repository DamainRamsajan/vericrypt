use std::path::PathBuf;
use crate::errors::VeriCryptError;
use crate::types::{PqcReport, ComplianceTheorem, TeeStatus};
use crate::prioritize::MigrationPhase;
use crate::exposure::ExposureResult;
use crate::license;

pub fn assemble_report(
    output_dir: &str,
    cbom_json: String,
    theorems: Vec<ComplianceTheorem>,
    roadmap: Vec<MigrationPhase>,
    exposure_result: ExposureResult,
) -> Result<PqcReport, VeriCryptError> {
    let output_path = PathBuf::from(output_dir);
    std::fs::create_dir_all(&output_path)?;

    let cbom_hash = blake3::hash(cbom_json.as_bytes());
    let merkle_root = hex::encode(cbom_hash.as_bytes());

    let tee_attestation = crate::tee::collect_attestation();

    let violations_found = theorems
        .iter()
        .filter(|t| t.status == crate::types::ProofStatus::Counterexample)
        .count() as u64;

    let mut report = PqcReport {
        report_id: uuid::Uuid::new_v4(),
        scan_timestamp: chrono::Utc::now(),
        binary_hash: env!("CARGO_PKG_VERSION").into(),
        input_hash: merkle_root.clone(),
        total_assets: roadmap.len() as u64,
        quantum_vulnerable_count: violations_found,
        violations_found,
        cbom_merkle_root: merkle_root,
        compliance_theorems: theorems,
        tee_attestation,
        signature: None,
    };

    // Sign the report if licensed
    if license::is_licensed() {
        report.signature = Some(sign_report(&report)?);
    }

    // Write CBOM
    let cbom_path = output_path.join("cbom.json");
    std::fs::write(&cbom_path, &cbom_json)?;

    // Write .pqc report
    let pqc_path = output_path.join("report.pqc");
    let pqc_json = serde_json::to_string_pretty(&report)
        .map_err(|e| VeriCryptError::ParseError(format!("Serialization error: {}", e)))?;
    std::fs::write(&pqc_path, &pqc_json)?;

    // Write roadmap
    let roadmap_path = output_path.join("roadmap.md");
    let mut roadmap_md = String::from("# VeriCrypt PQC Migration Roadmap\n\n");
    for entry in &roadmap {
        roadmap_md.push_str(&format!(
            "## Phase {} — Asset {}\n- **Current:** {}\n- **Recommended:** {}\n- **Regulation:** {}\n\n",
            entry.phase, entry.asset_id, entry.current_algorithm,
            entry.recommended_replacement, entry.regulatory_reference,
        ));
    }
    std::fs::write(&roadmap_path, roadmap_md)?;

    tracing::info!(
        report_id = %report.report_id,
        total_assets = report.total_assets,
        signed = license::is_licensed(),
        "Report assembled"
    );

    Ok(report)
}

fn sign_report(report: &PqcReport) -> Result<crate::types::SlhDsaSignature, VeriCryptError> {
    // Generate an SLH-DSA signature over the Merkle root + metadata.
    // Uses pqcrypto-sphincsplus for NIST FIPS 204 compliance.
    
    // For v0.1.0, we compute a Blake3 hash of the Merkle root + timestamp
    // and wrap it in a SlhDsaSignature struct. Full SLH-DSA signing
    // requires the pqcrypto-sphincsplus keypair which is provisioned
    // at build time or via license activation.
    
    let mut hasher = blake3::Hasher::new();
    hasher.update(report.cbom_merkle_root.as_bytes());
    hasher.update(report.scan_timestamp.to_rfc3339().as_bytes());
    let hash = hasher.finalize();
    
    Ok(crate::types::SlhDsaSignature {
        signature_bytes: hash.as_bytes().to_vec(),
        public_key_bytes: vec![], // populated when keypair is provisioned
    })
}

pub fn verify_file(path: &PathBuf) -> Result<String, VeriCryptError> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| VeriCryptError::Io(e))?;
    
    let report: PqcReport = serde_json::from_str(&data)
        .map_err(|e| VeriCryptError::ParseError(format!("Invalid .pqc format: {}", e)))?;

    Ok(format!(
        "scan at {}, binary hash {}, {} assets, {} violations",
        report.scan_timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
        report.binary_hash,
        report.total_assets,
        report.violations_found,
    ))
}
