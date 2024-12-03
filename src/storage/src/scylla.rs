use crate::core::Result;
use scylla::{Session, SessionBuilder, SerializeResult, Statement};
use bytes::Bytes;
use std::time::Duration;

pub struct ScyllaStorage {
    session: Session,
    prepared_statements: PreparedStatements,
}

struct PreparedStatements {
    write: Statement,
    read: Statement,
    delete: Statement,
}

impl ScyllaStorage {
    pub async fn new(nodes: Vec<String>, keyspace: String) -> Result<Self> {
        let session = SessionBuilder::new()
            .known_nodes(&nodes)
            .keyspace(keyspace)
            .connect()
            .await?;

        // Create keyspace and tables if they don't exist
        Self::initialize_schema(&session).await?;

        // Prepare statements
        let prepared = PreparedStatements {
            write: session.prepare("INSERT INTO messages (key, value, timestamp) VALUES (?, ?, ?)").await?,
            read: session.prepare("SELECT value FROM messages WHERE key = ?").await?,
            delete: session.prepare("DELETE FROM messages WHERE key = ?").await?,
        };

        Ok(Self {
            session,
            prepared_statements: prepared,
        })
    }

    pub async fn write(&self, key: &str, value: Bytes) -> Result<()> {
        self.session
            .execute(&self.prepared_statements.write, (
                key,
                value.to_vec(),
                chrono::Utc::now(),
            ))
            .await?;
        Ok(())
    }

    pub async fn read(&self, key: &str) -> Result<Option<Bytes>> {
        let result = self.session
            .execute(&self.prepared_statements.read, (key,))
            .await?;

        Ok(result.first_row().map(|row| {
            let value: Vec<u8> = row.get_bytes("value")
                .expect("value column missing")
                .to_vec();
            Bytes::from(value)
        }))
    }

    async fn initialize_schema(session: &Session) -> Result<()> {
        // Create keyspace if not exists
        session.query(
            "CREATE KEYSPACE IF NOT EXISTS rafka 
             WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 3}",
            &[],
        ).await?;

        // Create messages table
        session.query(
            "CREATE TABLE IF NOT EXISTS messages (
                key text PRIMARY KEY,
                value blob,
                timestamp timestamp
            )",
            &[],
        ).await?;

        Ok(())
    }
} 