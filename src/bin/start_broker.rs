use rafka::{StatelessBroker, Config, Result, FromEnv};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration from environment
    let config = Config::from_env()?;

    // Create and start broker
    let broker = StatelessBroker::new(config.clone());
    info!("Starting broker with config: {:?}", config);

    // Start network listeners
    broker.start_network().await?;

    // Start metrics endpoint
    broker.start_metrics().await?;

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("Shutting down broker...");

    Ok(())
}
