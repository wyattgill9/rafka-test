use crate::core::{Message, Result};
use bytes::Bytes;
use parking_lot::RwLock;
use std::collections::HashMap;

pub struct InMemoryStorage {
    data: RwLock<HashMap<String, Bytes>>,
    backup_client: Option<Box<dyn BackupClient>>,
}

impl InMemoryStorage {
    pub async fn store(&self, key: String, msg: Message) -> Result<()> {
        let serialized = bincode::serialize(&msg)?;
        
        // Store in memory
        self.data.write().insert(key.clone(), Bytes::from(serialized));
        
        // Async backup if configured
        if let Some(client) = &self.backup_client {
            tokio::spawn(async move {
                client.backup(key, serialized).await
            });
        }
        
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Option<Message>> {
        self.data.read()
            .get(key)
            .map(|bytes| bincode::deserialize(bytes))
            .transpose()
            .map_err(Into::into)
    }
}
