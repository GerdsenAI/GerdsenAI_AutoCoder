# 🍎 GerdsenAI Socrates - macOS Setup Guide

*Complete setup guide for macOS including Apple Silicon (M1/M2/M3) and Intel Macs*

---

## System Requirements

### Hardware Support
- **Apple Silicon (M1/M2/M3)**: ✅ Full native support with universal binaries
- **Intel-based Macs**: ✅ Full native support  
- **macOS Version**: 10.15 (Catalina) or later
- **Memory**: 8GB RAM minimum (16GB+ recommended for AI model performance)
- **Storage**: 4GB free space (for application + AI models)
- **Internet**: Required for initial setup and AI model downloads

### Architecture Detection
```bash
# Check your Mac's architecture
uname -m
# Apple Silicon: arm64
# Intel: x86_64
```

---

## 🚀 Quick Installation

### Option 1: Pre-built Universal Binary (Recommended)
1. **Download** the latest `.dmg` file from releases
2. **Double-click** the DMG to mount it
3. **Drag** GerdsenAI Socrates to your Applications folder
4. **Launch** from Applications or Spotlight
5. **Allow** system permissions when prompted

### Option 2: Build from Source
```bash
# Clone repository
git clone https://github.com/GerdsenAI/GerdsenAI_AutoCoder.git
cd GerdsenAI_AutoCoder

# Quick setup script
./scripts/setup-macos.sh

# Or manual setup (see below)
```

---

## 📦 Manual Installation

### Step 1: Install Dependencies

#### Using Homebrew (Recommended)
```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install core dependencies
brew install node rust python3 git

# Verify installations
node --version    # Should be 20.19+ or 22.12+
rustc --version   # Should be 1.70+
python3 --version # Should be 3.8+
```

#### Manual Installation
1. **Node.js**: Download from [nodejs.org](https://nodejs.org/) (LTS version)
2. **Rust**: Install via [rustup.rs](https://rustup.rs/)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```
3. **Python**: Pre-installed on macOS or via [python.org](https://python.org)
4. **Xcode Command Line Tools**:
   ```bash
   xcode-select --install
   ```

### Step 2: Install Rust Targets (For Building)
```bash
# For universal binary builds (recommended)
rustup target add aarch64-apple-darwin x86_64-apple-darwin

# For your specific architecture only
# Apple Silicon:
rustup target add aarch64-apple-darwin

# Intel:
rustup target add x86_64-apple-darwin
```

### Step 3: Install AI Services

#### Ollama (Required for AI functionality)
```bash
# Download and install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Or via Homebrew
brew install ollama

# Start Ollama service
ollama serve

# Install a starter model
ollama pull llama3.2:1b  # Lightweight, fast
# or for better quality:
ollama pull llama3.2     # Standard model
```

#### ChromaDB (Required for document management)
```bash
# Install via pip
pip3 install chromadb

# Start ChromaDB server
chroma run --host localhost --port 8000
```

#### SearXNG (Optional - enhances web search)
```bash
# Using Docker (recommended)
docker run -d \
  --name searxng \
  -p 8080:8080 \
  searxng/searxng

# Or follow manual installation: https://docs.searxng.org/admin/installation.html
```

### Step 4: Build GerdsenAI Socrates
```bash
# Clone and enter project directory
git clone https://github.com/GerdsenAI/GerdsenAI_AutoCoder.git
cd GerdsenAI_AutoCoder

# Install Node.js dependencies
npm install

# Build for your architecture
npm run tauri:build:macos           # Universal binary (recommended)
npm run tauri:build:macos:silicon   # Apple Silicon only
npm run tauri:build:macos:intel     # Intel only

# The .app and .dmg will be in src-tauri/target/*/release/bundle/macos/
```

---

## 🔧 Real-World Troubleshooting

### Common macOS Issues

#### "App is damaged and can't be opened" 
**Problem**: macOS Gatekeeper blocking unsigned app.

**Solutions**:
```bash
# Method 1: Remove quarantine attribute
xattr -dr com.apple.quarantine "/Applications/GerdsenAI Socrates.app"

# Method 2: Temporarily disable Gatekeeper (not recommended)
sudo spctl --master-disable
# Re-enable after installation:
sudo spctl --master-enable

# Method 3: Right-click app → Open → Open anyway
```

#### Permission Denied for Microphone/Camera
**Problem**: macOS requires explicit permission for certain features.

**Solutions**:
1. **System Settings** → **Privacy & Security** → **Microphone/Camera**
2. **Add** GerdsenAI Socrates to allowed apps
3. **Restart** the application

#### "Command not found: ollama" after installation
**Problem**: Ollama not in PATH or not properly installed.

**Solutions**:
```bash
# Check if Ollama is installed
which ollama

# If not found, add to PATH
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# Or reinstall Ollama
brew reinstall ollama
```

#### Rust compilation fails on Apple Silicon
**Problem**: Cross-compilation issues or missing targets.

**Solutions**:
```bash
# Ensure you have the right targets
rustup target list --installed

# Add missing targets
rustup target add aarch64-apple-darwin x86_64-apple-darwin

# Clear cache and rebuild
cargo clean
npm run tauri:build:macos
```

#### npm install fails with permission errors
**Problem**: npm global packages need permissions.

**Solutions**:
```bash
# Fix npm permissions
sudo chown -R $(whoami) $(npm config get prefix)/{lib/node_modules,bin,share}

# Or use n to manage Node versions
npm install -g n
sudo n latest
```

#### ChromaDB won't start
**Problem**: Port conflicts or Python environment issues.

**Solutions**:
```bash
# Check if port 8000 is in use
lsof -ti :8000

# Kill conflicting process
kill -9 $(lsof -ti :8000)

# Start ChromaDB on different port
chroma run --host localhost --port 8001

# Update configuration in GerdsenAI Socrates settings
```

### Performance Optimization for macOS

#### Apple Silicon Specific Optimizations
- **Use native builds**: `npm run tauri:build:macos:silicon` for best performance
- **Memory management**: 16GB+ RAM recommended for large AI models
- **Thermal management**: Monitor Activity Monitor during heavy AI usage

#### Intel Mac Optimizations  
- **Rosetta compatibility**: Universal binaries run well under Rosetta 2
- **Memory pressure**: Close unnecessary apps during AI model usage
- **CPU usage**: Consider smaller models like `llama3.2:1b` for better performance

---

## 🎯 Quick Verification Checklist

After installation, verify everything works:

```bash
# Check system architecture
uname -m

# Verify Node.js version
node --version  # Should be 20.19+ or 22.12+

# Verify Rust installation
rustc --version

# Check Ollama service
curl http://localhost:11434/api/tags

# Check ChromaDB service  
curl http://localhost:8000/api/v1/heartbeat

# Verify GerdsenAI Socrates installation
open "/Applications/GerdsenAI Socrates.app"
```

**Automated verification**:
```bash
cd GerdsenAI_AutoCoder
node scripts/verify-setup.js --macos
```

---

## 🛠 Development Setup

### For Contributors and Advanced Users

#### Set up development environment
```bash
# Install development dependencies
npm install
npm run tauri:dev  # Start development mode

# Enable file watching for hot reload
export TAURI_DEV_WATCHER=true
npm run tauri:dev
```

#### Build different variants
```bash
# Debug build (faster compilation)
npm run tauri build -- --debug

# Release build with debug symbols
npm run tauri build -- --debug --bundles app

# Universal binary for distribution
npm run tauri:build:macos
```

#### Code signing preparation (for distribution)
```bash
# Check available signing identities
security find-identity -v -p codesigning

# Build with signing (requires Apple Developer account)
npm run tauri build -- --target universal-apple-darwin --config '{"bundle":{"macOS":{"signingIdentity":"Developer ID Application: Your Name"}}}'
```

---

## 📞 Getting Help

### Architecture-Specific Issues

**Apple Silicon (M1/M2/M3) Users**:
- Use native builds when possible for best performance
- Some dependencies might need Rosetta 2 initially
- Report Apple Silicon-specific issues with `uname -m` output

**Intel Mac Users**:
- All features fully supported
- Universal binaries work excellently  
- Consider memory usage with large AI models

### System Information for Bug Reports
When reporting macOS-specific issues, include:

```bash
# System information
sw_vers
uname -m
sysctl -n machdep.cpu.brand_string

# Architecture and versions
node --version
rustc --version
python3 --version

# Service status
curl -s http://localhost:11434/api/tags | jq '.models | length' 2>/dev/null || echo "Ollama not responding"
curl -s http://localhost:8000/api/v1/heartbeat 2>/dev/null && echo "ChromaDB OK" || echo "ChromaDB not responding"
```

### Community Support
- **Issues**: Report bugs with full system information
- **Discussions**: Ask questions about macOS-specific usage
- **Contributing**: Submit PRs for macOS improvements

---

**Ready to revolutionize your coding workflow on macOS? Start with the Quick Installation above! 🚀**