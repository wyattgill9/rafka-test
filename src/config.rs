use std::env;
use crate::error::{Error, Result};

pub struct Config {
    pub nodes: Vec<String>,
    pub keyspace: String,
    pub port: u16,
    pub cache_ttl: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            nodes: env::var("RAFKA_NODES")
                .map_err(|e| Error::Config(e.to_string()))?
                .split(',')
                .map(String::from)
                .collect(),
            keyspace: env::var("RAFKA_KEYSPACE")
                .map_err(|e| Error::Config(e.to_string()))?,
            port: env::var("RAFKA_PORT")
                .map_err(|e| Error::Config(e.to_string()))?
                .parse()
                .map_err(|e| Error::Config(e.to_string()))?,
            cache_ttl: env::var("RAFKA_CACHE_TTL")
                .map_err(|e| Error::Config(e.to_string()))?
                .parse()
                .map_err(|e| Error::Config(e.to_string()))?,
        })
    }
} 