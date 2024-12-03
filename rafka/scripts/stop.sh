#!/bin/bash

# KILL
taskkill //F //IM "start_broker.exe" 2>/dev/null || true
taskkill //F //IM "start_consumer.exe" 2>/dev/null || true
taskkill //F //IM "start_producer.exe" 2>/dev/null || true

echo "All components stopped!" 