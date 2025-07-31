#!/bin/bash

# Stop SearXNG Docker containers
# This script provides a convenient way to stop the SearXNG service

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Stopping SearXNG development environment..."

# Stop services
docker-compose down

echo "✓ SearXNG services stopped successfully!"

# Optional: Remove volumes (uncomment if you want to reset data)
# echo "Removing persistent data volumes..."
# docker-compose down -v
# echo "✓ Volumes removed!"