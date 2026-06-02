//! Runtime taint engine with category‑aware coloring.
//!
//! Based on pyrograph (Rust crate, Apr 2026): GPU‑accelerated taint analysis
//! with category‑aware coloring for multi‑language supply chain security.
//! Also inspired by Tant (Bertolo, 2026) for type‑level taint qualifiers
//! and zeptoclaw for data‑flow‑aware agent safety.

use std::collections::{HashMap, HashSet};
use crate::value::Value;

// ── Taint categories ──

/// Taint source/sink categories — specific dangerous combinations trigger findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaintCategory {
    /// User input (untrusted).
    UserInput,
    /// Network‑sourced data.
    Network,
    /// File system data.
    FileSystem,
    /// Environment variables.
    EnvVar,
    /// LLM inference output.
    Inference,
    /// External agent message.
    AgentMessage,
    /// Database query result.
    Database,
    /// Clean / trusted.
    Clean,
}

/// Taint level — three‑level lattice: Clean ≤ Agnostic ≤ Tainted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaintLevel {
    Clean = 0,
    Agnostic = 1,
    Tainted = 2,
}

// ── Taint metadata ──

/// Metadata attached to every value as it flows through the system.
#[derive(Debug, Clone)]
pub struct TaintMeta {
    /// Current taint level.
    pub level: TaintLevel,
    /// Categories of taint sources that contributed.
    pub categories: HashSet<TaintCategory>,
    /// Sources (by name) that contributed taint.
    pub sources: Vec<String>,
    /// Propagation depth (how many steps from original source).
    pub depth: u32,
}

impl TaintMeta {
    /// Create clean metadata.
    pub fn clean() -> Self {
        Self { level: TaintLevel::Clean, categories: HashSet::new(), sources: vec![], depth: 0 }
    }

    /// Create tainted metadata from a single source.
    pub fn tainted(category: TaintCategory, source: impl Into<String>) -> Self {
        let mut categories = HashSet::new();
        categories.insert(category);
        Self { level: TaintLevel::Tainted, categories, sources: vec![source.into()], depth: 1 }
    }

    /// Join (lub) of two taint metadata values.
    pub fn join(&self, other: &TaintMeta) -> TaintMeta {
        let level = self.level.max(other.level);
        let mut categories = self.categories.clone();
        categories.extend(&other.categories);
        let mut sources = self.sources.clone();
        sources.extend(other.sources.clone());
        let depth = self.depth.max(other.depth) + 1;
        TaintMeta { level, categories, sources, depth }
    }
}

// ── Dangerous source‑sink combinations ──

/// A dangerous source→sink pair (from pyrograph's 35+ combinations).
#[derive(Debug, Clone)]
pub struct DangerRule {
    pub source_category: TaintCategory,
    pub sink_category: TaintCategory,
    pub severity: DangerSeverity,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DangerSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl DangerRule {
    /// The well‑known dangerous combinations.
    pub fn builtin_rules() -> Vec<DangerRule> {
        vec![
            DangerRule { source_category: TaintCategory::UserInput, sink_category: TaintCategory::Network, severity: DangerSeverity::High, name: "input→network".into(), description: "User input being sent over network".into() },
            DangerRule { source_category: TaintCategory::Inference, sink_category: TaintCategory::Network, severity: DangerSeverity::High, name: "inference→network".into(), description: "LLM output exfiltration".into() },
            DangerRule { source_category: TaintCategory::Network, sink_category::TaintCategory::FileSystem, severity: DangerSeverity::Medium, name: "network→filesystem".into(), description: "Network data written to disk".into() },
            DangerRule { source_category: TaintCategory::EnvVar, sink_category::TaintCategory::Network, severity: DangerSeverity::Critical, name: "envvar→network".into(), description: "Environment variable (credentials) sent over network".into() },
            DangerRule { source_category: TaintCategory::AgentMessage, sink_category::TaintCategory::Database, severity: DangerSeverity::Medium, name: "agent-msg→db".into(), description: "Agent message written to database".into() },
        ]
    }
}

// ── Taint engine ──

/// Runtime taint engine — tracks taint propagation and enforces rules.
pub struct TaintEngine {
    /// Current taint on each variable (by name).
    pub var_taint: HashMap<String, TaintMeta>,
    /// Program counter taint (branched on tainted conditions).
    pub pc_taint: TaintMeta,
    /// Dangerous source‑sink rules.
    pub rules: Vec<DangerRule>,
    /// Violations detected.
    pub violations: Vec<TaintViolation>,
}

#[derive(Debug, Clone)]
pub struct TaintViolation {
    pub rule_name: String,
    pub source: String,
    pub sink: String,
    pub description: String,
    pub severity: DangerSeverity,
}

impl TaintEngine {
    pub fn new() -> Self {
        Self {
            var_taint: HashMap::new(),
            pc_taint: TaintMeta::clean(),
            rules: DangerRule::builtin_rules(),
            violations: Vec::new(),
        }
    }

    /// Assign taint to a variable.
    pub fn taint_var(&mut self, name: &str, meta: TaintMeta) {
        self.var_taint.insert(name.to_string(), meta);
    }

    /// Propagate taint through an operation.
    pub fn propagate(&mut self, dest: &str, sources: &[&str]) -> TaintMeta {
        let mut combined = TaintMeta::clean();
        for src in sources {
            if let Some(meta) = self.var_taint.get(*src) {
                combined = combined.join(meta);
            }
        }
        // Also join with PC taint
        combined = combined.join(&self.pc_taint);
        self.var_taint.insert(dest.to_string(), combined.clone());
        combined
    }

    /// Check a sink operation against dangerous rules.
    pub fn check_sink(&mut self, source_var: &str, sink_category: TaintCategory, sink_description: &str) {
        if let Some(source_taint) = self.var_taint.get(source_var) {
            for rule in &self.rules {
                if source_taint.categories.contains(&rule.source_category) && rule.sink_category == sink_category {
                    self.violations.push(TaintViolation {
                        rule_name: rule.name.clone(),
                        source: format!("{:?}", source_taint.sources),
                        sink: sink_description.to_string(),
                        description: rule.description.clone(),
                        severity: rule.severity,
                    });
                }
            }
        }
    }

    /// Apply a sanitizer to a variable — reduces taint level.
    pub fn sanitize(&mut self, var_name: &str, policy: &SanitizePolicy) -> Result<(), String> {
        if let Some(meta) = self.var_taint.get_mut(var_name) {
            match policy {
                SanitizePolicy::StripAll => {
                    *meta = TaintMeta::clean();
                }
                SanitizePolicy::ReduceLevel(new_level) => {
                    meta.level = *new_level;
                }
                SanitizePolicy::RemoveCategory(cat) => {
                    meta.categories.remove(cat);
                    if meta.categories.is_empty() {
                        meta.level = TaintLevel::Clean;
                    }
                }
                SanitizePolicy::Validate(regex) => {
                    // If validation passes, reduce to Agnostic
                    meta.level = TaintLevel::Agnostic;
                }
            }
        }
        Ok(())
    }
}

/// Sanitization policies.
#[derive(Debug, Clone)]
pub enum SanitizePolicy {
    StripAll,
    ReduceLevel(TaintLevel),
    RemoveCategory(TaintCategory),
    Validate(String),
}
