#!/bin/bash
set -e

MESSAGE_COUNT=${1:-100}
MESSAGE_SIZE=${2:-1024}  # 1KB
REPORT_INTERVAL=${3:-10}  # Report every 10 messages
MAX_RETRIES=3
RETRY_DELAY=5

echo "Starting Rafka benchmark..."
echo "- Message count: $MESSAGE_COUNT"
echo "- Message size: $MESSAGE_SIZE bytes"
echo "- Report interval: $REPORT_INTERVAL messages"

start_broker() {
    local retry=0
    while [ $retry -lt $MAX_RETRIES ]; do
        echo "Starting broker (attempt $((retry + 1))/$MAX_RETRIES)..."
        if cargo run --bin start_broker > broker.log 2>&1 & then
            BROKER_PID=$!
            # Wait for broker to start
            sleep 2
            if kill -0 $BROKER_PID 2>/dev/null; then
                echo "Broker started successfully (PID: $BROKER_PID)"
                return 0
            fi
        fi
        retry=$((retry + 1))
        echo "Failed to start broker, retrying in $RETRY_DELAY seconds..."
        sleep $RETRY_DELAY
    done
    echo "ERROR: Failed to start broker after $MAX_RETRIES attempts"
    cat broker.log
    return 1
}

start_consumer() {
    echo "Starting benchmark consumer..."
    cargo run --bin benchmark_consumer > consumer.log 2>&1 &
    CONSUMER_PID=$!
    sleep 2
    
    if ! kill -0 $CONSUMER_PID 2>/dev/null; then
        echo "ERROR: Consumer failed to start"
        cat consumer.log
        return 1
    fi
    echo "Consumer started successfully (PID: $CONSUMER_PID)"
}

cleanup() {
    echo "Cleaning up processes..."
    if [ -n "$BROKER_PID" ]; then
        echo "Stopping broker..."
        kill $BROKER_PID 2>/dev/null || true
        wait $BROKER_PID 2>/dev/null || true
    fi
    if [ -n "$CONSUMER_PID" ]; then
        echo "Stopping consumer..."
        kill $CONSUMER_PID 2>/dev/null || true
        wait $CONSUMER_PID 2>/dev/null || true
    fi
}

# Run benchmark producer
echo "Starting benchmark producer..."
cargo run --bin benchmark_producer $MESSAGE_COUNT $MESSAGE_SIZE $REPORT_INTERVAL

# Wait for completion and cleanup
sleep 2
cleanup

echo "Benchmark completed!" 