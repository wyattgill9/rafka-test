use rafka_core::{Config, Result, FromEnv, Message};
use rafka_producer::Producer;
use tracing::info;
use tokio::time::Duration;
use futures_util::FutureExt;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let config = Config::from_env()?;
    
    info!("Starting producer with config: {:?}", config);

    let mut producer = Producer::connect(&config).await?;
    producer.identify().await?;
    
    let message = Message::new(
        Some("test-key".as_bytes().to_vec()),
        "Hello from producer!".as_bytes().to_vec(),
    );
    
    info!("Sending message with ID: {}", message.id);
    if let Err(e) = producer.send(message).await {
        tracing::error!("Failed to send message: {}", e);
    }

    tokio::time::sleep(Duration::from_secs(1)).await;
    info!("Producer shutting down...");

    Ok(())
} 