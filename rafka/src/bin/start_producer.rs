use rafka_producer::Producer;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let message = env::args()
        .nth(2)
        .unwrap_or_else(|| "Hello, World!".to_string());

    let mut producer = Producer::new("127.0.0.1:50051").await?;
    
    // Send hello world message
    producer.send(
        "hello".to_string(),
        message.as_bytes().to_vec(),
    ).await?;
    
    Ok(())
} 