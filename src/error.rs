use thiserror::Error;

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
}

pub type Result<T> = std::result::Result<T, Error>; 