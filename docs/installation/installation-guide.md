# GerdsenAI Socrates - Installation Guide

**Comprehensive Installation Instructions for All Platforms**

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

## 📋 System Requirements

### Minimum Requirements
- **Operating System**: 
  - Windows 10/11 (64-bit)
  - macOS 10.15+ (Catalina or later)
  - Linux (Ubuntu 20.04+, Debian 11+, Fedora 35+)
- **Hardware**:
  - 4-core CPU
  - 8GB RAM (16GB recommended)
  - 5GB free disk space
  - x86_64 or ARM64 processor
- **Software Dependencies**:
  - Ollama installed and running
  - SearXNG instance (optional)

### Recommended Configuration
- **CPU**: 8-core processor or better
- **RAM**: 16GB+ for large codebases
- **Storage**: SSD with 10GB+ free space
- **Network**: Broadband for model downloads

---

## 🖥️ Platform-Specific Installation

### Windows

#### Prerequisites
- Windows 10/11 (64-bit)
- Administrator privileges
- PowerShell 5.1+ (usually pre-installed)

#### Installation Options

**Option 1: Using Installer (Recommended)**
1. Download the `GerdsenAI_Socrates_Setup.exe` installer from GitHub releases
2. Right-click on the installer and select "Run as administrator"
3. Follow the on-screen instructions to complete the installation
4. Launch GerdsenAI Socrates from the Start menu or desktop shortcut

**Option 2: Using Setup Script**
1. Extract the GerdsenAI Socrates zip file to a location of your choice
2. Right-click on `WINDOWS_SETUP.BAT` and select "Run as administrator"
3. The script will automatically install dependencies and configure the application
4. Launch from the Start menu or desktop shortcut

**Option 3: Manual Installation**
1. Extract the zip file to your chosen directory
2. Open Command Prompt as administrator
3. Navigate to the extracted directory:
   ```batch
   cd path\to\GerdsenAI_Socrates
   ```
4. Run installation commands:
   ```batch
   npm install
   npm run tauri build
   ```
5. Find the installer in `target\release\bundle\msi\` directory
6. Run the installer and follow the on-screen instructions

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

#### Installation Options

**Option 1: Using DMG (Recommended)**
1. Download the `GerdsenAI_Socrates_x.x.x_universal.dmg` disk image
2. Open the disk image and drag GerdsenAI Socrates to the Applications folder
3. Right-click the app and select "Open" (required for first launch due to security)
4. Follow any security prompts to allow the application to run

**Option 2: Using Install Script**
1. Download or clone the repository
2. Open Terminal and navigate to the directory
3. Run the installer:
   ```bash
   chmod +x install.sh
   ./install.sh
   ```
4. Grant permissions as prompted
5. Launch from Applications folder or use Spotlight

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
- **Optimized Performance**: Native ARM64 support for M1/M2/M3/M4 Macs
- **Compatibility**: Full support for Intel-based Macs

### Linux

#### Prerequisites
- Ubuntu 20.04+, Fedora 32+, or equivalent
- `sudo` privileges
- `curl` and `bash` installed

#### Installation Options

**Option 1: Using AppImage**
```bash
# Make the AppImage executable
chmod +x gerdsenai-socrates_x.x.x_amd64.AppImage

# Run the AppImage
./gerdsenai-socrates_x.x.x_amd64.AppImage
```

**Option 2: Using DEB Package (Debian/Ubuntu)**
```bash
# Install the DEB package
sudo apt install ./gerdsenai-socrates_x.x.x_amd64.deb

# Launch from terminal
gerdsenai-socrates

# Or find in application menu
```

**Option 3: Using Install Script**
1. Update system: `sudo apt update` (Ubuntu/Debian)
2. Run the installer:
   ```bash
   chmod +x install.sh
   ./install.sh
   ```
3. Install dependencies as prompted
4. Launch: Run `gerdsenai-socrates` or find in applications menu

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

## 📦 IDE Extension Installation

### VS Code / VSCodium

1. Open VS Code or VSCodium
2. Go to Extensions (Ctrl+Shift+X or Cmd+Shift+X on macOS)
3. Click the "..." menu in the Extensions panel
4. Select "Install from VSIX..."
5. Navigate to and select the `auto-coder-companion.vsix` file
6. Restart the editor when prompted

### Visual Studio 2022+

1. Open Visual Studio
2. Go to Extensions → Manage Extensions
3. Click on "Install from VSIX..."
4. Navigate to and select the `AutoCoderCompanion.vsix` file
5. Restart Visual Studio when prompted

---

## ⚙️ Post-Installation Configuration

### First Launch

When you first launch GerdsenAI Socrates, it will:
- Detect your installed IDEs
- Ask for permission to integrate with them
- Download the default Ollama model (if not already present)

### Configuring Ollama Connection

GerdsenAI Socrates connects to Ollama by default at `http://localhost:11434`. To use a different Ollama instance:

1. Open GerdsenAI Socrates
2. Click on the model selector dropdown
3. Click "Custom Ollama URL"
4. Enter the URL of your Ollama instance
5. Click "Connect"

### Configuring SearXNG Connection

To configure a custom SearXNG instance:

1. Open GerdsenAI Socrates
2. Go to Settings (gear icon)
3. Navigate to the "Search" tab
4. Enter your SearXNG instance URL
5. Click "Save"

### IDE Integration

- **For VS Code**: The application will automatically dock to the right side
- **For Visual Studio**: The application will automatically dock to the right side
- You can undock/redock using the dock/undock button in the top-right corner

### Model Selection

- Click on the model dropdown to select your preferred Ollama model
- The application will download the model if it's not already available locally

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

### Common Installation Issues

#### Installation Fails
- **Check Permissions**: Ensure running as Administrator/sudo
- **Network Issues**: Verify internet connection and proxy settings
- **Disk Space**: Ensure at least 5GB free space
- **Antivirus**: Temporarily disable antivirus during installation

#### Application Doesn't Start
- Verify that Ollama is running and accessible
- Check if port 11434 is available
- Review installation logs for specific errors

#### IDE Integration Not Working
- Restart your IDE and the GerdsenAI Socrates application
- Check the Extensions panel to ensure it's installed
- Review extension logs for errors:
  - VS Code/VSCodium: Help → Toggle Developer Tools
  - Visual Studio: Help → Microsoft Visual Studio Extension Logs

#### Services Not Starting
- **Ollama**: Check if port 11434 is available
- **SearXNG**: Verify Docker is installed and running
- **Firewall**: Ensure required ports are not blocked

#### Connection Issues
- Ensure Ollama is running (`ollama serve` in terminal)
- Check firewall settings if using a remote Ollama instance
- Verify the correct URL in the connection settings

#### Models Not Loading
- Check your internet connection and Ollama configuration
- Verify the model is downloaded: `ollama list`

#### Performance Issues
- Try a smaller Ollama model
- Ensure your system meets the minimum requirements
- Close other resource-intensive applications

### Getting Help

1. **Check Documentation**: Review the [Troubleshooting Guide](../guides/troubleshooting.md)
2. **Log Files**: Check installation logs for specific errors
3. **GitHub Issues**: Report bugs and get community help
4. **Community Support**: Join Discord/discussions for real-time help

---

## 🗑️ Uninstallation

### Windows

**Using Control Panel:**
1. Open Control Panel → Programs → Uninstall a program
2. Select GerdsenAI Socrates and click Uninstall

**Using Settings:**
1. Open Windows Settings
2. Go to Apps > Installed Apps
3. Find "GerdsenAI Socrates" in the list
4. Click on the three dots and select "Uninstall"
5. Follow the on-screen instructions

### macOS

1. Drag GerdsenAI Socrates from Applications to Trash
2. Empty Trash

### Linux

**Debian/Ubuntu:**
```bash
sudo apt remove gerdsenai-socrates
```

**AppImage:**
Simply delete the AppImage file.

### IDE Extensions

- **VS Code/VSCodium**: Go to Extensions, find Auto-Coder Companion, click the gear icon, and select "Uninstall"
- **Visual Studio**: Go to Extensions → Manage Extensions, find Auto-Coder Companion, and click "Uninstall"

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

See [Contributing Guidelines](../development/contributing.md) for more information.

---

## 📄 License and Support

- **License**: See LICENSE file for terms
- **Support**: Community support via GitHub Issues
- **Enterprise**: Contact for enterprise support options
- **Documentation**: Visit our [GitHub repository](https://github.com/GerdsenAI/GerdsenAI_AutoCoder)

---

**Happy coding with GerdsenAI Socrates! 🚀**

*For platform-specific details, see: [Windows Setup](../setup/windows.md) | [macOS Setup](../setup/macos.md) | [Linux Setup](../setup/linux.md)*
