use crate::network::{Transport, MessageCodec};
use tokio::test;
use bytes::BytesMut;

#[test]
async fn test_message_codec() -> Result<()> {
    let mut codec = MessageCodec::new(1024 * 1024); // 1MB max frame
    let mut buf = BytesMut::new();

    // Test encoding
    let message = Message::new("test-key", "test-value".into());
    codec.encode(message.clone(), &mut buf)?;

    // Test decoding
    let decoded = codec.decode(&mut buf)?.unwrap();
    assert_eq!(decoded.payload, message.payload);
    
    Ok(())
}

#[test]
async fn test_transport_zero_copy() -> Result<()> {
    let (client, server) = tokio::io::duplex(1024);
    let mut client_transport = Transport::new(client);
    let mut server_transport = Transport::new(server);

    // Create large message
    let large_data = vec![0u8; 1024 * 1024];
    let message = Message::new("large-key", Bytes::from(large_data.clone()));

    // Send using zero-copy
    client_transport.send_message_zero_copy(message.clone()).await?;

    // Receive and verify
    let received = server_transport.receive_message().await?;
    assert_eq!(received.payload.len(), large_data.len());
    
    Ok(())
}

#[test]
async fn test_network_partitioning() -> Result<()> {
    let mut network = TestNetwork::new(3); // 3 nodes
    
    // Simulate network partition
    network.partition_node(1);
    
    // Verify messages still route correctly
    let message = Message::new("test-key", "test-value".into());
    network.send_message(0, message.clone()).await?;
    
    let received = network.receive_message(2).await?;
    assert_eq!(received.payload, message.payload);
    
    Ok(())
}
