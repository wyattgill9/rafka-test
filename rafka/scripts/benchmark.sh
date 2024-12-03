#!/bin/bash
set -e

# default
MESSAGES=${1:-10000}
CONCURRENT_CLIENTS=${2:-10}
MESSAGE_SIZE=${3:-1024}

echo "Running benchmark with:"
echo "- Number of messages: $MESSAGES"
echo "- Concurrent clients: $CONCURRENT_CLIENTS"
echo "- Message size: $MESSAGE_SIZE bytes"

# cargo bench with custom parameters
RAFKA_BENCH_MESSAGES=$MESSAGES \
RAFKA_BENCH_CLIENTS=$CONCURRENT_CLIENTS \
RAFKA_BENCH_MSG_SIZE=$MESSAGE_SIZE \
cargo bench 