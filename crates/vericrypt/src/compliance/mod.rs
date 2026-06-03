pub mod asl_runtime;

use crate::errors::VeriCryptError;
use crate::graph::CryptoGraph;
use crate::types::ComplianceTheorem;
use asl_runtime::AslRuntime;

/// Prove regulatory compliance using the ASL Virtual Machine.
///
/// Executes compiled regulatory bytecode against the cryptographic inventory.
/// Produces verifiable execution evidence (schedule trace, ProofMeta).
pub fn prove_compliance(graph: &CryptoGraph) -> Result<Vec<ComplianceTheorem>, VeriCryptError> {
    let runtime = AslRuntime::new();

    // Compute deterministic inventory hash for VM seed
    let inventory_hash = compute_inventory_hash(graph);

    let results = runtime.execute_all(&inventory_hash)?;

    let theorems: Vec<ComplianceTheorem> = results
        .into_iter()
        .map(|(vm_state, mut theorem)| {
            // Store VM state reference in theorem metadata
            theorem.asl_statement = format!(
                "ASL VM: {} instructions, proof verified: {}",
                vm_state.schedule_trace_len(),
                vm_state.proof_verified()
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

/// Compute a deterministic hash of the cryptographic inventory for VM seeding.
fn compute_inventory_hash(graph: &CryptoGraph) -> Vec<u8> {
    let mut hasher = blake3::Hasher::new();
    for asset in graph.get_all_assets() {
        hasher.update(asset.fingerprint.as_bytes());
        hasher.update(asset.algorithm.name.as_bytes());
    }
    hasher.finalize().as_bytes().to_vec()
}
