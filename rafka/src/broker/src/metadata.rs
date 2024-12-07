use crate::network::{NodeType, NodeInfo, SuperNodeManager};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use rafka_core::{Result, Error};
use std::time::{Duration, SystemTime};

pub struct MetadataManager {
    node_id: String,
    supernode_manager: Arc<SuperNodeManager>, // From network layer
    local_metadata: Arc<DashMap<String, LocalMetadata>>,
    performance_metrics: Arc<DashMap<String, PerformanceMetrics>>,
}

#[derive(Clone, Debug)]
struct LocalMetadata {
    partition_id: String,
    is_hot: bool,
    in_memory: bool,
    message_rate: f64,
    last_access: SystemTime,
    size_bytes: u64,
}

#[derive(Clone, Debug)]
struct PerformanceMetrics {
    latency_ms: f64,
    throughput: u64,
    cache_hit_rate: f64,
    last_updated: SystemTime,
}

impl MetadataManager {
    pub async fn new(
        node_id: String, 
        supernode_manager: Arc<SuperNodeManager>
    ) -> Self {
        Self {
            node_id,
            supernode_manager,
            local_metadata: Arc::new(DashMap::new()),
            performance_metrics: Arc::new(DashMap::new()),
        }
    }

    pub async fn start(&self) -> Result<()> {
        // Start performance monitoring
        self.monitor_performance().await?;
        
        // Start hot path detection
        self.monitor_hot_paths().await?;
        
        Ok(())
    }

    async fn monitor_hot_paths(&self) -> Result<()> {
        let manager = self.clone();
        tokio::spawn(async move {
            loop {
                for (partition_id, metadata) in manager.local_metadata.iter() {
                    if metadata.message_rate > 1000.0 && !metadata.is_hot {
                        manager.convert_to_hot_path(&partition_id).await?;
                    } else if metadata.message_rate < 100.0 && metadata.is_hot {
                        manager.convert_to_cold_path(&partition_id).await?;
                    }
                }
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
            #[allow(unreachable_code)]
            Ok::<(), Error>(())
        });
        Ok(())
    }

    async fn monitor_performance(&self) -> Result<()> {
        let manager = self.clone();
        tokio::spawn(async move {
            loop {
                let metrics = manager.collect_performance_metrics().await?;
                
                // Report to supernode if we're a regular node
                if let Some(supernode) = manager.supernode_manager.get_parent_supernode().await? {
                    manager.report_metrics_to_supernode(&supernode, &metrics).await?;
                }
                
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
            #[allow(unreachable_code)]
            Ok::<(), Error>(())
        });
        Ok(())
    }

    async fn collect_performance_metrics(&self) -> Result<PerformanceMetrics> {
        let mut metrics = PerformanceMetrics {
            latency_ms: 0.0,
            throughput: 0,
            cache_hit_rate: 0.0,
            last_updated: SystemTime::now(),
        };

        // Collect metrics from local partitions
        for metadata in self.local_metadata.iter() {
            metrics.throughput += metadata.message_rate as u64;
            metrics.cache_hit_rate += if metadata.in_memory { 1.0 } else { 0.0 };
        }

        // Normalize metrics
        let partition_count = self.local_metadata.len();
        if partition_count > 0 {
            metrics.cache_hit_rate /= partition_count as f64;
        }

        Ok(metrics)
    }

    async fn convert_to_hot_path(&self, partition_id: &str) -> Result<()> {
        if let Some(mut metadata) = self.local_metadata.get_mut(partition_id) {
            metadata.is_hot = true;
            metadata.in_memory = true;
            
            // Notify supernode through network layer
            if let Some(supernode) = self.supernode_manager.get_parent_supernode().await? {
                self.supernode_manager.notify_hot_path_change(
                    &supernode,
                    partition_id,
                    true
                ).await?;
            }
        }
        Ok(())
    }

    async fn convert_to_cold_path(&self, partition_id: &str) -> Result<()> {
        if let Some(mut metadata) = self.local_metadata.get_mut(partition_id) {
            metadata.is_hot = false;
            
            if metadata.last_access.elapsed()? > Duration::from_secs(3600) {
                metadata.in_memory = false;
            }
            
            // Notify supernode through network layer
            if let Some(supernode) = self.supernode_manager.get_parent_supernode().await? {
                self.supernode_manager.notify_hot_path_change(
                    &supernode,
                    partition_id,
                    false
                ).await?;
            }
        }
        Ok(())
    }
} 