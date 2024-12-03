use rafka::{Config, Result, FromEnv};
use rafka_consumer::Consumer;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let config = Config::from_env()?;
    
    info!("Starting consumer with config: {:?}", config);

    // Connect and identify as consumer
    let mut consumer = Consumer::connect(&config).await?;
    consumer.identify().await?;
    
    // Read messages in a loop
    while let Some(message) = consumer.receive().await? {
        info!("Received message: {:?}", message);
    }

    Ok(())
} 