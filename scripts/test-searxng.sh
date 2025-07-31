#!/bin/bash

# SearXNG Integration Test Runner
# This script sets up and runs comprehensive SearXNG integration tests

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DOCKER_DIR="$PROJECT_ROOT/docker/searxng"

echo "🔍 SearXNG Integration Test Runner"
echo "=================================="

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    echo "❌ Error: Docker is not installed or not in PATH"
    exit 1
fi

if ! docker info >/dev/null 2>&1; then
    echo "❌ Error: Docker is not running"
    exit 1
fi

# Function to check if SearXNG is healthy
check_searxng_health() {
    local max_attempts=30
    local attempt=0
    
    echo "⏳ Waiting for SearXNG to become healthy..."
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -f -s http://localhost:8080/healthz >/dev/null 2>&1; then
            echo "✅ SearXNG is healthy and ready!"
            return 0
        fi
        
        echo "   Attempt $((attempt + 1))/$max_attempts - waiting..."
        sleep 2
        attempt=$((attempt + 1))
    done
    
    echo "❌ SearXNG failed to become healthy within $((max_attempts * 2)) seconds"
    return 1
}

# Function to start SearXNG if not running
start_searxng() {
    echo "🐳 Checking SearXNG Docker containers..."
    
    cd "$DOCKER_DIR"
    
    # Check if containers are already running
    if docker-compose ps -q searxng | xargs docker inspect --format='{{.State.Status}}' 2>/dev/null | grep -q "running"; then
        echo "✅ SearXNG containers are already running"
        
        # Still check if service is healthy
        if ! check_searxng_health; then
            echo "🔄 Restarting SearXNG containers..."
            docker-compose restart
            check_searxng_health
        fi
    else
        echo "🚀 Starting SearXNG containers..."
        docker-compose up -d
        check_searxng_health
    fi
}

# Function to run tests
run_tests() {
    echo "🧪 Running SearXNG Integration Tests..."
    
    cd "$PROJECT_ROOT/src-tauri"
    
    # Set test environment variables
    export RUST_TEST_THREADS=1  # Run tests serially to avoid conflicts
    export RUST_LOG=debug
    
    # Run different test suites
    echo ""
    echo "📋 Running Integration Tests..."
    cargo test searxng_integration_tests --lib -- --nocapture
    
    echo ""
    echo "⚡ Running Performance Tests..."
    cargo test searxng_performance_tests --lib -- --nocapture
    
    echo ""
    echo "🏥 Running Health Tests..."
    cargo test searxng_health_tests --lib -- --nocapture
    
    echo ""
    echo "🔍 Running All SearXNG Tests..."
    cargo test searxng --lib -- --nocapture
}

# Function to clean up
cleanup() {
    echo ""
    echo "🧹 Cleanup options:"
    echo "  To stop SearXNG containers: cd $DOCKER_DIR && docker-compose down"
    echo "  To remove data volumes: cd $DOCKER_DIR && docker-compose down -v"
    echo "  To view logs: cd $DOCKER_DIR && docker-compose logs"
}

# Main execution
main() {
    local stop_after_tests=false
    local skip_start=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --stop-after)
                stop_after_tests=true
                shift
                ;;
            --skip-start)
                skip_start=true
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --stop-after    Stop SearXNG containers after tests complete"
                echo "  --skip-start    Skip starting SearXNG (assume it's already running)"
                echo "  --help          Show this help message"
                echo ""
                exit 0
                ;;
            *)
                echo "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done
    
    # Start SearXNG if needed
    if [ "$skip_start" = false ]; then
        start_searxng
    else
        echo "⏭️  Skipping SearXNG startup (--skip-start specified)"
        if ! check_searxng_health; then
            echo "❌ SearXNG is not healthy. Remove --skip-start to auto-start."
            exit 1
        fi
    fi
    
    # Run tests
    if run_tests; then
        echo ""
        echo "✅ All SearXNG tests passed successfully!"
        test_result=0
    else
        echo ""
        echo "❌ Some SearXNG tests failed!"
        test_result=1
    fi
    
    # Stop containers if requested
    if [ "$stop_after_tests" = true ]; then
        echo ""
        echo "🛑 Stopping SearXNG containers..."
        cd "$DOCKER_DIR"
        docker-compose down
    fi
    
    cleanup
    exit $test_result
}

# Run main function with all arguments
main "$@"