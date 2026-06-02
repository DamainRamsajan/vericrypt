//! Memory layer type definitions for the AGENT‑SEED VM.
//!
//! Each layer (L0‑L7) has a distinct schema, storage strategy,
//! decay function, and provenance tracking configuration.

use super::MemoryEntry;
use std::collections::HashMap;

// ── MemoryLayer enum ──

/// The eight memory layers, indexed 0..7.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryLayer {
    /// L0 — Working memory: session‑scoped, volatile, hot cache.
    Working = 0,
    /// L1 — Episodic memory: append‑only log, temporal/causal graphs.
    Episodic = 1,
    /// L2 — Semantic memory: multi‑graph, anti‑echo, ontology‑linked.
    Semantic = 2,
    /// L3 — Procedural memory: versioned skills, success‑rate tracked.
    Procedural = 3,
    /// L4 — Prospective memory: pending intentions, scheduler‑linked.
    Prospective = 4,
    /// L5 — Federated memory: CRDT‑backed, vector‑clocked, gossip.
    Federated = 5,
    /// L6 — Identity memory: protected, append‑only, DID + hash.
    Identity = 6,
    /// L7 — Provenance index: self‑anchored, Merkle‑proofed.
    ProvenanceIndex = 7,
}

impl TryFrom<u8> for MemoryLayer {
    type Error = crate::state::VmError;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::Working),
            1 => Ok(Self::Episodic),
            2 => Ok(Self::Semantic),
            3 => Ok(Self::Procedural),
            4 => Ok(Self::Prospective),
            5 => Ok(Self::Federated),
            6 => Ok(Self::Identity),
            7 => Ok(Self::ProvenanceIndex),
            _ => Err(crate::state::VmError::InvalidMemoryLayer { layer: v }),
        }
    }
}

// ── LayerStore — the abstract backing store ──

/// The storage backend for a single memory layer.
///
/// Layers can use either an append‑only log (for immutable layers
/// like Episodic and Identity) or a mutable store (for Working
/// and Semantic).
#[derive(Debug, Clone)]
pub enum LayerStore {
    /// Append‑only log: entries are never mutated, only appended.
    AppendOnly(Vec<MemoryEntry>),
    /// Mutable key‑value store: entries can be updated in place.
    Mutable(HashMap<String, MemoryEntry>),
}

impl LayerStore {
    /// Create a new append‑only log.
    pub fn new_append_only() -> Self {
        Self::AppendOnly(Vec::new())
    }

    /// Create a new mutable store.
    pub fn new_mutable() -> Self {
        Self::Mutable(HashMap::new())
    }

    /// Insert an entry into the store.
    pub fn insert(&mut self, entry: MemoryEntry) {
        match self {
            Self::AppendOnly(log) => log.push(entry),
            Self::Mutable(map) => {
                map.insert(entry.key.clone(), entry);
            }
        }
    }

    /// Get an entry by key.
    pub fn get(&self, key: &str) -> Option<&MemoryEntry> {
        match self {
            Self::AppendOnly(log) => log.iter().rev().find(|e| e.key == key),
            Self::Mutable(map) => map.get(key),
        }
    }

    /// Get a mutable reference to an entry by key.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut MemoryEntry> {
        match self {
            Self::AppendOnly(log) => log.iter_mut().rev().find(|e| e.key == key),
            Self::Mutable(map) => map.get_mut(key),
        }
    }

    /// Remove an entry by key (only meaningful for mutable stores).
    pub fn remove(&mut self, key: &str) {
        if let Self::Mutable(map) = self {
            map.remove(key);
        }
    }

    /// Return the number of entries.
    pub fn len(&self) -> usize {
        match self {
            Self::AppendOnly(log) => log.len(),
            Self::Mutable(map) => map.len(),
        }
    }

    /// Iterate over all entries.
    pub fn iter(&self) -> Box<dyn Iterator<Item = &MemoryEntry> + '_> {
        match self {
            Self::AppendOnly(log) => Box::new(log.iter()),
            Self::Mutable(map) => Box::new(map.values()),
        }
    }
}
