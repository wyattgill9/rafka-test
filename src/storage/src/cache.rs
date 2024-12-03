use crate::core::Result;
use bytes::Bytes;
use dashmap::DashMap;
use std::time::{Duration, Instant};

pub struct DistributedCache {
    local_cache: DashMap<String, CacheEntry>,
    ring: Arc<PartitionManager>,
    ttl: Duration,
}

struct CacheEntry {
    data: Bytes,
    expires_at: Instant,
}

impl DistributedCache {
    pub async fn get(&self, key: &str) -> Result<Option<Bytes>> {
        // Check if we own this key
        let owners = self.ring.get_partition_owners(key.as_bytes()).await;
        let local_node_id = self.get_local_node_id();
        
        if owners.contains(&local_node_id) {
            // We own this key, serve from local cache
            if let Some(entry) = self.local_cache.get(key) {
                if entry.expires_at > Instant::now() {
                    return Ok(Some(entry.data.clone()));
                }
                self.local_cache.remove(key);
            }
        } else {
            // Forward request to owner
            return self.forward_get_request(key, &owners[0]).await;
        }
        
        Ok(None)
    }

    pub async fn put(&self, key: String, value: Bytes) -> Result<()> {
        let owners = self.ring.get_partition_owners(key.as_bytes()).await;
        let local_node_id = self.get_local_node_id();

        if owners.contains(&local_node_id) {
            // We own this key, store locally
            self.local_cache.insert(key, CacheEntry {
                data: value,
                expires_at: Instant::now() + self.ttl,
            });
        } else {
            // Forward to owner
            self.forward_put_request(key, value, &owners[0]).await?;
        }
        
        Ok(())
    }
} 