use seedvm::{run_bytes, VMState};
use crate::errors::VeriCryptError;
use crate::types::{ComplianceTheorem, ProofStatus};
use std::collections::HashMap;

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/embedded_axioms.rs"));
}

pub struct AslRuntime {
    bytecode: HashMap<String, Vec<u8>>,
}

impl AslRuntime {
    pub fn new() -> Self {
        AslRuntime {
            bytecode: embedded::get_embedded_bytecode(),
        }
    }

    pub fn execute_framework(
        &self,
        framework: &str,
        inventory_hash: &[u8],
    ) -> Result<(VMState, ComplianceTheorem), VeriCryptError> {
        let bytecode = self.bytecode.get(framework)
            .ok_or_else(|| VeriCryptError::ParseError(
                format!("No bytecode found for framework: {}", framework)
            ))?;

        let seed = u64::from_le_bytes(
            inventory_hash[..8].try_into()
                .map_err(|_| VeriCryptError::ParseError("Invalid inventory hash length".into()))?
        );

        let vm_state = run_bytes(bytecode, seed)
            .map_err(|e| VeriCryptError::ParseError(
                format!("ASL VM execution failed: {}", e)
            ))?;

        let status = if vm_state.exit_code == 0 {
            ProofStatus::Proved
        } else {
            ProofStatus::Counterexample
        };

        let theorem = ComplianceTheorem {
            theorem_id: uuid::Uuid::new_v4(),
            regulation_reference: framework.to_string(),
            asl_statement: format!("ASL VM execution: {} steps, exit_code={}", 
                vm_state.schedule_trace.len(), vm_state.exit_code),
            status,
            counterexample_asset_id: None,
            remediation_recommendation: if vm_state.exit_code != 0 {
                Some("Review ASL VM execution trace for failed constraints".into())
            } else {
                None
            },
        };

        Ok((vm_state, theorem))
    }

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

    pub fn has_framework(&self, framework: &str) -> bool {
        self.bytecode.contains_key(framework)
    }

    pub fn available_frameworks(&self) -> Vec<&String> {
        self.bytecode.keys().collect()
    }
}
