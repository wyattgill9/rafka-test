#!/bin/bash
set -e

# Base Path
BASE_PATH="./k8s/base/"

# Install ScyllaDB CRDs first
echo "Installing ScyllaDB CRDs..." #FIX GITHUB AND ITS ALL GOOD!!!
kubectl apply -f https://raw.githubusercontent.com/scylladb/scylla-operator/v1.7.2/config/crd/bases/scylla.scylladb.com_scyllaclusters.yaml

# Wait a moment for CRD to register
sleep 5

# Continue with rest of deployment...
echo "Setting up In Memory Data Store configuration..."
# kubectl apply -f "$BASE_PATH/skytable/config.yaml" OR REDIS OR MEMCACHED 

# Apply monitoring stack
echo "Setting up monitoring..."
kubectl apply -f "$BASE_PATH/monitoring/secrets.yaml"
kubectl apply -f "$BASE_PATH/monitoring/prometheus-config.yaml"
kubectl apply -f "$BASE_PATH/monitoring/prometheus-deployment.yaml"
kubectl apply -f "$BASE_PATH/monitoring/grafana-deployment.yaml"
kubectl apply -f "$BASE_PATH/monitoring/services.yaml"

# Apply Rafka resources
echo "Deploying Rafka..."
kubectl apply -f "$BASE_PATH/rafka/deployment.yaml"
kubectl apply -f "$BASE_PATH/rafka/headless-service.yaml"
kubectl apply -f "$BASE_PATH/rafka/hpa.yaml"
kubectl apply -f "$BASE_PATH/rafka/pdb.yaml"

# Apply ScyllaDB resources
echo "Setting up ScyllaDB..."
kubectl apply -f "$BASE_PATH/scylla/rbac.yaml"
kubectl apply -f "$BASE_PATH/scylla/scylla-operator.yaml"
kubectl wait --for=condition=ready pod -l app=scylla-operator --timeout=300s
kubectl apply -f "$BASE_PATH/scylla/scyllacluster.yaml"
kubectl apply -f "$BASE_PATH/scylla/service.yml"

echo "Deployment completed successfully!"

# Wait for all pods to be ready
kubectl wait --for=condition=ready pod --all --timeout=300s

echo "All systems are up and running!"
