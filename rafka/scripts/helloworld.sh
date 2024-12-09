#!/bin/bash
set -e

echo "Starting Rafka Hello World demo..."

# Start broker in the background
echo "Starting broker..."
cargo run --bin start_broker &
BROKER_PID=$!
sleep 2

# Start consumer in the background
echo "Starting consumer..."
cargo run --bin start_consumer &
CONSUMER_PID=$!
sleep 2

# Run producer with hello world message
echo "Sending Hello World message..."
cargo run --bin start_producer "Hello, World!"

# Wait for the message to be processed
sleep 1

# Clean up
echo "Message sent! Cleaning up..."
kill $BROKER_PID $CONSUMER_PID 2>/dev/null || true
wait

./kill.sh

echo "Demo completed!"