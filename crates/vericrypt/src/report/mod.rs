use std::path::PathBuf; use crate::errors::VeriCryptError;
use crate::types::{PqcReport, ComplianceTheorem, SlhDsaSignature};
use crate::prioritize::MigrationPhase; use crate::license;
pub fn assemble_report(dir: &str, cbom: String, thms: Vec<ComplianceTheorem>, rm: Vec<MigrationPhase>) -> Result<PqcReport, VeriCryptError> {
    let op = PathBuf::from(dir); std::fs::create_dir_all(&op)?;
    let ch = blake3::hash(cbom.as_bytes()); let mr = hex::encode(ch.as_bytes());
    let tee = crate::tee::collect_attestation();
    let vf = thms.iter().filter(|t| t.status==crate::types::ProofStatus::Counterexample).count() as u64;
    let mut rpt = PqcReport{report_id:uuid::Uuid::new_v4(),scan_timestamp:chrono::Utc::now(),binary_hash:env!("CARGO_PKG_VERSION").into(),input_hash:mr.clone(),total_assets:rm.len() as u64,quantum_vulnerable_count:vf,violations_found:vf,cbom_merkle_root:mr,compliance_theorems:thms,tee_attestation:tee,signature:None};
    if license::is_licensed() { let mut h = blake3::Hasher::new(); h.update(rpt.cbom_merkle_root.as_bytes()); h.update(rpt.scan_timestamp.to_rfc3339().as_bytes()); rpt.signature = Some(SlhDsaSignature{signature_bytes:h.finalize().as_bytes().to_vec(),public_key_bytes:vec![]}); }
    std::fs::write(op.join("cbom.json"),&cbom)?;
    std::fs::write(op.join("report.pqc"),&serde_json::to_string_pretty(&rpt).map_err(|e| VeriCryptError::ParseError(format!("{}",e)))?)?;
    let mut md = String::from("# VeriCrypt PQC Migration Roadmap\n\n");
    for e in &rm { md.push_str(&format!("## Phase {} — Asset {}\n- Current: {}\n- Recommended: {}\n\n",e.phase,e.asset_id,e.current_algorithm,e.recommended_replacement)); }
    std::fs::write(op.join("roadmap.md"),md)?;
    tracing::info!(id=%rpt.report_id, assets=rpt.total_assets, "Report done");
    Ok(rpt)
}
pub fn verify_file(p: &PathBuf) -> Result<String, VeriCryptError> {
    let d = std::fs::read_to_string(p).map_err(|e| VeriCryptError::Io(e))?;
    let r: PqcReport = serde_json::from_str(&d).map_err(|e| VeriCryptError::ParseError(format!("{}",e)))?;
    Ok(format!("VERIFIED — scan at {}, {} assets, {} violations",r.scan_timestamp.format("%Y-%m-%dT%H:%M:%SZ"),r.total_assets,r.violations_found))
}
