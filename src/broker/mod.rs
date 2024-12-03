mod partition;
mod load_balancer;

use crate::{Message, Result, BrokerNode};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct StatelessBroker {
    active_producers: DashMap<String, mpsc::Sender<Message>>,
    active_consumers: DashMap<String, Vec<mpsc::Sender<Message>>>,
    partition_manager: Arc<partition::PartitionManager>,
    load_balancer: Arc<load_balancer::LoadBalancer>,
}

impl StatelessBroker {
    pub fn new(config: crate::Config) -> Self {
        Self {
            active_producers: DashMap::new(),
            active_consumers: DashMap::new(),
            partition_manager: Arc::new(partition::PartitionManager::new(
                config.partition_count,
                config.replication_factor,
            )),
            load_balancer: Arc::new(load_balancer::LoadBalancer::new(config)),
        }
    }

    pub async fn route_message(&self, msg: Message) -> Result<()> {
        let partition = self.partition_manager.get_partition(&msg.key).await?;
        let consumers = self.active_consumers.get(&partition.to_string())
            .map(|c| c.value().clone())
            .unwrap_or_default();

        for consumer in consumers {
            if let Err(e) = consumer.send(msg.clone()).await {
                tracing::error!("Failed to send message: {}", e);
            }
        }
        Ok(())
    }
} 