//! Adaptive memory structure selector.
//!
//! Based on FluxMem (Feb 2026): equips agents with multiple complementary
//! memory structures and explicitly learns to select among them based on
//! interaction‑level features.
//!
//! Structures:
//!   - raw_buffer: ephemeral ring buffer (last N interactions)
//!   - summarized_context: LLM‑generated summaries
//!   - entity_index: structured knowledge graph
//!   - vector_store: embedding‑indexed
//!   - rule_base: extracted patterns

use crate::value::Value;
use std::collections::{HashMap, VecDeque};

/// Available memory structures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryStructure {
    RawBuffer,
    SummarizedContext,
    EntityIndex,
    VectorStore,
    RuleBase,
}

/// The adaptive selector.
pub struct AdaptiveSelector {
    /// Currently active structure.
    pub active: MemoryStructure,
    /// Feature weights for structure selection (learned via RL or heuristics).
    pub weights: HashMap<MemoryStructure, f64>,
    /// Raw buffer (ring buffer).
    pub raw_buffer: VecDeque<(String, Value)>,
    /// Buffer capacity.
    pub buffer_capacity: usize,
    /// Selection history for learning.
    pub selection_history: Vec<(MemoryStructure, f64)>,
}

impl AdaptiveSelector {
    pub fn new() -> Self {
        let mut weights = HashMap::new();
        weights.insert(MemoryStructure::RawBuffer, 0.5);
        weights.insert(MemoryStructure::SummarizedContext, 0.7);
        weights.insert(MemoryStructure::EntityIndex, 0.6);
        weights.insert(MemoryStructure::VectorStore, 0.4);
        weights.insert(MemoryStructure::RuleBase, 0.3);

        Self {
            active: MemoryStructure::RawBuffer,
            weights,
            raw_buffer: VecDeque::new(),
            buffer_capacity: 100,
            selection_history: Vec::new(),
        }
    }

    /// Select the best memory structure for a query.
    pub fn select(&mut self, query_type: &str, interaction_length: usize) -> MemoryStructure {
        let score = |s: MemoryStructure| -> f64 {
            let base = self.weights[&s];
            match s {
                MemoryStructure::RawBuffer if interaction_length < 5 => base + 0.3,
                MemoryStructure::SummarizedContext if interaction_length > 10 => base + 0.2,
                MemoryStructure::EntityIndex if query_type == "entity" => base + 0.4,
                MemoryStructure::VectorStore if query_type == "semantic" => base + 0.3,
                MemoryStructure::RuleBase if query_type == "procedural" => base + 0.3,
                _ => base,
            }
        };

        let structures = [
            MemoryStructure::RawBuffer,
            MemoryStructure::SummarizedContext,
            MemoryStructure::EntityIndex,
            MemoryStructure::VectorStore,
            MemoryStructure::RuleBase,
        ];

        let best = structures
            .iter()
            .max_by(|a, b| score(**a).partial_cmp(&score(**b)).unwrap())
            .copied()
            .unwrap_or(MemoryStructure::RawBuffer);

        self.selection_history.push((best, score(best)));
        self.active = best;
        best
    }

    /// Push to the raw buffer.
    pub fn buffer_push(&mut self, key: String, value: Value) {
        if self.raw_buffer.len() >= self.buffer_capacity {
            self.raw_buffer.pop_front();
        }
        self.raw_buffer.push_back((key, value));
    }

    /// Query the raw buffer.
    pub fn buffer_query(&self, key: &str) -> Option<&Value> {
        self.raw_buffer
            .iter()
            .rev()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }
}
