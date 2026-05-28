use crate::errors::VeriCryptError;
use crate::graph::CryptoGraph;
use crate::types::{ExposureResult, ComplianceTheorem};

/// A migration roadmap entry.
#[derive(Debug, Clone, serde::Serialize)]
pub struct MigrationPhase {
    pub phase: u32,
    pub asset_id: uuid::Uuid,
    pub current_algorithm: String,
    pub recommended_replacement: String,
    pub regulatory_reference: String,
    pub estimated_complexity: String,
}

/// Generate a risk-prioritized migration roadmap.
///
/// Pre-conditions:
/// - exposure_result contains Shapley values
/// - graph provides dependency structure
///
/// Post-conditions:
/// - Returns Vec<MigrationPhase> with Phase 1/2/3 assignments
pub fn generate_roadmap(
    exposure_result: &ExposureResult,
    graph: &CryptoGraph,
) -> Result<Vec<MigrationPhase>, VeriCryptError> {
    // Sort assets by Shapley value (highest first)
    let mut entries: Vec<(uuid::Uuid, f64)> = exposure_result
        .shapley_values
        .iter()
        .map(|(k, v)| (*k, *v))
        .collect();
    entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let total = entries.len();
    let phase1_cutoff = total / 3;
    let phase2_cutoff = 2 * total / 3;

    let roadmap: Vec<MigrationPhase> = entries
        .iter()
        .enumerate()
        .map(|(i, (asset_id, shapley))| {
            let phase = if i < phase1_cutoff {
                1
            } else if i < phase2_cutoff {
                2
            } else {
                3
            };

            MigrationPhase {
                phase,
                asset_id: *asset_id,
                current_algorithm: "To be classified".into(),
                recommended_replacement: "ML-DSA (NIST FIPS 204)".into(),
                regulatory_reference: format!("DORA Art. 12.3; PQFIF Phase {}", phase),
                estimated_complexity: match phase {
                    1 => "High priority — remediate within 12 months".into(),
                    2 => "Medium priority — remediate within 24 months".into(),
                    _ => "Standard priority — remediate within 36 months".into(),
                },
            }
        })
        .collect();

    tracing::info!(phases = roadmap.len(), "Migration roadmap generated");
    Ok(roadmap)
}
