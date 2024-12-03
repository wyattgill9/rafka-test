use thiserror::Error;
use std::str::Utf8Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Cache error: {0}")]
    Cache(String),
    #[error("Config error: {0}")]
    Config(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Invalid connection type")]
    InvalidConnectionType,
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Send error: {0}")]
    SendError(String),
    #[error("Broker error: {0}")]
    BrokerError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<std::str::Utf8Error> for Error {
    fn from(error: Utf8Error) -> Self {
        Error::InvalidInput(error.to_string())
    }
} 