use crate::storage::{StorageManager, ScyllaStorage, Result};
use scylla::{Session, SessionBuilder};
use tokio::test;
use uuid::Uuid;

#[test]
async fn test_scylla_storage_basic() -> Result<()> {
    let storage = ScyllaStorage::new(get_test_session().await?);
    
    // Test basic write and read
    let key = Uuid::new_v4().to_string();
    let value = b"test value".to_vec();
    
    storage.write(&key, &value).await?;
    let read_value = storage.read(&key).await?;
    
    assert_eq!(read_value, Some(value));
    Ok(())
}

#[test]
async fn test_storage_caching() -> Result<()> {
    let storage = StorageManager::new(
        ScyllaStorage::new(get_test_session().await?),
        CacheConfig::default(),
    );

    // Write data
    let key = Uuid::new_v4().to_string();
    let value = b"cached value".to_vec();
    storage.write(&key, &value).await?;

    // First read should hit ScyllaDB
    let first_read = storage.read(&key).await?;
    assert_eq!(first_read, Some(value.clone()));
    assert_eq!(storage.get_cache_stats().await.misses, 1);

    // Second read should hit cache
    let second_read = storage.read(&key).await?;
    assert_eq!(second_read, Some(value));
    assert_eq!(storage.get_cache_stats().await.hits, 1);

    Ok(())
}

async fn get_test_session() -> Result<Session> {
    SessionBuilder::new()
        .known_node("localhost:9042")
        .keyspace("rafka_test")
        .build()
        .await
        .map_err(Into::into)
}
