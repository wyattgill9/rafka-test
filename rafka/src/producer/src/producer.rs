use rafka_core::{Message, Result, Config};
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use tokio::io::AsyncReadExt;
use rafka_core::Error;

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
        Ok(())
    }

    pub async fn send(&mut self, message: Message) -> Result<()> {
        let msg_bytes = bincode::serialize(&message)?;
        
        // Add message length prefix
        let msg_len = msg_bytes.len() as u32;
        self.stream.write_all(&msg_len.to_be_bytes()).await?;
        self.stream.write_all(&msg_bytes).await?;
        
        // Wait for acknowledgment with timeout
        let mut buf = [0u8; 3];
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            self.stream.read_exact(&mut buf)
        ).await {
            Ok(result) => {
                result?;  // Handle IO error
                match &buf {
                    b"ACK" => Ok(()),
                    b"ERR" => Err(Error::InvalidInput("Broker failed to process message".into())),
                    _ => Err(Error::InvalidInput("Invalid acknowledgment from broker".into()))
                }
            },
            Err(_elapsed) => Err(Error::InvalidInput("Timeout waiting for broker acknowledgment".into()))
        }
    }
}
