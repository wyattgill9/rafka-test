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
    
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        
        let message = Message::new(
            Some("test-key".as_bytes().to_vec()),
            "Hello from producer!".as_bytes().to_vec(),
        );
        
        if let Err(e) = producer.send(message).await {
            tracing::error!("Failed to send message: {}", e);
        }

        if tokio::signal::ctrl_c().now_or_never().is_some() {
            info!("Shutting down producer...");
            break;
        }
    }

    Ok(())
} 