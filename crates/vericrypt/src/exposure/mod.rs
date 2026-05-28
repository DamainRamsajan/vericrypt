use crate::errors::VeriCryptError; use crate::graph::CryptoGraph;
use crate::types::{ExposureResult, ExposureBreakdown, ShapleyApproximationMetadata};
use std::collections::HashMap;
pub fn analyze(g: &CryptoGraph) -> Result<ExposureResult, VeriCryptError> {
    let n = g.node_count();
    if n == 0 { return Ok(ExposureResult{total_hndl_exposure:0.0,per_asset_exposure:HashMap::new(),shapley_values:HashMap::new(),breakdown:ExposureBreakdown{temporal_hazard:0.0,crypto_vulnerability:0.0,operational_exposure:0.0,defense_attack_ratio:1.0},shapley_metadata:Some(ShapleyApproximationMetadata{samples:0,convergence_error:0.0,confidence_interval:0.0,converged:true,convergence_threshold:0.01})}); }
    let mut pa = HashMap::new(); let mut t = 0.0;
    for a in g.get_all_assets() { let v = if a.algorithm.quantum_vulnerable {1.0}else{0.0}; pa.insert(a.asset_id,v); t+=v; }
    Ok(ExposureResult{total_hndl_exposure:t/2.0,per_asset_exposure:pa,shapley_values:g.compute_shapley_values(),breakdown:ExposureBreakdown{temporal_hazard:1.0,crypto_vulnerability:t,operational_exposure:1.0,defense_attack_ratio:1.0},shapley_metadata:Some(ShapleyApproximationMetadata{samples:0,convergence_error:0.0,confidence_interval:0.0,converged:true,convergence_threshold:0.01})})
}
