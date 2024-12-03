use rafka_core::{Message, Result, Config};
use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tracing::info;

pub struct Consumer {
    stream: TcpStream,
}

impl Consumer {
    pub async fn connect(config: &Config) -> Result<Self> {
        let stream = TcpStream::connect(format!("127.0.0.1:{}", config.broker_port)).await?;
        Ok(Self { stream })
    }

    pub async fn identify(&mut self) -> Result<()> {
        self.stream.write_all(b"consumer").await?;
        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Option<Message>> {
        let mut buf = [0u8; 1024];
        let n = self.stream.read(&mut buf).await?;
        
        if n == 0 {
            return Ok(None);
        }

        let message: Message = bincode::deserialize(&buf[..n])?;
        Ok(Some(message))
    }
}
