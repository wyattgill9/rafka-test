use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub key: Option<Vec<u8>>,
    pub payload: Vec<u8>,
    pub timestamp: SystemTime,
    pub headers: MessageHeaders,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializedMessage {
    pub id: String,
    pub key: Option<Vec<u8>>,
    pub payload: Vec<u8>,
    pub timestamp: u64,
    pub headers: MessageHeaders,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MessageHeaders {
    pub partition_hint: Option<u32>,
    pub priority: Option<u8>,
    pub compression: Option<CompressionType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    LZ4,
    Snappy,
}

impl Message {
    pub fn new(key: Option<impl Into<Vec<u8>>>, payload: impl Into<Vec<u8>>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            key: key.map(Into::into),
            payload: payload.into(),
            timestamp: SystemTime::now(),
            headers: MessageHeaders::default(),
        }
    }

    pub fn into_bytes(self) -> Bytes {
        Bytes::from(self.payload)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BrokerNode {
    pub id: String,
    pub address: String,
    pub port: u16,
}
