# SearXNG Docker Setup

This directory contains the Docker configuration for SearXNG integration with CSE-Icon AutoCoder.

## Quick Start

### Windows
```batch
start-searxng.bat
```

### Linux/macOS
```bash
./start-searxng.sh
```

## Files

| File | Purpose |
|------|---------|
| `docker-compose.yml` | Docker services configuration |
| `searxng-settings.yml` | SearXNG engine and behavior settings |
| `limiter.toml` | Rate limiting configuration |
| `start-searxng.sh/.bat` | Startup scripts with health checks |
| `stop-searxng.sh/.bat` | Shutdown scripts |

## Services

- **SearXNG**: Search engine aggregator (port 8080)
- **Redis**: Caching and rate limiting (internal)

## Health Check

After starting, verify the service is running:
- Web interface: http://localhost:8080
- Health endpoint: http://localhost:8080/healthz

## Configuration

### Enabled Search Engines

- GitHub (for code and documentation)
- Stack Overflow (for programming questions)
- Google (general search)
- DuckDuckGo (privacy-focused search)
- Bing (alternative general search)
- Brave (privacy-focused search)
- Wikipedia (knowledge base)
- Reddit (community discussions)
- arXiv (academic papers)

### Development Settings

- **Rate Limiting**: Lenient for localhost/development
- **Debug Mode**: Disabled by default
- **Secret Key**: Development-only key (change for production)
- **Timeout**: 30 seconds for search requests

## Usage

### Manual Docker Commands

```bash
# Start services
docker-compose up -d

# View logs
docker-compose logs -f searxng

# Stop services
docker-compose down

# Reset (removes data)
docker-compose down -v
```

### Integration with Application

The SearXNG service is automatically used by:
- Rust backend commands in `src-tauri/src/searxng_commands.rs`
- React SearchPanel component
- Integration tests

## Troubleshooting

### Common Issues

1. **Port 8080 in use**: Change port mapping in docker-compose.yml
2. **Service won't start**: Check Docker is running
3. **No search results**: Verify internet connectivity
4. **Health check fails**: Wait 30-60 seconds for full startup

### Debug Commands

```bash
# Check container status
docker-compose ps

# View detailed logs
docker-compose logs searxng

# Test health endpoint
curl http://localhost:8080/healthz

# Access container shell
docker exec -it searxng-dev /bin/bash
```

## Development

### Modifying Configuration

1. Edit `searxng-settings.yml` or `limiter.toml`
2. Restart services: `docker-compose restart`
3. Verify changes: Check logs and test searches

### Adding Search Engines

Edit the `engines` section in `searxng-settings.yml`:

```yaml
engines:
  - name: new_engine
    engine: engine_name
    shortcut: ne
    categories: [general]
    disabled: false
```

### Performance Tuning

Monitor resource usage:
```bash
docker stats searxng-dev redis
```

Adjust memory limits in docker-compose.yml if needed.

## Security Notes

⚠️ **Development Configuration**: This setup is optimized for development and includes:
- Default secret keys
- Permissive rate limiting
- No HTTPS/authentication

For production deployment, review and update security settings.

## Documentation

- [Setup Guide](../../docs/searxng-setup.md)
- [Troubleshooting Guide](../../docs/searxng-troubleshooting.md)
- [SearXNG Official Docs](https://docs.searxng.org/)