#!/bin/bash
set -e

NAMESPACE="rafka"
CLUSTER_NAME="rafka-cluster"
REGION="us-west-2"  # We can change this as needed

GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

log() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

command -v kubectl >/dev/null 2>&1 || error "kubectl is required but not installed"

log "Creating namespace ${NAMESPACE}..."
kubectl create namespace ${NAMESPACE} --dry-run=client -o yaml | kubectl apply -f -

log "Applying Rafka configurations..."

log "Setting up storage..."
kubectl apply -f ../k8s/storage/storage-class.yaml
kubectl apply -f ../k8s/storage/persistent-volumes.yaml

log "Deploying core components..."
kubectl apply -f ../k8s/core/configmap.yaml
kubectl apply -f ../k8s/core/secrets.yaml
kubectl apply -f ../k8s/core/service-accounts.yaml

log "Deploying Rafka broker..."
kubectl apply -f ../k8s/broker/statefulset.yaml
kubectl apply -f ../k8s/broker/service.yaml
kubectl apply -f ../k8s/broker/hpa.yaml

log "Setting up monitoring..."
kubectl apply -f ../k8s/monitoring/alerts.yaml
kubectl apply -f ../k8s/monitoring/grafana.yaml
kubectl apply -f ../k8s/monitoring/prometheus.yaml

log "Applying network policies..."
kubectl apply -f ../k8s/network/policies.yaml

log "Deployment complete! Waiting for pods to be ready..."
kubectl wait --for=condition=ready pod -l app=rafka -n ${NAMESPACE} --timeout=300s

log "Rafka deployment successful!" 