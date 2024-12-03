use rafka_core::Result;
use tokio::net::TcpStream;

pub struct Transport;

impl Transport {
    pub fn new() -> Self {
        Self
    }

    pub async fn connect(&self) -> Result<TcpStream> {
        todo!()
    }
} 