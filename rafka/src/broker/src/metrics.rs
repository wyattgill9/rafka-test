use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use rafka_core::{Result, Error};
use std::time::{Duration, SystemTime};
use metrics::{counter, gauge, histogram};

pub struct PerformanceMonitor {
    node_id: String,
    metrics_store: Arc<DashMap<String, PartitionMetrics>>,
    memory_tracker: Arc<MemoryTracker>,
    sampling_interval: Duration,
}

#[derive(Clone, Debug)]
struct PartitionMetrics {
    // Throughput metrics
    messages_per_second: f64,
    bytes_per_second: f64,
    
    // Latency metrics
    p50_latency_ms: f64,
    p95_latency_ms: f64,
    p99_latency_ms: f64,
    
    // Memory metrics
    memory_usage_bytes: u64,
    cache_hit_ratio: f64,
    
    // I/O metrics
    disk_reads_per_sec: f64,
    disk_writes_per_sec: f64,
    
    // Network metrics
    network_in_bytes: u64,
    network_out_bytes: u64,
    
    last_updated: SystemTime,
}

struct MemoryTracker {
    total_memory: u64,
    used_memory: RwLock<u64>,
    memory_pressure: RwLock<f64>,
    eviction_threshold: f64,
}

impl PerformanceMonitor {
    pub async fn new(node_id: String) -> Self {
        Self {
            node_id,
            metrics_store: Arc::new(DashMap::new()),
            memory_tracker: Arc::new(MemoryTracker::new()),
            sampling_interval: Duration::from_secs(1),
        }
    }

    pub async fn start(&self) -> Result<()> {
        // Start metrics collection
        self.start_metrics_collection().await?;
        
        // Start memory optimization
        self.start_memory_optimization().await?;
        
        Ok(())
    }

    async fn collect_partition_metrics(&self, partition_id: &str) -> Result<PartitionMetrics> {
        let mut metrics = PartitionMetrics {
            messages_per_second: gauge!("messages_per_second", partition_id),
            bytes_per_second: gauge!("bytes_per_second", partition_id),
            p50_latency_ms: histogram!("latency_ms", 0.5, partition_id),
            p95_latency_ms: histogram!("latency_ms", 0.95, partition_id),
            p99_latency_ms: histogram!("latency_ms", 0.99, partition_id),
            memory_usage_bytes: gauge!("memory_usage_bytes", partition_id),
            cache_hit_ratio: gauge!("cache_hit_ratio", partition_id),
            disk_reads_per_sec: counter!("disk_reads", partition_id),
            disk_writes_per_sec: counter!("disk_writes", partition_id),
            network_in_bytes: counter!("network_in_bytes", partition_id),
            network_out_bytes: counter!("network_out_bytes", partition_id),
            last_updated: SystemTime::now(),
        };

        // Calculate derived metrics
        metrics.cache_hit_ratio = self.calculate_cache_hit_ratio(partition_id).await?;
        
        Ok(metrics)
    }

    async fn optimize_memory(&self) -> Result<()> {
        let pressure = *self.memory_tracker.memory_pressure.read().await;
        
        if pressure > self.memory_tracker.eviction_threshold {
            // Find candidates for eviction
            let candidates = self.find_eviction_candidates().await?;
            
            for candidate in candidates {
                // Evict from memory but keep in Scylla if hot
                self.evict_partition(&candidate).await?;
                
                // Update metrics
                if let Some(mut metrics) = self.metrics_store.get_mut(&candidate) {
                    metrics.memory_usage_bytes = 0;
                    metrics.last_updated = SystemTime::now();
                }
            }
        }
        Ok(())
    }

    async fn find_eviction_candidates(&self) -> Result<Vec<String>> {
        let mut candidates = Vec::new();
        let mut partition_scores = Vec::new();

        // Score each partition based on:
        // - Access frequency
        // - Memory usage
        // - Cache hit ratio
        // - Message rate
        for entry in self.metrics_store.iter() {
            let score = self.calculate_partition_score(entry.value());
            partition_scores.push((entry.key().clone(), score));
        }

        // Sort by score (lower = better eviction candidate)
        partition_scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // Take lowest scoring partitions until we free enough memory
        let mut memory_to_free = self.calculate_memory_to_free().await?;
        for (partition_id, _) in partition_scores {
            if let Some(metrics) = self.metrics_store.get(&partition_id) {
                candidates.push(partition_id);
                memory_to_free = memory_to_free.saturating_sub(metrics.memory_usage_bytes);
                if memory_to_free == 0 {
                    break;
                }
            }
        }

        Ok(candidates)
    }

    fn calculate_partition_score(&self, metrics: &PartitionMetrics) -> f64 {
        let recency_score = 1.0 / (1.0 + metrics.last_updated.elapsed().unwrap().as_secs_f64());
        let throughput_score = metrics.messages_per_second / 1000.0;
        let cache_score = metrics.cache_hit_ratio;
        
        // Weight the factors
        recency_score * 0.4 + throughput_score * 0.4 + cache_score * 0.2
    }

    async fn calculate_memory_to_free(&self) -> Result<u64> {
        let used = *self.memory_tracker.used_memory.read().await;
        let target = (self.memory_tracker.total_memory as f64 * 0.8) as u64;
        
        if used > target {
            Ok(used - target)
        } else {
            Ok(0)
        }
    }
}

impl MemoryTracker {
    fn new() -> Self {
        Self {
            total_memory: sys_info::mem_info().unwrap().total,
            used_memory: RwLock::new(0),
            memory_pressure: RwLock::new(0.0),
            eviction_threshold: 0.85,
        }
    }
} 