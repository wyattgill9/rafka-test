#!/bin/bash
set -e

# config
if [ -f .env ]; then
    source .env
fi

export RAFKA_NODES=${RAFKA_NODES:-"localhost:9092"}
export RAFKA_KEYSPACE=${RAFKA_KEYSPACE:-"default"}
export RAFKA_BROKER_PORT=${RAFKA_BROKER_PORT:-8080}
export RAFKA_PRODUCER_PORT=${RAFKA_PRODUCER_PORT:-8081}
export RAFKA_CONSUMER_PORT=${RAFKA_CONSUMER_PORT:-8082}
export RAFKA_CACHE_TTL=${RAFKA_CACHE_TTL:-60}
export RUST_LOG=${RUST_LOG:-"info"}

echo "Starting Rafka components..."

# Start broker first
cd "$(pwd)" && cargo run --bin start_broker &
sleep 5

# Start consumer
cd "$(pwd)" && cargo run --bin start_consumer &
sleep 3

# Start producer
cd "$(pwd)" && cargo run --bin start_producer &
sleep 3

echo "All components started in background!"
echo "Use 'jobs' to see running processes"
echo "Press Ctrl+C to stop all components"

# Wait for all background processes
wait