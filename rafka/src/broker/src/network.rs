use tokio::net::TcpStream;
use std::sync::Arc;
use rafka_core::{Message, Result, Error};
use dashmap::DashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct NetworkLayer {
    connections: Arc<DashMap<String, TcpStream>>,
    producers: Arc<DashMap<String, TcpStream>>,
    consumers: Arc<DashMap<String, Vec<TcpStream>>>,
}

impl NetworkLayer {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
            producers: Arc::new(DashMap::new()),
            consumers: Arc::new(DashMap::new()),
        }
    }

    pub async fn add_producer(&self, producer_id: String, socket: TcpStream) -> Result<()> {
        self.producers.insert(producer_id.clone(), socket);
        self.connections.insert(producer_id, socket);
        Ok(())
    }

    pub async fn add_consumer(&self, consumer_id: String, socket: TcpStream) -> Result<()> {
        // Read partition assignment
        let mut buf = [0u8; 4];
        socket.readable().await?;
        socket.try_read(&mut buf)?;
        let partition = u32::from_be_bytes(buf);
        
        let partition_key = partition.to_string();
        self.consumers
            .entry(partition_key)
            .or_insert_with(Vec::new)
            .push(socket);
            
        self.connections.insert(consumer_id, socket);
        Ok(())
    }

    pub async fn replicate_message(&self, partition: u32, message: &Message) -> Result<()> {
        let partition_key = partition.to_string();
        
        if let Some(consumers) = self.consumers.get(&partition_key) {
            let msg_bytes = bincode::serialize(message)?;
            let msg_len = msg_bytes.len() as u32;
            
            for consumer in consumers.value() {
                if let Ok(mut stream) = consumer.try_clone() {
                    if let Err(e) = stream.write_all(&msg_len.to_be_bytes()).await {
                        tracing::error!("Failed to write message length: {}", e);
                        continue;
                    }
                    if let Err(e) = stream.write_all(&msg_bytes).await {
                        tracing::error!("Failed to write message: {}", e);
                        continue;
                    }
                    tracing::info!("Replicated message to consumer");
                }
            }
        }
        
        Ok(())
    }
} 