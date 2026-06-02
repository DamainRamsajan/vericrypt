//! Sanitizer — applies policies to reduce or strip taint from values.
//!
//! Based on SONAR (May 2026): sentence‑relation‑based prompt sanitization,
//! CodeQL sanitizer/validator barrier guards (Apr 2026), and the OWASP
//! 2026 three‑tier hardening blueprint (Input Sanitization, Execution
//! Isolation, Output Filtering).

use crate::value::Value;
use crate::taint::{TaintMeta, TaintLevel, SanitizePolicy};

/// The sanitizer applies registered policies to values.
pub struct Sanitizer {
    /// Registered sanitization policies, keyed by policy name.
    pub policies: Vec<NamedPolicy>,
}

#[derive(Debug, Clone)]
pub struct NamedPolicy {
    pub name: String,
    pub policy: SanitizePolicy,
    /// Whether this sanitizer is trusted (breaks taint chains completely).
    pub trusted: bool,
}

impl Sanitizer {
    pub fn new() -> Self {
        Self { policies: Vec::new() }
    }

    /// Register a sanitization policy.
    pub fn register(&mut self, policy: NamedPolicy) {
        self.policies.push(policy);
    }

    /// Apply all matching policies to a value and return sanitized output.
    pub fn apply(&self, value: &Value, taint: &TaintMeta) -> (Value, TaintMeta) {
        let mut new_taint = taint.clone();
        for named in &self.policies {
            if named.trusted {
                new_taint = TaintMeta::clean();
                break;
            }
            match &named.policy {
                SanitizePolicy::StripAll => {
                    new_taint = TaintMeta::clean();
                }
                SanitizePolicy::ReduceLevel(level) => {
                    new_taint.level = *level;
                }
                SanitizePolicy::RemoveCategory(_cat) => {
                    // Category removal is handled by taint.rs
                }
                SanitizePolicy::Validate(_regex) => {
                    new_taint.level = TaintLevel::Agnostic;
                }
            }
        }
        (value.clone(), new_taint)
    }

    /// Check whether a value needs sanitization.
    pub fn needs_sanitization(taint: &TaintMeta) -> bool {
        taint.level > TaintLevel::Clean
    }
}
