use std::collections::{HashMap, HashSet};
use tokio::sync::{RwLock, broadcast};
use std::sync::Arc;
use rafka_core::{Result, Error};

pub struct ConsumerGroup {
    group_id: String,
    members: Arc<RwLock<HashMap<String, ConsumerMember>>>,
    partitions: Arc<RwLock<HashSet<u32>>>,
    coordinator: Arc<GroupCoordinator>,
    assignment_strategy: AssignmentStrategy,
}

struct ConsumerMember {
    consumer_id: String,
    heartbeat: SystemTime,
    assigned_partitions: HashSet<u32>,
}

#[derive(Clone)]
enum AssignmentStrategy {
    RangeAssignment,
    RoundRobin,
    Sticky,
}

impl ConsumerGroup {
    pub async fn new(group_id: String, coordinator: Arc<GroupCoordinator>) -> Self {
        Self {
            group_id,
            members: Arc::new(RwLock::new(HashMap::new())),
            partitions: Arc::new(RwLock::new(HashSet::new())),
            coordinator,
            assignment_strategy: AssignmentStrategy::RangeAssignment,
        }
    }

    pub async fn join(&self, consumer_id: String) -> Result<Vec<u32>> {
        let mut members = self.members.write().await;
        members.insert(consumer_id.clone(), ConsumerMember {
            consumer_id: consumer_id.clone(),
            heartbeat: SystemTime::now(),
            assigned_partitions: HashSet::new(),
        });

        // Trigger rebalance
        self.rebalance().await
    }

    pub async fn leave(&self, consumer_id: &str) -> Result<()> {
        let mut members = self.members.write().await;
        members.remove(consumer_id);

        // Trigger rebalance
        self.rebalance().await?;
        Ok(())
    }

    async fn rebalance(&self) -> Result<Vec<u32>> {
        let members = self.members.read().await;
        let partitions = self.partitions.read().await;

        match self.assignment_strategy {
            AssignmentStrategy::RangeAssignment => {
                self.assign_range(&members, &partitions).await
            }
            AssignmentStrategy::RoundRobin => {
                self.assign_round_robin(&members, &partitions).await
            }
            AssignmentStrategy::Sticky => {
                self.assign_sticky(&members, &partitions).await
            }
        }
    }

    async fn assign_range(
        &self,
        members: &HashMap<String, ConsumerMember>,
        partitions: &HashSet<u32>
    ) -> Result<Vec<u32>> {
        // Implement range assignment strategy
        let mut assignments = Vec::new();
        let partition_count = partitions.len();
        let member_count = members.len();
        
        if member_count == 0 {
            return Ok(vec![]);
        }

        let partitions_per_consumer = partition_count / member_count;
        let remainder = partition_count % member_count;

        // Sort partitions and members for consistent assignment
        let mut sorted_partitions: Vec<_> = partitions.iter().collect();
        sorted_partitions.sort();

        let mut sorted_members: Vec<_> = members.keys().collect();
        sorted_members.sort();

        // Assign partitions to consumers
        let mut current_partition = 0;
        for (i, member_id) in sorted_members.iter().enumerate() {
            let mut member_partitions = partitions_per_consumer;
            if i < remainder {
                member_partitions += 1;
            }

            for _ in 0..member_partitions {
                if current_partition < sorted_partitions.len() {
                    assignments.push(**sorted_partitions[current_partition]);
                    current_partition += 1;
                }
            }
        }

        Ok(assignments)
    }
}
