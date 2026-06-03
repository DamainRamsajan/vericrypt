use std::path::PathBuf;
use crate::types::ComplianceTheorem;
use crate::errors::VeriCryptError;

/// Write a human-readable violations file for immediate CISO action.
///
/// Each violation includes: regulatory article, affected asset, and remediation path.
pub fn write_violations(
    output_dir: &PathBuf,
    theorems: &[ComplianceTheorem],
) -> Result<(), VeriCryptError> {
    let violations_path = output_dir.join("violations.txt");

    let violations: Vec<&ComplianceTheorem> = theorems
        .iter()
        .filter(|t| t.status == crate::types::ProofStatus::Counterexample)
        .collect();

    if violations.is_empty() {
        return Ok(());
    }

    let mut content = String::from("VERICRYPT COMPLIANCE VIOLATIONS\n");
    content.push_str("================================\n\n");
    content.push_str("The following compliance violations were detected during the scan.\n");
    content.push_str("Each violation includes the specific regulatory article, the affected asset,\n");
    content.push_str("and a recommended remediation path.\n\n");

    for theorem in violations {
        content.push_str(&format!(
            "VIOLATION: {}\n  Regulation: {}\n  Asset ID: {}\n  Remediation: {}\n\n",
            theorem.asl_statement,
            theorem.regulation_reference,
            theorem.counterexample_asset_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            theorem.remediation_recommendation
                .as_deref()
                .unwrap_or("No remediation recommendation available"),
        ));
    }

    std::fs::write(&violations_path, content)
        .map_err(|e| VeriCryptError::Io(e))
}
