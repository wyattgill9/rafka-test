use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use rafka_core::{Message, Result, Error};
use std::time::{Duration, SystemTime};

pub struct InMemoryStore {
    store: Arc<DashMap<String, PartitionStore>>,
    capacity_manager: Arc<CapacityManager>,
}

struct PartitionStore {
    messages: Vec<Message>,
    metadata: PartitionMetadata,
    indexes: MessageIndexes,
}

struct PartitionMetadata {
    leader_id: String,
    followers: Vec<String>,
    size_bytes: u64,
    last_access: SystemTime,
}

struct MessageIndexes {
    timestamp_index: DashMap<SystemTime, u64>,
    key_index: DashMap<Vec<u8>, Vec<u64>>,
}

impl InMemoryStore {
    pub fn new(capacity_bytes: u64) -> Self {
        Self {
            store: Arc::new(DashMap::new()),
            capacity_manager: Arc::new(CapacityManager::new(capacity_bytes)),
        }
    }

    pub async fn get_or_route(&self, key: &str) -> Result<StoreResponse> {
        // Fast path: check if data is in memory
        if let Some(data) = self.store.get(key) {
            return Ok(StoreResponse::InMemory(data.messages.clone()));
        }

        // Slow path: need to route to broker
        Ok(StoreResponse::NeedsRouting)
    }

    pub async fn store_message(&self, partition: &str, message: Message) -> Result<()> {
        let mut partition_store = self.store
            .entry(partition.to_string())
            .or_insert_with(|| PartitionStore {
                messages: Vec::new(),
                metadata: PartitionMetadata {
                    leader_id: "".to_string(),
                    followers: Vec::new(),
                    size_bytes: 0,
                    last_access: SystemTime::now(),
                },
                indexes: MessageIndexes {
                    timestamp_index: DashMap::new(),
                    key_index: DashMap::new(),
                },
            });

        // Check capacity before inserting
        let message_size = estimate_message_size(&message);
        self.capacity_manager.reserve_space(message_size).await?;

        // Update indexes
        partition_store.indexes.timestamp_index.insert(
            message.timestamp,
            partition_store.messages.len() as u64,
        );

        if let Some(key) = &message.key {
            partition_store.indexes.key_index
                .entry(key.clone())
                .or_default()
                .push(partition_store.messages.len() as u64);
        }

        // Store message
        partition_store.messages.push(message);
        partition_store.metadata.size_bytes += message_size;
        partition_store.metadata.last_access = SystemTime::now();

        Ok(())
    }

    pub async fn forward_to_follower(&self, partition: &str, follower: String) -> Result<()> {
        if let Some(mut store) = self.store.get_mut(partition) {
            store.metadata.followers.push(follower);
        }
        Ok(())
    }
}

pub enum StoreResponse {
    InMemory(Vec<Message>),
    NeedsRouting,
}

struct CapacityManager {
    max_bytes: u64,
    used_bytes: Arc<RwLock<u64>>,
    eviction_policy: EvictionPolicy,
}

enum EvictionPolicy {
    LRU,
    Size,
    Custom(Box<dyn Fn(&PartitionStore) -> f64 + Send + Sync>),
}

impl CapacityManager {
    fn new(max_bytes: u64) -> Self {
        Self {
            max_bytes,
            used_bytes: Arc::new(RwLock::new(0)),
            eviction_policy: EvictionPolicy::LRU,
        }
    }

    async fn reserve_space(&self, needed_bytes: u64) -> Result<()> {
        let mut used = self.used_bytes.write().await;
        
        if *used + needed_bytes > self.max_bytes {
            // Need to evict
            self.evict_until_space_available(needed_bytes).await?;
        }

        *used += needed_bytes;
        Ok(())
    }

    async fn evict_until_space_available(&self, needed_bytes: u64) -> Result<()> {
        // Implement eviction based on policy
        Ok(())
    }
}

fn estimate_message_size(message: &Message) -> u64 {
    // Basic size estimation
    let mut size = std::mem::size_of::<Message>() as u64;
    size += message.payload.len() as u64;
    if let Some(key) = &message.key {
        size += key.len() as u64;
    }
    size
} 