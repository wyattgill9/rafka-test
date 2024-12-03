use crate::{StatelessBroker, Message, Result};
use tokio::test;
use std::time::Duration;

#[test]
async fn test_broker_message_routing() -> Result<()> {
    let broker = StatelessBroker::new(Default::default());
    let message = Message::new("test-key", "test-value".into());
    
    broker.route_message(message.clone()).await?;
    
    // Verify message reached correct partition
    let partition = broker.calculate_partition(&message);
    let received = broker.get_message(partition, message.id).await?;
    assert_eq!(received.payload, message.payload);
    
    Ok(())
}

#[test]
async fn test_broker_load_balancing() -> Result<()> {
    let broker = StatelessBroker::new(Default::default());
    let messages: Vec<_> = (0..1000)
        .map(|i| Message::new(format!("key-{}", i), vec![]))
        .collect();

    // Send messages
    for msg in messages.clone() {
        broker.route_message(msg).await?;
    }

    // Verify even distribution
    let partition_counts = broker.get_partition_message_counts().await;
    let max_diff = partition_counts.values().max().unwrap() - partition_counts.values().min().unwrap();
    assert!(max_diff <= 2, "Partition distribution is uneven");
    
    Ok(())
}

#[test]
async fn test_broker_failure_handling() -> Result<()> {
    let broker = StatelessBroker::new(Default::default());
    let message = Message::new("test-key", "test-value".into());

    // Simulate node failure
    broker.simulate_node_failure(1).await;
    
    // Message should still be routed successfully
    broker.route_message(message.clone()).await?;
    
    // Verify message was rerouted to available node
    let received = broker.get_message_from_any_node(message.id).await?;
    assert_eq!(received.payload, message.payload);
    
    Ok(())
}
