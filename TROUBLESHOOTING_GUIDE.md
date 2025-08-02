# GerdsenAI Socrates - Troubleshooting Guide

**Quick Reference for Common Issues and Solutions**

---

## Quick Diagnostic Checklist

Before diving into specific issues, run through this quick checklist:

- [ ] Is Ollama installed and running? (`ollama --version`)
- [ ] Are required ports available? (11434 for Ollama, 8080 for SearXNG)
- [ ] Is the application running with proper permissions?
- [ ] Are all dependencies installed? (Node.js, Rust, etc.)
- [ ] Is there sufficient disk space and memory?
- [ ] Are antivirus/firewall blocking the application?

---

## Installation Issues

### Application Won't Install

#### Windows MSI Installation Fails
**Symptoms**: Installation stops with error message, installer crashes

**Solutions**:
1. **Run as Administrator**:
   ```batch
   # Right-click installer → "Run as administrator"
   ```

2. **Check Windows Installer Service**:
   ```batch
   # Open Services → Windows Installer → Ensure it's running
   sc query msiserver
   ```

3. **Clear Windows Installer Cache**:
   ```batch
   # Run as admin
   net stop msiserver
   rd /s "%WINDIR%\Installer\$PatchCache$"
   net start msiserver
   ```

4. **Use Alternative Installation**:
   ```batch
   # Use manual installation method
   INSTALL_DEPENDENCIES.BAT
   ```

#### macOS DMG Installation Problems
**Symptoms**: "App is damaged", Gatekeeper blocks installation

**Solutions**:
1. **Bypass Gatekeeper Temporarily**:
   ```bash
   # Allow app from anywhere
   sudo spctl --master-disable
   # After installation, re-enable
   sudo spctl --master-enable
   ```

2. **Clear Quarantine Attribute**:
   ```bash
   # Remove quarantine flag
   xattr -d com.apple.quarantine /Applications/GerdsenAI\ Socrates.app
   ```

3. **Verify Download Integrity**:
   ```bash
   # Check file integrity
   shasum -a 256 GerdsenAI-Socrates-*.dmg
   ```

#### Linux AppImage Issues
**Symptoms**: Permission denied, FUSE errors, won't execute

**Solutions**:
1. **Make Executable**:
   ```bash
   chmod +x GerdsenAI-Socrates-*.AppImage
   ```

2. **Install FUSE (if missing)**:
   ```bash
   # Ubuntu/Debian
   sudo apt install fuse libfuse2
   
   # Fedora
   sudo dnf install fuse fuse-libs
   ```

3. **Extract and Run Manually**:
   ```bash
   # Extract AppImage contents
   ./GerdsenAI-Socrates-*.AppImage --appimage-extract
   # Run directly
   ./squashfs-root/AppRun
   ```

---

## Service Connection Issues

### Ollama Connection Problems

#### "Failed to connect to Ollama" Error
**Symptoms**: Red Ollama indicator, chat doesn't work

**Diagnosis**:
```bash
# Check if Ollama is running
ps aux | grep ollama

# Test Ollama API
curl http://localhost:11434/api/version

# Check port availability
netstat -an | grep 11434
```

**Solutions**:
1. **Start Ollama Service**:
   ```bash
   # Start Ollama
   ollama serve
   
   # Or as background service
   nohup ollama serve > /dev/null 2>&1 &
   ```

2. **Check Firewall Settings**:
   ```bash
   # Windows
   netsh firewall set portopening TCP 11434 "Ollama" ENABLE
   
   # Linux (UFW)
   sudo ufw allow 11434
   
   # macOS
   sudo pfctl -f /etc/pf.conf
   ```

3. **Restart Ollama Service**:
   ```bash
   # Kill existing processes
   pkill ollama
   
   # Restart service
   ollama serve
   ```

4. **Check Ollama Configuration**:
   ```bash
   # Verify Ollama environment
   ollama list
   
   # Test model availability
   ollama run qwen2.5-coder "Hello"
   ```

#### Models Not Loading or Available
**Symptoms**: Empty model dropdown, "No models available"

**Solutions**:
1. **Download Models**:
   ```bash
   # Download recommended models
   ollama pull qwen2.5-coder
   ollama pull codellama
   ollama pull deepseek-coder
   ```

2. **Verify Model Installation**:
   ```bash
   # List installed models
   ollama list
   
   # Test model
   ollama run qwen2.5-coder "test"
   ```

3. **Clear Ollama Cache**:
   ```bash
   # Clear model cache
   rm -rf ~/.ollama/models/*
   
   # Re-download models
   ollama pull qwen2.5-coder
   ```

### SearXNG Search Issues

#### "Search service unavailable" Error
**Symptoms**: Red search indicator, web search doesn't work

**Diagnosis**:
```bash
# Check Docker is running
docker ps

# Test SearXNG directly
curl "http://localhost:8080/search?q=test&format=json"

# Check SearXNG logs
docker logs searxng-searxng-1
```

**Solutions**:
1. **Start SearXNG Service**:
   ```bash
   # Navigate to docker directory
   cd docker/searxng/
   
   # Start services
   ./start-searxng.sh
   
   # Or use Docker Compose directly
   docker-compose up -d
   ```

2. **Restart Docker Services**:
   ```bash
   # Stop services
   ./stop-searxng.sh
   
   # Start services
   ./start-searxng.sh
   ```

3. **Check Docker Installation**:
   ```bash
   # Verify Docker is installed and running
   docker --version
   docker info
   
   # Start Docker service if needed
   sudo systemctl start docker  # Linux
   # Or start Docker Desktop manually
   ```

4. **Alternative Search Configuration**:
   - Disable SearXNG in settings
   - Use direct web search APIs (if available)
   - Configure alternative search engines

### ChromaDB Connection Issues

#### "RAG database unavailable" Error
**Symptoms**: RAG features don't work, document upload fails

**Diagnosis**:
```bash
# Check ChromaDB installation
pip list | grep chroma
# or
python -c "import chromadb; print('ChromaDB OK')"

# Check database files
ls -la ~/.local/share/GerdsenAI/Socrates/chroma/
```

**Solutions**:
1. **Install ChromaDB**:
   ```bash
   # Install ChromaDB
   pip install chromadb
   
   # Or using conda
   conda install -c conda-forge chromadb
   ```

2. **Reset Database**:
   ```bash
   # Backup current data
   cp -r ~/.local/share/GerdsenAI/Socrates/chroma/ ~/chroma_backup/
   
   # Clear database
   rm -rf ~/.local/share/GerdsenAI/Socrates/chroma/
   
   # Restart application to recreate
   ```

3. **Check File Permissions**:
   ```bash
   # Fix permissions
   chmod -R 755 ~/.local/share/GerdsenAI/Socrates/
   chown -R $USER ~/.local/share/GerdsenAI/Socrates/
   ```

---

## Performance Issues

### High Memory Usage

#### Application Using Too Much RAM
**Symptoms**: System slowdown, high memory usage in task manager

**Diagnosis**:
```bash
# Check memory usage
top -p $(pgrep "gerdsenai")  # Linux/macOS
# Or use Activity Monitor/Task Manager

# Check application logs
tail -f ~/.local/share/GerdsenAI/Socrates/logs/application.log
```

**Solutions**:
1. **Optimize Document Collections**:
   - Remove unnecessary documents
   - Use smaller document chunks
   - Limit collection size

2. **Adjust Memory Settings**:
   - Settings → Performance → Memory Limit
   - Reduce concurrent operations
   - Clear cache regularly

3. **Model Optimization**:
   - Use smaller AI models
   - Limit context window size
   - Reduce model concurrency

4. **System Optimization**:
   ```bash
   # Clear system cache (Linux)
   sudo sh -c 'echo 3 > /proc/sys/vm/drop_caches'
   
   # Restart application
   pkill gerdsenai-socrates
   ```

### Slow Response Times

#### AI Responses Taking Too Long
**Symptoms**: Long delays in chat responses, timeouts

**Solutions**:
1. **Model Optimization**:
   - Switch to faster models (e.g., qwen2.5-coder instead of larger models)
   - Reduce context size
   - Use CPU vs GPU settings appropriately

2. **Hardware Optimization**:
   - Close unnecessary applications
   - Ensure adequate cooling (CPU throttling)
   - Add more RAM if consistently hitting limits

3. **Network Optimization**:
   - Check network latency for external services
   - Use local-only mode if possible
   - Optimize proxy settings

4. **Cache Optimization**:
   ```bash
   # Clear response cache
   rm -rf ~/.local/share/GerdsenAI/Socrates/cache/
   
   # Restart application
   ```

### Application Freezing or Crashing

#### Frequent Crashes or Hangs
**Symptoms**: Application becomes unresponsive, crashes unexpectedly

**Diagnosis**:
1. **Check Crash Logs**:
   ```bash
   # Application logs
   cat ~/.local/share/GerdsenAI/Socrates/logs/crash.log
   
   # System logs (Linux)
   journalctl -u gerdsenai-socrates
   
   # Windows Event Viewer
   # macOS Console app
   ```

2. **Resource Monitoring**:
   ```bash
   # Monitor resource usage during crash
   top -p $(pgrep gerdsenai)
   iostat 1 10
   ```

**Solutions**:
1. **Update Graphics Drivers**:
   - Update to latest graphics drivers
   - Try different WebView2 versions (Windows)
   - Disable hardware acceleration if needed

2. **System Optimization**:
   ```bash
   # Increase system limits (Linux)
   ulimit -n 4096
   ulimit -u 2048
   
   # Clear temporary files
   rm -rf /tmp/gerdsenai-*
   ```

3. **Application Reset**:
   ```bash
   # Backup settings
   cp -r ~/.config/GerdsenAI/ ~/gerdsenai_backup/
   
   # Reset to defaults
   rm -rf ~/.config/GerdsenAI/
   
   # Restart application
   ```

---

## UI and Display Issues

### Interface Problems

#### Text Not Displaying Correctly
**Symptoms**: Garbled text, missing characters, incorrect rendering

**Solutions**:
1. **Font Issues**:
   ```bash
   # Update font cache (Linux)
   fc-cache -fv
   
   # Install additional fonts
   sudo apt install fonts-liberation fonts-dejavu
   ```

2. **Display Scaling**:
   - Adjust system display scaling
   - Set application-specific DPI settings
   - Try different zoom levels in settings

3. **Theme Issues**:
   - Switch between light/dark themes
   - Reset theme to default
   - Check for theme corruption

#### Window Sizing or Layout Problems
**Symptoms**: Window too small/large, overlapping elements

**Solutions**:
1. **Reset Window State**:
   ```bash
   # Delete window state file
   rm ~/.config/GerdsenAI/window-state.json
   ```

2. **Display Configuration**:
   - Check multi-monitor setup
   - Verify resolution settings
   - Test on single monitor

3. **Manual Reset**:
   - Delete application preferences
   - Restart with default settings
   - Reconfigure manually

### Dark/Light Theme Issues

#### Theme Not Applying Correctly
**Symptoms**: Mixed theme elements, incorrect colors

**Solutions**:
1. **Theme Reset**:
   - Settings → Appearance → Reset Theme
   - Clear theme cache
   - Restart application

2. **System Theme Sync**:
   - Verify system theme detection
   - Manually set theme preference
   - Check for theme conflicts

---

## File and Data Issues

### Document Upload Problems

#### RAG Documents Won't Upload
**Symptoms**: Upload fails, files not appearing in collections

**Solutions**:
1. **File Format Check**:
   - Verify supported file formats (PDF, TXT, MD, etc.)
   - Check file size limits
   - Ensure files aren't corrupted

2. **Permissions Check**:
   ```bash
   # Check file permissions
   ls -la /path/to/document
   
   # Fix permissions if needed
   chmod 644 /path/to/document
   ```

3. **Storage Space**:
   ```bash
   # Check available space
   df -h ~/.local/share/GerdsenAI/
   
   # Clear old documents if needed
   ```

#### Search Not Finding Documents
**Symptoms**: Documents uploaded but search returns no results

**Solutions**:
1. **Reindex Documents**:
   - Settings → RAG → Reindex Collections
   - Wait for indexing to complete
   - Test search again

2. **Search Query Optimization**:
   - Use different search terms
   - Try broader queries
   - Check for typos

3. **Database Integrity**:
   ```bash
   # Check database status
   sqlite3 ~/.local/share/GerdsenAI/chroma/chroma.sqlite3 ".schema"
   ```

### Configuration Issues

#### Settings Not Saving
**Symptoms**: Changes revert after restart, preferences lost

**Solutions**:
1. **File Permissions**:
   ```bash
   # Check config directory permissions
   ls -la ~/.config/GerdsenAI/
   
   # Fix permissions
   chmod -R 755 ~/.config/GerdsenAI/
   chown -R $USER ~/.config/GerdsenAI/
   ```

2. **Disk Space**:
   - Ensure sufficient disk space
   - Check for read-only file systems
   - Verify write permissions

3. **Configuration Reset**:
   ```bash
   # Backup current config
   cp ~/.config/GerdsenAI/settings.json ~/settings_backup.json
   
   # Delete corrupted config
   rm ~/.config/GerdsenAI/settings.json
   
   # Restart and reconfigure
   ```

---

## Network and Connectivity Issues

### Proxy and Firewall Problems

#### Application Can't Connect Through Corporate Proxy
**Symptoms**: External services fail, update checks fail

**Solutions**:
1. **Proxy Configuration**:
   ```bash
   # Set proxy environment variables
   export HTTP_PROXY=http://proxy.company.com:8080
   export HTTPS_PROXY=http://proxy.company.com:8080
   export NO_PROXY=localhost,127.0.0.1
   ```

2. **Application Settings**:
   - Settings → Network → Proxy Configuration
   - Enter proxy server details
   - Test connection

3. **Certificate Issues**:
   ```bash
   # Add corporate certificates (Linux)
   sudo cp corporate-cert.crt /usr/local/share/ca-certificates/
   sudo update-ca-certificates
   ```

#### Firewall Blocking Connections
**Symptoms**: Services can't connect, external features fail

**Solutions**:
1. **Windows Firewall**:
   ```batch
   # Add firewall rules
   netsh advfirewall firewall add rule name="GerdsenAI Socrates" dir=in action=allow program="C:\Program Files\GerdsenAI\Socrates\gerdsenai-socrates.exe"
   ```

2. **Linux Firewall (UFW)**:
   ```bash
   # Allow application ports
   sudo ufw allow 11434
   sudo ufw allow 8080
   ```

3. **Corporate Firewall**:
   - Contact IT to whitelist application
   - Use local-only mode if available
   - Configure alternative ports

---

## Advanced Troubleshooting

### Log Analysis

#### Understanding Log Files
Application logs provide detailed information for troubleshooting:

**Log Locations**:
- **Windows**: `%APPDATA%\GerdsenAI\Socrates\logs\`
- **macOS**: `~/Library/Application Support/GerdsenAI/Socrates/logs/`
- **Linux**: `~/.local/share/GerdsenAI/Socrates/logs/`

**Key Log Files**:
1. **application.log**: Main application events
2. **ollama.log**: AI model communication
3. **searxng.log**: Web search functionality
4. **chromadb.log**: Document indexing and RAG
5. **error.log**: Error messages and stack traces

**Log Analysis Commands**:
```bash
# View recent errors
tail -n 100 ~/.local/share/GerdsenAI/Socrates/logs/error.log

# Search for specific errors
grep -i "connection" ~/.local/share/GerdsenAI/Socrates/logs/*.log

# Monitor logs in real-time
tail -f ~/.local/share/GerdsenAI/Socrates/logs/application.log
```

### System Information Collection

#### Gathering Debug Information
When reporting issues, collect this information:

```bash
# System information
uname -a
lsb_release -a  # Linux
sw_vers       # macOS
systeminfo    # Windows

# Application version
gerdsenai-socrates --version

# Service status
ollama version
docker --version
python --version

# Resource usage
free -h
df -h
ps aux | grep -E "(ollama|gerdsenai|docker)"

# Network connectivity
netstat -an | grep -E "(11434|8080)"
curl -I http://localhost:11434/api/version
```

### Database Recovery

#### ChromaDB Database Corruption
If the document database becomes corrupted:

1. **Backup Current State**:
   ```bash
   cp -r ~/.local/share/GerdsenAI/Socrates/chroma/ ~/chroma_backup_$(date +%Y%m%d)/
   ```

2. **Validate Database**:
   ```bash
   sqlite3 ~/.local/share/GerdsenAI/chroma/chroma.sqlite3 "PRAGMA integrity_check;"
   ```

3. **Repair or Recreate**:
   ```bash
   # Try repair
   sqlite3 ~/.local/share/GerdsenAI/chroma/chroma.sqlite3 "VACUUM;"
   
   # Or recreate if repair fails
   rm -rf ~/.local/share/GerdsenAI/Socrates/chroma/
   # Restart application to recreate
   ```

4. **Restore Documents**:
   - Re-upload important documents
   - Recreate collections
   - Verify search functionality

### Performance Profiling

#### Identifying Performance Bottlenecks

1. **Resource Monitoring**:
   ```bash
   # Continuous monitoring
   while true; do
     echo "$(date): CPU: $(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)%, MEM: $(free | grep Mem | awk '{printf("%.1f%%", $3/$2 * 100.0)}')"
     sleep 5
   done
   ```

2. **Application Profiling**:
   - Enable debug mode in settings
   - Monitor response times in logs
   - Identify slow operations

3. **Optimization Strategies**:
   - Reduce model size
   - Limit concurrent operations
   - Optimize document collection size
   - Clear caches regularly

---

## Emergency Recovery Procedures

### Complete Application Reset

If the application is completely unusable:

1. **Stop All Services**:
   ```bash
   # Kill application processes
   pkill gerdsenai-socrates
   pkill ollama
   docker stop $(docker ps -q)
   ```

2. **Backup Critical Data**:
   ```bash
   # Create backup directory
   mkdir ~/gerdsenai_emergency_backup_$(date +%Y%m%d)
   
   # Backup conversations
   cp -r ~/.local/share/GerdsenAI/Socrates/conversations/ ~/gerdsenai_emergency_backup_$(date +%Y%m%d)/
   
   # Backup documents
   cp -r ~/.local/share/GerdsenAI/Socrates/documents/ ~/gerdsenai_emergency_backup_$(date +%Y%m%d)/
   
   # Backup settings
   cp ~/.config/GerdsenAI/settings.json ~/gerdsenai_emergency_backup_$(date +%Y%m%d)/
   ```

3. **Complete Reset**:
   ```bash
   # Remove all application data
   rm -rf ~/.local/share/GerdsenAI/
   rm -rf ~/.config/GerdsenAI/
   rm -rf ~/.cache/GerdsenAI/
   ```

4. **Reinstall Application**:
   - Download fresh installer
   - Install with default settings
   - Restore backed-up data selectively

### Service Recovery

If external services are broken:

1. **Ollama Recovery**:
   ```bash
   # Completely reinstall Ollama
   ollama stop
   # Uninstall Ollama
   # Download and reinstall from ollama.ai
   ollama serve
   ollama pull qwen2.5-coder
   ```

2. **Docker/SearXNG Recovery**:
   ```bash
   # Stop and remove containers
   docker stop $(docker ps -q)
   docker rm $(docker ps -aq)
   
   # Remove images
   docker rmi $(docker images -q)
   
   # Restart SearXNG
   cd docker/searxng/
   ./start-searxng.sh
   ```

---

## Getting Additional Help

### Self-Diagnosis Tools

Run the built-in diagnostic tool:
```bash
# If available
gerdsenai-socrates --diagnose

# Or manual checks
curl http://localhost:11434/api/version
curl http://localhost:8080/search?q=test
python -c "import chromadb; print('ChromaDB OK')"
```

### Reporting Issues

When reporting issues, include:
1. **System Information**: OS, version, architecture
2. **Application Version**: Exact version number
3. **Error Messages**: Complete error text
4. **Steps to Reproduce**: Detailed reproduction steps
5. **Log Files**: Relevant log excerpts
6. **Screenshots**: Visual issues require screenshots

### Emergency Contacts

- **GitHub Issues**: Report bugs and get community help
- **Documentation**: Check latest troubleshooting updates
- **Community Forums**: Ask questions and share solutions
- **Professional Support**: Enterprise customers contact support team

---

## Prevention and Maintenance

### Regular Maintenance Tasks

1. **Weekly**:
   - Clear application cache
   - Update Ollama models
   - Check disk space

2. **Monthly**:
   - Update application to latest version
   - Optimize document collections
   - Review and clean conversation history

3. **Quarterly**:
   - Full backup of settings and data
   - Performance optimization review
   - Security updates and patches

### Monitoring and Alerts

Set up monitoring for:
- **Service Health**: Automated checks for Ollama, SearXNG
- **Resource Usage**: Memory and disk space alerts
- **Error Rates**: Unusual error frequency
- **Performance**: Response time degradation

This comprehensive troubleshooting guide should help resolve most common issues. For complex problems or if issues persist, don't hesitate to seek help from the community or professional support team.

---

**GerdsenAI Socrates Troubleshooting Guide v1.0**
*Last Updated: January 2025*