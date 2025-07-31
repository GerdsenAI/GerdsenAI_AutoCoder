# SearXNG Integration Setup Guide

This guide covers setting up and configuring SearXNG integration for the CSE-Icon AutoCoder application.

## Table of Contents
- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Docker Setup](#docker-setup)
- [Configuration](#configuration)
- [Testing](#testing)
- [Troubleshooting](#troubleshooting)
- [Development](#development)

## Prerequisites

- **Docker Desktop**: Ensure Docker is installed and running
  - Windows: Download from [Docker Desktop for Windows](https://docs.docker.com/desktop/windows/install/)
  - macOS: Download from [Docker Desktop for Mac](https://docs.docker.com/desktop/mac/install/)
  - Linux: Install using your distribution's package manager

- **Rust & Cargo**: Required for running integration tests
- **Node.js**: Required for frontend development

## Quick Start

### 1. Start SearXNG Service

**Windows:**
```batch
cd docker\searxng
start-searxng.bat
```

**Linux/macOS:**
```bash
cd docker/searxng
./start-searxng.sh
```

### 2. Verify Service Health

Open your browser and navigate to:
- **SearXNG Interface**: http://localhost:8080
- **Health Check**: http://localhost:8080/healthz

### 3. Run Integration Tests

```bash
# Run comprehensive test suite
./scripts/test-searxng.sh

# Or on Windows
scripts\test-searxng.bat
```

## Docker Setup

### Architecture

The SearXNG setup includes:
- **SearXNG**: Main search engine service (port 8080)
- **Redis**: Caching and rate limiting (internal)
- **Docker Networks**: Isolated network for services
- **Persistent Volumes**: Data persistence across restarts

### Configuration Files

| File | Purpose |
|------|---------|
| `docker-compose.yml` | Service orchestration |
| `searxng-settings.yml` | SearXNG configuration |
| `limiter.toml` | Rate limiting rules |
| `start-searxng.sh/.bat` | Startup scripts |
| `stop-searxng.sh/.bat` | Shutdown scripts |

### Service Management

**Start Services:**
```bash
cd docker/searxng
docker-compose up -d
```

**Stop Services:**
```bash
docker-compose down
```

**View Logs:**
```bash
docker-compose logs -f searxng
```

**Restart Services:**
```bash
docker-compose restart
```

**Remove All Data:**
```bash
docker-compose down -v  # Warning: This removes all data!
```

### Port Configuration

- **8080**: SearXNG web interface and API
- **6379**: Redis (internal only)

## Configuration

### SearXNG Settings

Key configuration options in `searxng-settings.yml`:

```yaml
# Enable engines suitable for development
engines:
  - name: github
    disabled: false
  - name: stackoverflow  
    disabled: false
  - name: google
    disabled: false
  - name: duckduckgo
    disabled: false

# Development-friendly settings
general:
  debug: false
  instance_name: "CSE-Icon AutoCoder SearXNG"

server:
  port: 8080
  secret_key: "super-secret-key-for-development-only"
```

### Rate Limiting

Rate limiting is configured in `limiter.toml` to be lenient during development:

```toml
[botdetection.ip_limit]
# Allow localhost and private networks
pass_ip = [
    '127.0.0.1',
    '::1', 
    '192.168.0.0/16',
    '172.16.0.0/12',
    '10.0.0.0/8'
]
```

### Application Integration

The application connects to SearXNG via:
- **Base URL**: `http://localhost:8080`
- **Health Check**: `/healthz` endpoint
- **Search API**: `/search` endpoint with JSON format
- **Default Engines**: github, stackoverflow, google, duckduckgo

## Testing

### Test Suites

1. **Integration Tests** (`searxng_integration_tests.rs`)
   - Basic search functionality
   - Engine-specific searches
   - Error handling
   - Connection management

2. **Performance Tests** (`searxng_performance_tests.rs`)
   - Response time validation
   - Concurrent search handling
   - Large result set processing
   - Memory usage monitoring

3. **Health Tests** (`searxng_health_tests.rs`)
   - Service availability detection
   - Health check consistency
   - Recovery scenarios
   - Metrics collection

### Running Tests

**All Tests:**
```bash
./scripts/test-searxng.sh
```

**Specific Test Suite:**
```bash
cd src-tauri
cargo test searxng_integration_tests --lib -- --nocapture
```

**With Options:**
```bash
# Stop services after testing
./scripts/test-searxng.sh --stop-after

# Skip service startup (assume already running)
./scripts/test-searxng.sh --skip-start
```

### Test Requirements

- SearXNG service must be running on localhost:8080
- Tests run serially to avoid conflicts
- Each test includes proper cleanup
- Health checks verify service readiness

## Troubleshooting

### Common Issues

#### 1. Service Won't Start

**Symptoms:**
- Docker containers fail to start
- Port 8080 already in use
- Permission errors

**Solutions:**
```bash
# Check if port is in use
netstat -an | grep 8080  # Linux/macOS
netstat -an | findstr 8080  # Windows

# Kill process using port 8080
sudo lsof -ti:8080 | xargs sudo kill -9  # Linux/macOS

# Check Docker status
docker info
docker-compose ps
```

#### 2. Health Checks Fail

**Symptoms:**
- `/healthz` returns errors
- Application shows "Search Service Offline"
- Tests fail with connection errors

**Diagnostic Steps:**
```bash
# Check service logs
docker-compose logs searxng

# Verify network connectivity
curl http://localhost:8080/healthz

# Check container status
docker-compose ps
docker inspect searxng-dev
```

#### 3. Search Results Empty

**Symptoms:**
- Search completes but returns no results
- Specific engines not working
- Timeout errors

**Solutions:**
1. Check engine configuration in `searxng-settings.yml`
2. Verify internet connectivity from container
3. Check rate limiting in `limiter.toml`
4. Review SearXNG logs for engine-specific errors

#### 4. Performance Issues

**Symptoms:**
- Slow search responses
- Tests timing out
- High CPU/memory usage

**Diagnostic Steps:**
```bash
# Monitor resource usage
docker stats searxng-dev

# Check for DNS issues
docker exec searxng-dev nslookup google.com

# Review rate limiting logs
docker-compose logs searxng | grep -i limit
```

### Debug Mode

Enable debug logging:

1. Edit `searxng-settings.yml`:
```yaml
general:
  debug: true
```

2. Restart services:
```bash
docker-compose restart
```

3. View detailed logs:
```bash
docker-compose logs -f searxng
```

### Container Management

**Reset Everything:**
```bash
cd docker/searxng
docker-compose down -v
docker-compose pull
docker-compose up -d
```

**Check Container Health:**
```bash
docker exec searxng-dev curl -f http://localhost:8080/healthz
```

**Access Container Shell:**
```bash
docker exec -it searxng-dev /bin/bash
```

## Development

### Development Workflow

1. **Start Services**: Use startup scripts
2. **Code Changes**: Modify Rust backend or React frontend
3. **Test Integration**: Run test suites
4. **Health Monitoring**: Check service status
5. **Debug Issues**: Use logs and health checks

### Integration Points

**Rust Backend:**
- `src-tauri/src/searxng_client.rs` - HTTP client
- `src-tauri/src/searxng_commands.rs` - Tauri commands
- `src-tauri/src/tests/` - Test suites

**React Frontend:**
- `src/components/SearchPanel.tsx` - Search interface
- `src/components/SearchHealthIndicator.tsx` - Health status
- `src/hooks/useSearchHealth.ts` - Health monitoring

### Adding New Features

1. **New Search Engines**: Edit `searxng-settings.yml`
2. **Additional Commands**: Add to `searxng_commands.rs`
3. **UI Enhancements**: Modify React components
4. **Test Coverage**: Add tests for new functionality

### Performance Monitoring

Monitor key metrics:
- **Response Time**: < 30 seconds per search
- **Health Check**: < 10 seconds
- **Memory Usage**: Monitor container resources
- **Success Rate**: > 90% for healthy service

### Security Considerations

**Development Only:**
- Default secret key is for development only
- Rate limiting is disabled for localhost
- Debug logging may expose sensitive data
- No HTTPS in development setup

**Production Checklist:**
- Change secret key
- Enable proper rate limiting
- Configure HTTPS
- Disable debug mode
- Set up proper monitoring

## Support

### Getting Help

1. **Check Logs**: Always start with container logs
2. **Test Suite**: Run tests to identify specific issues
3. **Health Status**: Monitor service health indicators
4. **Documentation**: Review SearXNG official docs

### Reporting Issues

When reporting issues, include:
- Operating system and Docker version
- Container logs (`docker-compose logs`)
- Health check status
- Steps to reproduce
- Expected vs actual behavior

### Useful Commands

```bash
# Complete service status
docker-compose ps && curl -s http://localhost:8080/healthz

# Full log tail
docker-compose logs -f --tail=100

# Service metrics
docker stats --no-stream searxng-dev redis

# Network connectivity test
docker exec searxng-dev ping google.com
```