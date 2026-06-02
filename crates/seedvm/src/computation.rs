//! The unified effect metadata wrapper `Computation<T, ε>`.
//!
//! Every side‑effecting expression produces a `Computation`,
//! and every `perform` must occur inside a `discharge` block that
//! checks the accumulated thresholds (uncertainty, taint, cost, capability).
//!
//! Implemented per ASL v15.2 Patches 15.7–15.20.

use crate::value::Value;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Computation {
    pub value: Box<Value>, // <-- CHANGE THIS LINE
    pub uncertainty_lo: f64,
    pub uncertainty_hi: f64,
    pub taint_influence: f64,
    pub cost_tokens_min: u64,
    pub cost_tokens_max: u64,
    pub capabilities: Vec<String>,
    pub provenance_refs: Vec<u64>,
    pub effect_set: Vec<String>,
}

impl Computation {
    pub fn pure(value: Value) -> Self {
        Self {
            value: Box::new(value),
            uncertainty_lo: 1.0,
            uncertainty_hi: 1.0,
            taint_influence: 0.0,
            cost_tokens_min: 0,
            cost_tokens_max: 0,
            capabilities: Vec::new(),
            provenance_refs: Vec::new(),
            effect_set: Vec::new(),
        }
    }

    pub fn uncertain(value: Value, lo: f64, hi: f64) -> Self {
        let mut comp = Self::pure(value);
        comp.uncertainty_lo = lo.clamp(0.0, 1.0);
        comp.uncertainty_hi = hi.clamp(0.0, 1.0);
        comp
    }

    pub fn merge(prev: &Computation, next: &Computation) -> Self {
        let lo = (prev.uncertainty_lo * next.uncertainty_lo).clamp(0.0, 1.0);
        let hi = (prev.uncertainty_hi * next.uncertainty_hi).clamp(0.0, 1.0);
        let taint = prev.taint_influence.max(next.taint_influence);
        let cost_min = prev.cost_tokens_min.saturating_add(next.cost_tokens_min);
        let cost_max = prev.cost_tokens_max.saturating_add(next.cost_tokens_max);
        let mut capabilities = prev.capabilities.clone();
        for c in &next.capabilities {
            if !capabilities.contains(c) {
                capabilities.push(c.clone());
            }
        }
        let mut provenance_refs = prev.provenance_refs.clone();
        provenance_refs.extend_from_slice(&next.provenance_refs);
        let mut effect_set = prev.effect_set.clone();
        for e in &next.effect_set {
            if !effect_set.contains(e) {
                effect_set.push(e.clone());
            }
        }
        Self {
            value: next.value.clone(),
            uncertainty_lo: lo,
            uncertainty_hi: hi,
            taint_influence: taint,
            cost_tokens_min: cost_min,
            cost_tokens_max: cost_max,
            capabilities,
            provenance_refs,
            effect_set,
        }
    }

    pub fn check_thresholds(
        &self,
        confidence_threshold: f64,
        taint_threshold: f64,
        budget_remaining: u64,
    ) -> Result<(), crate::state::VmError> {
        use crate::state::VmError;
        if self.uncertainty_hi < confidence_threshold {
            return Err(VmError::LowConfidence {
                actual: self.uncertainty_hi,
                threshold: confidence_threshold,
            });
        }
        if self.taint_influence > taint_threshold {
            return Err(VmError::LowConfidence {
                actual: self.taint_influence,
                threshold: taint_threshold,
            });
        }
        if self.cost_tokens_max > budget_remaining {
            return Err(VmError::LowConfidence {
                actual: self.cost_tokens_max as f64,
                threshold: budget_remaining as f64,
            });
        }
        Ok(())
    }

    pub fn into_value(self) -> Value {
        *self.value
    }
}

impl fmt::Display for Computation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Computation({}, u=[{:.2}, {:.2}], taint={:.2}, cost=[{}, {}])",
            self.value,
            self.uncertainty_lo,
            self.uncertainty_hi,
            self.taint_influence,
            self.cost_tokens_min,
            self.cost_tokens_max,
        )
    }
}
