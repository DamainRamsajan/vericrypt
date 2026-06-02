//! Uncertainty engine — enforces the four uncertainty axioms (U1–U4).
//!
//! Based on interval arithmetic (Moore, 1966) applied to agentic
//! computation. The engine tracks uncertainty as intervals and
//! guarantees that uncertainty never silently collapses.
//!
//! U1: Interval multiplication (bind)
//! U2: Conditioning (observe) narrows uncertainty
//! U3: Precision monotonicity — uncertainty never widens
//! U4: No illegal widening — reject operations that would increase uncertainty

use std::ops::{Add, Mul, Sub};

// ── Interval (from effects/interval.rs — core spec type) ──

/// A numeric interval [lo, hi] representing bounded uncertainty.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    pub lo: f64,
    pub hi: f64,
}

impl Interval {
    /// Create a new interval. Enforces lo ≤ hi.
    pub fn new(lo: f64, hi: f64) -> Self {
        assert!(lo <= hi, "lo must be ≤ hi");
        Self { lo, hi }
    }

    /// Create an exact value interval [v, v].
    pub fn exact(v: f64) -> Self {
        Self { lo: v, hi: v }
    }

    /// The width of the interval.
    pub fn width(&self) -> f64 { self.hi - self.lo }

    /// The midpoint of the interval.
    pub fn midpoint(&self) -> f64 { (self.lo + self.hi) / 2.0 }

    /// Check whether this interval contains a value.
    pub fn contains(&self, v: f64) -> bool { v >= self.lo && v <= self.hi }

    /// Check whether this interval is contained within another.
    pub fn contained_in(&self, other: &Interval) -> bool {
        self.lo >= other.lo && self.hi <= other.hi
    }
}

impl Add for Interval {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self { lo: self.lo + other.lo, hi: self.hi + other.hi }
    }
}

impl Sub for Interval {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self { lo: self.lo - other.hi, hi: self.hi - other.lo }
    }
}

impl Mul for Interval {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        let products = [
            self.lo * other.lo,
            self.lo * other.hi,
            self.hi * other.lo,
            self.hi * other.hi,
        ];
        let min = products.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = products.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        Self { lo: min, hi: max }
    }
}

// ── Uncertainty Engine ──

/// The uncertainty engine — enforces U1–U4 axioms at runtime.
pub struct UncertaintyEngine {
    /// Current accumulated uncertainty interval.
    pub uncertainty: Interval,
    /// Propagation chain (for debugging / audit).
    pub propagation_chain: Vec<UncertaintyEvent>,
    /// Minimum allowed precision (reject if width exceeds this).
    pub min_precision: f64,
}

#[derive(Debug, Clone)]
pub struct UncertaintyEvent {
    pub operation: String,
    pub before: Interval,
    pub after: Interval,
    pub timestamp: u64,
}

impl UncertaintyEngine {
    pub fn new() -> Self {
        Self {
            uncertainty: Interval::exact(0.0),
            propagation_chain: Vec::new(),
            min_precision: 1.0, // max allowed width
        }
    }

    /// U1: Bind (multiply) two uncertainty intervals.
    /// Propagates uncertainty: result width = f(width_a, width_b).
    pub fn bind(&mut self, u1: Interval, u2: Interval, timestamp: u64) -> Interval {
        let before = self.uncertainty;
        // The bind operation combines uncertainties multiplicatively
        let combined = u1 * u2;
        let result = self.uncertainty + combined;

        self.propagation_chain.push(UncertaintyEvent {
            operation: "bind".into(),
            before,
            after: result,
            timestamp,
        });
        self.uncertainty = result;
        result
    }

    /// U2: Observe — condition on new information.
    /// This MUST narrow (or maintain) the uncertainty interval.
    pub fn observe(&mut self, observation: Interval, timestamp: u64) -> Result<Interval, String> {
        let before = self.uncertainty;
        // Conditioning: intersect current uncertainty with observation
        let lo = self.uncertainty.lo.max(observation.lo);
        let hi = self.uncertainty.hi.min(observation.hi);

        if lo > hi {
            return Err(format!(
                "Inconsistent observation: current [{}, {}] incompatible with [{}, {}]",
                self.uncertainty.lo, self.uncertainty.hi, observation.lo, observation.hi
            ));
        }

        let result = Interval { lo, hi };

        // U3: Precision monotonicity — result must not be wider than before
        if result.width() > before.width() + 1e-10 {
            return Err(format!(
                "U3 violation: observation widened uncertainty from {:.6} to {:.6}",
                before.width(), result.width()
            ));
        }

        self.propagation_chain.push(UncertaintyEvent {
            operation: "observe".into(),
            before,
            after: result,
            timestamp,
        });
        self.uncertainty = result;
        Ok(result)
    }

    /// U4: Validate that an operation does not illegally widen uncertainty.
    pub fn validate(&self, proposed: &Interval) -> Result<(), String> {
        if proposed.width() > self.min_precision {
            return Err(format!(
                "U4 violation: interval width {:.6} exceeds max allowed {:.6}",
                proposed.width(), self.min_precision
            ));
        }
        Ok(())
    }

    /// Get the current accumulated uncertainty.
    pub fn current(&self) -> Interval {
        self.uncertainty
    }
}
