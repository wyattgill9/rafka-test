mod broker;
pub use broker::StatelessBroker;

use rafka_core::{Config, Result};
use tracing::{info, error};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct Broker {
    config: Config,
}

impl Broker {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting broker on port {}", self.config.broker_port);
        
        // Create and bind TCP listener
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.config.broker_port)).await?;
        
        // Accept and handle connections
        loop {
            match listener.accept().await {
                Ok((mut socket, addr)) => {
                    info!("New connection from {}", addr);
                    tokio::spawn(async move {
                        // Use a proper buffer for reading
                        let mut buf = vec![0u8; 1024];
                        loop {
                            match socket.read(&mut buf).await {
                                Ok(0) => {
                                    // Connection closed normally
                                    info!("Connection from {} closed", addr);
                                    break;
                                }
                                Ok(n) => {
                                    info!("Received {} bytes from {}", n, addr);
                                    // Instead of echoing, try to parse and handle the message properly
                                    match String::from_utf8(buf[..n].to_vec()) {
                                        Ok(message) => {
                                            info!("Received message: {}", message);
                                            // Acknowledge receipt instead of echoing
                                            if let Err(e) = socket.write_all(b"ACK").await {
                                                error!("Error sending ACK: {}", e);
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            error!("Invalid UTF-8 sequence: {}", e);
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Error reading from socket: {}", e);
                                    break;
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}
