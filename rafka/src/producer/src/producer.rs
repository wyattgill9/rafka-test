use rafka_core::{Message, Result, Config};
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use tracing::info;

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
        self.stream.write_all(&msg_bytes).await?;
        Ok(())
    }
}
