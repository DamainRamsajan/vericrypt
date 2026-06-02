use crate::types::{CryptoAsset, DependencyType};
use std::collections::HashMap;
use uuid::Uuid;

/// Coalition types for structured Shapley value computation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoalitionType {
    TrustChain,
    Encryption,
    Configuration,
    Container,
    Isolated,
}

/// Assign a coalition type based on dependency edge type.
pub fn coalition_for_dependency(dep_type: &DependencyType) -> CoalitionType {
    match dep_type {
        DependencyType::Trusts | DependencyType::Signs => CoalitionType::TrustChain,
        DependencyType::Encrypts | DependencyType::Uses => CoalitionType::Encryption,
        DependencyType::Configures => CoalitionType::Configuration,
        DependencyType::Contains => CoalitionType::Container,
    }
}

/// Group assets into coalitions based on their dependency edges.
pub fn group_into_coalitions(
    assets: &[CryptoAsset],
    edges: &[(Uuid, Uuid, DependencyType)],
) -> HashMap<CoalitionType, Vec<Uuid>> {
    let mut coalitions: HashMap<CoalitionType, Vec<Uuid>> = HashMap::new();

    for (source_id, target_id, dep_type) in edges {
        let coalition = coalition_for_dependency(dep_type);
        coalitions.entry(coalition).or_default().extend([*source_id, *target_id]);
    }

    let connected: std::collections::HashSet<Uuid> = edges
        .iter()
        .flat_map(|(s, t, _)| [*s, *t])
        .collect();

    let isolated: Vec<Uuid> = assets
        .iter()
        .map(|a| a.asset_id)
        .filter(|id| !connected.contains(id))
        .collect();

    if !isolated.is_empty() {
        coalitions.insert(CoalitionType::Isolated, isolated);
    }

    coalitions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coalition_assignment() {
        assert_eq!(coalition_for_dependency(&DependencyType::Trusts), CoalitionType::TrustChain);
        assert_eq!(coalition_for_dependency(&DependencyType::Signs), CoalitionType::TrustChain);
        assert_eq!(coalition_for_dependency(&DependencyType::Encrypts), CoalitionType::Encryption);
        assert_eq!(coalition_for_dependency(&DependencyType::Uses), CoalitionType::Encryption);
        assert_eq!(coalition_for_dependency(&DependencyType::Configures), CoalitionType::Configuration);
        assert_eq!(coalition_for_dependency(&DependencyType::Contains), CoalitionType::Container);
    }
}
