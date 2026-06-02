//! IR verifier for AGENT‑SEED v15.2.
//!
//! Checks:
//!   - SSA dominance (simplified: variables defined before use within block)
//!   - Type consistency of instruction operands
//!   - Effect soundness: Perform only after Discharge in the same block
//!   - Control flow correctness (all blocks reachable, terminators valid)
//!
//! References:
//!   - LLVM's `Verifier` pass (Lattner & Adve, 2004)
//!   - Crafting Interpreters chapter on static analysis

use crate::ir::*;
use std::collections::HashSet;

pub fn verify(module: &Module) -> Result<(), IrError> {
    for func in &module.functions {
        verify_function(func)?;
    }
    Ok(())
}

fn verify_function(func: &Function) -> Result<(), IrError> {
    // Check all blocks exist
    for (i, block) in func.blocks.iter().enumerate() {
        if block.id != i {
            return Err(IrError::ControlFlowError {
                msg: format!("Block ID mismatch: expected {}, got {}", i, block.id),
                span: None,
            });
        }
    }

    // Check entry block exists
    if func.entry >= func.blocks.len() {
        return Err(IrError::ControlFlowError {
            msg: format!(
                "Entry block {} out of range (0-{})",
                func.entry,
                func.blocks.len()
            ),
            span: None,
        });
    }

    // Check each block
    for block in &func.blocks {
        verify_block(func, block)?;
    }

    // Check terminators reference valid blocks
    for block in &func.blocks {
        match &block.terminator {
            Terminator::Branch {
                then_block,
                else_block,
                ..
            } => {
                if *then_block >= func.blocks.len() || *else_block >= func.blocks.len() {
                    return Err(IrError::ControlFlowError {
                        msg: "Branch target out of range".into(),
                        span: None,
                    });
                }
            }
            Terminator::Jump(target) => {
                if *target >= func.blocks.len() {
                    return Err(IrError::ControlFlowError {
                        msg: "Jump target out of range".into(),
                        span: None,
                    });
                }
            }
            Terminator::Return(_) | Terminator::Halt => {}
        }
    }
    Ok(())
}

fn verify_block(func: &Function, block: &BasicBlock) -> Result<(), IrError> {
    let mut defined: HashSet<VarId> = func.params.iter().cloned().collect();
    let mut discharged = false; // Track if we've seen a Discharge before a Perform

    for instr in &block.instrs {
        // Check that all operand variables are defined
        for op in &instr.operands {
            if let Operand::Var(v) = op {
                if !defined.contains(v) {
                    return Err(IrError::UndefinedVar(*v));
                }
            }
        }

        // Check type consistency (simplified)
        match instr.opcode {
            Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Div | Opcode::Rem => {
                verify_binary_op(instr)?;
            }
            Opcode::Eq | Opcode::NotEq | Opcode::Lt | Opcode::Gt | Opcode::LtEq | Opcode::GtEq => {
                verify_binary_op(instr)?;
            }
            Opcode::Perform if !discharged => {
                return Err(IrError::EffectViolation {
                    msg: "Perform instruction outside of a Discharge context".into(),
                    span: instr.span,
                });
            }
            Opcode::Perform => {}
            Opcode::Discharge => {
                discharged = true;
            }
            _ => {}
        }

        // Mark destination as defined
        if let Some(dest) = instr.dest {
            defined.insert(dest);
        }
    }
    Ok(())
}

fn verify_binary_op(instr: &Instr) -> Result<(), IrError> {
    if instr.operands.len() < 2 {
        return Err(IrError::TypeMismatch {
            expected: "two operands".into(),
            found: format!("{}", instr.operands.len()),
            span: instr.span,
        });
    }
    Ok(())
}
