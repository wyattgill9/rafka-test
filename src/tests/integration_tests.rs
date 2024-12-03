use crate::{StatelessBroker, AsyncProducer, StatelessConsumer};
use tokio::test;

#[test]
async fn test_end_to_end_messaging() -> Result<()> {
    // Setup components
    let broker = StatelessBroker::new(Default::default());
    let producer = AsyncProducer::new(Default::default());
    let mut consumer = StatelessConsumer::new("test-group", Default::default());

    // Send messages
    for i in 0..100 {
        let message = Message::new(
            format!("key-{}", i),
            format!("value-{}", i).into(),
        );
        producer.send(message).await?;
    }

    // Consume messages
    let mut received = 0;
    while received < 100 {
        if let Some(msg) = consumer.poll_message().await? {
            received += 1;
            assert!(msg.payload.starts_with(b"value-"));
        }
    }

    Ok(())
}

#[test]
async fn test_fault_tolerance() -> Result<()> {
    let mut cluster = TestCluster::new(3); // 3 node cluster
    
    // Send initial messages
    let producer = cluster.create_producer();
    for i in 0..100 {
        producer.send(Message::new(format!("key-{}", i), vec![])).await?;
    }

    // Kill a node
    cluster.kill_node(1);

    // Should still be able to send and receive
    producer.send(Message::new("test-key", "test-value".into())).await?;
    
    let mut consumer = cluster.create_consumer("test-group");
    let msg = consumer.poll_message().await?.unwrap();
    assert_eq!(msg.payload, Bytes::from("test-value"));
    
    Ok(())
} 