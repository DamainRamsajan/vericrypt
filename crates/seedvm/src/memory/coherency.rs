//! Memory coherency controller: MESI protocol, CRDT merging, and anti‑entropy gossip.
//!
//! Inspired by CPU cache coherence protocols (MESI) applied to
//! multi‑agent memory systems (Yu et al., 2026).

use crate::value::Value;
use std::collections::{HashMap, HashSet};

// ── MESI state ──

/// Cache line state for the MESI coherence protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MesiState {
    /// Modified — local copy is dirty, must be written back before sharing.
    Modified,
    /// Exclusive — local copy is clean, no other agent has it.
    Exclusive,
    /// Shared — multiple agents have a clean copy.
    Shared,
    /// Invalid — this copy is stale and must be re‑fetched.
    Invalid,
}

/// A cache line tracked by the coherence controller.
#[derive(Debug, Clone)]
pub struct CacheLine {
    pub key: String,
    pub value: Value,
    pub state: MesiState,
    pub layer: u8,
}

// ── MESI controller ──

/// A MESI‑inspired coherence controller for multi‑agent memory.
#[derive(Debug)]
pub struct MesiController {
    /// Cache lines indexed by key.
    lines: HashMap<String, CacheLine>,
    /// Peers that share each key (for invalidation broadcasts).
    shared_by: HashMap<String, HashSet<String>>,
}

impl MesiController {
    pub fn new() -> Self {
        Self {
            lines: HashMap::new(),
            shared_by: HashMap::new(),
        }
    }

    /// Read a key locally. Transitions state as needed.
    pub fn local_read(&mut self, key: &str) -> Option<&Value> {
        if let Some(line) = self.lines.get(key) {
            // Local read does not change MESI state.
            Some(&line.value)
        } else {
            None
        }
    }

    /// Write a key locally. Invalidates all shared copies.
    pub fn local_write(&mut self, key: &str, value: Value, layer: u8) {
        // Invalidate all peers holding this key
        if let Some(_peers) = self.shared_by.get(key) {
            // In a real implementation, broadcast invalidation to peers.
            // Here we simply clear the set.
            // Peers will have their state set to Invalid on next access.
        }
        self.lines.insert(
            key.to_string(),
            CacheLine {
                key: key.to_string(),
                value,
                state: MesiState::Modified,
                layer,
            },
        );
        self.shared_by.insert(key.to_string(), HashSet::new());
    }

    /// Receive a remote write notification — invalidate our copy.
    pub fn remote_write(&mut self, key: &str) {
        if let Some(line) = self.lines.get_mut(key) {
            line.state = MesiState::Invalid;
        }
    }

    /// Share a key with a peer.
    pub fn share_with(&mut self, key: &str, peer_id: &str) {
        self.shared_by
            .entry(key.to_string())
            .or_default()
            .insert(peer_id.to_string());
        if let Some(line) = self.lines.get_mut(key) {
            if line.state == MesiState::Exclusive || line.state == MesiState::Modified {
                line.state = MesiState::Shared;
            }
        }
    }
}

// ── CRDT manager ──

/// A simple CRDT‑backed store for federated memory (L5).
///
/// Uses a last‑writer‑wins register per key with vector clock.
/// Full CRDT integration uses the `automerge` crate; this is a
/// minimal implementation for the VM.
#[derive(Debug, Clone)]
pub struct CrdtManager {
    /// Vector clock: node_id → counter.
    clock: HashMap<String, u64>,
    /// Data store.
    store: HashMap<String, (u64, Value)>,
    node_id: String,
}

impl CrdtManager {
    pub fn new(node_id: impl Into<String>) -> Self {
        Self {
            clock: HashMap::new(),
            store: HashMap::new(),
            node_id: node_id.into(),
        }
    }

    /// Local write: increment our own clock and store the value.
    pub fn local_set(&mut self, key: &str, value: Value) {
        let counter = self.clock.entry(self.node_id.clone()).or_insert(0);
        *counter += 1;
        self.store.insert(key.to_string(), (*counter, value));
    }

    /// Merge a remote update (LWW: last‑writer‑wins).
    pub fn merge(&mut self, key: &str, remote_counter: u64, remote_value: Value) {
        let local_counter = self.store.get(key).map(|(c, _)| *c).unwrap_or(0);
        if remote_counter > local_counter {
            self.store
                .insert(key.to_string(), (remote_counter, remote_value));
        }
    }

    /// Read a key.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.store.get(key).map(|(_, v)| v)
    }
}

// ── Anti‑entropy gossip ──

/// A minimal anti‑entropy gossip manager.
///
/// Tracks which keys have been modified since the last sync
/// and exchanges them with peers.
#[derive(Debug)]
pub struct GossipManager {
    /// Keys modified since last full sync.
    dirty_keys: HashSet<String>,
    /// Known peers.
    peers: Vec<String>,
}

impl GossipManager {
    pub fn new() -> Self {
        Self {
            dirty_keys: HashSet::new(),
            peers: Vec::new(),
        }
    }

    /// Mark a key as dirty (modified locally).
    pub fn mark_dirty(&mut self, key: &str) {
        self.dirty_keys.insert(key.to_string());
    }

    /// Add a peer to the gossip mesh.
    pub fn add_peer(&mut self, peer_id: &str) {
        self.peers.push(peer_id.to_string());
    }

    /// Run a gossip round: exchange dirty keys with a randomly selected peer.
    /// Returns the set of keys to push to that peer.
    pub fn gossip_round(&mut self) -> (String, Vec<String>) {
        // Select a random peer (simplified: round‑robin)
        let peer = self.peers.first().cloned().unwrap_or_default();
        let keys: Vec<String> = self.dirty_keys.iter().cloned().collect();
        self.dirty_keys.clear();
        (peer, keys)
    }

    /// Receive dirty keys from a peer.
    pub fn receive_dirty(&mut self, keys: &[String]) {
        for k in keys {
            self.dirty_keys.insert(k.clone());
        }
    }
}

// ── CoherencyController ──

/// Top‑level coherency controller combining MESI, CRDT, and gossip.
#[derive(Debug)]
pub struct CoherencyController {
    pub mesi: MesiController,
    pub crdt: CrdtManager,
    pub gossip: GossipManager,
}

impl CoherencyController {
    pub fn new(node_id: impl Into<String>) -> Self {
        Self {
            mesi: MesiController::new(),
            crdt: CrdtManager::new(node_id),
            gossip: GossipManager::new(),
        }
    }
}
