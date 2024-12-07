use crate::p2p::NodeInfo;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use rafka_core::{Result, Error};

pub struct DHT {
    node_id: String,
    routing_table: Arc<RoutingTable>,
    data_store: Arc<DashMap<String, Vec<u8>>>,
    replication_factor: usize,
}

impl DHT {
    pub fn new(node_id: String, replication_factor: usize) -> Self {
        Self {
            node_id,
            routing_table: Arc::new(RoutingTable::new(&node_id, 4)), // 16 bits per digit
            data_store: Arc::new(DashMap::new()),
            replication_factor,
        }
    }

    pub async fn put(&self, key: String, value: Vec<u8>) -> Result<()> {
        // Find target nodes for replication
        let target_nodes = self.find_target_nodes(&key, self.replication_factor);
        
        // Store locally if we're one of the target nodes
        if target_nodes.iter().any(|node| node.id == self.node_id) {
            self.data_store.insert(key.clone(), value.clone());
        }

        // Replicate to other target nodes
        let mut tasks = Vec::new();
        for node in target_nodes {
            if node.id != self.node_id {
                let key = key.clone();
                let value = value.clone();
                tasks.push(tokio::spawn(async move {
                    // Send replication request to node
                    self.replicate_to_node(&node, &key, &value).await
                }));
            }
        }

        // Wait for majority acknowledgment
        let results = futures::future::join_all(tasks).await;
        let successful = results.iter()
            .filter(|r| r.as_ref().map_or(false, |inner| inner.is_ok()))
            .count();

        if successful >= self.replication_factor / 2 {
            Ok(())
        } else {
            Err(Error::Replication("Failed to replicate to majority".into()))
        }
    }

    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        // Check local store first
        if let Some(value) = self.data_store.get(key) {
            return Ok(Some(value.clone()));
        }

        // Find nodes that should have the data
        let target_nodes = self.find_target_nodes(key, self.replication_factor);
        
        // Query each node until we find the value
        for node in target_nodes {
            if node.id != self.node_id {
                if let Ok(Some(value)) = self.query_node(&node, key).await {
                    return Ok(Some(value));
                }
            }
        }

        Ok(None)
    }

    fn find_target_nodes(&self, key: &str, count: usize) -> Vec<NodeInfo> {
        let mut nodes = Vec::new();
        let mut current_node = self.routing_table.route(key);

        while nodes.len() < count {
            if let Some(node) = current_node {
                if !nodes.iter().any(|n: &NodeInfo| n.id == node.id) {
                    nodes.push(node.clone());
                }
                // Get next closest node
                current_node = self.routing_table.next_closest(key, &nodes);
            } else {
                break;
            }
        }

        nodes
    }

    async fn replicate_to_node(&self, node: &NodeInfo, key: &str, value: &[u8]) -> Result<()> {
        // Implement replication protocol
        // Send PUT request to node
        // Wait for acknowledgment
        Ok(())
    }

    async fn query_node(&self, node: &NodeInfo, key: &str) -> Result<Option<Vec<u8>>> {
        // Implement node query protocol
        // Send GET request to node
        // Wait for response
        Ok(None)
    }
} 