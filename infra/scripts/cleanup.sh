#!/bin/bash
set -e

NAMESPACE="rafka"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

log() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

# Remove all Rafka resources
log "Cleaning up Rafka deployment..."

kubectl delete namespace ${NAMESPACE} --ignore-not-found=true
kubectl delete pv -l app=rafka --ignore-not-found=true
kubectl delete storageclass rafka-storage --ignore-not-found=true

log "Cleanup complete!" 