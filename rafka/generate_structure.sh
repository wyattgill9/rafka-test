#!/bin/bash

# Define the base directory where the structure will be created
BASE_DIR="./"

# List of directories to create
DIRS=(
    "src/broker"
    "src/core"
    "src/storage"
    "src/network"
    "src/consumer"
    "src/producer"
    "src/config"
    "src/utils"
    "src/tests"
    "docs"
    "bin"
    "examples"
    "scripts"
)

# Create the directories
for DIR in "${DIRS[@]}"; do
    mkdir -p "$BASE_DIR/$DIR"
done

# Create the files
touch "$BASE_DIR/src/broker/broker.rs"
touch "$BASE_DIR/src/broker/partition.rs"
touch "$BASE_DIR/src/broker/load_balancer.rs"

touch "$BASE_DIR/src/core/message.rs"
touch "$BASE_DIR/src/core/thread_pool.rs"
touch "$BASE_DIR/src/core/logger.rs"

touch "$BASE_DIR/src/storage/storage.rs"
touch "$BASE_DIR/src/storage/backup.rs"

touch "$BASE_DIR/src/network/p2p.rs"
touch "$BASE_DIR/src/network/transport.rs"
touch "$BASE_DIR/src/network/client.rs"

touch "$BASE_DIR/src/consumer/consumer.rs"
touch "$BASE_DIR/src/consumer/group.rs"
touch "$BASE_DIR/src/consumer/ack.rs"

touch "$BASE_DIR/src/producer/producer.rs"
touch "$BASE_DIR/src/producer/publisher.rs"
touch "$BASE_DIR/src/producer/serializer.rs"

touch "$BASE_DIR/src/config/config.rs"
touch "$BASE_DIR/src/config/defaults.rs"

touch "$BASE_DIR/src/utils/error.rs"
touch "$BASE_DIR/src/utils/timer.rs"
touch "$BASE_DIR/src/utils/metrics.rs"

touch "$BASE_DIR/src/tests/broker_tests.rs"
touch "$BASE_DIR/src/tests/consumer_tests.rs"
touch "$BASE_DIR/src/tests/producer_tests.rs"
touch "$BASE_DIR/src/tests/network_tests.rs"
touch "$BASE_DIR/src/tests/storage_tests.rs"

touch "$BASE_DIR/docs/architecture.md"
touch "$BASE_DIR/docs/getting_started.md"
touch "$BASE_DIR/docs/troubleshooting.md"

touch "$BASE_DIR/bin/start_broker.rs"
touch "$BASE_DIR/bin/start_consumer.rs"
touch "$BASE_DIR/bin/start_producer.rs"

touch "$BASE_DIR/examples/producer_example.rs"
touch "$BASE_DIR/examples/consumer_example.rs"

touch "$BASE_DIR/scripts/deploy_prod.sh"
touch "$BASE_DIR/scripts/deploy_dev.sh"
touch "$BASE_DIR/scripts/build.sh"
touch "$BASE_DIR/scripts/test.sh"

touch "$BASE_DIR/Cargo.toml"
touch "$BASE_DIR/README.md"
touch "$BASE_DIR/.gitignore"

echo "Folder structure and files have been created!"
