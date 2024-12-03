mod message;
mod error;
mod config;
mod thread_pool;


use bincode;

pub use message::{Message, BrokerNode, MessageHeaders, CompressionType};
pub use error::{Error, Result};
pub use config::{Config, FromEnv};
pub use thread_pool::ThreadPool;

impl From<Box<bincode::ErrorKind>> for Error {
    fn from(error: Box<bincode::ErrorKind>) -> Self {
        Error::Serialization(error.to_string())  
    }
}