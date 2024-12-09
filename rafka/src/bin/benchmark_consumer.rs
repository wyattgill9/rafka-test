use rafka_consumer::Consumer;
use std::time::Instant;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut consumer = Consumer::new("127.0.0.1:50051").await?;
    consumer.subscribe("benchmark".to_string()).await?;
    
    println!("Starting benchmark consumer...");
    
    let mut message_count = 0;
    let mut total_latency = 0i64;
    let mut error_count = 0;
    let mut error_types = std::collections::HashMap::new();
    let start_time = Instant::now();
    
    // Create a future that completes when ctrl-c is pressed
    let shutdown = signal::ctrl_c();
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            result = consumer.receive_benchmark() => {
                match result {
                    Ok(msg) => {
                        message_count += 1;
                        total_latency += msg.latency_ms;
                        
                        if message_count % 10 == 0 {
                            println!(
                                "Received {} messages, avg latency: {:.2?}ms (errors: {})",
                                message_count,
                                total_latency as f64 / message_count as f64,
                                error_count
                            );
                        }
                    }
                    Err(e) => {
                        error_count += 1;
                        *error_types.entry(e.to_string()).or_insert(0) += 1;
                        
                        eprintln!("Error receiving message: {}", e);
                        if error_count >= 5 {
                            println!("\nStopping due to too many errors!");
                            break;
                        }
                        
                        // Attempt to reconnect
                        consumer = Consumer::new("127.0.0.1:50051").await?;
                        consumer.subscribe("benchmark".to_string()).await?;
                    }
                }
            }
            _ = &mut shutdown => {
                println!("Shutdown signal received, exiting...");
                break;
            }
        }
    }
    
    let duration = start_time.elapsed();
    
    println!("\nBenchmark Results:");
    println!("==================");
    println!("Duration: {:.2?}", duration);
    println!("Total messages received: {}", message_count);
    println!("Total errors: {}", error_count);
    if message_count > 0 {
        println!("Average latency: {:.2?}ms", total_latency as f64 / message_count as f64);
        println!("Messages/sec: {:.2}", message_count as f64 / duration.as_secs_f64());
    }
    
    if !error_types.is_empty() {
        println!("\nError Breakdown:");
        println!("===============");
        for (error, count) in error_types {
            println!("- {} occurred {} times", error, count);
        }
    }
    
    std::process::exit(if error_count > 0 { 1 } else { 0 });
} 