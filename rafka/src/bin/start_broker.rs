use rafka_broker::Broker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let broker = Broker::new();
    broker.serve("127.0.0.1:50051").await?;
    Ok(())
} 