use crate::{AsyncProducer, Message, Result, ProducerConfig};
use tokio::test;
use bytes::Bytes;

#[test]
async fn test_producer_batching() -> Result<()> {
    let producer = AsyncProducer::new(ProducerConfig {
        batch_size: 10,
        compression: CompressionType::LZ4,
        ..Default::default()
    });

    // Send messages that should be batched
    let mut sent_messages = Vec::new();
    for i in 0..20 {
        let msg = Message::new(
            format!("key-{}", i),
            format!("value-{}", i).into(),
        );
        sent_messages.push(msg.clone());
        producer.send(msg).await?;
    }

    // Verify batches were created correctly
    let batches = producer.get_pending_batches().await;
    assert!(batches.len() <= 2, "Messages should be in at most 2 batches");
    
    // Verify compression
    for batch in batches {
        assert!(batch.is_compressed());
        assert_eq!(batch.compression_type(), CompressionType::LZ4);
    }

    Ok(())
}

#[test]
async fn test_producer_backpressure() -> Result<()> {
    let producer = AsyncProducer::new(ProducerConfig {
        max_in_flight: 100,
        ..Default::default()
    });

    // Send messages until backpressure kicks in
    let start = std::time::Instant::now();
    let mut sent = 0;
    
    while sent < 1000 && start.elapsed() < std::time::Duration::from_secs(1) {
        if producer.send(Message::new("key", vec![0; 1024 * 1024])).await.is_ok() {
            sent += 1;
        }
    }

    assert!(sent < 1000, "Backpressure should have limited sending");
    Ok(())
}
