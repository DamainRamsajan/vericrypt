use crate::types::{ComplianceConfidence, ComplianceTheorem, ProofStatus, InventoryConfidence};

/// Compute compliance confidence as specified in Addendum 3 §3.
///
/// compliance_confidence = proof_confidence × inventory_confidence × regulatory_axiom_confidence
///
/// Where:
///   proof_confidence = fraction of theorems PROVED
///   inventory_confidence = visibility_score from inventory assessment
///   regulatory_axiom_confidence = 1.0 for axioms reviewed by Verity Regulatory Advisory Board
pub fn compute_compliance_confidence(
    theorems: &[ComplianceTheorem],
    inventory: &InventoryConfidence,
) -> ComplianceConfidence {
    let proof_confidence = if theorems.is_empty() {
        0.0
    } else {
        let proved = theorems
            .iter()
            .filter(|t| t.status == ProofStatus::Proved)
            .count() as f64;
        proved / theorems.len() as f64
    };

    let inventory_confidence = inventory.visibility_score;
    let regulatory_axiom_confidence = 1.0;

    ComplianceConfidence {
        proof_confidence,
        inventory_confidence,
        regulatory_axiom_confidence,
        composite_confidence: proof_confidence * inventory_confidence * regulatory_axiom_confidence,
    }
}

/// Compute inventory confidence from scan results.
pub fn compute_inventory_confidence(
    total_assets: u64,
    unreachable: u64,
    unsupported: &[String],
    encrypted: u64,
) -> InventoryConfidence {
    let mut visibility = 1.0_f64;

    if unreachable > 0 {
        visibility -= 0.05 * (unreachable as f64 / total_assets.max(1) as f64).min(1.0);
    }
    if !unsupported.is_empty() {
        visibility -= 0.10 * (unsupported.len() as f64 / 10.0).min(1.0);
    }
    if encrypted > 0 {
        visibility -= 0.05 * (encrypted as f64 / total_assets.max(1) as f64).min(1.0);
    }

    visibility = visibility.max(0.0).min(1.0);

    let confidence_level = if visibility > 0.95 {
        crate::types::ConfidenceLevel::Complete
    } else if visibility > 0.80 {
        crate::types::ConfidenceLevel::High
    } else if visibility > 0.50 {
        crate::types::ConfidenceLevel::Partial
    } else if visibility > 0.20 {
        crate::types::ConfidenceLevel::Low
    } else {
        crate::types::ConfidenceLevel::Unknown
    };

    InventoryConfidence {
        visibility_score: visibility,
        unreachable_assets: unreachable,
        unsupported_formats: unsupported.to_vec(),
        encrypted_uninspectable: encrypted,
        inferred_dependencies: 0,
        confidence_level,
    }
}
