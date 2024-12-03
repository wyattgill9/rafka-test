use bytes::Bytes;
use serde::{Serialize, Deserialize};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

#[derive(Clone, Debug)]
pub struct Message {
    pub id: String,
    pub key: Option<String>,
    pub payload: Bytes,
    pub timestamp: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SerdeMessage {
    id: String,
    key: Option<String>,
    #[serde(with = "bytes_serde")]
    payload: Vec<u8>,
    timestamp: i64,
}

mod bytes_serde {
    use super::*;
    use serde::{Serializer, Deserializer};

    pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let b64 = BASE64.encode(bytes);
        serializer.serialize_str(&b64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        String::deserialize(deserializer)
            .and_then(|string| BASE64.decode(string.as_bytes())
                .map_err(|err| Error::custom(err.to_string())))
    }
}

#[derive(Clone, Debug)]
pub struct BrokerNode {
    pub id: String,
    pub address: String,
    pub port: u16,
}

impl Message {
    pub fn new(key: impl Into<String>, payload: impl Into<Bytes>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            key: Some(key.into()),
            payload: payload.into(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
} 