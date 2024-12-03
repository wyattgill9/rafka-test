pub use rafka_core::{Message, BrokerNode, Config, Error, Result};
pub use rafka_broker::StatelessBroker;
pub use rafka_storage::Storage;
pub use rafka_network::Transport;

// Create a wrapper struct for configuration
pub struct ConfigBuilder;

// Define a trait for environment configuration
pub trait FromEnv {
    fn from_env() -> Result<Self> where Self: Sized;
}

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
            port: parse_env("PORT", "8080")? as u16,
            partition_count: parse_env("PARTITION_COUNT", "32")? as usize,
            replication_factor: parse_env("REPLICATION_FACTOR", "3")? as usize,
        })
    }
}

// Implement the trait for Config
impl FromEnv for Config {
    fn from_env() -> Result<Self> {
        ConfigBuilder::from_env()
    }
}