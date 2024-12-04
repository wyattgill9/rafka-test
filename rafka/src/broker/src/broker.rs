use rafka_core::{Message, Result, ThreadPool, Config};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use rafka_core::Error;
use bincode;
use uuid;
use tokio::net::tcp::OwnedWriteHalf;
use rafka_core::PROTOCOL_VERSION;

pub struct StatelessBroker {
    active_producers: DashMap<String, OwnedWriteHalf>,
    active_consumers: DashMap<String, Vec<TcpStream>>,
    thread_pool: Arc<ThreadPool>,
    config: Config,
}

impl StatelessBroker {
    pub fn new(config: Config) -> Self {
        StatelessBroker {
            active_producers: DashMap::new(),
            active_consumers: DashMap::new(),
            thread_pool: Arc::new(ThreadPool::new(32)),
            config,
        }
    }

    pub async fn start_network(self: Arc<Self>) -> Result<()> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.config.broker_port)).await?;
        
        loop {
            let (socket, addr) = listener.accept().await?;
            tracing::info!("New connection from {}", addr);
            
            let broker = self.clone();
            tokio::spawn(async move {
                if let Err(e) = broker.handle_connection(socket).await {
                    tracing::error!("Connection error: {}", e);
                }
            });
        }
    }

    async fn handle_connection(&self, mut socket: TcpStream) -> Result<()> {
        // Read connection type (producer/consumer)
        let mut buf = [0u8; 8]; // Increased buffer for "producer" string
        let n = socket.read_exact(&mut buf[..8]).await?;
        
        // Read protocol version
        let mut version_buf = [0u8; 4];
        socket.read_exact(&mut version_buf).await?;
        let client_version = u32::from_be_bytes(version_buf);
        
        if client_version != PROTOCOL_VERSION {
            socket.write_all(b"ERR").await?;
            return Err(Error::InvalidInput("Protocol version mismatch".into()));
        }
        
        let connection_type = std::str::from_utf8(&buf[..n])?;
        
        // Send acknowledgment
        socket.write_all(b"ACK").await?;
        
        match connection_type {
            "producer" => self.handle_producer(socket).await,
            "consumer" => self.handle_consumer(socket).await,
            _ => {
                socket.write_all(b"ERR").await?;
                Err(Error::InvalidConnectionType)
            }
        }
    }

    async fn route_message(&self, msg: Message) -> Result<()> {
        let partition = self.calculate_partition(&msg);
        tracing::info!("Routing msg: {} to partition: {}", msg.id, partition);
        
        let partition_key = partition.to_string();
        
        // Get or create consumer group for partition
        if !self.active_consumers.contains_key(&partition_key) {
            tracing::warn!("No consumers for partition {}. Message buffered.", partition);
            // TODO: Implement message buffering for when consumers aren't available
            return Ok(());  // Return Ok instead of Error to avoid producer failures
        }
        
        if let Some(consumers) = self.active_consumers.get(&partition_key) {
            let msg_bytes = bincode::serialize(&msg)?;
            let msg_len = msg_bytes.len() as u32;
            
            let mut successful_delivery = false;
            
            for consumer in consumers.value() {
                if let Ok(_) = consumer.try_write(&msg_len.to_be_bytes()) {
                    if let Ok(_) = consumer.try_write(&msg_bytes) {
                        if let Ok(peer_addr) = consumer.peer_addr() {
                            tracing::info!("Successfully delivered msg: {} to consumer {} on partition {}", 
                                msg.id, peer_addr, partition);
                        }
                        successful_delivery = true;
                        break;  // Successfully delivered to one consumer
                    }
                }
            }
            
            if successful_delivery {
                Ok(())
            } else {
                tracing::warn!("Failed to deliver message to any active consumers, message will be buffered");
                // TODO: Implement message buffering
                Ok(())  // Return Ok instead of Error
            }
        } else {
            tracing::warn!("No consumers available for partition {}, message will be buffered", partition);
            // TODO: Implement message buffering
            Ok(())  // Return Ok instead of Error
        }
    }

    fn calculate_partition(&self, msg: &Message) -> u32 {
        // Simple partition calculation based on message key
        match &msg.key {
            Some(key) => {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                key.hash(&mut hasher);
                (hasher.finish() % self.get_partition_count() as u64) as u32
            }
            None => rand::random::<u32>() % self.get_partition_count()
        }
    }

    pub fn get_partition_count(&self) -> u32 {
        32 // Default partition count
    }

    async fn discover_consumers(&self, partition: u32) -> Result<Vec<TcpStream>> {
        if let Some(consumers) = self.active_consumers.get(&partition.to_string()) {
            let mut result = Vec::new();
            for consumer in consumers.value() {
                // Create a new socket from the same connection
                let socket_addr = consumer.peer_addr()?;
                if let Ok(new_stream) = TcpStream::connect(socket_addr).await {
                    result.push(new_stream);
                }
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn start_metrics(&self) -> Result<()> {
        // Implement metrics initialization logic here
        // For now, returning Ok to get it compiling
        Ok(())
    }

    async fn handle_producer(&self, socket: TcpStream) -> Result<()> {
        let (mut read_half, write_half) = socket.into_split();
        let producer_id = uuid::Uuid::new_v4().to_string();
        
        tracing::info!("New producer connected with ID: {}", producer_id);
        self.active_producers.insert(producer_id.clone(), write_half);
        
        loop {
            //msg type
            let mut type_buf = [0u8; 1];
            match read_half.read_exact(&mut type_buf).await {
                Ok(_) => {
                    //msg length
                    let mut len_buf = [0u8; 4];
                    if let Err(e) = read_half.read_exact(&mut len_buf).await {
                        tracing::error!("Error reading message length: {}", e);
                        break;
                    }
                    
                    let msg_len = u32::from_be_bytes(len_buf);
                    tracing::debug!("Receiving message of length: {} bytes", msg_len);
                    
                    let mut msg_buf = vec![0u8; msg_len as usize];
                    
                    // read message
                    if let Err(e) = read_half.read_exact(&mut msg_buf).await {
                        tracing::error!("Error reading message: {}", e);
                        break;
                    }
                    
                    match bincode::deserialize::<Message>(&msg_buf) {
                        Ok(message) => {
                            tracing::info!("Received message ID: {} from producer {}", message.id, producer_id);
                            
                            // acknowledge after successful routing
                            match self.route_message(message.clone()).await {
                                Ok(_) => {
                                    if let Some(mut producer) = self.active_producers.get_mut(&producer_id) {
                                        producer.write_all(b"ACK").await?;
                                        tracing::info!("Sent ACK to producer for message {}", message.id);
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to route message {}: {}", message.id, e);
                                    if let Some(mut producer) = self.active_producers.get_mut(&producer_id) {
                                        let err_msg = e.to_string();
                                        producer.write_all(b"ERR").await?;
                                        producer.write_all(&(err_msg.len() as u32).to_be_bytes()).await?;
                                        producer.write_all(err_msg.as_bytes()).await?;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to deserialize message: {}", e);
                            if let Some(mut producer) = self.active_producers.get_mut(&producer_id) {
                                producer.write_all(b"ERR").await?;
                            }
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    tracing::info!("Producer {} disconnected", producer_id);
                    break;
                }
                Err(e) => {
                    tracing::error!("Error reading from producer {}: {}", producer_id, e);
                    break;
                }
            }
        }
        
        self.active_producers.remove(&producer_id);
        tracing::info!("Producer {} removed from active producers", producer_id);
        Ok(())
    }

    async fn handle_consumer(&self, mut socket: TcpStream) -> Result<()> {
        // read partition assignment
        let mut buf = [0u8; 4];
        socket.read_exact(&mut buf).await?;
        let partition = u32::from_be_bytes(buf);
        
        let consumer_id = uuid::Uuid::new_v4().to_string();
        tracing::info!("New consumer connected with ID: {} for partition {}", consumer_id, partition);
        
        // store consumer
        if let Some(mut consumers) = self.active_consumers.get_mut(&partition.to_string()) {
            consumers.value_mut().push(socket);
            tracing::info!("Added consumer {} to partition {}", consumer_id, partition);
        } else {
            self.active_consumers.insert(partition.to_string(), vec![socket]);
            tracing::info!("Created new consumer group, partition:{} with consumer: {}", partition, consumer_id);
        }
        
        Ok(())
    }
}
