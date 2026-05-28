use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use uuid::Uuid;
use crate::errors::VeriCryptError;
use crate::types::{CryptoAsset, CryptoDependency, DependencyType};

pub struct CryptoGraph {
    graph: DiGraph<CryptoAsset, DependencyType>,
    asset_index: HashMap<Uuid, NodeIndex>,
    assets: Vec<CryptoAsset>,
}

impl CryptoGraph {
    pub fn build(assets: Vec<CryptoAsset>) -> Result<Self, VeriCryptError> {
        let mut graph = DiGraph::new();
        let mut asset_index = HashMap::new();
        let assets_clone = assets.clone();

        for asset in assets {
            let idx = graph.add_node(asset.clone());
            asset_index.insert(asset.asset_id, idx);
        }

        let crypto_graph = CryptoGraph {
            graph,
            asset_index,
            assets: assets_clone,
        };

        tracing::info!(
            node_count = crypto_graph.graph.node_count(),
            edge_count = crypto_graph.graph.edge_count(),
            "Knowledge graph built"
        );

        Ok(crypto_graph)
    }

    pub fn get_all_assets(&self) -> &Vec<CryptoAsset> {
        &self.assets
    }

    pub fn compute_shapley_values(&self) -> HashMap<Uuid, f64> {
        let node_count = self.graph.node_count();
        if node_count == 0 {
            return HashMap::new();
        }

        let equal_share = 1.0 / node_count as f64;
        let mut shapley = HashMap::new();
        for node_idx in self.graph.node_indices() {
            let asset = &self.graph[node_idx];
            shapley.insert(asset.asset_id, equal_share);
        }
        shapley
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
}

pub fn build_graph(assets: Vec<CryptoAsset>) -> Result<CryptoGraph, VeriCryptError> {
    CryptoGraph::build(assets)
}
