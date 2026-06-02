//! Merkle integrity manager for the memory subsystem.
//!
//! Maintains a time‑aware Merkle tree over memory writes,
//! producing tamper‑evident proofs for audit trails.
//!
//! Uses blake3 for hashing, producing a 256‑bit (32‑byte) digest.
//! Based on ChronoMerkle design patterns.

/// A node in the Merkle tree.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct MerkleNode {
    hash: String,
    timestamp: u64,
    left: Option<Box<MerkleNode>>,
    right: Option<Box<MerkleNode>>,
}

/// Merkle integrity manager — one root per memory layer.
#[derive(Debug)]
pub struct MerkleIntegrityManager {
    /// The current leaves (content hashes) per layer, in insertion order.
    leaves: [Vec<(String, u64)>; 8],
    /// Cached Merkle roots per layer.
    roots: [Option<String>; 8],
}

impl MerkleIntegrityManager {
    pub fn new() -> Self {
        Self {
            leaves: Default::default(),
            roots: Default::default(),
        }
    }

    /// Update a layer's Merkle tree with a new content hash.
    pub fn update(&mut self, layer: u8, hash: &str, timestamp: u64) {
        let idx = layer as usize;
        self.leaves[idx].push((hash.to_string(), timestamp));
        // Recompute the root for this layer
        self.roots[idx] = Some(self.compute_root(&self.leaves[idx]));
    }

    /// Verify that a leaf hash is part of the tree for a given layer.
    pub fn verify(&self, layer: u8, hash: &str) -> bool {
        let idx = layer as usize;
        self.leaves[idx].iter().any(|(h, _)| h == hash)
    }

    /// Get the current root hash for a layer.
    pub fn root(&self, layer: u8) -> Option<&str> {
        self.roots[layer as usize].as_deref()
    }

    /// Compute a Merkle root from a list of (hash, timestamp) pairs.
    fn compute_root(&self, leaves: &[(String, u64)]) -> String {
        if leaves.is_empty() {
            return "0".repeat(64);
        }

        let mut hashes: Vec<String> = leaves.iter().map(|(h, _)| h.clone()).collect();

        // Build tree bottom‑up
        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in hashes.chunks(2) {
                let combined = if chunk.len() == 2 {
                    format!("{}{}", chunk[0], chunk[1])
                } else {
                    chunk[0].clone()
                };
                let hash = blake3::hash(combined.as_bytes());
                next_level.push(hex::encode(hash.as_bytes()));
            }
            hashes = next_level;
        }

        hashes[0].clone()
    }

    /// Generate a proof of inclusion for a leaf hash.
    /// Returns a vector of sibling hashes forming the proof path.
    pub fn generate_proof(&self, layer: u8, hash: &str) -> Option<Vec<String>> {
        let idx = layer as usize;
        let pos = self.leaves[idx].iter().position(|(h, _)| h == hash)?;

        let mut proof = Vec::new();
        let mut hashes: Vec<String> = self.leaves[idx].iter().map(|(h, _)| h.clone()).collect();
        let mut pos = pos;

        while hashes.len() > 1 {
            let sibling_idx = if pos % 2 == 0 { pos + 1 } else { pos - 1 };
            if sibling_idx < hashes.len() {
                proof.push(hashes[sibling_idx].clone());
            }
            pos /= 2;
            let mut next_level = Vec::new();
            for chunk in hashes.chunks(2) {
                let combined = if chunk.len() == 2 {
                    format!("{}{}", chunk[0], chunk[1])
                } else {
                    chunk[0].clone()
                };
                let hash = blake3::hash(combined.as_bytes());
                next_level.push(hex::encode(hash.as_bytes()));
            }
            hashes = next_level;
        }
        Some(proof)
    }
}
