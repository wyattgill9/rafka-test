use rafka_consumer::Consumer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut consumer = Consumer::new("127.0.0.1:50051").await?;
    consumer.subscribe("greetings".to_string()).await?;
    println!("Consumer ready - listening for messages on 'greetings' topic");
    consumer.start_consuming().await?;
    Ok(())
} 