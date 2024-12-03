use bytes::Bytes;
use std::sync::Arc;

pub struct Storage {
    scylla: Arc<ScyllaStorage>,
    cache: Arc<SharedCache>,
}

impl Storage {
    pub async fn new(nodes: Vec<String>, keyspace: String) -> Result<Self> {
        Ok(Self {
            scylla: Arc::new(ScyllaStorage::new(nodes, keyspace).await?),
            cache: Arc::new(SharedCache::new(
                Duration::from_secs(3600),
                3,
                ConsistencyLevel::Quorum,
            )),
        })
    }

    pub async fn get(&self, key: &str) -> Result<Option<Bytes>> {
        if let Some(value) = self.cache.get(key).await? {
            return Ok(Some(value));
        }

        if let Some(value) = self.scylla.read(key).await? {
            self.cache.put(key.to_string(), value.clone()).await?;
            return Ok(Some(value));
        }

        Ok(None)
    }

    pub async fn put(&self, key: String, value: Bytes) -> Result<()> {
        self.scylla.write(&key, value.clone()).await?;
        self.cache.put(key, value).await?;
        Ok(())
    }
}
