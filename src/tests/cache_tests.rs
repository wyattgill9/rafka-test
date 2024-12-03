use crate::storage::{SharedCache, DistributedCache, ConsistencyLevel};
use tokio::test;
use bytes::Bytes;
use std::time::Duration;

#[test]
async fn test_shared_cache_basic_operations() -> Result<()> {
    let cache = SharedCache::new(
        Duration::from_secs(60),
        3,
        ConsistencyLevel::Quorum,
    );

    // Test put
    cache.put("test-key".into(), Bytes::from("test-value")).await?;
    
    // Test get
    let value = cache.get("test-key").await?;
    assert_eq!(value.unwrap(), Bytes::from("test-value"));
    
    Ok(())
}

#[test]
async fn test_shared_cache_consistency_levels() -> Result<()> {
    let cache = SharedCache::new(
        Duration::from_secs(60),
        3,
        ConsistencyLevel::Quorum,
    );

    // Test with different consistency levels
    for level in [ConsistencyLevel::One, ConsistencyLevel::Quorum, ConsistencyLevel::All] {
        cache.set_consistency_level(level);
        
        cache.put("key".into(), Bytes::from("value")).await?;
        let value = cache.get("key").await?;
        
        assert_eq!(value.unwrap(), Bytes::from("value"));
    }
    
    Ok(())
}

#[test]
async fn test_distributed_cache_partitioning() -> Result<()> {
    let cache = DistributedCache::new(Default::default());
    
    // Insert data across different partitions
    for i in 0..100 {
        let key = format!("key-{}", i);
        let value = format!("value-{}", i);
        cache.put(key, Bytes::from(value)).await?;
    }

    // Verify partition distribution
    let partition_stats = cache.get_partition_stats().await;
    assert!(partition_stats.len() > 1, "Data should be distributed across partitions");
    
    Ok(())
} 