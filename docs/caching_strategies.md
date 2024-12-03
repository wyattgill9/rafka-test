# Rafka Caching Strategies

Rafka supports two primary caching strategies: Broker-Specific and Shared Cache. Each has its own advantages and use cases.

## 1. Broker-Specific Cache (Traditional)

### Characteristics
- Each broker maintains cache for its owned partitions
- Direct routing to partition owners
- Strong consistency within partition boundaries
- Minimal cross-broker communication

### Best For
- High-throughput scenarios
- When data locality is important
- When strong partition-level consistency is required
- Systems with predictable access patterns

### Implementation 

```rust
// Example broker-specific cache usage
let cache = DistributedCache::new(config);
cache.put("key1", value).await?;
let value = cache.get("key1").await?;
```

## 2. Shared Cache Model

### Characteristics
- Any broker can serve any request
- Eventually consistent updates across brokers
- Configurable consistency levels
- Higher availability for reads

### Best For
- Read-heavy workloads
- Systems requiring high availability
- When data access patterns are unpredictable
- When network latency between brokers is low

### Implementation

```rust
// Example shared cache usage
let cache = SharedCache::new(config);
cache.put_with_consistency("key1", value, ConsistencyLevel::Quorum).await?;
let value = cache.get_with_consistency("key1", ConsistencyLevel::One).await?;
```
## Choosing a Strategy

### Use Broker-Specific When:
- You need strong consistency
- Your data naturally partitions well
- Network bandwidth between brokers is limited
- Write performance is critical

### Use Shared Cache When:
- Read availability is paramount
- You can tolerate eventual consistency
- Network latency between brokers is low
- You need flexible consistency levels