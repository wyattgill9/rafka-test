use rafka_consumer::Consumer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut consumer = Consumer::new("127.0.0.1:50051").await?;
    
    // Subscribe to the hello world topic
    consumer.subscribe("hello".to_string()).await?;
    
    // Start consuming messages
    consumer.start_consuming().await?;
    
    Ok(())
} 