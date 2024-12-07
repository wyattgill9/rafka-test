use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageType {
    Produce,
    Fetch,
    JoinNetwork,
    LeaveNetwork,
    PartitionTransfer,
    Heartbeat,
    JoinGroup,
    MetadataRequest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub msg_type: MessageType,
    pub source_node: String,
    pub target_node: Option<String>,
    pub payload: Message,
}

impl Default for NetworkMessage {
    fn default() -> Self {
        Self {
            msg_type: MessageType::Heartbeat,
            source_node: String::new(),
            target_node: None,
            payload: Message::new(
                String::new(),
                None::<Vec<u8>>,
                Vec::<u8>::new()
            ),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub topic: String,
    pub partition: Option<u32>,
    pub key: Option<Vec<u8>>,
    pub payload: Vec<u8>,
    pub timestamp: SystemTime,
    pub headers: MessageHeaders,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MessageHeaders {
    pub partition_hint: Option<u32>,
    pub replication_factor: Option<u8>,
    pub compression: Option<CompressionType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    LZ4,
    Snappy,
}

impl Message {
    pub fn new(topic: String, key: Option<impl Into<Vec<u8>>>, payload: impl Into<Vec<u8>>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            topic,
            partition: None,
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

impl Default for Message {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            topic: String::new(),
            partition: None,
            key: None,
            payload: Vec::new(),
            timestamp: SystemTime::now(),
            headers: MessageHeaders::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BrokerNode {
    pub id: String,
    pub address: String,
    pub port: u16,
}
