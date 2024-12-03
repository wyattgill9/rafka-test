use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub broker: BrokerConfig,
    pub network: NetworkConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Deserialize)]
pub struct BrokerConfig {
    pub thread_pool_size: usize,
    pub max_message_size: usize,
    pub partition_strategy: PartitionStrategy,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // Load conf from env
        let config = Config {
            broker: BrokerConfig {
                thread_pool_size: env::var("BROKER_THREAD_POOL_SIZE")
                    .unwrap_or_else(|_| "32".to_string())
                    .parse()?,
                max_message_size: env::var("MAX_MESSAGE_SIZE")
                    .unwrap_or_else(|_| "1048576".to_string())
                    .parse()?,
                partition_strategy: env::var("PARTITION_STRATEGY")
                    .map(|s| s.parse().unwrap_or(PartitionStrategy::Consistent))
                    .unwrap_or(PartitionStrategy::Consistent),
            },
            // ... other config sections
        };
        
        Ok(config)
    }
}
