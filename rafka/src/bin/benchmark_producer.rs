use rafka_producer::Producer;
use std::env;
use std::time::Instant;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let message_count: usize = env::args()
        .nth(1)
        .unwrap_or("10000".to_string())
        .parse()?;
    let message_size: usize = env::args()
        .nth(2)
        .unwrap_or("1024".to_string())
        .parse()?;
    let report_interval: usize = env::args()
        .nth(3)
        .unwrap_or("1000".to_string())
        .parse()?;

    let mut producer = Producer::new("127.0.0.1:50051").await?;
    let payload = vec![0u8; message_size];
    
    println!("Starting benchmark with:");
    println!("- Message count: {}", message_count);
    println!("- Message size: {} bytes", message_size);
    
    let start = Instant::now();
    let mut last_report = start;
    let mut messages_since_report = 0;
    let mut error_count = 0;
    let mut error_types = std::collections::HashMap::new();
    let mut retry_count = 0;
    const MAX_RETRIES: usize = 3;

    for i in 0..message_count {
        match producer.send_benchmark("benchmark".to_string(), payload.clone()).await {
            Ok(_) => {
                messages_since_report += 1;
                retry_count = 0;
            }
            Err(e) => {
                error_count += 1;
                *error_types.entry(e.to_string()).or_insert(0) += 1;
                
                retry_count += 1;
                if retry_count >= MAX_RETRIES {
                    println!("\nStopping due to consecutive failures!");
                    break;
                }
                eprintln!("Error sending message, retrying ({}/{}): {}", retry_count, MAX_RETRIES, e);
                sleep(Duration::from_millis(100)).await;
                producer = Producer::new("127.0.0.1:50051").await?;
            }
        }

        if messages_since_report >= report_interval {
            let now = Instant::now();
            let duration = now.duration_since(last_report);
            let throughput = messages_since_report as f64 / duration.as_secs_f64();
            
            println!(
                "Sent {} messages ({:.2} msgs/sec, errors: {})", 
                i + 1,
                throughput,
                error_count
            );
            
            messages_since_report = 0;
            last_report = now;
        }
    }

    let total_duration = start.elapsed();
    let total_throughput = message_count as f64 / total_duration.as_secs_f64();
    
    println!("\nBenchmark Results:");
    println!("==================");
    println!("Duration: {:.2?}", total_duration);
    println!("Messages sent: {}", message_count);
    println!("Average throughput: {:.2} msgs/sec", total_throughput);
    println!("Total errors: {}", error_count);
    
    if !error_types.is_empty() {
        println!("\nError Breakdown:");
        println!("===============");
        for (error, count) in error_types {
            println!("- {} occurred {} times", error, count);
        }
    }
    
    std::process::exit(if error_count > 0 { 1 } else { 0 });
} 