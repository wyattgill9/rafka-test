use crate::core::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct ConsumerGroup {
    group_id: String,
    members: RwLock<HashMap<String, ConsumerMember>>,
    partition_strategy: PartitionStrategy,
}

impl ConsumerGroup {
    pub async fn join(&self, consumer_id: String) -> Result<Vec<u32>> {
        let mut members = self.members.write().await;
        members.insert(consumer_id.clone(), ConsumerMember::new());
        
        // Dynamically reassign partitions
        self.rebalance_partitions(&members).await
    }

    async fn rebalance_partitions(
        &self,
        members: &HashMap<String, ConsumerMember>,
    ) -> Result<Vec<u32>> {
        match self.partition_strategy {
            PartitionStrategy::RangeAssignment => {
                self.assign_range_partitions(members).await
            }
            PartitionStrategy::RoundRobin => {
                self.assign_round_robin_partitions(members).await
            }
        }
    }
}
