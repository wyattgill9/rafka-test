use crate::core::Result;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

//MAYBE THIS IS NOT NEEDED bc were doing decentralized routing


pub struct LoadBalancer {
    node_stats: Arc<DashMap<String, NodeStats>>,
    strategy: LoadBalancingStrategy,
    health_checker: Arc<HealthChecker>,
}

impl LoadBalancer {
    pub async fn select_node(&self, message_key: Option<&[u8]>) -> Result<String> {
        match self.strategy {
            LoadBalancingStrategy::ConsistentHashing => {
                // Use consistent hashing for sticky routing when key present
                if let Some(key) = message_key {
                    return self.consistent_hash_node(key);
                }
                self.least_loaded_node().await
            }
            LoadBalancingStrategy::RoundRobin => {
                self.next_round_robin_node().await
            }
            LoadBalancingStrategy::LeastLoaded => {
                self.least_loaded_node().await
            }
        }
    }

    async fn least_loaded_node(&self) -> Result<String> {
        self.node_stats
            .iter()
            .filter(|node| self.health_checker.is_healthy(&node.key()))
            .min_by_key(|node| node.value().current_load)
            .map(|node| node.key().clone())
            .ok_or_else(|| Error::NoAvailableNodes)
    }
}
