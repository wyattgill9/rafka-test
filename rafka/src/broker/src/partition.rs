use crate::core::Result;
use consistent_hash_ring::{HashRing, Node};
use std::sync::Arc;

pub struct PartitionManager {
    ring: Arc<RwLock<HashRing<BrokerNode>>>,
    partition_count: usize,
    replication_factor: usize,
}

impl PartitionManager {
    pub fn new(partition_count: usize, replication_factor: usize) -> Self {
        Self {
            ring: Arc::new(RwLock::new(HashRing::new())),
            partition_count,
            replication_factor,
        }
    }

    pub async fn get_partition_owners(&self, key: &[u8]) -> Vec<BrokerNode> {
        let ring = self.ring.read().await;
        ring.get_nodes(key, self.replication_factor)
    }

    pub async fn add_broker(&self, broker: BrokerNode) -> Result<()> {
        let mut ring = self.ring.write().await;
        ring.add_node(broker);
        self.rebalance_partitions().await
    }

    async fn rebalance_partitions(&self) -> Result<()> {
        // Minimal data movement during rebalancing (idk lol)
        let ring = self.ring.read().await;
        for partition in 0..self.partition_count {
            let key = format!("partition-{}", partition);
            let new_owners = ring.get_nodes(&key.as_bytes(), self.replication_factor);
            // Implement rebalancing logic here (idk lol)
        }
        Ok(())
    }
}
