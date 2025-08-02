# GerdsenAI Socrates - Installation Guide

**Quick and Easy Installation for All Platforms**

---

## 🚀 Quick Start

### One-Line Installation

**Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/GerdsenAI/GerdsenAI_AutoCoder/main/install.sh | bash
```

**Windows (PowerShell as Administrator):**
```powershell
iwr -UseBasicParsing https://raw.githubusercontent.com/GerdsenAI/GerdsenAI_AutoCoder/main/scripts/install-simplified.ps1 | iex
```

### Local Installation

If you've downloaded the repository:

**Linux/macOS:**
```bash
chmod +x install.sh
./install.sh
```

**Windows:**
```batch
# Right-click and "Run as Administrator"
install.bat
```

---

## 📋 Installation Options

### 1. 🚀 Full Installation (Recommended)
- **What it does**: Complete automated setup
- **Installs**: All dependencies, AI models, optional services
- **Time**: 15-30 minutes depending on internet speed
- **Best for**: New users, production use

### 2. 🔧 Custom Installation
- **What it does**: Choose specific components
- **Options**: Skip dependencies, models, or Docker
- **Time**: 5-20 minutes
- **Best for**: Users with existing setups

### 3. 📦 Development Installation
- **What it does**: Full development environment
- **Includes**: Source code, build tools, testing frameworks
- **Time**: 20-45 minutes
- **Best for**: Contributors, developers

### 4. 🔄 Update Installation
- **What it does**: Updates existing installation
- **Preserves**: Configuration, documents, models
- **Time**: 5-15 minutes
- **Best for**: Existing users upgrading

---

## 🖥️ Platform-Specific Instructions

### Windows

#### Prerequisites
- Windows 10/11 (64-bit)
- Administrator privileges
- PowerShell 5.1+ (usually pre-installed)

#### Installation Steps
1. **Download**: Get the installer from GitHub releases or clone repository
2. **Run as Administrator**: Right-click `install.bat` → "Run as administrator"
3. **Follow Prompts**: The installer will guide you through the process
4. **Launch**: Find "GerdsenAI Socrates" in Start Menu

#### What Gets Installed
- **Chocolatey**: Package manager for Windows
- **Node.js 20**: JavaScript runtime
- **Rust**: Systems programming language
- **Ollama**: AI model runtime
- **Docker Desktop**: Container platform (optional)
- **Python**: For ChromaDB support
- **GerdsenAI Socrates**: The main application

### macOS

#### Prerequisites
- macOS 10.15+ (Catalina or later)
- Administrator privileges
- Xcode Command Line Tools (installer will prompt if needed)

#### Installation Steps
1. **Download**: Clone repository or download installer
2. **Run Installer**: Execute `./install.sh` in Terminal
3. **Grant Permissions**: Allow installation of dependencies
4. **Launch**: Find in Applications folder or use Spotlight

#### What Gets Installed
- **Homebrew**: Package manager for macOS
- **Node.js 20**: JavaScript runtime
- **Rust**: Systems programming language
- **Ollama**: AI model runtime
- **Docker Desktop**: Container platform (optional, manual install)
- **Python**: For ChromaDB support
- **GerdsenAI Socrates**: Universal binary app

#### Apple Silicon vs Intel
- **Universal Binary**: Single installation works on both architectures
- **Optimized Performance**: Native ARM64 support for M1/M2/M3 Macs
- **Compatibility**: Full support for Intel-based Macs

### Linux

#### Prerequisites
- Ubuntu 20.04+, Fedora 32+, or equivalent
- `sudo` privileges
- `curl` and `bash` installed

#### Installation Steps
1. **Update System**: `sudo apt update` (Ubuntu/Debian)
2. **Run Installer**: Execute `./install.sh`
3. **Install Dependencies**: Installer handles system packages
4. **Launch**: Run `gerdsenai-socrates` or find in applications menu

#### What Gets Installed
- **System Dependencies**: WebKit, SSL, GTK development libraries
- **Node.js 20**: JavaScript runtime
- **Rust**: Systems programming language
- **Ollama**: AI model runtime
- **Docker**: Container platform (optional)
- **Python**: For ChromaDB support
- **GerdsenAI Socrates**: AppImage or system package

#### Supported Distributions
- **Ubuntu/Debian**: Native APT package support
- **Fedora/RHEL**: DNF package support
- **Arch Linux**: Pacman package support
- **Other**: Generic AppImage installation

---

## 🔧 Manual Installation

If automatic installation fails, you can install components manually:

### Step 1: Install Prerequisites

**Windows:**
```powershell
# Install Chocolatey
Set-ExecutionPolicy Bypass -Scope Process -Force; iex ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1'))

# Install packages
choco install nodejs rust ollama docker-desktop python3 -y
```

**macOS:**
```bash
# Install Homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install packages
brew install node rust ollama docker python@3.11
```

**Linux (Ubuntu):**
```bash
# Update system
sudo apt update

# Install system dependencies
sudo apt install -y curl build-essential libwebkit2gtk-4.0-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Install Docker
curl -fsSL https://get.docker.com | sudo sh
sudo usermod -aG docker $USER

# Install Python packages
pip3 install --user chromadb
```

### Step 2: Install Application

```bash
# Clone repository
git clone https://github.com/GerdsenAI/GerdsenAI_AutoCoder.git
cd GerdsenAI_AutoCoder

# Install npm dependencies
npm install

# Build application
npm run build
npm run tauri build
```

### Step 3: Install AI Models

```bash
# Start Ollama service
ollama serve &

# Download recommended models
ollama pull qwen2.5-coder:7b
ollama pull codellama:7b
```

### Step 4: Setup Optional Services

```bash
# Start SearXNG (if Docker is installed)
cd docker/searxng
./start-searxng.sh
```

---

## 🔍 Verification

After installation, verify everything is working:

### Service Status Check
```bash
# Check Ollama
curl http://localhost:11434/api/version

# Check SearXNG (optional)
curl "http://localhost:8080/search?q=test&format=json"

# Check ChromaDB
python3 -c "import chromadb; print('ChromaDB OK')"
```

### Application Launch
- **Windows**: Start Menu → GerdsenAI Socrates
- **macOS**: Applications folder or Spotlight search
- **Linux**: Application menu or run `gerdsenai-socrates`

### Feature Testing
1. **Chat Interface**: Send a test message to verify AI responses
2. **Model Selection**: Check available models in dropdown
3. **RAG System**: Upload a test document
4. **Web Search**: Try a search query (if SearXNG is running)
5. **Health Indicators**: Verify all services show green status

---

## 🚨 Troubleshooting

### Common Issues

#### Installation Fails
- **Check Permissions**: Ensure running as Administrator/sudo
- **Network Issues**: Verify internet connection and proxy settings
- **Disk Space**: Ensure at least 5GB free space
- **Antivirus**: Temporarily disable antivirus during installation

#### Services Not Starting
- **Ollama**: Check if port 11434 is available
- **SearXNG**: Verify Docker is installed and running
- **Firewall**: Ensure required ports are not blocked

#### Application Won't Launch
- **Dependencies**: Verify all prerequisites are installed
- **Permissions**: Check file permissions and ownership
- **Logs**: Check application logs for specific errors

### Getting Help

1. **Check Documentation**: Review TROUBLESHOOTING_GUIDE.md
2. **Log Files**: Check installation logs for specific errors
3. **GitHub Issues**: Report bugs and get community help
4. **Community Support**: Join Discord/discussions for real-time help

---

## 📊 System Requirements

### Minimum Requirements
- **OS**: Windows 10, macOS 10.15, or Linux (Ubuntu 20.04+)
- **CPU**: 4-core processor
- **RAM**: 8GB (16GB recommended)
- **Storage**: 5GB free space
- **Network**: Internet connection for downloads

### Recommended Configuration
- **CPU**: 8-core processor or better
- **RAM**: 16GB+ for large codebases
- **Storage**: SSD with 10GB+ free space
- **Network**: Broadband for model downloads

### External Dependencies
- **Ollama**: Required for AI functionality
- **Docker**: Optional for web search
- **Python**: Optional for enhanced RAG features

---

## 🔄 Updates and Maintenance

### Automatic Updates
- Application includes update notifications
- Models can be updated through Ollama
- Dependencies managed by system package managers

### Manual Updates
```bash
# Update application
git pull origin main
npm install
npm run build
npm run tauri build

# Update models
ollama pull qwen2.5-coder:latest

# Update system packages
# Windows: choco upgrade all
# macOS: brew upgrade
# Linux: sudo apt upgrade
```

### Backup and Restore
- **Configuration**: Backed up automatically during updates
- **Documents**: RAG documents preserved across updates
- **Models**: Ollama models persist across updates

---

## 🤝 Contributing

### Development Setup
```bash
# Clone repository
git clone https://github.com/GerdsenAI/GerdsenAI_AutoCoder.git
cd GerdsenAI_AutoCoder

# Install development dependencies
npm install
cd src-tauri && cargo build

# Run in development mode
npm run tauri:dev
```

### Testing Installer
```bash
# Test on clean system or VM
# Run installer and verify all components work
# Report issues and contribute fixes
```

---

## 📄 License and Support

- **License**: See LICENSE file for terms
- **Support**: Community support via GitHub Issues
- **Enterprise**: Contact for enterprise support options
- **Contributing**: See CONTRIBUTING.md for guidelines

---

**Happy coding with GerdsenAI Socrates! 🚀**

*For the latest documentation and updates, visit our [GitHub repository](https://github.com/GerdsenAI/GerdsenAI_AutoCoder)*