use crate::core::{Message, Result};
use crossbeam::channel;
use parking_lot::RwLock;

pub struct AsyncProducer {
    batch_size: usize,
    message_queue: channel::Sender<Message>,
    compression: CompressionType,
    batch_processor: Arc<RwLock<BatchProcessor>>,
}

impl AsyncProducer {
    pub async fn send(&self, msg: Message) -> Result<()> {
        if self.should_batch() {
            self.batch_processor.write().add(msg);
        } else {
            self.direct_send(msg).await?;
        }
        Ok(())
    }

    async fn flush_batch(&self) -> Result<()> {
        let batch = self.batch_processor.write().take_batch();
        if !batch.is_empty() {
            self.send_batch(batch).await?;
        }
        Ok(())
    }
}
