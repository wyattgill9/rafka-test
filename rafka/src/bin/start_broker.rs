use rafka::{StatelessBroker, Config, Result};
use rafka_core::FromEnv;
use tracing::info;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // init logging
    tracing_subscriber::fmt::init();

    // load config
    let config = Config::from_env()?;

    // create and start broker
    let broker = StatelessBroker::new(config.clone());
    info!("Starting broker with config: {:?}", config);

    let broker = Arc::new(broker);
    broker.clone().start_network().await?;

    // metrics endpoint
    broker.start_metrics().await?;

    // shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("Shutting down broker...");

    Ok(())
}
