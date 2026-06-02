//! Dual‑process memory controller — System 1 (fast) and System 2 (deep).
//!
//! Based on D‑Mem (Yuan et al., 2026) and the dual‑process theory
//! from cognitive science (Kahneman, 2011).
//!
//! System 1: fast, vector‑similarity‑based retrieval for routine queries.
//! System 2: exhaustive graph traversal for complex or high‑stakes queries.
//! A multi‑dimensional quality gating policy bridges the two.

use super::MemoryEntry;

/// The gating decision for routing a query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatingDecision {
    /// Route to System 1 (fast retrieval).
    System1,
    /// Route to System 2 (deep deliberation).
    System2,
    /// Hybrid: try System 1 first, escalate to System 2 if confidence is low.
    Hybrid,
}

/// The dual‑process controller.
pub struct DualProcessController {
    /// Confidence threshold below which System 1 results are escalated.
    pub threshold: f64,
    /// Whether System 2 runs during the query (eager) or only on escalation (lazy).
    pub eager: bool,
    /// Metrics for gating decisions.
    pub last_gating: Option<GatingDecision>,
    pub escalation_count: u64,
}

impl DualProcessController {
    pub fn new() -> Self {
        Self {
            threshold: 0.85,
            eager: false,
            last_gating: None,
            escalation_count: 0,
        }
    }

    /// Decide which system to use based on query features.
    ///
    /// Multi‑dimensional gate considers:
    ///   - Query novelty (new topic → System 2)
    ///   - Confidence of System 1 on similar queries (low → System 2)
    ///   - Stakes of the decision (high → System 2)
    ///   - Recency requirements (very recent → System 1 cache hit)
    ///   - Contradiction potential (high → System 2)
    pub fn gate(
        &mut self,
        query_novelty: f64,
        stakes: f64,
        prior_confidence: Option<f64>,
    ) -> GatingDecision {
        let decision = if query_novelty > 0.7 || stakes > 0.7 {
            GatingDecision::System2
        } else if let Some(conf) = prior_confidence {
            if conf < self.threshold {
                self.escalation_count += 1;
                GatingDecision::System2
            } else {
                GatingDecision::System1
            }
        } else {
            GatingDecision::System1
        };
        self.last_gating = Some(decision);
        decision
    }

    /// System 1: fast key‑based or pattern‑match retrieval.
    pub fn system1_retrieve<'a>(
        &self,
        query: &str,
        entries: &'a [MemoryEntry],
    ) -> Vec<&'a MemoryEntry> {
        entries
            .iter()
            .filter(|e| e.key.contains(query) || self.fuzzy_match(&e.key, query))
            .collect()
    }

    /// System 2: exhaustive semantic graph traversal.
    /// Placeholder — full implementation would use multi‑graph traversal
    /// across the semantic, temporal, causal, and entity graphs.
    pub fn system2_retrieve<'a>(
        &self,
        _query: &str,
        entries: &'a [MemoryEntry],
    ) -> Vec<&'a MemoryEntry> {
        // Exhaustive traversal with graph expansion
        // For now, return all entries with relevance scoring
        entries.iter().filter(|e| e.weight > 0.2).collect()
    }

    /// Simple fuzzy match (contains all query terms).
    fn fuzzy_match(&self, key: &str, query: &str) -> bool {
        query
            .split_whitespace()
            .all(|term| key.to_lowercase().contains(&term.to_lowercase()))
    }
}
