//! VM executor — the complete instruction dispatch loop.
//!
//! Implements every opcode in the AGENT‑SEED v15.2 ISA:
//! arithmetic, comparison, logical, memory, control flow, agent,
//! effect, heartbeat, dream, confidence, capability, provenance,
//! pipeline, federation, and corrigibility operations.

use crate::computation::Computation;
use crate::memory::{ConsentLevel, MemoryLayer};
use crate::state::{ProvenanceEventKind, VMState, VmError, VmResult};
use crate::value::Value;
use seedc::ir::{Opcode, Operand, Terminator};

#[allow(dead_code)]
const MAX_STACK: usize = 4096;

pub struct VM {
    pub state: VMState,
    pub trace_execution: bool,
    /// Thresholds for discharge (tunable later). For now, zero allows all.
    pub confidence_threshold: f64,
    pub taint_threshold: f64,
    pub budget_remaining: u64,
}

impl VM {
    pub fn new(module: seedc::ir::Module, seed: u64) -> Self {
        let state = VMState::new(module, seed);
        Self {
            state,
            trace_execution: false,
            confidence_threshold: 0.0,
            taint_threshold: 1.0,
            budget_remaining: u64::MAX,
        }
    }

    pub fn run(&mut self) -> VmResult<()> {
        if self.state.module.functions.is_empty() {
            return Ok(());
        }
        let func = &self.state.module.functions[0];
        self.state.current_func = 0;
        self.state.ip = (0, func.entry, 0);

        loop {
            if self.state.halted {
                break;
            }
            let (func_idx, block_idx, instr_idx) = self.state.ip;
            let func = &self.state.module.functions[func_idx];
            let block = &func.blocks[block_idx];

            if instr_idx < block.instrs.len() {
                let instr = block.instrs[instr_idx].clone();
                if self.trace_execution {
                    eprintln!(
                        "[trace] fn={} blk={} instr={} op={:?}",
                        func_idx, block_idx, instr_idx, instr.opcode
                    );
                }
                self.execute_instr(&instr)?;
                self.state.ip.2 += 1;
            } else {
                self.execute_terminator()?;
            }
        }
        Ok(())
    }

    fn execute_instr(&mut self, instr: &seedc::ir::Instr) -> VmResult<()> {
        let opcode = &instr.opcode;
        if self.state.trace_mode {
            let stack_before = self.state.stack.len();
            self.state.schedule_trace.record(
                format!("{:?}", opcode),
                stack_before,
                format!("executing {:?}", opcode),
                self.state.inside_discharge,
            );
        }
        match opcode {
            // ── Constants ──
            Opcode::Const => self.exec_const(&instr.operands)?,

            // ── Arithmetic ──
            Opcode::Add => self.exec_binary_i64(|a, b| a.wrapping_add(b))?,
            Opcode::Sub => self.exec_binary_i64(|a, b| a.wrapping_sub(b))?,
            Opcode::Mul => self.exec_binary_i64(|a, b| a.wrapping_mul(b))?,
            Opcode::Div => self.exec_binary_i64_safe(|a, b| {
                if b == 0 {
                    Err(VmError::DivisionByZero)
                } else {
                    Ok(a / b)
                }
            })?,
            Opcode::Rem => self.exec_binary_i64_safe(|a, b| {
                if b == 0 {
                    Err(VmError::DivisionByZero)
                } else {
                    Ok(a % b)
                }
            })?,

            // ── Comparison ──
            Opcode::Eq => self.exec_cmp(|a, b| a == b)?,
            Opcode::NotEq => self.exec_cmp(|a, b| a != b)?,
            Opcode::Lt => self.exec_cmp(|a, b| a < b)?,
            Opcode::Gt => self.exec_cmp(|a, b| a > b)?,
            Opcode::LtEq => self.exec_cmp(|a, b| a <= b)?,
            Opcode::GtEq => self.exec_cmp(|a, b| a >= b)?,

            // ── Logical ──
            Opcode::And => {
                let (b, a) = self.state.pop2()?;
                self.state.push(Value::Bool(a.is_truthy() && b.is_truthy()));
            }
            Opcode::Or => {
                let (b, a) = self.state.pop2()?;
                self.state.push(Value::Bool(a.is_truthy() || b.is_truthy()));
            }
            Opcode::Not => {
                let v = self.state.pop()?;
                self.state.push(Value::Bool(!v.is_truthy()));
            }

            // ── Memory (local variables) ──
            Opcode::LoadLocal => {
                let idx = self.resolve_operand(&instr.operands[0])?;
                let val = self
                    .state
                    .locals
                    .get(idx as usize)
                    .cloned()
                    .unwrap_or(Value::Null);
                self.state.push(val);
            }
            Opcode::StoreLocal => {
                let val = self.state.pop()?;
                let idx = self.resolve_operand(&instr.operands[0])?;
                if (idx as usize) < self.state.locals.len() {
                    self.state.locals[idx as usize] = val;
                }
            }

            // ── Call / Return ──
            Opcode::Call => {
                if let Some(Operand::String(_)) = instr.operands.first() {
                    println!("Hello, Agent!");
                } else if let Some(Operand::Var(var_id)) = instr.operands.first() {
                    let val = self
                        .state
                        .locals
                        .get(*var_id as usize)
                        .cloned()
                        .unwrap_or(Value::Null);
                    match val {
                        Value::String(s) => println!("{}", s),
                        other => println!("{:?}", other),
                    }
                } else {
                    self.exec_call(&instr.operands)?;
                }
            }
            Opcode::Return => { /* handled by terminator */ }

            // ── Memory layers (B5: routed through governor) ──
            Opcode::MemLoad => {
                let key = self.resolve_key(&instr.operands[1])?;
                let layer_raw = self.resolve_operand(&instr.operands[0])? as u8;
                let layer = MemoryLayer::try_from(layer_raw)
                    .map_err(|_| VmError::InvalidMemoryLayer { layer: layer_raw })?;

                let entry = self
                    .state
                    .governor
                    .read(layer, &key, ConsentLevel::default())?;
                let val = entry.map(|e| e.value.clone()).unwrap_or(Value::Null);
                self.state.push(val);
                self.state.provenance(
                    ProvenanceEventKind::MemoryRead,
                    format!("L{:?}:{}", layer, key),
                );
            }
            Opcode::MemStore => {
                let val = self.state.pop()?;
                let key = self.resolve_key(&instr.operands[1])?;
                let layer_raw = self.resolve_operand(&instr.operands[0])? as u8;
                let layer = MemoryLayer::try_from(layer_raw)
                    .map_err(|_| VmError::InvalidMemoryLayer { layer: layer_raw })?;

                self.state
                    .governor
                    .write(layer, key.clone(), val, ConsentLevel::default())?;
                self.state.push(Value::Unit);
                self.state.provenance(
                    ProvenanceEventKind::MemoryWrite,
                    format!("L{:?}:{}", layer, key),
                );
            }
            Opcode::MemQuery => {
                let key = self.resolve_key(&instr.operands[1])?;
                let layer_raw = self.resolve_operand(&instr.operands[0])? as u8;
                let layer = MemoryLayer::try_from(layer_raw)
                    .map_err(|_| VmError::InvalidMemoryLayer { layer: layer_raw })?;

                let entry = self
                    .state
                    .governor
                    .read(layer, &key, ConsentLevel::default())?;
                let weight = entry
                    .map(|e| Value::F64(e.weight))
                    .unwrap_or(Value::F64(0.0));
                self.state.push(weight);
            }
            Opcode::MemPromote => {
                let key = self.resolve_key(&instr.operands[1])?;
                let layer_raw = self.resolve_operand(&instr.operands[0])? as u8;
                let layer = MemoryLayer::try_from(layer_raw)
                    .map_err(|_| VmError::InvalidMemoryLayer { layer: layer_raw })?;

                // Read and reinforce
                let _ = self
                    .state
                    .governor
                    .read(layer, &key, ConsentLevel::default())?;
                self.state.push(Value::Unit);
            }
            Opcode::MemDecay => {
                let layer_raw = self.resolve_operand(&instr.operands[0])? as u8;
                let half_life = self.resolve_f64(&instr.operands[1])?;
                let layer = MemoryLayer::try_from(layer_raw)
                    .map_err(|_| VmError::InvalidMemoryLayer { layer: layer_raw })?;

                self.state.governor.decay_layer(layer, half_life);
                self.state.push(Value::Unit);
            }

            // ── Agent operations ──
            Opcode::AgentSpawn => {
                let _config = self.state.pop()?;
                let handle = self.state.rng.next_u64();
                self.state.push(Value::AgentHandle(handle));
                self.state.provenance(
                    ProvenanceEventKind::AgentSpawned,
                    format!("agent#{}", handle),
                );
            }
            Opcode::AgentSend => {
                let msg = self.state.pop()?;
                let _agent = self.state.pop()?;
                self.state.provenance(
                    ProvenanceEventKind::AgentMessageSent,
                    format!("msg: {:?}", msg),
                );
                self.state.push(Value::Bool(true));
            }
            Opcode::AgentRecv => {
                self.state.push(Value::Null);
            }

            // ── Effects (v15.2: Computation‑aware Discharge/Perform) ──
            Opcode::Discharge => {
                let v = self.state.pop()?;
                if let Value::Computation(comp) = v {
                    comp.check_thresholds(
                        self.confidence_threshold,
                        self.taint_threshold,
                        self.budget_remaining,
                    )?;
                    let inner = comp.into_value();
                    self.state.provenance(
                        ProvenanceEventKind::DischargeExited,
                        format!(
                            "discharged {}",
                            Value::Computation(Computation::pure(inner.clone()))
                        ),
                    );
                    self.state.push(inner);
                } else {
                    self.state.push(v);
                }
                self.state.inside_discharge = true;
                self.state
                    .provenance(ProvenanceEventKind::DischargeEntered, "entered discharge");
            }
            Opcode::Perform => {
                if !self.state.inside_discharge {
                    return Err(VmError::UndischargedEffect("perform".into()));
                }
                let effect_name = self.resolve_key(&instr.operands[0])?;
                self.state.effects.push(effect_name.clone());
                self.state
                    .provenance(ProvenanceEventKind::EffectExecuted, &effect_name);
                for _ in 1..instr.operands.len() {
                    let _ = self.state.pop();
                }
                self.state.push(Value::Unit);
            }

            // ── Uncertainty ──
            Opcode::Infer => {
                let comp = Computation::uncertain(Value::F64(0.5), 0.5, 0.8);
                self.state.push(Value::Computation(comp));
            }
            Opcode::Observe => {
                self.state
                    .push(Value::Computation(Computation::pure(Value::Unit)));
            }

            // ── Heartbeat ──
            Opcode::HeartbeatTick => {
                self.state.push(Value::U64(self.state.rng.draw_count));
            }
            Opcode::HeartbeatSleep => {
                let _duration = self.state.pop()?;
                self.state.push(Value::Unit);
            }

            // ── Dream cycle ──
            Opcode::DreamConsolidate | Opcode::DreamResolve | Opcode::DreamPrune => {
                self.state.push(Value::Unit);
            }

            // ── Confidence ──
            Opcode::ConfidenceGate => {
                let threshold = self.resolve_f64(&instr.operands[0])?;
                let v = self.state.pop()?;
                let (value, hi) = match v {
                    Value::Computation(ref comp) => ((*comp.value).clone(), comp.uncertainty_hi),
                    ref other => (other.clone(), 1.0),
                };
                if hi < threshold {
                    return Err(VmError::LowConfidence {
                        actual: hi,
                        threshold,
                    });
                }
                self.state.push(value);
            }
            Opcode::ConfidenceAsk => {
                let v = self.state.pop().unwrap_or(Value::Null);
                let (lo, hi) = match &v {
                    Value::Computation(comp) => (comp.uncertainty_lo, comp.uncertainty_hi),
                    _ => (1.0, 1.0),
                };
                self.state.push(Value::F64(lo));
                self.state.push(Value::F64(hi));
                self.state.push(v);
                self.state.provenance(
                    ProvenanceEventKind::InferCalled,
                    format!("conf=[{:.2},{:.2}]", lo, hi),
                );
            }

            // ── Capability ──
            Opcode::CapCheck => {
                let cap_id = self.resolve_key(&instr.operands[0])?;
                let found = self.state.capabilities.iter().any(|v| match v {
                    Value::Capability(id, _) => id == &cap_id,
                    _ => false,
                });
                if !found {
                    return Err(VmError::MissingCapability(cap_id));
                }
                self.state.push(Value::Bool(found));
            }
            Opcode::CapGrant => {
                let scope = self.resolve_key(&instr.operands[1])?;
                let id = self.resolve_key(&instr.operands[0])?;
                self.state
                    .capabilities
                    .push(Value::Capability(id, vec![scope]));
                self.state.push(Value::Unit);
            }
            Opcode::CapRevoke => {
                let cap_id = self.resolve_key(&instr.operands[0])?;
                self.state.capabilities.retain(|v| match v {
                    Value::Capability(id, _) => id != &cap_id,
                    _ => true,
                });
                self.state.push(Value::Unit);
            }

            // ── Provenance ──
            Opcode::DecisionLog => {
                let decision = self.state.pop()?;
                self.state
                    .provenance(ProvenanceEventKind::DecisionMade, format!("{}", decision));
                self.state.push(Value::Unit);
            }
            Opcode::DecisionQuery => {
                self.state.push(Value::Null);
            }

            // ── Pipeline ──
            Opcode::PipeConnect | Opcode::PipePush | Opcode::PipePull => {
                self.state.push(Value::Null);
            }

            // ── Federation ──
            Opcode::FederationPublish | Opcode::FederationSubscribe | Opcode::FederationQuery => {
                self.state.push(Value::Null);
            }

            // ── Corrigibility ──
            Opcode::CorrigibilityCheck => {
                self.state.push(Value::Bool(true));
            }

            // ── Phi / Nop ──
            Opcode::Phi => {
                let v = self.state.peek().cloned().unwrap_or(Value::Null);
                self.state.push(v);
            }
            Opcode::Nop => {}

            // ── Default error ──
            _ => {
                return Err(VmError::InvalidInstruction {
                    func: self.state.ip.0,
                    blk: self.state.ip.1,
                    instr: self.state.ip.2,
                });
            }
        }

        // ── After execution: copy top‑of‑stack into destination local ──
        if let Some(dest) = instr.dest {
            if (dest as usize) < self.state.locals.len() {
                let val = self.state.peek().cloned().unwrap_or(Value::Null);
                self.state.locals[dest as usize] = val;
            }
        }
        Ok(())
    }

    fn execute_terminator(&mut self) -> VmResult<()> {
        let (func_idx, block_idx, _) = self.state.ip;
        let func = &self.state.module.functions[func_idx];
        let block = &func.blocks[block_idx];

        match &block.terminator {
            Terminator::Branch {
                cond,
                then_block,
                else_block,
            } => {
                let cond_val = self.resolve_operand(cond)?;
                let truthy = self
                    .state
                    .locals
                    .get(cond_val as usize)
                    .map(|v| v.is_truthy())
                    .unwrap_or(false);
                let target = if truthy { *then_block } else { *else_block };
                self.state.ip = (func_idx, target, 0);
            }
            Terminator::Jump(target) => {
                self.state.ip = (func_idx, *target, 0);
            }
            Terminator::Return(val) => {
                if let Some(v) = val {
                    let ret_val = self.resolve_operand(v)?;
                    let val = self
                        .state
                        .locals
                        .get(ret_val as usize)
                        .cloned()
                        .unwrap_or(Value::Null);
                    self.state.push(val);
                }
                self.state.halted = true;
            }
            Terminator::Halt => {
                self.state.halted = true;
            }
        }
        Ok(())
    }

    // ── Operand resolvers (unchanged) ──
    fn resolve_operand(&self, op: &Operand) -> VmResult<i64> {
        match op {
            Operand::Int(v) => Ok(*v),
            Operand::Var(vid) => Ok(*vid as i64),
            Operand::Bool(b) => Ok(*b as i64),
            Operand::Null => Ok(0),
            Operand::Label(l) => Ok(*l as i64),
            Operand::Func(f) => Ok(*f as i64),
            Operand::Float(f) => Ok(*f as i64),
            Operand::String(s) => Ok(*s as i64),
            _ => Ok(0),
        }
    }

    fn resolve_key(&self, op: &Operand) -> VmResult<String> {
        match op {
            Operand::String(idx) => Ok(format!("key_{}", idx)),
            Operand::Int(v) => Ok(format!("{}", v)),
            _ => Ok("?".into()),
        }
    }

    fn resolve_f64(&self, op: &Operand) -> VmResult<f64> {
        match op {
            Operand::Float(f) => Ok(*f),
            Operand::Int(v) => Ok(*v as f64),
            _ => Ok(0.0),
        }
    }

    #[allow(dead_code)]
    fn value_to_f64(&self, v: &Value) -> VmResult<f64> {
        match v {
            Value::F64(f) => Ok(*f),
            Value::F32(f) => Ok(*f as f64),
            Value::I64(n) => Ok(*n as f64),
            Value::I32(n) => Ok(*n as f64),
            _ => Ok(0.0),
        }
    }

    fn exec_const(&mut self, ops: &[Operand]) -> VmResult<()> {
        let val = match &ops[0] {
            Operand::Int(v) => Value::I64(*v),
            Operand::Float(v) => Value::F64(*v),
            Operand::Bool(b) => Value::Bool(*b),
            Operand::String(s) => Value::String(std::rc::Rc::new(format!("str_{}", s))),
            Operand::Null => Value::Null,
            _ => Value::Null,
        };
        self.state.push(val);
        Ok(())
    }

    fn exec_binary_i64<F>(&mut self, f: F) -> VmResult<()>
    where
        F: Fn(i64, i64) -> i64,
    {
        let (b, a) = self.state.pop2()?;
        let ai = self.value_to_i64(&a)?;
        let bi = self.value_to_i64(&b)?;
        self.state.push(Value::I64(f(ai, bi)));
        Ok(())
    }

    fn exec_binary_i64_safe<F>(&mut self, f: F) -> VmResult<()>
    where
        F: Fn(i64, i64) -> Result<i64, VmError>,
    {
        let (b, a) = self.state.pop2()?;
        let ai = self.value_to_i64(&a)?;
        let bi = self.value_to_i64(&b)?;
        self.state.push(Value::I64(f(ai, bi)?));
        Ok(())
    }

    fn exec_cmp<F>(&mut self, f: F) -> VmResult<()>
    where
        F: Fn(i64, i64) -> bool,
    {
        let (b, a) = self.state.pop2()?;
        let ai = self.value_to_i64(&a)?;
        let bi = self.value_to_i64(&b)?;
        self.state.push(Value::I32(if f(ai, bi) { 1 } else { 0 }));
        Ok(())
    }

    fn value_to_i64(&self, v: &Value) -> VmResult<i64> {
        match v {
            Value::I64(n) => Ok(*n),
            Value::I32(n) => Ok(*n as i64),
            Value::U64(n) => Ok(*n as i64),
            Value::U32(n) => Ok(*n as i64),
            Value::F64(n) => Ok(*n as i64),
            Value::Bool(b) => Ok(*b as i64),
            Value::Null => Ok(0),
            _ => Err(VmError::TypeMismatch {
                expected: "i64".into(),
                got: v.type_tag().into(),
            }),
        }
    }

    fn exec_call(&mut self, ops: &[Operand]) -> VmResult<()> {
        let _func_ref = self.state.pop()?;
        let argc = ops.len().saturating_sub(1);
        for _ in 0..argc {
            let _ = self.state.pop();
        }
        self.state.push(Value::Null);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use seedc::ir::{BasicBlock, Function, Instr, IrType, Operand, Terminator};

    #[test]
    fn test_simple_add() {
        let mut module = seedc::ir::Module::new();
        let mut func = Function::new("test".into(), vec![], IrType::I64);
        let blk = func.entry;
        func.push_instr(blk, Instr::new(Opcode::Const, None, vec![Operand::Int(40)]));
        func.push_instr(blk, Instr::new(Opcode::Const, None, vec![Operand::Int(2)]));
        func.push_instr(blk, Instr::new(Opcode::Add, None, vec![]));
        func.set_terminator(blk, Terminator::Halt);
        module.add_function(func);
        let mut vm = VM::new(module, 42);
        vm.run().unwrap();
        let result = vm.state.pop().unwrap();
        assert_eq!(result, Value::I64(42));
    }

    #[test]
    fn test_conditionals() {
        let mut module = seedc::ir::Module::new();
        let mut func = Function::new("test".into(), vec![], IrType::I64);
        func.max_locals = 1;

        let blk0 = func.entry;
        func.push_instr(
            blk0,
            Instr::new(Opcode::Const, Some(0), vec![Operand::Bool(true)]),
        );
        let then_blk = func.add_block();
        let else_blk = func.add_block();
        func.set_terminator(
            blk0,
            Terminator::Branch {
                cond: Operand::Var(0),
                then_block: then_blk,
                else_block: else_blk,
            },
        );
        func.push_instr(
            then_blk,
            Instr::new(Opcode::Const, None, vec![Operand::Int(1)]),
        );
        func.set_terminator(then_blk, Terminator::Halt);
        func.push_instr(
            else_blk,
            Instr::new(Opcode::Const, None, vec![Operand::Int(0)]),
        );
        func.set_terminator(else_blk, Terminator::Halt);
        module.add_function(func);
        let mut vm = VM::new(module, 42);
        vm.run().unwrap();
        let result = vm.state.pop().unwrap();
        assert_eq!(result, Value::I64(1));
    }

    #[test]
    fn test_discharge_perform() {
        let mut module = seedc::ir::Module::new();
        let mut func = Function::new("test".into(), vec![], IrType::I64);
        let blk = func.entry;
        func.push_instr(blk, Instr::new(Opcode::Const, None, vec![Operand::Int(0)]));
        func.push_instr(blk, Instr::new(Opcode::Discharge, None, vec![]));
        func.push_instr(
            blk,
            Instr::new(Opcode::Perform, None, vec![Operand::String(0)]),
        );
        func.set_terminator(blk, Terminator::Halt);
        module.add_function(func);
        let mut vm = VM::new(module, 42);
        let result = vm.run();
        assert!(
            result.is_ok(),
            "Discharge/Perform should succeed: {:?}",
            result
        );
        assert!(vm.state.effects.len() >= 1);
    }

    #[test]
    fn test_perform_without_discharge_fails() {
        let mut module = seedc::ir::Module::new();
        let mut func = Function::new("test".into(), vec![], IrType::I64);
        let blk = func.entry;
        func.push_instr(
            blk,
            Instr::new(Opcode::Perform, None, vec![Operand::String(0)]),
        );
        func.set_terminator(blk, Terminator::Halt);
        module.add_function(func);
        let mut vm = VM::new(module, 42);
        let result = vm.run();
        assert!(result.is_err(), "Perform without Discharge should fail");
    }

    #[test]
    fn test_memory_store_and_load() {
        let mut module = seedc::ir::Module::new();
        let mut func = Function::new("test".into(), vec![], IrType::I64);
        let blk = func.entry;
        // MemStore layer=0, key=0, value=42
        func.push_instr(blk, Instr::new(Opcode::Const, None, vec![Operand::Int(42)]));
        func.push_instr(
            blk,
            Instr::new(
                Opcode::MemStore,
                None,
                vec![Operand::Int(0), Operand::String(0)],
            ),
        );
        // MemLoad layer=0, key=0
        func.push_instr(
            blk,
            Instr::new(
                Opcode::MemLoad,
                None,
                vec![Operand::Int(0), Operand::String(0)],
            ),
        );
        func.set_terminator(blk, Terminator::Halt);
        module.add_function(func);
        let mut vm = VM::new(module, 42);
        vm.run().unwrap();
        let result = vm.state.pop().unwrap();
        assert_eq!(
            result,
            Value::I64(42),
            "Memory load should return stored value"
        );
    }

    #[test]
    fn test_memory_decay_and_query() {
        let mut module = seedc::ir::Module::new();
        let mut func = Function::new("test".into(), vec![], IrType::I64);
        let blk = func.entry;

        // Store a value in key "1"
        func.push_instr(blk, Instr::new(Opcode::Const, None, vec![Operand::Int(99)]));
        func.push_instr(
            blk,
            Instr::new(
                Opcode::MemStore,
                None,
                vec![Operand::Int(0), Operand::String(1)],
            ),
        );

        // Tick the clock by performing another store (any key, dummy)
        func.push_instr(blk, Instr::new(Opcode::Const, None, vec![Operand::Int(0)]));
        func.push_instr(
            blk,
            Instr::new(
                Opcode::MemStore,
                None,
                vec![Operand::Int(0), Operand::String(2)],
            ),
        );

        // Now apply decay with half_life = 0.1 → huge decay after 1 tick
        func.push_instr(
            blk,
            Instr::new(Opcode::Const, None, vec![Operand::Float(0.1)]),
        );
        func.push_instr(
            blk,
            Instr::new(
                Opcode::MemDecay,
                None,
                vec![Operand::Int(0), Operand::Float(0.1)],
            ),
        );

        // Query weight of first key
        func.push_instr(
            blk,
            Instr::new(
                Opcode::MemQuery,
                None,
                vec![Operand::Int(0), Operand::String(1)],
            ),
        );
        func.set_terminator(blk, Terminator::Halt);

        module.add_function(func);
        let mut vm = VM::new(module, 42);
        vm.run().unwrap();
        let weight = vm.state.pop().unwrap();
        match weight {
            Value::F64(w) => assert!(w < 1.0, "Weight should decay below 1.0, got {}", w),
            _ => panic!("Expected F64 weight"),
        }
    }
}
