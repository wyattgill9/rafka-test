use rafka_core::{Config, Result};
use tracing::info;

pub struct Broker {
    config: Config,
}

impl Broker {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting broker on port {}", self.config.port);
        Ok(())
    }
}

mod broker;
pub use broker::StatelessBroker;