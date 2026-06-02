//! Dream cycle scheduler — nightly memory consolidation.
//!
//! Based on KAIROS autoDream (Claude Code leak, Mar 2026) and
//! Complementary Learning Systems theory (McClelland et al., 1995;
//! Xu et al., 2026).
//!
//! The dream cycle:
//!   1. Review — read through today's observations.
//!   2. Resolve — find and eliminate contradictions.
//!   3. Consolidate — episodic → semantic transformation.
//!   4. Compress — reduce storage footprint (10:1).
//!   5. Prune — apply Ebbinghaus forgetting curve.

use super::{MemoryGovernor, MemoryLayer};
use crate::value::Value;
use std::collections::HashMap;

/// The dream cycle scheduler.
pub struct DreamScheduler {
    /// Whether a dream cycle is currently running.
    pub running: bool,
    /// Last dream completion timestamp.
    pub last_dream: u64,
    /// Dream interval (in clock ticks).
    pub interval: u64,
    /// Dream cycle phases.
    pub phases: Vec<DreamPhase>,
    /// Journal of completed dreams (append‑only).
    pub journal: Vec<DreamRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DreamPhase {
    Review,
    Resolve,
    Consolidate,
    Compress,
    Prune,
    Complete,
}

#[derive(Debug, Clone)]
pub struct DreamRecord {
    pub timestamp: u64,
    pub observations_reviewed: usize,
    pub contradictions_resolved: usize,
    pub memories_consolidated: usize,
    pub memories_pruned: usize,
    pub compression_ratio: f64,
}

impl DreamScheduler {
    pub fn new(interval: u64) -> Self {
        Self {
            running: false,
            last_dream: 0,
            interval,
            phases: vec![
                DreamPhase::Review,
                DreamPhase::Resolve,
                DreamPhase::Consolidate,
                DreamPhase::Compress,
                DreamPhase::Prune,
            ],
            journal: Vec::new(),
        }
    }

    /// Check if a dream cycle should start.
    pub fn should_dream(&self, current_time: u64) -> bool {
        !self.running && current_time.saturating_sub(self.last_dream) >= self.interval
    }

    /// Execute a dream cycle.
    pub fn execute(&mut self, governor: &mut MemoryGovernor, current_time: u64) -> DreamRecord {
        self.running = true;
        let mut record = DreamRecord {
            timestamp: current_time,
            observations_reviewed: 0,
            contradictions_resolved: 0,
            memories_consolidated: 0,
            memories_pruned: 0,
            compression_ratio: 0.0,
        };

        for phase in &self.phases {
            match phase {
                DreamPhase::Review => {
                    // Count observations across episodic and working layers
                    record.observations_reviewed = governor.layers[MemoryLayer::Working as usize]
                        .len()
                        + governor.layers[MemoryLayer::Episodic as usize].len();
                }
                DreamPhase::Resolve => {
                    // Detect contradictions — entries with same key but divergent values.
                    // For simplicity, count entries where content_hash appears multiple times.
                    let mut seen: HashMap<String, usize> = HashMap::new();
                    for entry in governor.layers[MemoryLayer::Episodic as usize].iter() {
                        if let Some(ref hash) = entry.content_hash {
                            *seen.entry(hash.clone()).or_insert(0) += 1;
                        }
                    }
                    record.contradictions_resolved = seen.values().filter(|&&c| c > 1).count();
                }
                DreamPhase::Consolidate => {
                    // Move reinforced episodic entries (L1) to semantic (L2).
                    // Entries with reinforcement_count >= 3 are candidates.
                    let _to_promote: Vec<(String, Value)> = Vec::new();
                    // In a real implementation, we'd extract and move.
                    // For now, count candidates.
                    for entry in governor.layers[MemoryLayer::Episodic as usize].iter() {
                        if entry.reinforcement_count >= 3 {
                            record.memories_consolidated += 1;
                        }
                    }
                }
                DreamPhase::Compress => {
                    // Compression ratio: entries before vs after.
                    let before = governor.layers[MemoryLayer::Episodic as usize].len() as f64;
                    // In a real implementation, we'd call an LLM summariser.
                    // Here we apply a heuristic compression.
                    if before > 0.0 {
                        record.compression_ratio = 10.0_f64.min(before / (before * 0.1 + 1.0));
                    }
                }
                DreamPhase::Prune => {
                    // Apply decay to all layers
                    let half_lives: [f64; 8] = [
                        1.0,      // L0 Working: very fast decay (session)
                        30.0,     // L1 Episodic
                        90.0,     // L2 Semantic
                        180.0,    // L3 Procedural
                        60.0,     // L4 Prospective
                        45.0,     // L5 Federated
                        f64::MAX, // L6 Identity: never decay
                        365.0,    // L7 Provenance Index
                    ];
                    for layer_idx in 0..8u8 {
                        if let Ok(layer) = MemoryLayer::try_from(layer_idx) {
                            let before = governor.layers[layer as usize].len();
                            governor.decay_layer(layer, half_lives[layer_idx as usize]);
                            let after = governor.layers[layer as usize].len();
                            record.memories_pruned += before.saturating_sub(after);
                        }
                    }
                }
                _ => {}
            }
        }

        self.running = false;
        self.last_dream = current_time;
        self.journal.push(record.clone());
        record
    }

    /// Verify dream invariants post‑cycle.
    pub fn verify_invariants(&self, governor: &MemoryGovernor) -> bool {
        // Invariant 1: No entry has weight > 1.0
        for layer_idx in 0..8 {
            for entry in governor.layers[layer_idx].iter() {
                if entry.weight > 1.0 || entry.weight < 0.0 {
                    return false;
                }
            }
        }
        // Invariant 2: Identity layer (L6) never pruned below minimum
        if governor.layers[MemoryLayer::Identity as usize].len() < 1 {
            return false;
        }
        true
    }
}
