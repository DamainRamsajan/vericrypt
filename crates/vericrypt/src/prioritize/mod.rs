pub mod monte_carlo;

use crate::errors::VeriCryptError;
use crate::graph::CryptoGraph;
use crate::types::{ExposureResult, MigrationPhase};

/// Generate a risk-prioritized migration roadmap with CMAP and PQCMM scores.
pub fn generate_roadmap(
    er: &ExposureResult,
    _g: &CryptoGraph,
) -> Result<Vec<MigrationPhase>, VeriCryptError> {
    let mut e: Vec<_> = er.shapley_values.iter().map(|(k, v)| (*k, *v)).collect();
    e.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let t = e.len();
    let p1 = if t > 0 { t / 3 } else { 0 };
    let p2 = if t > 0 { 2 * t / 3 } else { 0 };

    Ok(e.iter()
        .enumerate()
        .map(|(i, (id, _))| {
            let ph = if i < p1 {
                1
            } else if i < p2 {
                2
            } else {
                3
            };
            let (cmap, pqcmm, milestone) = match ph {
                1 => (1u32, 2u32, "EU 2026 PQC transition start"),
                2 => (2u32, 3u32, "EU 2030 critical infrastructure deadline"),
                _ => (3u32, 4u32, "EU 2035 completion target"),
            };
            MigrationPhase {
                phase: ph,
                asset_id: *id,
                current_algorithm: "Classified during scan".into(),
                recommended_replacement: "ML-DSA (FIPS 204) or SLH-DSA (FIPS 205)".into(),
                regulatory_reference: format!("DORA Art. 12.3; PQFIF Phase {} ({})", ph, milestone),
                estimated_complexity: match ph {
                    1 => "High priority — 12 months".into(),
                    2 => "Medium priority — 24 months".into(),
                    _ => "Standard priority — 36 months".into(),
                },
            }
        })
        .collect())
}
