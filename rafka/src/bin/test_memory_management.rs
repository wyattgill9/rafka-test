use rafka::{
    broker::{
        memory_store::InMemoryStore,
        metrics::PerformanceMonitor,
        metadata::MetadataManager,
    },
    network::SuperNodeManager,
};
use std::sync::Arc;
use tokio::time::Duration;
use rand::{thread_rng, Rng};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting memory management tests...");

    let test_env = TestEnvironment::new().await?;
    
    // Run test scenarios
    test_memory_pressure(&test_env).await?;
    test_eviction_strategy(&test_env).await?;
    test_hot_partition_retention(&test_env).await?;
    test_memory_recovery(&test_env).await?;
    test_concurrent_memory_ops(&test_env).await?;

    println!("All memory tests completed successfully!");
    Ok(())
}

struct TestEnvironment {
    memory_store: Arc<InMemoryStore>,
    performance_monitor: Arc<PerformanceMonitor>,
    metadata_manager: Arc<MetadataManager>,
}

impl TestEnvironment {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let node_id = "test-node-1".to_string();
        let supernode_manager = Arc::new(SuperNodeManager::new(node_id.clone()).await?);
        
        Ok(Self {
            memory_store: Arc::new(InMemoryStore::new(node_id.clone())),
            performance_monitor: Arc::new(PerformanceMonitor::new(node_id.clone()).await?),
            metadata_manager: Arc::new(MetadataManager::new(
                node_id,
                supernode_manager,
            ).await?),
        })
    }
}

async fn test_memory_pressure(env: &TestEnvironment) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nTesting memory pressure handling...");

    // Fill memory to trigger pressure
    let mut rng = thread_rng();
    let test_data = generate_test_data(1024 * 1024); // 1MB chunks

    for i in 0..1000 {
        let partition_id = format!("pressure-test-{}", i);
        env.memory_store.store(
            &partition_id,
            test_data.clone(),
            Some(Duration::from_secs(rng.gen_range(1..100))),
        ).await?;

        if i % 100 == 0 {
            let metrics = env.performance_monitor.collect_partition_metrics(&partition_id).await?;
            println!("Memory usage after {} insertions: {} bytes", i, metrics.memory_usage_bytes);
        }
    }

    // Verify memory pressure triggered eviction
    let final_metrics = env.performance_monitor.collect_partition_metrics("pressure-test-0").await?;
    assert!(final_metrics.memory_usage_bytes < env.memory_store.get_capacity_limit());

    Ok(())
}

async fn test_eviction_strategy(env: &TestEnvironment) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nTesting eviction strategy...");

    // Create partitions with different access patterns
    let hot_partition = "hot-partition";
    let warm_partition = "warm-partition";
    let cold_partition = "cold-partition";

    // Set up test data
    let test_data = generate_test_data(1024 * 1024); // 1MB
    
    // Store data with different access patterns
    env.memory_store.store(hot_partition, test_data.clone(), None).await?;
    env.memory_store.store(warm_partition, test_data.clone(), None).await?;
    env.memory_store.store(cold_partition, test_data.clone(), None).await?;

    // Simulate access patterns
    for _ in 0..1000 {
        env.memory_store.get(hot_partition).await?;
        if thread_rng().gen_bool(0.5) {
            env.memory_store.get(warm_partition).await?;
        }
        // Cold partition not accessed
    }

    // Force memory pressure
    let large_data = generate_test_data(1024 * 1024 * 100); // 100MB
    env.memory_store.store("pressure-trigger", large_data, None).await?;

    // Verify eviction preferences
    assert!(env.memory_store.get(hot_partition).await?.is_some(), "Hot partition was evicted");
    assert!(env.memory_store.get(cold_partition).await?.is_none(), "Cold partition was not evicted");

    Ok(())
}

async fn test_hot_partition_retention(env: &TestEnvironment) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nTesting hot partition retention...");

    let hot_partition = "retention-hot";
    let test_data = generate_test_data(1024 * 1024); // 1MB

    // Mark partition as hot
    env.metadata_manager.convert_to_hot_path(hot_partition).await?;
    env.memory_store.store(hot_partition, test_data.clone(), None).await?;

    // Fill memory to trigger eviction
    for i in 0..100 {
        env.memory_store.store(
            &format!("filler-{}", i),
            test_data.clone(),
            None,
        ).await?;
    }

    // Verify hot partition retained
    assert!(env.memory_store.get(hot_partition).await?.is_some(), "Hot partition was incorrectly evicted");
    
    // Verify metrics
    let metrics = env.performance_monitor.collect_partition_metrics(hot_partition).await?;
    assert!(metrics.cache_hit_ratio > 0.9, "Hot partition cache hit ratio too low");

    Ok(())
}

async fn test_memory_recovery(env: &TestEnvironment) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nTesting memory recovery...");

    // Fill memory
    let test_data = generate_test_data(1024 * 1024); // 1MB
    for i in 0..100 {
        env.memory_store.store(
            &format!("recovery-test-{}", i),
            test_data.clone(),
            None,
        ).await?;
    }

    // Record initial memory usage
    let initial_usage = env.performance_monitor
        .collect_partition_metrics("recovery-test-0")
        .await?
        .memory_usage_bytes;

    // Force eviction
    env.memory_store.force_eviction().await?;

    // Verify memory recovered
    let final_usage = env.performance_monitor
        .collect_partition_metrics("recovery-test-0")
        .await?
        .memory_usage_bytes;

    assert!(final_usage < initial_usage, "Memory not recovered after eviction");

    Ok(())
}

async fn test_concurrent_memory_ops(env: &TestEnvironment) -> Result<(), Box<dyn std::error::Error>> {
    println!("\nTesting concurrent memory operations...");

    let test_data = generate_test_data(1024 * 1024); // 1MB
    let mut handles = vec![];

    // Spawn concurrent operations
    for i in 0..10 {
        let store = env.memory_store.clone();
        let data = test_data.clone();
        let handle = tokio::spawn(async move {
            for j in 0..10 {
                let partition_id = format!("concurrent-{}-{}", i, j);
                store.store(&partition_id, data.clone(), None).await?;
                store.get(&partition_id).await?;
                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            }
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.await??;
    }

    // Verify memory consistency
    let metrics = env.performance_monitor.collect_partition_metrics("concurrent-0-0").await?;
    assert!(metrics.memory_usage_bytes > 0, "Memory operations failed");

    Ok(())
}

fn generate_test_data(size: usize) -> Vec<u8> {
    let mut rng = thread_rng();
    let mut data = Vec::with_capacity(size);
    for _ in 0..size {
        data.push(rng.gen());
    }
    data
} 