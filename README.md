# Rafka

Rafka is an blazingly fast experimental distributed async message broker written in Rust, currently in early development. It uses a experimental "Pastry" based, structure pear-to-pear broker "mesh" architecture with Redis/Memcached acting as a "sidecar" cache. Along with a Supernode/Ordinary node based system for distributed metadata. Designed to effortlessly deploy with Kubernetes, ensuring Maximum possible scalability, fault tolerance and efficiency.

## Current Status: Early Development

This project is in active development and **Not ready for production use**. 

#### We are Currently building a TCP version first, but we will migrate to gRPC after.

### What's Working

- ✅ Basic client-server message routing
- ✅ ScyllaDB integration for persistence
- ✅ Redis/Memcached caching layer
- ✅ Simple producer/consumer clients
- ✅ Basic broker implementation
- ✅ Kubernetes deployment configs

### Under Development

- � Peer-to-peer networking
- 🚧 Distributed broker coordination
- 🚧 Advanced message routing
- 🚧 Fault tolerance
- 🚧 Production hardening

## Architecture

Current implementation:

```plaintext
                   ┌──────────────┐
                   │    Broker    │
┌──────────────┐   │              │   ┌──────────────┐
│   Producer   │─▶│   Message    │──▶│   Consumer   │
└──────────────┘   │   Routing    │   └──────────────┘
                   └──────┬───────┘
                          │
                    ┌─────┴─────┐
                    │  Storage  │
                    │  Layer    │
                    └─────┬─────┘
                          │                       
                    ┌─────▼─────┐           
                    │   Redis/  │           
                    │ Memcached │           
                    └───────────┘
```

## Components

### Broker
**Currently implemented:**
- Single-node message routing
- Basic client connection handling
- Simple message dispatching
- Storage integration

**Goal implementation:**
- Multi-node structured peer-to-peer message routing and replication
- Distributed metadata management
- Advanced client connection handling
- Advanced message dispatching
- Advanced storage integration

### Storage
**Currently supported:**
- **Cache**: ScyllaDB

**Goal implementation:**
- **Cache**: Redis/Memcached/Skytable/ScyllaDB

### Client Libraries
Basic implementations for:
- Producer client
- Consumer client with simple acknowledgments

## Getting Started

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install dependencies
sudo apt-get install -y build-essential protobuf-compiler

# Start ScyllaDB (required)
docker run -d --name scylla -p 9042:9042 scylladb/scylla

# Start Redis (optional, for caching)
docker run -d --name redis -p 6379:6379 redis
```

### Quick Start

```bash
# Clone repository
git clone https://github.com/yourusername/rafka.git
cd rafka

# Set environment variables
export RAFKA_PORT=8080
export RAFKA_NODES=localhost:9042
export RAFKA_KEYSPACE=default
export RAFKA_CACHE_TTL=60

# Start all components
./scripts/start.sh
```

### Basic Usage

Start a broker:
```rust
// src/bin/start_broker.rs
#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::from_env()?;
    let broker = StatelessBroker::new(config);
    broker.start().await?;
    Ok(())
}
```

Send a message:
```rust
// Example producer usage
let mut producer = Producer::connect(&config).await?;
producer.send(Message::new(
    "test-topic",
    "test-key",
    "Hello World!".as_bytes(),
)).await?;
```

Receive messages:
```rust
// Example consumer usage
let mut consumer = Consumer::connect(&config).await?;
while let Some(msg) = consumer.receive().await? {
    println!("Received: {:?}", msg);
    consumer.acknowledge(msg.id).await?;
}
```

## Configuration

Current environment variables:

```bash
# Required
RAFKA_PORT=8080              # Broker port
RAFKA_NODES=localhost:9042   # ScyllaDB nodes
RAFKA_KEYSPACE=default       # ScyllaDB keyspace

# Optional
RAFKA_CACHE_TTL=60          # Cache TTL in seconds
RUST_LOG=info               # Log level
```

## Development

```bash
# Run tests
cargo test

# Run with hot reload
cargo watch -x run

# Format code
cargo fmt
```

## Roadmap

1. **Phase 1 - In Progress**
   - ✅ (Very) Basic message routing
   - 🚧 ScyllaDB integration
   - 🚧Simple client libraries

2. **Phase 2 - In Progress**
   - 🚧 Peer-to-peer networking
   - 🚧 Distributed coordination
   - 🚧 Advanced routing

3. **Phase 3 - Planned**
   - 📋 Fault tolerance
   - 📋 Production hardening
   - 📋 Performance optimization

## Contributing

We welcome all contributions! The project is in very early stages, so there are many areas to help:

1. Core broker functionality
2. Storage engine optimizations
3. P2P networking implementation
4. Documentation improvements
5. Test coverage
6. Kubernetes deployment configs
7. Client libraries
8. Performance optimizations
9. Bug fixes
10. Feature requests

## License

Apache License 2.0 - See [LICENSE](./LICENSE) for details.
```