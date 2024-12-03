use rafka_core::{Message, Result, Config, Error};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct Consumer {
    stream: TcpStream,
}

impl Consumer {
    pub async fn connect(config: &Config) -> Result<Self> {
        let stream = TcpStream::connect(format!("127.0.0.1:{}", config.broker_port)).await?;
        Ok(Self { stream })
    }

    pub async fn identify(&mut self) -> Result<()> {
        // Send consumer identifier
        self.stream.write_all(b"consumer").await?;
        self.stream.write_all(&rafka_core::PROTOCOL_VERSION.to_be_bytes()).await?;
        
        // Register for all partitions (0-31) for testing
        for partition in 0..32u32 {
            self.stream.write_all(&partition.to_be_bytes()).await?;
            
            // Wait for acknowledgment for each partition
            let mut buf = [0u8; 3];
            self.stream.read_exact(&mut buf).await?;
            
            match &buf {
                b"ACK" => {
                    tracing::info!("Successfully registered for partition {}", partition);
                }
                _ => return Err(Error::InvalidInput("Failed to register for partition".into())),
            }
        }
        
        tracing::info!("Successfully identified as consumer for all partitions");
        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Option<Message>> {
        loop {
            let mut len_buf = [0u8; 4];
            
            match self.stream.read_exact(&mut len_buf).await {
                Ok(_) => {
                    let msg_len = u32::from_be_bytes(len_buf) as usize;
                    let mut msg_buf = vec![0u8; msg_len];
                    
                    self.stream.read_exact(&mut msg_buf).await?;
                    
                    match bincode::deserialize::<Message>(&msg_buf) {
                        Ok(message) => {
                            tracing::info!("Consumer received message ID: {}", message.id);
                            return Ok(Some(message));
                        }
                        Err(e) => {
                            tracing::error!("Failed to deserialize received message: {}", e);
                            return Err(Error::Serialization(e.to_string()));
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    continue;
                }
                Err(e) => return Err(Error::Io(e)),
            }
        }
    }
}
