//! Episodic context reconstruction — master‑assistant two‑agent retrieval.
//!
//! Based on E‑mem (Wang et al., May 2026): shifting from memory
//! preprocessing to episodic context reconstruction inspired by
//! biological engrams. Achieves 54% F1, +7.75% over SOTA.
//!
//! Architecture: heterogeneous hierarchical.
//!   - Master agent: global planning, compressed index, full context.
//!   - Assistant agents: local reasoning in activated segments,
//!     uncompressed memory contexts.

use super::{MemoryEntry, MemoryGovernor, MemoryLayer};

/// The episodic reconstructor.
pub struct EpisodicReconstructor {
    /// Number of assistant agents to deploy.
    pub assistant_count: usize,
    /// Activation threshold for including a memory segment.
    pub activation_threshold: f64,
}

/// A reconstructed episodic trace.
#[derive(Debug, Clone)]
pub struct EpisodicTrace {
    pub session_id: String,
    pub entries: Vec<MemoryEntry>,
    pub confidence: f64,
}

impl EpisodicReconstructor {
    pub fn new() -> Self {
        Self {
            assistant_count: 3,
            activation_threshold: 0.3,
        }
    }

    /// Reconstruct the episodic context for a given query.
    ///
    /// The master agent selects relevant segments from L1 (episodic),
    /// then assistant agents perform local reasoning within each
    /// activated segment before aggregation.
    pub fn reconstruct(&self, query: &str, governor: &MemoryGovernor) -> EpisodicTrace {
        let entries: Vec<&MemoryEntry> = governor.layers[MemoryLayer::Episodic as usize]
            .iter()
            .filter(|e| e.weight > self.activation_threshold)
            .collect();

        // Master agent: global planning — select relevant entries.
        let relevant: Vec<&MemoryEntry> = entries
            .iter()
            .filter(|e| e.key.contains(query) || e.value.to_string().contains(query))
            .cloned()
            .collect();

        // Assistant agents: local reasoning (simulated by scoring).
        let segment_size = (relevant.len() / self.assistant_count).max(1);
        let mut scored: Vec<(f64, &MemoryEntry)> = Vec::new();

        for chunk in relevant.chunks(segment_size) {
            for entry in chunk {
                let score = entry.weight * (entry.reinforcement_count as f64 + 1.0).ln();
                scored.push((score, entry));
            }
        }

        // Sort by score descending
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let entries: Vec<MemoryEntry> = scored.into_iter().map(|(_, e)| e.clone()).collect();
        let confidence = if entries.is_empty() {
            0.0
        } else {
            0.5 + 0.5 / entries.len() as f64
        };

        EpisodicTrace {
            session_id: format!("recon-{}", governor.clock()),
            entries,
            confidence,
        }
    }
}
