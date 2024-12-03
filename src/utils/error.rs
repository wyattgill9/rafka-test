use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RafkaError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("No available nodes")]
    NoAvailableNodes,

    #[error("Message too large: {size} bytes (max: {max} bytes)")]
    MessageTooLarge { size: usize, max: usize },

    #[error("Operation timeout after {0:?}")]
    Timeout(std::time::Duration),
}

pub type Result<T> = std::result::Result<T, RafkaError>;
