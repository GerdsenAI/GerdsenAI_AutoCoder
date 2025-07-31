#!/bin/bash

# Start SearXNG Docker containers for development
# This script provides a convenient way to start the SearXNG service

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Starting SearXNG development environment..."

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    echo "Error: Docker is not running. Please start Docker first."
    exit 1
fi

# Pull latest images
echo "Pulling latest SearXNG and Redis images..."
docker-compose pull

# Start services
echo "Starting SearXNG and Redis services..."
docker-compose up -d

# Wait for services to be healthy
echo "Waiting for services to become healthy..."
timeout=60
elapsed=0

while [ $elapsed -lt $timeout ]; do
    if docker-compose ps -q searxng | xargs docker inspect --format='{{.State.Health.Status}}' | grep -q "healthy"; then
        echo "✓ SearXNG is healthy and ready!"
        echo "✓ SearXNG is available at: http://localhost:8080"
        echo "✓ Health check endpoint: http://localhost:8080/healthz"
        exit 0
    fi
    
    echo "Waiting for SearXNG to become healthy... ($elapsed/${timeout}s)"
    sleep 5
    elapsed=$((elapsed + 5))
done

echo "Warning: SearXNG may not be fully ready yet. Check with: docker-compose logs searxng"
echo "Access SearXNG at: http://localhost:8080"