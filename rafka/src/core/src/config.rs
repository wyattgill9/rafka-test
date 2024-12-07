use std::env;
use crate::error::{Error, Result};

#[derive(Clone, Debug)]
pub struct Config {
    pub nodes: Vec<String>,
    pub keyspace: String,
    pub broker_port: u16,
    pub producer_port: u16,
    pub consumer_port: u16,
    pub cache_ttl: u64,
    pub partition_count: u32,
    pub replication_factor: usize,
    pub node_id: String,
}

pub trait FromEnv {
    fn from_env() -> Result<Self> where Self: Sized;
}

impl FromEnv for Config {
    fn from_env() -> Result<Self> {
        Ok(Self {
            nodes: env::var("RAFKA_NODES")
                .unwrap_or_else(|_| "127.0.0.1".to_string())
                .split(',')
                .map(String::from)
                .collect(),
            keyspace: env::var("RAFKA_KEYSPACE")
                .unwrap_or_else(|_| "rafka".to_string()),
            broker_port: env::var("RAFKA_BROKER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|e: std::num::ParseIntError| Error::Config(e.to_string()))?,
            producer_port: env::var("RAFKA_PRODUCER_PORT")
                .unwrap_or_else(|_| "8081".to_string())
                .parse()
                .map_err(|e: std::num::ParseIntError| Error::Config(e.to_string()))?,
            consumer_port: env::var("RAFKA_CONSUMER_PORT")
                .unwrap_or_else(|_| "8082".to_string())
                .parse()
                .map_err(|e: std::num::ParseIntError| Error::Config(e.to_string()))?,
            cache_ttl: env::var("RAFKA_CACHE_TTL")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .map_err(|e: std::num::ParseIntError| Error::Config(e.to_string()))?,
            partition_count: env::var("RAFKA_PARTITION_COUNT")
                .unwrap_or_else(|_| "32".to_string())
                .parse()
                .map_err(|e: std::num::ParseIntError| Error::Config(e.to_string()))?,
            replication_factor: env::var("RAFKA_REPLICATION_FACTOR")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .map_err(|e: std::num::ParseIntError| Error::Config(e.to_string()))?,
            node_id: env::var("RAFKA_NODE_ID")
                .unwrap_or_else(|_| "node_1".to_string()),
        })
    }
} 