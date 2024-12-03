#[derive(Clone, Debug)]
pub struct Config {
    pub nodes: Vec<String>,
    pub keyspace: String,
    pub port: u16,
    pub cache_ttl: u64,
    pub partition_count: usize,
    pub replication_factor: usize,
} 