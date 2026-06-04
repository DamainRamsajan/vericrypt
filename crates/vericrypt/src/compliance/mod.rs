pub mod asl_runtime;

use crate::errors::VeriCryptError;
use crate::graph::CryptoGraph;
use crate::types::ComplianceTheorem;
use asl_runtime::AslRuntime;

pub fn prove_compliance(graph: &CryptoGraph) -> Result<Vec<ComplianceTheorem>, VeriCryptError> {
    let runtime = AslRuntime::new();
    let inventory_hash = compute_inventory_hash(graph);
    let results = runtime.execute_all(&inventory_hash)?;

    let theorems: Vec<ComplianceTheorem> = results
        .into_iter()
        .map(|(vm_state, mut theorem)| {
            theorem.asl_statement = format!(
                "ASL VM: {} steps, exit_code={}",
                vm_state.schedule_trace.len(),
                vm_state.exit_code
            );
            theorem
        })
        .collect();

    tracing::info!(
        frameworks = runtime.available_frameworks().len(),
        theorems = theorems.len(),
        "ASL VM compliance verification complete"
    );

    Ok(theorems)
}

fn compute_inventory_hash(graph: &CryptoGraph) -> Vec<u8> {
    let mut hasher = blake3::Hasher::new();
    for asset in graph.get_all_assets() {
        hasher.update(asset.fingerprint.as_bytes());
        hasher.update(asset.algorithm.name.as_bytes());
    }
    hasher.finalize().as_bytes().to_vec()
}
