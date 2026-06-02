//! Deterministic random number generator for the AGENT-SEED VM.
//!
//! Uses `rand_pcg::Pcg64Mcg` seeded with a user-provided `u64`.
//! This ensures that identical seeds produce identical execution traces
//! — a critical requirement for the "Deterministic Replay" system axiom.

use rand::Rng;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;

/// A deterministic, seedable PRNG backed by PCG64 (Mcg variant).
///
/// # Security
///
/// This is **not** cryptographically secure. It is designed for
/// deterministic replay, not for key generation or secure randomness.
/// Cryptographic randomness should use the host-provided entropy source.
#[derive(Debug, Clone)]
pub struct DeterministicRng {
    inner: Pcg64Mcg,
    seed: u64,
    /// Counter of how many random values have been drawn.
    pub draw_count: u64,
}

impl DeterministicRng {
    /// Create a new deterministic RNG from a 64-bit seed.
    pub fn new(seed: u64) -> Self {
        // Use a simple mixing step to avoid zero-seed issues
        let mixed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let inner = Pcg64Mcg::seed_from_u64(mixed);
        Self {
            inner,
            seed,
            draw_count: 0,
        }
    }

    /// Return the original seed (for replay verification).
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Generate a uniformly distributed `u64`.
    pub fn next_u64(&mut self) -> u64 {
        self.draw_count += 1;
        self.inner.gen()
    }

    /// Generate a uniformly distributed `f64` in [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        self.draw_count += 1;
        self.inner.gen()
    }

    /// Generate a uniformly distributed `i64`.
    pub fn next_i64(&mut self) -> i64 {
        self.draw_count += 1;
        self.inner.gen()
    }

    /// Generate a uniformly distributed `u32`.
    pub fn next_u32(&mut self) -> u32 {
        self.draw_count += 1;
        self.inner.gen()
    }

    /// Generate a uniformly distributed `i32`.
    pub fn next_i32(&mut self) -> i32 {
        self.draw_count += 1;
        self.inner.gen()
    }
}
