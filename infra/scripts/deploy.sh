#!/bin/bash
set -e

# Base Path
BASE_PATH="./k8s/base/"

# Wait a moment for CRD to register
sleep 5

# Continue with rest of deployment...
echo "Setting up In Memory Data Store configuration..."
# kubectl apply -f "$BASE_PATH/skytable/config.yaml" OR REDIS OR MEMCACHED 

# Apply monitoring stack
echo "Setting up monitoring..."

# Apply Rafka resources
echo "Deploying Rafka..."
kubectl apply -f "$BASE_PATH/rafka/deployment.yaml"
kubectl apply -f "$BASE_PATH/rafka/headless-service.yaml"
kubectl apply -f "$BASE_PATH/rafka/hpa.yaml"
kubectl apply -f "$BASE_PATH/rafka/pdb.yaml"

echo "Deployment completed successfully!"

# Wait for all pods to be ready
kubectl wait --for=condition=ready pod --all --timeout=300s

echo "All systems are up and running!"
