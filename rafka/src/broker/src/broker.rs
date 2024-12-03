use rafka_core::{Message, Result, ThreadPool, Config};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use rafka_core::Error;
use bincode;
use uuid;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::io::Result as IoResult;

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
        let mut buf = [0u8; 1024];
        let n = socket.read(&mut buf).await?;
        
        let connection_type = std::str::from_utf8(&buf[..n])?;
        match connection_type {
            "producer" => self.handle_producer(socket).await,
            "consumer" => self.handle_consumer(socket).await,
            _ => Err(Error::InvalidConnectionType),
        }
    }

    async fn route_message(&self, msg: Message) -> Result<()> {
        let partition = self.calculate_partition(&msg);
        let consumers = self.discover_consumers(partition).await?;
        
        let msg_bytes = bincode::serialize(&msg)?;
        
        for mut consumer in consumers {
            let msg_bytes = msg_bytes.clone();
            tokio::spawn(async move {
                consumer.write_all(&msg_bytes).await
            });
        }
        
        Ok(())
    }

    fn calculate_partition(&self, msg: &Message) -> u32 {
        // Dynamic partition calculation based on message key
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
        // Return a default partition count (you can adjust this value based on your needs)
        32
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
        
        // Store write half for acknowledgments
        self.active_producers.insert(producer_id.clone(), write_half);
        
        let mut len_buf = [0u8; 4];
        loop {
            // Read message length
            match read_half.read_exact(&mut len_buf).await {
                Ok(_) => {
                    let msg_len = u32::from_be_bytes(len_buf);
                    let mut msg_buf = vec![0u8; msg_len as usize];
                    
                    // Read the actual message
                    if let Err(e) = read_half.read_exact(&mut msg_buf).await {
                        tracing::error!("Error reading message: {}", e);
                        break;
                    }
                    
                    match bincode::deserialize::<Message>(&msg_buf) {
                        Ok(message) => {
                            if let Err(e) = self.route_message(message).await {
                                tracing::error!("Failed to route message: {}", e);
                                if let Some(mut producer) = self.active_producers.get_mut(&producer_id) {
                                    producer.write_all(b"ERR").await?;
                                }
                            } else {
                                if let Some(mut producer) = self.active_producers.get_mut(&producer_id) {
                                    producer.write_all(b"ACK").await?;
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
                    // Connection closed normally
                    break;
                }
                Err(e) => {
                    tracing::error!("Error reading message length: {}", e);
                    break;
                }
            }
        }
        
        self.active_producers.remove(&producer_id);
        Ok(())
    }

    async fn handle_consumer(&self, socket: TcpStream) -> Result<()> {
        // Read partition assignment
        let mut buf = [0u8; 4];
        let mut socket = socket;
        socket.read_exact(&mut buf).await?;
        let partition = u32::from_be_bytes(buf);
        
        // Store consumer
        if let Some(mut consumers) = self.active_consumers.get_mut(&partition.to_string()) {
            consumers.value_mut().push(socket);
        } else {
            self.active_consumers.insert(partition.to_string(), vec![socket]);
        }
        
        Ok(())
    }
}
