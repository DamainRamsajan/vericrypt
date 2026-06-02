//! Prism evolutionary memory substrate.
//!
//! Based on Prism (Apr 2026): unifies four independently developed
//! paradigms under a single decision‑theoretic framework with eight
//! interconnected subsystems.
//!
//! Four paradigms:
//!   1. Layered file persistence (hot / warm / cold)
//!   2. Vector‑augmented semantic
//!   3. Graph‑structured relational
//!   4. Multi‑agent evolutionary search
//!
//! Eight subsystems: encoder, indexer, retriever, consolidator,
//! pruner, evolver, verifier, governor.

use std::collections::HashMap;

/// Configuration tier for memory persistence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistenceTier {
    Hot,  // Frequently accessed, in‑memory.
    Warm, // Infrequently accessed, cached on disk.
    Cold, // Rarely accessed, compressed archive.
}

/// The Prism evolutionary substrate.
pub struct PrismSubstrate {
    /// Tier assignment per key.
    pub tiers: HashMap<String, PersistenceTier>,
    /// Access frequency per key.
    pub frequencies: HashMap<String, u64>,
    /// Promotion threshold (access count) for cold → warm → hot.
    pub promotion_threshold: u64,
    /// Demotion interval (clock ticks without access).
    pub demotion_interval: u64,
    /// Last access clock per key.
    pub last_access: HashMap<String, u64>,
    /// Current clock.
    clock: u64,
    /// Subsystem health status.
    pub subsystems: SubsystemHealth,
}

#[derive(Debug, Clone)]
pub struct SubsystemHealth {
    pub encoder: bool,
    pub indexer: bool,
    pub retriever: bool,
    pub consolidator: bool,
    pub pruner: bool,
    pub evolver: bool,
    pub verifier: bool,
    pub governor: bool,
}

impl Default for SubsystemHealth {
    fn default() -> Self {
        Self {
            encoder: true,
            indexer: true,
            retriever: true,
            consolidator: true,
            pruner: true,
            evolver: true,
            verifier: true,
            governor: true,
        }
    }
}

impl PrismSubstrate {
    pub fn new() -> Self {
        Self {
            tiers: HashMap::new(),
            frequencies: HashMap::new(),
            promotion_threshold: 10,
            demotion_interval: 1000,
            last_access: HashMap::new(),
            clock: 0,
            subsystems: SubsystemHealth::default(),
        }
    }

    /// Tick the internal clock.
    pub fn tick(&mut self) -> u64 {
        self.clock += 1;
        self.clock
    }

    /// Access a key — increment frequency, update last access, handle promotion.
    pub fn access(&mut self, key: &str) {
        *self.frequencies.entry(key.to_string()).or_insert(0) += 1;
        self.last_access.insert(key.to_string(), self.clock);

        // Promote if threshold exceeded
        if self.frequencies[key] >= self.promotion_threshold {
            let current = self
                .tiers
                .get(key)
                .copied()
                .unwrap_or(PersistenceTier::Cold);
            let new_tier = match current {
                PersistenceTier::Cold => PersistenceTier::Warm,
                PersistenceTier::Warm => PersistenceTier::Hot,
                PersistenceTier::Hot => PersistenceTier::Hot,
            };
            self.tiers.insert(key.to_string(), new_tier);
        }
    }

    /// Apply demotion: keys not accessed within `demotion_interval` are demoted.
    pub fn demote_stale(&mut self) {
        let clock = self.clock;
        let interval = self.demotion_interval;
        let mut to_demote: Vec<String> = Vec::new();

        for (key, last) in &self.last_access {
            if clock.saturating_sub(*last) > interval {
                let current = self.tiers.get(key).copied().unwrap_or(PersistenceTier::Hot);
                let new_tier = match current {
                    PersistenceTier::Hot => PersistenceTier::Warm,
                    PersistenceTier::Warm => PersistenceTier::Cold,
                    PersistenceTier::Cold => PersistenceTier::Cold,
                };
                if new_tier != current {
                    to_demote.push(key.clone());
                }
            }
        }

        for key in to_demote {
            let current = self
                .tiers
                .get(&key)
                .copied()
                .unwrap_or(PersistenceTier::Hot);
            let new_tier = match current {
                PersistenceTier::Hot => PersistenceTier::Warm,
                PersistenceTier::Warm => PersistenceTier::Cold,
                PersistenceTier::Cold => PersistenceTier::Cold,
            };
            self.tiers.insert(key, new_tier);
        }
    }

    /// Evolutionary search: optimise memory configuration.
    /// Placeholder — full implementation uses genetic algorithms.
    pub fn evolve(&mut self, _population: Vec<HashMap<String, PersistenceTier>>) {
        // In production, this would run a genetic algorithm to optimise
        // tier assignments, decay rates, and retrieval strategies.
        self.tick();
    }
}
