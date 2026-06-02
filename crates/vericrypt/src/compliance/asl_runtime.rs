use seedvm::{run_bytes, VMState};
use crate::errors::VeriCryptError;
use crate::types::{ComplianceTheorem, ProofStatus};
use std::collections::HashMap;

/// ASL Virtual Machine runtime for compliance verification.
///
/// Replaces the Lean 4 bridge (ADR-021). Executes compiled ASL bytecode
/// against the cryptographic inventory and produces verifiable execution evidence.
pub struct AslRuntime {
    bytecode: HashMap<String, Vec<u8>>,
}

impl AslRuntime {
    /// Create a new ASL runtime with embedded bytecode.
    /// Bytecode is compiled at build time from the regulatory axiom library.
    pub fn new() -> Self {
        AslRuntime {
            bytecode: include!(concat!(env!("OUT_DIR"), "/embedded_axioms.rs")).get_embedded_bytecode(),
        }
    }

    /// Execute a regulatory framework's bytecode against the inventory.
    ///
    /// Returns a VMState containing the schedule trace and ProofMeta.
    /// The seed is derived deterministically from the inventory hash for
    /// bit-identical reproducibility.
    pub fn execute_framework(
        &self,
        framework: &str,
        inventory_hash: &[u8],
    ) -> Result<(VMState, ComplianceTheorem), VeriCryptError> {
        let bytecode = self.bytecode.get(framework)
            .ok_or_else(|| VeriCryptError::ParseError(
                format!("No bytecode found for framework: {}", framework)
            ))?;

        // Derive deterministic seed from inventory hash
        let seed = u64::from_le_bytes(
            inventory_hash[..8].try_into()
                .map_err(|_| VeriCryptError::ParseError("Invalid inventory hash length".into()))?
        );

        let vm_state = run_bytes(bytecode, seed)
            .map_err(|e| VeriCryptError::ParseError(
                format!("ASL VM execution failed: {}", e)
            ))?;

        let status = if vm_state.proof_verified() {
            ProofStatus::Proved
        } else {
            ProofStatus::Counterexample
        };

        let theorem = ComplianceTheorem {
            theorem_id: uuid::Uuid::new_v4(),
            regulation_reference: framework.to_string(),
            lean4_statement: format!("ASL VM execution: {} instructions traced", vm_state.schedule_trace_len()),
            status,
            counterexample_asset_id: None,
            remediation_recommendation: if !vm_state.proof_verified() {
                Some("Review ASL VM execution trace for failed constraints".into())
            } else {
                None
            },
        };

        Ok((vm_state, theorem))
    }

    /// Execute all embedded regulatory frameworks.
    pub fn execute_all(
        &self,
        inventory_hash: &[u8],
    ) -> Result<Vec<(VMState, ComplianceTheorem)>, VeriCryptError> {
        let mut results = Vec::new();

        for framework in self.bytecode.keys() {
            match self.execute_framework(framework, inventory_hash) {
                Ok(result) => results.push(result),
                Err(e) => {
                    tracing::warn!(framework = %framework, error = %e, "Framework execution failed");
                }
            }
        }

        Ok(results)
    }

    /// Check if bytecode is available for a specific framework.
    pub fn has_framework(&self, framework: &str) -> bool {
        self.bytecode.containsKey(framework)
    }

    /// List all available regulatory frameworks.
    pub fn available_frameworks(&self) -> Vec<&String> {
        self.bytecode.keys().collect()
    }
}
