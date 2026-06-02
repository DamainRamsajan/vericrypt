//! Cognitive Mesh — semantic infrastructure for multi-agent LLM systems.
//!
//! Based on the Mesh Memory Protocol (MMP, Xu, Apr 2026, arXiv:2604.19540).
//! Four composable primitives:
//!   1. CAT7 — fixed seven‑field schema for every Cognitive Memory Block (CMB)
//!   2. SVAF — role‑indexed field evaluation gate (Symbolic‑Vector Attention
//!      Fusion, arXiv:2604.03955)
//!   3. Inter‑agent lineage — content‑hash parent tracking, anti‑echo
//!   4. Remix — store receiver's own role‑evaluated understanding only

use std::collections::{HashMap, HashSet};

// ── CAT7: Cognitive Memory Block ──

/// The seven fields of a Cognitive Memory Block (CMB).
/// Fixed schema applied to every inter‑agent cognitive signal.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CognitiveMemoryBlock {
    /// The subject or focus of this cognitive block.
    pub focus: String,
    /// The specific issue or sub‑problem addressed.
    pub issue: String,
    /// The intent behind this communication.
    pub intent: String,
    /// The motivation or rationale.
    pub motivation: String,
    /// The commitment level (e.g., "firm", "tentative", "exploratory").
    pub commitment: String,
    /// The perspective or viewpoint of the sender.
    pub perspective: String,
    /// The emotional or affective tone.
    pub mood: String,
}

impl CognitiveMemoryBlock {
    /// Create a new CMB with all seven fields.
    pub fn new(
        focus: impl Into<String>,
        issue: impl Into<String>,
        intent: impl Into<String>,
        motivation: impl Into<String>,
        commitment: impl Into<String>,
        perspective: impl Into<String>,
        mood: impl Into<String>,
    ) -> Self {
        Self {
            focus: focus.into(),
            issue: issue.into(),
            intent: intent.into(),
            motivation: motivation.into(),
            commitment: commitment.into(),
            perspective: perspective.into(),
            mood: mood.into(),
        }
    }

    /// Compute a content hash for lineage tracking (blake3).
    pub fn content_hash(&self) -> String {
        let content = format!(
            "{}|{}|{}|{}|{}|{}|{}",
            self.focus,
            self.issue,
            self.intent,
            self.motivation,
            self.commitment,
            self.perspective,
            self.mood
        );
        let hash = blake3::hash(content.as_bytes());
        hex::encode(hash.as_bytes())
    }
}

// ── SVAF: Symbolic‑Vector Attention Fusion ──

/// Role‑indexed anchors for SVAF evaluation.
/// Each agent role has its own acceptance criteria for each CAT7 field.
#[derive(Debug, Clone)]
pub struct SvafAnchors {
    /// Role name (e.g., "researcher", "validator", "auditor").
    pub role: String,
    /// Per‑field acceptance thresholds: (field_name, threshold).
    pub field_thresholds: HashMap<String, f64>,
    /// Global acceptance threshold.
    pub global_threshold: f64,
}

/// The SVAF evaluator — gate for field‑level acceptance.
pub struct SvafEvaluator {
    /// Role‑indexed anchors.
    pub anchors: HashMap<String, SvafAnchors>,
}

impl SvafEvaluator {
    pub fn new() -> Self {
        Self {
            anchors: HashMap::new(),
        }
    }

    /// Register anchors for a role.
    pub fn register_role(&mut self, anchors: SvafAnchors) {
        self.anchors.insert(anchors.role.clone(), anchors);
    }

    /// Evaluate a CMB against a receiver role's anchors.
    /// Returns a map of field → score in [0, 1], plus a global accept/reject decision.
    pub fn evaluate(&self, role: &str, cmb: &CognitiveMemoryBlock) -> SvafResult {
        let anchors = match self.anchors.get(role) {
            Some(a) => a,
            None => {
                return SvafResult {
                    accepted: false,
                    scores: HashMap::new(),
                    reason: format!("No anchors for role '{}'", role),
                }
            }
        };

        let fields: HashMap<&str, &str> = HashMap::from([
            ("focus", cmb.focus.as_str()),
            ("issue", cmb.issue.as_str()),
            ("intent", cmb.intent.as_str()),
            ("motivation", cmb.motivation.as_str()),
            ("commitment", cmb.commitment.as_str()),
            ("perspective", cmb.perspective.as_str()),
            ("mood", cmb.mood.as_str()),
        ]);

        let mut scores = HashMap::new();
        let mut total = 0.0;
        let mut count = 0;

        for (field, value) in &fields {
            let score = self.score_field(value);
            let threshold = anchors.field_thresholds.get(*field).copied().unwrap_or(0.5);
            scores.insert(field.to_string(), score);
            if score >= threshold {
                total += 1.0;
            }
            count += 1;
        }

        let acceptance_ratio = if count > 0 { total / count as f64 } else { 0.0 };
        let accepted = acceptance_ratio >= anchors.global_threshold;

        SvafResult {
            accepted,
            scores,
            reason: if accepted {
                format!(
                    "Accepted by role '{}' (ratio {:.2})",
                    role, acceptance_ratio
                )
            } else {
                format!(
                    "Rejected by role '{}' (ratio {:.2} < {:.2})",
                    role, acceptance_ratio, anchors.global_threshold
                )
            },
        }
    }

    /// Score a single field value based on information density.
    fn score_field(&self, value: &str) -> f64 {
        if value.is_empty() {
            return 0.0;
        }
        let len = value.len() as f64;
        // Simple heuristic: longer, more specific answers score higher
        (len / 200.0).min(1.0)
    }
}

#[derive(Debug, Clone)]
pub struct SvafResult {
    pub accepted: bool,
    pub scores: HashMap<String, f64>,
    pub reason: String,
}

// ── Inter‑agent lineage ──

/// Lineage tracker — content‑hash chain for echo detection and provenance.
pub struct LineageTracker {
    /// All content hashes ever seen by this agent.
    pub seen_hashes: HashSet<String>,
    /// Parent→child relationships for tracing claim origins.
    pub parent_map: HashMap<String, Vec<String>>,
}

impl LineageTracker {
    pub fn new() -> Self {
        Self {
            seen_hashes: HashSet::new(),
            parent_map: HashMap::new(),
        }
    }

    /// Record a new CMB and its parent hashes.
    /// Returns `true` if this is a new (non‑echo) block.
    pub fn record(&mut self, hash: &str, parent_hashes: &[String]) -> bool {
        // Echo detection: if we've seen this hash before, it's an echo
        if self.seen_hashes.contains(hash) {
            return false;
        }
        self.seen_hashes.insert(hash.to_string());
        for parent in parent_hashes {
            self.parent_map
                .entry(parent.clone())
                .or_default()
                .push(hash.to_string());
        }
        true
    }

    /// Check whether a hash is an echo (already seen).
    pub fn is_echo(&self, hash: &str) -> bool {
        self.seen_hashes.contains(hash)
    }

    /// Trace the lineage chain from a hash back to origin.
    pub fn trace(&self, hash: &str) -> Vec<String> {
        let mut chain = vec![hash.to_string()];
        let mut current = hash.to_string();
        // Walk backwards through parent_map
        loop {
            let parents: Vec<String> = self
                .parent_map
                .iter()
                .filter(|(_, children)| children.contains(&current))
                .map(|(parent, _)| parent.clone())
                .collect();
            if parents.is_empty() {
                break;
            }
            current = parents[0].clone();
            chain.push(current.clone());
        }
        chain.reverse();
        chain
    }
}

// ── Remix processor ──

/// Remix processor — stores receiver's own understanding, never raw peer signal.
/// This is the key insight of MMP: each agent remixes accepted CMBs into
/// its own cognitive frame before storing.
pub struct RemixProcessor {
    /// The receiving agent's role.
    pub receiver_role: String,
}

impl RemixProcessor {
    pub fn new(role: impl Into<String>) -> Self {
        Self {
            receiver_role: role.into(),
        }
    }

    /// Remix an accepted CMB into the receiver's own understanding.
    /// The resulting remixed block is tagged with the receiver's role.
    pub fn remix(
        &self,
        cmb: &CognitiveMemoryBlock,
        _svaf_result: &SvafResult,
    ) -> CognitiveMemoryBlock {
        CognitiveMemoryBlock {
            focus: format!("[remixed by {}] {}", self.receiver_role, cmb.focus),
            issue: cmb.issue.clone(),
            intent: format!("Re‑interpreted: {}", cmb.intent),
            motivation: cmb.motivation.clone(),
            commitment: cmb.commitment.clone(),
            perspective: format!("{} (via {})", self.receiver_role, cmb.perspective),
            mood: cmb.mood.clone(),
        }
    }
}

// ── Cognitive Mesh ──

/// The full cognitive mesh — combines CAT7, SVAF, lineage, and remix.
pub struct CognitiveMesh {
    pub evaluator: SvafEvaluator,
    pub lineage: LineageTracker,
    pub remixer: RemixProcessor,
    /// Stored CMBs indexed by content hash.
    pub blocks: HashMap<String, CognitiveMemoryBlock>,
}

impl CognitiveMesh {
    pub fn new(receiver_role: impl Into<String>) -> Self {
        Self {
            evaluator: SvafEvaluator::new(),
            lineage: LineageTracker::new(),
            remixer: RemixProcessor::new(receiver_role),
            blocks: HashMap::new(),
        }
    }

    /// Process an incoming CMB: evaluate → lineage check → remix → store.
    pub fn process(
        &mut self,
        role: &str,
        cmb: CognitiveMemoryBlock,
        parent_hashes: &[String],
    ) -> ProcessResult {
        let hash = cmb.content_hash();

        // Echo detection
        if self.lineage.is_echo(&hash) {
            return ProcessResult {
                accepted: false,
                hash,
                reason: "Echo detected — already processed".into(),
                remixed: None,
            };
        }

        // SVAF evaluation
        let svaf = self.evaluator.evaluate(role, &cmb);
        if !svaf.accepted {
            return ProcessResult {
                accepted: false,
                hash,
                reason: svaf.reason,
                remixed: None,
            };
        }

        // Lineage recording
        self.lineage.record(&hash, parent_hashes);

        // Remix
        let remixed = self.remixer.remix(&cmb, &svaf);
        let remixed_hash = remixed.content_hash();

        // Store
        self.blocks.insert(remixed_hash.clone(), remixed.clone());

        ProcessResult {
            accepted: true,
            hash: remixed_hash,
            reason: svaf.reason,
            remixed: Some(remixed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessResult {
    pub accepted: bool,
    pub hash: String,
    pub reason: String,
    pub remixed: Option<CognitiveMemoryBlock>,
}
