use rafka_broker::Broker;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let port = args.iter()
        .position(|arg| arg == "--port")
        .and_then(|i| args.get(i + 1))
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(50051);

    let partition_id = args.iter()
        .position(|arg| arg == "--partition")
        .and_then(|i| args.get(i + 1))
        .and_then(|p| p.parse::<u32>().ok())
        .unwrap_or(0);

    let total_partitions = args.iter()
        .position(|arg| arg == "--total-partitions")
        .and_then(|i| args.get(i + 1))
        .and_then(|p| p.parse::<u32>().ok())
        .unwrap_or(1);

    println!("Starting Rafka broker on 127.0.0.1:{} (partition {}/{})", 
             port, partition_id, total_partitions);

    let broker = Broker::new(partition_id, total_partitions);
    broker.serve(&format!("127.0.0.1:{}", port)).await?;
    Ok(())
} 