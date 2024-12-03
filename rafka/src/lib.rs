pub use rafka_core::{Config, Result, FromEnv};
use rafka_core::Error;
pub use rafka_broker::StatelessBroker;

// Create a wrapper struct for configuration
pub struct ConfigBuilder;

impl ConfigBuilder {
    pub fn from_env() -> Result<Config> {
        let parse_env = |key: &str, default: &str| -> Result<u64> {
            std::env::var(key)
                .unwrap_or_else(|_| default.to_string())
                .parse()
                .map_err(|e| Error::Config(format!("Failed to parse {}: {}", key, e)))
        };

        Ok(Config {
            cache_ttl: parse_env("CACHE_TTL", "60")?,
            keyspace: std::env::var("KEYSPACE")
                .unwrap_or_else(|_| "default_keyspace".to_string()),
            nodes: std::env::var("NODES")
                .unwrap_or_else(|_| "localhost:9092".to_string())
                .split(',')
                .map(String::from)
                .collect(),
            broker_port: parse_env("BROKER_PORT", "8080")? as u16,
            producer_port: parse_env("PRODUCER_PORT", "8081")? as u16,
            consumer_port: parse_env("CONSUMER_PORT", "8082")? as u16,
            partition_count: parse_env("PARTITION_COUNT", "32")? as usize,
            replication_factor: parse_env("REPLICATION_FACTOR", "3")? as usize,
        })
    }
}

// Create a newtype wrapper around Config
pub struct RafkaConfig(pub Config);

// Implement FromEnv for our newtype wrapper instead
impl FromEnv for RafkaConfig {
    fn from_env() -> Result<Self> {
        ConfigBuilder::from_env().map(RafkaConfig)
    }
}

// Add a conversion method if needed
impl RafkaConfig {
    pub fn into_inner(self) -> Config {
        self.0
    }
}