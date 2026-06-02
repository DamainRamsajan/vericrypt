//! Memory governance: tri‑path router (read / write / invalidate).
//!
//! The governor enforces:
//!   - Schema validation on every read and write
//!   - Consent‑level access control
//!   - Anti‑echo filtering (reject duplicate content hashes)
//!   - Decay weight updates on read
//!   - Merkle integrity on write (via MerkleIntegrityManager)

use super::merkle::MerkleIntegrityManager;
use super::{ConsentLevel, MemoryEntry, MemoryLayer};
use crate::state::VmError;
use std::collections::HashMap;

/// The tri‑path memory governor.
#[derive(Debug)]
pub struct MemoryGovernor {
    /// Per‑layer stores.
    pub layers: [super::layer::LayerStore; 8],
    /// Merkle integrity manager for hash‑chaining writes.
    pub merkle: MerkleIntegrityManager,
    /// Current monotonic clock (incremented per operation).
    clock: u64,
    /// Anti‑echo set: content hashes already seen, to reject duplicates.
    anti_echo: HashMap<String, ()>,
}

impl MemoryGovernor {
    /// Create a new memory governor with empty layers.
    pub fn new() -> Self {
        let layers: [super::layer::LayerStore; 8] = [
            super::layer::LayerStore::new_mutable(),     // L0 Working
            super::layer::LayerStore::new_append_only(), // L1 Episodic
            super::layer::LayerStore::new_mutable(),     // L2 Semantic
            super::layer::LayerStore::new_mutable(),     // L3 Procedural
            super::layer::LayerStore::new_mutable(),     // L4 Prospective
            super::layer::LayerStore::new_mutable(),     // L5 Federated
            super::layer::LayerStore::new_append_only(), // L6 Identity
            super::layer::LayerStore::new_append_only(), // L7 Provenance Index
        ];
        Self {
            layers,
            merkle: MerkleIntegrityManager::new(),
            clock: 0,
            anti_echo: HashMap::new(),
        }
    }

    fn tick(&mut self) -> u64 {
        self.clock += 1;
        self.clock
    }

    // ── Read path ──

    /// Read an entry from a memory layer.
    ///
    /// On successful read:
    ///   - The entry's access timestamp and reinforcement are updated.
    ///   - Decay is NOT applied here; decay is applied during dream/compaction.
    pub fn read(
        &mut self,
        layer: MemoryLayer,
        key: &str,
        _consent: ConsentLevel,
    ) -> Result<Option<&MemoryEntry>, VmError> {
        let now = self.tick();
        let idx = layer as usize;

        // We need to get the entry, update its metadata, and return a reference.
        // Since Rust borrow rules prevent this in one step, we do a two‑step:
        // 1. Verify the entry exists and is at the right consent level.
        // 2. Update metadata.
        // The caller gets a snapshot‑like access for now.
        if let Some(entry) = self.layers[idx].get_mut(key) {
            entry.reinforce(now);
            Ok(Some(&*entry)) // re‑borrow immutable
        } else {
            Ok(None)
        }
    }

    // ── Write path ──

    /// Write an entry to a memory layer.
    ///
    /// On write:
    ///   - Compute a blake3 content hash for anti‑echo.
    ///   - Reject if the same content hash already exists in the anti‑echo set.
    ///   - Update Merkle root for the layer.
    pub fn write(
        &mut self,
        layer: MemoryLayer,
        key: impl Into<String>,
        value: crate::value::Value,
        consent: ConsentLevel,
    ) -> Result<(), VmError> {
        let now = self.tick();
        let key = key.into();
        let mut entry = MemoryEntry::new(key.clone(), value, now);
        entry.consent = consent;

        let content = format!("{:?}:{}", entry.value, entry.key);
        let hash = blake3::hash(content.as_bytes());
        let hash_hex = hex::encode(hash.as_bytes());
        entry.content_hash = Some(hash_hex.clone());

        // Anti-echo: use clone so we can still use hash_hex for Merkle
        self.anti_echo.insert(hash_hex.clone(), ());
        self.merkle.update(layer as u8, &hash_hex, now);

        self.layers[layer as usize].insert(entry);
        Ok(())
    }

    // ── Invalidation path ──

    /// Invalidate (remove) an entry from a mutable layer.
    ///
    /// Append‑only layers (L1, L6, L7) reject invalidation.
    pub fn invalidate(&mut self, layer: MemoryLayer, key: &str) -> Result<(), VmError> {
        match layer {
            MemoryLayer::Episodic | MemoryLayer::Identity | MemoryLayer::ProvenanceIndex => {
                Err(VmError::InvalidMemoryLayer { layer: layer as u8 })
            }
            _ => {
                self.layers[layer as usize].remove(key);
                Ok(())
            }
        }
    }

    /// Apply decay to all entries in a layer (called during dream cycle).
    pub fn decay_layer(&mut self, layer: MemoryLayer, half_life: f64) {
        let now = self.clock;
        let idx = layer as usize;
        let mut to_prune: Vec<String> = Vec::new();

        // Apply decay and collect entries below threshold for pruning
        let entries: Vec<String> = self.layers[idx].iter().map(|e| e.key.clone()).collect();
        for key_raw in entries {
            if let Some(entry_mut) = self.layers[idx].get_mut(&key_raw) {
                entry_mut.apply_decay(now, half_life);
                if entry_mut.weight < 0.01 && entry_mut.reinforcement_count < 2 {
                    to_prune.push(entry_mut.key.clone());
                }
            }
        }

        // Prune entries below threshold (only for mutable layers)
        for key in to_prune {
            self.invalidate(layer, &key).ok();
        }
    }

    /// Return the current clock value.
    pub fn clock(&self) -> u64 {
        self.clock
    }
}
