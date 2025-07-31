# SearXNG Troubleshooting Guide

This guide provides solutions for common issues with the SearXNG integration in CSE-Icon AutoCoder.

## Quick Diagnostics

### System Check Script

Run this quick diagnostic to identify common issues:

```bash
#!/bin/bash
echo "=== SearXNG Diagnostic Check ==="

# Check Docker
echo "1. Docker Status:"
if docker info >/dev/null 2>&1; then
    echo "   ✅ Docker is running"
else
    echo "   ❌ Docker is not running or not accessible"
fi

# Check containers
echo "2. Container Status:"
if docker-compose ps -q searxng >/dev/null 2>&1; then
    echo "   ✅ SearXNG container exists"
    status=$(docker inspect --format='{{.State.Status}}' $(docker-compose ps -q searxng))
    echo "   Status: $status"
else
    echo "   ❌ SearXNG container not found"
fi

# Check port availability
echo "3. Port 8080 Status:"
if curl -s http://localhost:8080/healthz >/dev/null; then
    echo "   ✅ Port 8080 accessible and healthy"
else
    echo "   ❌ Port 8080 not accessible or unhealthy"
fi

# Check network connectivity
echo "4. Network Test:"
if docker exec $(docker-compose ps -q searxng) ping -c 1 google.com >/dev/null 2>&1; then
    echo "   ✅ Container has internet connectivity"
else
    echo "   ❌ Container network issues"
fi
```

## Common Issues & Solutions

### 1. "Search Service Offline" Error

**Symptoms:**
- Red indicator in SearchPanel
- Health check fails
- Searches return immediate errors

**Root Causes & Solutions:**

#### A. Docker Not Running
```bash
# Check Docker status
docker info

# Start Docker (varies by OS)
# Windows: Start Docker Desktop
# macOS: Start Docker Desktop  
# Linux: sudo systemctl start docker
```

#### B. SearXNG Container Not Started
```bash
# Check container status
docker-compose ps

# If not running, start it
cd docker/searxng
docker-compose up -d

# Check logs for startup issues
docker-compose logs searxng
```

#### C. Port 8080 Conflicts
```bash
# Find what's using port 8080
netstat -tulpn | grep 8080  # Linux
netstat -an | findstr 8080  # Windows
lsof -i :8080              # macOS

# Kill conflicting process
kill -9 <PID>

# Or change SearXNG port in docker-compose.yml
ports:
  - "8081:8080"  # Use port 8081 instead
```

#### D. Container Health Issues
```bash
# Check container health
docker inspect searxng-dev | grep -A 10 Health

# Check specific health endpoint
docker exec searxng-dev curl -f http://localhost:8080/healthz

# Restart container if unhealthy
docker-compose restart searxng
```

### 2. Search Returns No Results

**Symptoms:**
- Search completes successfully
- Empty results array returned
- No error messages

**Root Causes & Solutions:**

#### A. Search Engine Configuration
```bash
# Check enabled engines in settings
docker exec searxng-dev cat /etc/searxng/settings.yml | grep -A 20 engines

# Verify engines are not disabled
# Edit searxng-settings.yml and set disabled: false for required engines
```

#### B. Network Connectivity Issues
```bash
# Test DNS resolution from container
docker exec searxng-dev nslookup google.com
docker exec searxng-dev nslookup github.com

# Test HTTP connectivity
docker exec searxng-dev curl -s -o /dev/null -w "%{http_code}" https://google.com
```

#### C. Rate Limiting
```bash
# Check rate limiting logs
docker-compose logs searxng | grep -i limit
docker-compose logs searxng | grep -i block

# Review limiter configuration
cat docker/searxng/limiter.toml

# Temporarily disable rate limiting (development only)
# Edit limiter.toml and set more permissive rules
```

#### D. Search Query Issues
```bash
# Test with basic query
curl "http://localhost:8080/search?q=test&format=json"

# Check for special characters
# Some characters might need URL encoding
```

### 3. Slow Search Performance

**Symptoms:**
- Searches take > 30 seconds
- Timeout errors
- Application becomes unresponsive

**Root Causes & Solutions:**

#### A. Container Resource Limits
```bash
# Check container resource usage
docker stats searxng-dev

# If high CPU/memory usage, adjust Docker resources:
# Docker Desktop -> Settings -> Resources -> Advanced
# Increase CPU cores and memory allocation
```

#### B. Engine Response Times
```bash
# Check which engines are slow
docker-compose logs searxng | grep -i timeout
docker-compose logs searxng | grep -i slow

# Disable slow engines temporarily in searxng-settings.yml
engines:
  - name: slow_engine
    disabled: true
```

#### C. Network Latency
```bash
# Test network latency to common search engines
docker exec searxng-dev ping -c 5 google.com
docker exec searxng-dev ping -c 5 api.github.com

# Consider using regional search engines or CDNs
```

#### D. Redis Performance
```bash
# Check Redis status
docker exec searxng-redis redis-cli ping

# Clear Redis cache if needed
docker exec searxng-redis redis-cli FLUSHALL

# Check Redis memory usage
docker exec searxng-redis redis-cli INFO memory
```

### 4. Application Won't Connect to SearXNG

**Symptoms:**
- Rust application cannot invoke SearXNG commands
- Connection timeout errors
- Health checks always fail

**Root Causes & Solutions:**

#### A. Base URL Configuration
```rust
// Check base URL in SearXNGClient
// src-tauri/src/searxng_client.rs
let client = SearXNGClient::new(Some("http://localhost:8080".to_string()));

// Try different URL formats
let client = SearXNGClient::new(Some("http://127.0.0.1:8080".to_string()));
```

#### B. Firewall/Network Issues
```bash
# Check if localhost resolves correctly
ping localhost
ping 127.0.0.1

# Verify port is open
telnet localhost 8080

# Check Windows Firewall (Windows only)
# Allow Docker Desktop through firewall
```

#### C. Docker Network Configuration
```bash
# Check Docker networks
docker network ls

# Inspect SearXNG network
docker network inspect searxng_searxng-network

# Verify container connectivity
docker exec searxng-dev curl http://localhost:8080/healthz
```

### 5. Tests Failing

**Symptoms:**
- Integration tests timeout
- Health tests report service unavailable
- Performance tests exceed limits

**Root Causes & Solutions:**

#### A. Service Not Ready
```bash
# Wait for service to be fully ready before testing
./scripts/test-searxng.sh --skip-start

# Check service startup logs
docker-compose logs searxng | tail -50
```

#### B. Test Environment Issues
```bash
# Ensure tests run serially
export RUST_TEST_THREADS=1

# Increase test timeouts in test files
// Increase timeout duration in tests
const TEST_TIMEOUT: Duration = Duration::from_secs(90);
```

#### C. Resource Constraints
```bash
# Close other applications during testing
# Ensure sufficient Docker resources allocated
# Run tests on a clean Docker environment

# Reset Docker state
docker system prune -f
cd docker/searxng && docker-compose down -v && docker-compose up -d
```

### 6. Memory/Resource Issues

**Symptoms:**
- High memory usage
- Docker running out of disk space
- Container restarts frequently

**Root Causes & Solutions:**

#### A. Memory Leaks
```bash
# Monitor memory usage over time
while true; do
  docker stats --no-stream searxng-dev
  sleep 30
done

# Restart container if memory grows continuously
docker-compose restart searxng
```

#### B. Disk Space
```bash
# Clean up Docker resources
docker system df
docker system prune -a

# Clean up specific volumes
docker volume prune
docker-compose down -v  # Warning: removes data
```

#### C. Log File Growth
```bash
# Check log sizes
docker-compose logs searxng | wc -l

# Rotate logs by restarting
docker-compose restart searxng

# Configure log rotation in docker-compose.yml
logging:
  driver: "json-file"
  options:
    max-size: "10m"
    max-file: "3"
```

## Advanced Debugging

### Enable Debug Mode

1. **Edit Configuration:**
```yaml
# docker/searxng/searxng-settings.yml
general:
  debug: true

# Enable more verbose logging
server:
  log_level: DEBUG
```

2. **Restart Service:**
```bash
docker-compose restart searxng
```

3. **Monitor Debug Logs:**
```bash
docker-compose logs -f searxng | grep -i debug
```

### Network Debugging

```bash
# Check container network configuration
docker exec searxng-dev ip addr show
docker exec searxng-dev ip route show

# Test specific endpoints
docker exec searxng-dev curl -v http://localhost:8080/search?q=test
docker exec searxng-dev curl -v http://localhost:8080/healthz

# Check DNS resolution
docker exec searxng-dev cat /etc/resolv.conf
docker exec searxng-dev nslookup search-engine-domain.com
```

### Application Integration Debugging

```rust
// Add debug logging to Rust client
// src-tauri/src/searxng_client.rs

impl SearXNGClient {
    pub async fn search(&self, /* params */) -> Result<Vec<SearchResult>, Box<dyn Error>> {
        let base_url = self.base_url.lock().await.clone();
        println!("DEBUG: Using base URL: {}", base_url);
        
        let response = self.client.get(&url)
            .query(&params)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await?;
            
        println!("DEBUG: Response status: {}", response.status());
        // ... rest of implementation
    }
}
```

### Performance Profiling

```bash
# Monitor system resources during searches
top -p $(docker inspect --format='{{.State.Pid}}' searxng-dev)

# Profile memory usage
docker exec searxng-dev cat /proc/meminfo

# Monitor network traffic
docker exec searxng-dev netstat -i
```

## Recovery Procedures

### Complete Reset

When all else fails, perform a complete reset:

```bash
# 1. Stop and remove everything
cd docker/searxng
docker-compose down -v

# 2. Clean up Docker resources
docker system prune -a -f

# 3. Remove any cached images
docker rmi searxng/searxng:latest redis:7-alpine

# 4. Restart Docker (varies by OS)
# Windows/macOS: Restart Docker Desktop
# Linux: sudo systemctl restart docker

# 5. Start fresh
docker-compose pull
docker-compose up -d

# 6. Wait for health check
./start-searxng.sh
```

### Service Recovery

For automatic service recovery, consider adding this to your monitoring:

```bash
#!/bin/bash
# Health check and auto-recovery script

check_and_recover() {
    if ! curl -f -s http://localhost:8080/healthz >/dev/null; then
        echo "Service unhealthy, attempting recovery..."
        
        # Try restart first
        docker-compose restart searxng
        sleep 30
        
        # If still unhealthy, full reset
        if ! curl -f -s http://localhost:8080/healthz >/dev/null; then
            echo "Restart failed, performing full reset..."
            docker-compose down
            docker-compose up -d
        fi
    fi
}

# Run every 5 minutes
while true; do
    check_and_recover
    sleep 300
done
```

## Prevention

### Best Practices

1. **Regular Health Monitoring**: Implement the health indicator in UI
2. **Resource Monitoring**: Keep an eye on Docker resource usage
3. **Log Rotation**: Configure proper log management
4. **Backup Configuration**: Keep backup of working configurations
5. **Version Pinning**: Use specific Docker image versions

### Monitoring Setup

```yaml
# Add to docker-compose.yml for better monitoring
services:
  searxng:
    # ... existing config
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/healthz"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    
    # Resource limits
    deploy:
      resources:
        limits:
          memory: 1G
          cpus: '0.5'
        reservations:
          memory: 512M
          cpus: '0.25'
```

## Getting Help

### Support Channels

1. **Application Issues**: Check application logs and health indicators
2. **Docker Issues**: Review Docker Desktop logs and settings
3. **SearXNG Issues**: Check [SearXNG Documentation](https://docs.searxng.org/)
4. **Performance Issues**: Profile using Docker stats and system monitoring

### Information to Gather

When seeking help, collect:

```bash
# System information
uname -a                    # OS info
docker version             # Docker version
docker-compose version     # Compose version

# Service status
docker-compose ps          # Container status
docker-compose logs searxng # Service logs
curl -v http://localhost:8080/healthz # Health check

# Resource usage
docker stats --no-stream   # Resource usage
df -h                      # Disk space
free -h                    # Memory usage (Linux)
```

### Common Solutions Summary

| Issue | Quick Fix |
|-------|-----------|
| Service offline | `docker-compose restart searxng` |
| Port conflict | Change port in docker-compose.yml |
| No results | Check engine configuration |
| Slow performance | Increase Docker resources |
| Tests failing | Ensure service is ready first |
| Memory issues | `docker system prune -f` |
| Network problems | Restart Docker daemon |
| Config issues | Reset to default config |