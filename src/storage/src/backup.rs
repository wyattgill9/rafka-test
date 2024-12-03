use async_trait::async_trait;
use bytes::Bytes;
use std::time::Duration;

#[async_trait]
pub trait BackupClient: Send + Sync {
    async fn backup(&self, key: String, data: Bytes) -> Result<()>;
    async fn restore(&self, key: &str) -> Result<Option<Bytes>>;
}

pub struct CloudBackupClient {
    client: Arc<CloudStorageClient>,
    retry_policy: RetryPolicy,
    compression: CompressionType,
}

impl CloudBackupClient {
    pub async fn backup_with_retry(&self, key: String, data: Bytes) -> Result<()> {
        let compressed = self.compress(data)?;
        
        let mut attempts = 0;
        loop {
            match self.client.upload(&key, compressed.clone()).await {
                Ok(_) => return Ok(()),
                Err(e) if attempts < self.retry_policy.max_attempts => {
                    attempts += 1;
                    tokio::time::sleep(self.retry_policy.backoff(attempts)).await;
                    continue;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}
