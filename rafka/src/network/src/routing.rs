use crate::p2p::{NodeInfo, P2PNetwork};
use dashmap::DashMap;
use std::sync::Arc;

pub struct RoutingTable {
    // Prefix routing table - each row i contains nodes that share i digits with local node
    rows: Vec<DashMap<String, NodeInfo>>,
    leaf_set: Arc<DashMap<String, NodeInfo>>,
    neighborhood_set: Arc<DashMap<String, NodeInfo>>,
}

impl RoutingTable {
    pub fn new(node_id: &str, b: u8) -> Self {
        let rows = (0..b).map(|_| DashMap::new()).collect();
        
        Self {
            rows,
            leaf_set: Arc::new(DashMap::new()),
            neighborhood_set: Arc::new(DashMap::new()),
        }
    }

    pub fn route(&self, target: &str) -> Option<NodeInfo> {
        // First check leaf set
        if let Some(node) = self.route_via_leaf_set(target) {
            return Some(node);
        }

        // Then try prefix routing
        self.route_via_prefix(target)
    }

    fn route_via_leaf_set(&self, target: &str) -> Option<NodeInfo> {
        self.leaf_set.iter()
            .min_by_key(|entry| calculate_distance(entry.key(), target))
            .map(|entry| entry.value().clone())
    }

    fn route_via_prefix(&self, target: &str) -> Option<NodeInfo> {
        let shared_prefix = calculate_shared_prefix(target);
        
        if let Some(row) = self.rows.get(shared_prefix) {
            row.iter()
                .min_by_key(|entry| calculate_distance(entry.key(), target))
                .map(|entry| entry.value().clone())
        } else {
            None
        }
    }
}

fn calculate_distance(id1: &str, id2: &str) -> u64 {
    // Simple XOR distance metric
    let n1 = u64::from_str_radix(id1, 16).unwrap_or(0);
    let n2 = u64::from_str_radix(id2, 16).unwrap_or(0);
    n1 ^ n2
}

fn calculate_shared_prefix(id: &str) -> usize {
    // Calculate number of shared prefix digits
    id.chars().take_while(|c| c.is_ascii_hexdigit()).count()
} 