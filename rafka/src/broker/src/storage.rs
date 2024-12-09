use rafka_core::{Message, Result};
use std::sync::Arc;
use dashmap::DashMap;

pub struct Storage {
    messages: Arc<DashMap<String, Vec<Message>>>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(DashMap::new()),
        }
    }

    pub async fn store_message(&self, partition: u32, message: Message) -> Result<()> {
        let partition_key = partition.to_string();
        
        self.messages
            .entry(partition_key)
            .or_insert_with(Vec::new)
            .push(message);
            
        Ok(())
    }

    pub async fn get_messages(&self, partition: u32) -> Result<Vec<Message>> {
        let partition_key = partition.to_string();
        
        Ok(self.messages
            .get(&partition_key)
            .map(|msgs| msgs.clone())
            .unwrap_or_default())
    }
} 