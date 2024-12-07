use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use rafka_core::{Message, Result, Error};
use std::time::{Duration, SystemTime};
use crate::memory_store::{InMemoryStore, StoreResponse};

pub struct RoutingOptimizer {
    memory_store: Arc<InMemoryStore>,
    route_cache: Arc<DashMap<String, RouteInfo>>,
    partition_stats: Arc<DashMap<String, PartitionStats>>,
}

struct RouteInfo {
    preferred_node: String,
    alternate_nodes: Vec<String>,
    last_updated: SystemTime,
    performance_score: f64,
}

struct PartitionStats {
    hit_rate: f64,
    avg_latency: Duration,
    throughput: u64,
    last_overflow: Option<SystemTime>,
}

impl RoutingOptimizer {
    pub fn new(memory_store: Arc<InMemoryStore>) -> Self {
        Self {
            memory_store,
            route_cache: Arc::new(DashMap::new()),
            partition_stats: Arc::new(DashMap::new()),
        }
    }

    pub async fn optimize_route(&self, message: &Message) -> Result<RoutingDecision> {
        // Fast path: Check in-memory store first
        match self.memory_store.get_or_route(&message.topic).await? {
            StoreResponse::InMemory(_) => {
                self.update_stats(&message.topic, true).await;
                return Ok(RoutingDecision::UseMemoryStore);
            }
            StoreResponse::NeedsRouting => {
                self.update_stats(&message.topic, false).await;
            }
        }

        // Check route cache
        if let Some(route) = self.route_cache.get(&message.topic) {
            if route.performance_score > 0.8 { // High performance threshold
                return Ok(RoutingDecision::DirectRoute(route.preferred_node.clone()));
            }
        }

        // Determine optimal route based on current conditions
        self.calculate_optimal_route(message).await
    }

    async fn calculate_optimal_route(&self, message: &Message) -> Result<RoutingDecision> {
        let stats = self.partition_stats.get(&message.topic);
        
        match stats {
            Some(stats) => {
                if stats.hit_rate > 0.7 && stats.avg_latency < Duration::from_millis(10) {
                    // High performance path
                    Ok(RoutingDecision::DirectRoute(
                        self.select_best_node(&message.topic).await?
                    ))
                } else if let Some(last_overflow) = stats.last_overflow {
                    if last_overflow.elapsed().unwrap() < Duration::from_secs(60) {
                        // Recent overflow, use distributed path
                        Ok(RoutingDecision::DistributedRoute(
                            self.get_available_nodes(&message.topic).await?
                        ))
                    } else {
                        Ok(RoutingDecision::StandardBrokerRoute)
                    }
                } else {
                    Ok(RoutingDecision::StandardBrokerRoute)
                }
            }
            None => Ok(RoutingDecision::StandardBrokerRoute)
        }
    }

    async fn update_stats(&self, topic: &str, was_hit: bool) {
        let mut stats = self.partition_stats
            .entry(topic.to_string())
            .or_insert_with(|| PartitionStats {
                hit_rate: 0.0,
                avg_latency: Duration::from_millis(0),
                throughput: 0,
                last_overflow: None,
            });

        // Update hit rate with exponential moving average
        const ALPHA: f64 = 0.1;
        stats.hit_rate = stats.hit_rate * (1.0 - ALPHA) + (if was_hit { 1.0 } else { 0.0 }) * ALPHA;
        stats.throughput += 1;
    }

    pub async fn record_overflow(&self, topic: &str) {
        if let Some(mut stats) = self.partition_stats.get_mut(topic) {
            stats.last_overflow = Some(SystemTime::now());
        }
    }
}

#[derive(Debug, Clone)]
pub enum RoutingDecision {
    UseMemoryStore,
    DirectRoute(String),
    DistributedRoute(Vec<String>),
    StandardBrokerRoute,
} 