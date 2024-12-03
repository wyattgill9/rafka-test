use rafka_core::{Config, Result, Error};
use scylla::{Session, SessionBuilder};
use bytes::Bytes;

pub struct Storage {
    session: Session,
}

impl Storage {
    pub async fn new(config: Config) -> Result<Self> {
        let session = SessionBuilder::new()
            .known_nodes(&config.nodes)
            .build()
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        // Use the keyspace
        session.use_keyspace(&config.keyspace, false)
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        Ok(Self { session })
    }

    pub async fn get(&self, _key: &str) -> Result<Option<Bytes>> {
        // TODO: Implement get logic
        Ok(None)
    }

    pub async fn put(&self, _key: String, _value: Bytes) -> Result<()> {
        // TODO: Implement put logic
        Ok(())
    }
} 