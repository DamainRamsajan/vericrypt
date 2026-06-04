pub mod temporal;

use crate::errors::VeriCryptError;
use crate::graph::CryptoGraph;
use crate::types::{ExposureBreakdown, ExposureResult, ShapleyApproximationMetadata};
use std::collections::HashMap;

/// Analyze quantum exposure using the multiplicative HNDL model
/// with per-asset temporal hazard via Ld > Ha condition.
pub fn analyze(g: &CryptoGraph) -> Result<ExposureResult, VeriCryptError> {
    let n = g.node_count();
    if n == 0 {
        return Ok(ExposureResult {
            total_hndl_exposure: 0.0,
            per_asset_exposure: HashMap::new(),
            shapley_values: HashMap::new(),
            breakdown: ExposureBreakdown {
                temporal_hazard: 0.0,
                crypto_vulnerability: 0.0,
                operational_exposure: 0.0,
                defense_attack_ratio: 1.0,
            },
            shapley_metadata: Some(ShapleyApproximationMetadata {
                samples: 0,
                convergence_error: 0.0,
                confidence_interval: 0.0,
                converged: true,
                convergence_threshold: 0.01,
            }),
        });
    }

    let attacker_horizon = temporal::default_attacker_horizon();
    let defense_attack_ratio = 1.0;
    let mut per_asset = HashMap::new();
    let mut total_vulnerability_exposure = 0.0;

    for a in g.get_all_assets() {
        let temporal_hazard = temporal::compute_temporal_hazard(a, attacker_horizon);
        let vuln = if a.algorithm.quantum_vulnerable {
            temporal_hazard * 1.0
        } else {
            0.0
        };
        per_asset.insert(a.asset_id, vuln);
        total_vulnerability_exposure += vuln;
    }

    let total_hndl_exposure = total_vulnerability_exposure / (1.0 + defense_attack_ratio);
    let shapley_values = g.compute_shapley_values();

    Ok(ExposureResult {
        total_hndl_exposure,
        per_asset_exposure: per_asset,
        shapley_values,
        breakdown: ExposureBreakdown {
            temporal_hazard: 1.0,
            crypto_vulnerability: total_vulnerability_exposure,
            operational_exposure: 1.0,
            defense_attack_ratio,
        },
        shapley_metadata: Some(ShapleyApproximationMetadata {
            samples: 0,
            convergence_error: 0.0,
            confidence_interval: 0.0,
            converged: true,
            convergence_threshold: 0.01,
        }),
    })
}
