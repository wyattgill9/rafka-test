use rafka_core::{Message, Result, Config};
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;
use rafka_core::Error;
use rafka_core::PROTOCOL_VERSION;

pub struct Producer {
    stream: TcpStream,
}

impl Producer {
    pub async fn connect(config: &Config) -> Result<Self> {
        let stream = TcpStream::connect(format!("127.0.0.1:{}", config.broker_port)).await?;
        Ok(Self { stream })
    }

    pub async fn identify(&mut self) -> Result<()> {
        self.stream.write_all(b"producer").await?;
        self.stream.write_all(&PROTOCOL_VERSION.to_be_bytes()).await?;
        
        let mut buf = [0u8; 3];
        self.stream.read_exact(&mut buf).await?;
        
        match &buf {
            b"ACK" => Ok(()),
            _ => Err(Error::InvalidInput("Failed to identify with broker".into())),
        }
    }

    pub async fn send(&mut self, message: Message) -> Result<()> {
        tracing::info!("Sending message with ID: {}", message.id);
        
        let msg_bytes = bincode::serialize(&message)?;
        let msg_len = msg_bytes.len() as u32;
        
        self.stream.write_all(&[1u8]).await?;
        
        self.stream.write_all(&msg_len.to_be_bytes()).await?;
        
        self.stream.write_all(&msg_bytes).await?;
        self.stream.flush().await?;
        
        match self.wait_for_ack().await {
            Ok(_) => {
                tracing::info!("Message {} successfully sent and acknowledged", message.id);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to send message {}: {}", message.id, e);
                Err(e)
            }
        }
    }

    async fn wait_for_ack(&mut self) -> Result<()> {
        let mut buf = [0u8; 3];
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            self.stream.read_exact(&mut buf)
        ).await {
            Ok(result) => {
                result?;
                match &buf {
                    b"ACK" => Ok(()),
                    b"ERR" => {
                        let mut len_buf = [0u8; 4];
                        self.stream.read_exact(&mut len_buf).await?;
                        let len = u32::from_be_bytes(len_buf);
                        
                        let mut err_buf = vec![0u8; len as usize];
                        self.stream.read_exact(&mut err_buf).await?;
                        let err_msg = String::from_utf8_lossy(&err_buf);
                        
                        Err(Error::InvalidInput(format!("Broker error: {}", err_msg)))
                    },
                    _ => Err(Error::InvalidInput("Invalid acknowledgment from broker".into()))
                }
            },
            Err(_) => Err(Error::InvalidInput("Timeout waiting for broker acknowledgment".into()))
        }
    }
}
