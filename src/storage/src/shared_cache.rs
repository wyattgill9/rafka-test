use crate::core::Result;
use bytes::Bytes;
use dashmap::DashMap;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

pub struct SharedCache {
    local_cache: DashMap<String, CacheEntry>,
    update_channel: broadcast::Sender<CacheUpdate>,
    replication_factor: usize,
    ttl: Duration,
    consistency_level: ConsistencyLevel,
}

#[derive(Clone, Debug)]
struct CacheUpdate {
    key: String,
    value: Bytes,
    version: u64,
    source_node: String,
}

#[derive(Clone, Copy, Debug)]
pub enum ConsistencyLevel {
    One,
    Quorum,
    All,
}

impl SharedCache {
    pub async fn get(&self, key: &str) -> Result<Option<Bytes>> {
        // Try local cache first
        if let Some(entry) = self.local_cache.get(key) {
            if entry.expires_at > Instant::now() {
                return Ok(Some(entry.data.clone()));
            }
            self.local_cache.remove(key);
        }

        // If not in local cache, try other nodes
        match self.consistency_level {
            ConsistencyLevel::One => self.get_from_any_node(key).await,
            ConsistencyLevel::Quorum => self.get_with_quorum(key).await,
            ConsistencyLevel::All => self.get_from_all_nodes(key).await,
        }
    }

    pub async fn put(&self, key: String, value: Bytes) -> Result<()> {
        let update = CacheUpdate {
            key: key.clone(),
            value: value.clone(),
            version: self.generate_version(),
            source_node: self.get_node_id(),
        };

        // Update local cache
        self.local_cache.insert(key, CacheEntry {
            data: value,
            expires_at: Instant::now() + self.ttl,
            version: update.version,
        });

        // Propagate update to other nodes
        self.update_channel.send(update)?;
        
        Ok(())
    }

    async fn handle_updates(&self) {
        let mut rx = self.update_channel.subscribe();
        while let Ok(update) = rx.recv().await {
            if update.source_node != self.get_node_id() {
                self.local_cache.insert(update.key, CacheEntry {
                    data: update.value,
                    expires_at: Instant::now() + self.ttl,
                    version: update.version,
                });
            }
        }
    }

    async fn get_from_any_node(&self, key: &str) -> Result<Option<Bytes>> {
        let nodes = self.get_available_nodes().await?;
        for node in nodes {
            if let Ok(Some(value)) = self.request_from_node(&node, key).await {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }

    async fn get_with_quorum(&self, key: &str) -> Result<Option<Bytes>> {
        let nodes = self.get_available_nodes().await?;
        let quorum_size = (nodes.len() / 2) + 1;
        let mut responses = Vec::new();

        for node in nodes {
            if let Ok(Some(value)) = self.request_from_node(&node, key).await {
                responses.push(value);
                if responses.len() >= quorum_size {
                    // Return most recent version based on timestamps
                    return Ok(Some(self.select_most_recent(responses)));
                }
            }
        }
        Ok(None)
    }

    async fn get_from_all_nodes(&self, key: &str) -> Result<Option<Bytes>> {
        let nodes = self.get_available_nodes().await?;
        let mut responses = Vec::new();

        for node in nodes {
            if let Ok(Some(value)) = self.request_from_node(&node, key).await {
                responses.push(value);
            } else {
                return Err(Error::ConsistencyError);
            }
        }

        if responses.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.select_most_recent(responses)))
        }
    }

    fn select_most_recent(&self, values: Vec<Bytes>) -> Bytes {
        // Select the value with the highest version number
        values.into_iter()
            .max_by_key(|v| self.extract_version(v))
            .unwrap()
    }

    fn generate_version(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
} 