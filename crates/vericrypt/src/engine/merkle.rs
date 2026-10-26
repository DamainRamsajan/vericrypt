//! Binary Merkle tree over report items (BLAKE3, domain-separated leaves and nodes).
//! An odd node at any level is carried up unchanged (never duplicated), which avoids the
//! duplicate-leaf ambiguity of naive Bitcoin-style trees.
use serde::{Deserialize, Serialize};

pub type Hash = [u8; 32];

pub fn leaf(kind: &str, id: &str, canonical: &[u8]) -> Hash {
    let mut h = blake3::Hasher::new();
    h.update(&[0x00]);
    h.update(kind.as_bytes());
    h.update(&[0x00]);
    h.update(id.as_bytes());
    h.update(&[0x00]);
    h.update(canonical);
    *h.finalize().as_bytes()
}

fn node(l: &Hash, r: &Hash) -> Hash {
    let mut h = blake3::Hasher::new();
    h.update(&[0x01]);
    h.update(l);
    h.update(r);
    *h.finalize().as_bytes()
}

fn levels(leaves: &[Hash]) -> Vec<Vec<Hash>> {
    let mut lv = vec![leaves.to_vec()];
    while lv.last().unwrap().len() > 1 {
        let cur = lv.last().unwrap();
        let mut next = Vec::with_capacity((cur.len() + 1) / 2);
        for pair in cur.chunks(2) {
            next.push(if pair.len() == 2 { node(&pair[0], &pair[1]) } else { pair[0] });
        }
        lv.push(next);
    }
    lv
}

pub fn root(leaves: &[Hash]) -> Hash {
    if leaves.is_empty() {
        return *blake3::hash(b"vericrypt-empty-tree").as_bytes();
    }
    *levels(leaves).last().unwrap().first().unwrap()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub sibling: String,
    /// true when the sibling is the left child
    pub left: bool,
}

pub fn proof(leaves: &[Hash], mut idx: usize) -> Vec<Step> {
    let mut steps = Vec::new();
    for level in levels(leaves).iter().take_while(|l| l.len() > 1) {
        if idx % 2 == 0 {
            if idx + 1 < level.len() {
                steps.push(Step { sibling: hex::encode(level[idx + 1]), left: false });
            }
        } else {
            steps.push(Step { sibling: hex::encode(level[idx - 1]), left: true });
        }
        idx /= 2;
    }
    steps
}

pub fn verify_proof(leaf_hash: &Hash, steps: &[Step], expected_root: &Hash) -> bool {
    let mut cur = *leaf_hash;
    for s in steps {
        let sib = match hex::decode(&s.sibling) {
            Ok(b) if b.len() == 32 => {
                let mut a = [0u8; 32];
                a.copy_from_slice(&b);
                a
            }
            _ => return false,
        };
        cur = if s.left { node(&sib, &cur) } else { node(&cur, &sib) };
    }
    &cur == expected_root
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn proofs_verify_for_every_leaf_and_sizes() {
        for n in 1..=9usize {
            let leaves: Vec<Hash> = (0..n).map(|i| leaf("t", &i.to_string(), b"x")).collect();
            let r = root(&leaves);
            for i in 0..n {
                let p = proof(&leaves, i);
                assert!(verify_proof(&leaves[i], &p, &r), "n={} i={}", n, i);
            }
        }
    }
    #[test]
    fn tampered_leaf_fails() {
        let leaves: Vec<Hash> = (0..5).map(|i| leaf("t", &i.to_string(), b"x")).collect();
        let r = root(&leaves);
        let p = proof(&leaves, 2);
        let bad = leaf("t", "2", b"y");
        assert!(!verify_proof(&bad, &p, &r));
    }
}
