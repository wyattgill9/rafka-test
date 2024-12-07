use rafka_core::{Message, NetworkMessage, MessageType, Result, Config};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use rafka_core::Error;
use rafka_core::PROTOCOL_VERSION;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn hash_fn(data: &[u8]) -> u32 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish() as u32
}

#[derive(Clone, Default)]
pub struct TopicMetadata {
    pub partition_count: u32,
    pub partition_owners: HashMap<u32, String>,
}

pub struct Producer {
    stream: Option<TcpStream>,
    node_id: String,
    topic_metadata: HashMap<String, TopicMetadata>,
}

impl Producer {
    pub async fn connect(config: &Config) -> Result<Self> {
        let stream = TcpStream::connect(format!("127.0.0.1:{}", config.broker_port)).await?;
        Ok(Self { 
            stream: Some(stream), 
            node_id: config.node_id.clone(), 
            topic_metadata: HashMap::new() 
        })
    }

    pub async fn identify(&mut self) -> Result<()> {
        let stream = self.stream.as_mut().ok_or_else(|| Error::InvalidInput("No connection".into()))?;
        stream.write_all(b"producer").await?;
        stream.write_all(&PROTOCOL_VERSION.to_be_bytes()).await?;
        
        let mut buf = [0u8; 3];
        stream.read_exact(&mut buf).await?;
        
        match &buf {
            b"ACK" => Ok(()),
            _ => Err(Error::InvalidInput("Failed to identify with broker".into())),
        }
    }

    pub async fn send(&mut self, message: Message) -> Result<()> {
        // Get topic metadata if not cached
        let metadata = self.get_topic_metadata(&message.topic).await?;
        
        // Calculate target partition
        let partition = self.calculate_partition(&message, &metadata);
        
        // Find broker for partition
        let target_node = metadata.partition_owners.get(&partition)
            .ok_or_else(|| Error::InvalidInput("No broker found for partition".into()))?;

        // Create network message
        let network_msg = NetworkMessage {
            msg_type: MessageType::Produce,
            source_node: self.node_id.clone(),
            target_node: Some(target_node.clone()),
            payload: message,
        };

        // Serialize and send
        self.send_network_message(network_msg).await
    }

    async fn get_topic_metadata(&mut self, topic: &str) -> Result<TopicMetadata> {
        if let Some(metadata) = self.topic_metadata.get(topic) {
            if !self.is_metadata_stale(&metadata) {
                return Ok(metadata.clone());
            }
        }

        // Query DHT for topic metadata
        self.refresh_topic_metadata(topic).await
    }

    async fn refresh_topic_metadata(&mut self, topic: &str) -> Result<TopicMetadata> {
        // Create metadata request
        let request = NetworkMessage {
            msg_type: MessageType::MetadataRequest,
            source_node: self.node_id.clone(),
            target_node: None, // Broker will handle routing
            payload: Message {
                topic: topic.to_string(),
                ..Default::default()
            },
        };

        self.send_network_message(request).await?;
        
        // Read response
        let metadata = self.read_metadata_response().await?;
        self.topic_metadata.insert(topic.to_string(), metadata.clone());
        
        Ok(metadata)
    }

    async fn send_network_message(&mut self, message: NetworkMessage) -> Result<()> {
        let stream = self.stream.as_mut().ok_or_else(|| Error::InvalidInput("No connection".into()))?;
        
        let msg_bytes = bincode::serialize(&message)?;
        let msg_len = msg_bytes.len() as u32;
        
        stream.write_all(&[1u8]).await?;
        stream.write_all(&msg_len.to_be_bytes()).await?;
        stream.write_all(&msg_bytes).await?;
        stream.flush().await?;
        
        self.wait_for_ack().await
    }

    async fn wait_for_ack(&mut self) -> Result<()> {
        let mut buf = [0u8; 3];
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            self.stream.as_mut().ok_or_else(|| Error::InvalidInput("No connection".into()))?.read_exact(&mut buf)
        ).await {
            Ok(result) => {
                result?;
                match &buf {
                    b"ACK" => Ok(()),
                    b"ERR" => {
                        let mut len_buf = [0u8; 4];
                        self.stream.as_mut().ok_or_else(|| Error::InvalidInput("No connection".into()))?.read_exact(&mut len_buf).await?;
                        let len = u32::from_be_bytes(len_buf);
                        
                        let mut err_buf = vec![0u8; len as usize];
                        self.stream.as_mut().ok_or_else(|| Error::InvalidInput("No connection".into()))?.read_exact(&mut err_buf).await?;
                        let err_msg = String::from_utf8_lossy(&err_buf);
                        
                        Err(Error::InvalidInput(format!("Broker error: {}", err_msg)))
                    },
                    _ => Err(Error::InvalidInput("Invalid acknowledgment from broker".into()))
                }
            },
            Err(_) => Err(Error::InvalidInput("Timeout waiting for broker acknowledgment".into()))
        }
    }

    fn calculate_partition(&self, message: &Message, metadata: &TopicMetadata) -> u32 {
        // Implement partition calculation logic based on message and metadata
        // For simplicity, let's use a hash-based approach
        let hash = hash_fn(message.payload.as_slice());
        let partition_count = metadata.partition_count as u32;
        hash % partition_count
    }

    async fn read_metadata_response(&mut self) -> Result<TopicMetadata> {
        // Implement logic to read metadata response from broker
        // For simplicity, let's return a dummy metadata
        let metadata = TopicMetadata {
            partition_count: 4,
            partition_owners: HashMap::new(),
            ..Default::default()
        };
        Ok(metadata)
    }

    // Helper method to check if metadata is stale
    fn is_metadata_stale(&self, metadata: &TopicMetadata) -> bool {
        // Implement staleness check logic here
        // For example, check if metadata is older than 5 minutes
        false // Temporary implementation
    }
}
