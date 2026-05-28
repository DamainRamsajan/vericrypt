use petgraph::graph::DiGraph; use std::collections::HashMap; use uuid::Uuid;
use crate::errors::VeriCryptError; use crate::types::{CryptoAsset, DependencyType};
pub struct CryptoGraph { graph: DiGraph<CryptoAsset, DependencyType>, assets: Vec<CryptoAsset> }
impl CryptoGraph {
    pub fn build(assets: Vec<CryptoAsset>) -> Result<Self, VeriCryptError> {
        let mut g = DiGraph::new(); let a = assets.clone();
        for asset in assets { g.add_node(asset); }
        Ok(CryptoGraph { graph: g, assets: a })
    }
    pub fn get_all_assets(&self) -> &Vec<CryptoAsset> { &self.assets }
    pub fn compute_shapley_values(&self) -> HashMap<Uuid, f64> {
        let n = self.graph.node_count(); if n == 0 { return HashMap::new(); }
        let s = 1.0 / n as f64;
        self.graph.node_indices().map(|i| (self.graph[i].asset_id, s)).collect()
    }
    pub fn node_count(&self) -> usize { self.graph.node_count() }
    pub fn edge_count(&self) -> usize { self.graph.edge_count() }
}
pub fn build_graph(assets: Vec<CryptoAsset>) -> Result<CryptoGraph, VeriCryptError> { CryptoGraph::build(assets) }
