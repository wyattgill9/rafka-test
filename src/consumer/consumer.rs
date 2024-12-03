use crate::core::{Message, Result};
use async_trait::async_trait;
use tokio::sync::mpsc;

pub struct StatelessConsumer {
    consumer_id: String,
    message_channel: mpsc::Receiver<Message>,
    processor: Box<dyn MessageProcessor>,
}

#[async_trait]
impl Consumer for StatelessConsumer {
    async fn start(&mut self) -> Result<()> {
        while let Some(msg) = self.message_channel.recv().await {
            self.process_message(msg).await?;
        }
        Ok(())
    }

    async fn process_message(&self, msg: Message) -> Result<()> {
        // Process in parallel using thread pool
        self.processor.process(msg).await
    }
}
