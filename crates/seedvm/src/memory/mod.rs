//! AGENT‑SEED v15.2 memory subsystem.
//!
//! Implements the eight‑layer memory hierarchy from the architecture:
//!
//! | Layer | Name            | Key Properties |
//! |-------|-----------------|----------------|
//! | L0    | Working Memory  | Session‑scoped, volatile, hot cache |
//! | L1    | Episodic Memory | Append‑only, temporal/causal graphs, Ebbinghaus decay |
//! | L2    | Semantic Memory | Multi‑graph, anti‑echo, ontology‑linked |
//! | L3    | Procedural      | Versioned, success‑rate tracked, causal graph |
//! | L4    | Prospective     | Pending intentions, scheduler‑linked |
//! | L5    | Federated       | CRDT‑backed, vector‑clocked, gossip protocol |
//! | L6    | Identity        | Protected, append‑only, DID + binary hash |
//! | L7    | Provenance Index| Self‑anchored, Merkle‑proofed, exportable JSON‑LD |
//!
//! References:
//!   - Memanto (Abtahi et al., 2026) — typed semantic memory, 89.8% SOTA
//!   - MAGMA (Jiang et al., 2026) — multi‑graph orthogonal memory
//!   - Continuum Memory Architecture (Logan, 2026)
//!   - Engram (engramai crate) — Ebbinghaus forgetting, ACT‑R activation
//!   - ChronoMerkle — time‑aware Merkle trees
//!   - Automerge — CRDT for federated memory

pub mod adaptive;
pub mod coherency;
pub mod dream;
pub mod dual;
pub mod episodic;
pub mod evolutionary;
pub mod governance;
pub mod layer;
pub mod merkle;

pub use adaptive::AdaptiveSelector;
pub use coherency::CoherencyController;
pub use dream::DreamScheduler;
pub use dual::DualProcessController;
pub use episodic::EpisodicReconstructor;
pub use evolutionary::PrismSubstrate;
pub use governance::MemoryGovernor;
pub use layer::MemoryLayer;
pub use merkle::MerkleIntegrityManager;

use crate::value::Value;

/// A key‑value entry stored in a memory layer.
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub key: String,
    pub value: Value,
    /// Reinforcement count — incremented on each access, boosted by spaced repetition.
    pub reinforcement_count: u32,
    /// Timestamp of creation (monotonic counter).
    pub created_at: u64,
    /// Timestamp of last access.
    pub last_accessed: u64,
    /// Current weight in [0, 1] — decays via Ebbinghaus curve, boosted by access.
    pub weight: f64,
    /// Consent level: public, private, sensitive.
    pub consent: ConsentLevel,
    /// Content hash (blake3) for anti‑echo and integrity.
    pub content_hash: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsentLevel {
    Public,
    Private,
    Sensitive,
}

impl Default for ConsentLevel {
    fn default() -> Self {
        Self::Private
    }
}

impl MemoryEntry {
    /// Create a new memory entry with initial weight 1.0.
    pub fn new(key: impl Into<String>, value: Value, timestamp: u64) -> Self {
        Self {
            key: key.into(),
            value,
            reinforcement_count: 0,
            created_at: timestamp,
            last_accessed: timestamp,
            weight: 1.0,
            consent: ConsentLevel::default(),
            content_hash: None,
        }
    }

    /// Apply Ebbinghaus exponential decay: R = e^(-t/S).
    /// S is the decay half‑life in time units.
    pub fn apply_decay(&mut self, now: u64, half_life: f64) {
        let elapsed = (now.saturating_sub(self.last_accessed)) as f64;
        self.weight *= (-elapsed / half_life).exp();
        self.weight = self.weight.clamp(0.001, 1.0);
    }

    /// Boost weight on access (spaced repetition).
    pub fn reinforce(&mut self, now: u64) {
        self.reinforcement_count += 1;
        self.last_accessed = now;
        // Boost: weight approaches 1.0 with each reinforcement
        let boost = 0.1 * (1.0 - self.weight);
        self.weight = (self.weight + boost).min(1.0);
    }
}
