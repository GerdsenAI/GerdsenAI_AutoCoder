#!/bin/bash

# GerdsenAI Socrates - Simplified Cross-Platform Installer
# Copyright (c) 2025 GerdsenAI. All rights reserved.
# This script automates the complete installation process

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script configuration
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
INSTALL_LOG="/tmp/gerdsenai-install.log"
BACKUP_DIR="$HOME/.gerdsenai-backup-$(date +%Y%m%d_%H%M%S)"

# Function definitions
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1" | tee -a "$INSTALL_LOG"
}

success() {
    echo -e "${GREEN}✅ $1${NC}" | tee -a "$INSTALL_LOG"
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}" | tee -a "$INSTALL_LOG"
}

error() {
    echo -e "${RED}❌ $1${NC}" | tee -a "$INSTALL_LOG"
    exit 1
}

# Platform detection
detect_platform() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        PLATFORM="linux"
        if command -v lsb_release &> /dev/null; then
            DISTRO=$(lsb_release -si)
            VERSION=$(lsb_release -sr)
        elif [ -f /etc/os-release ]; then
            . /etc/os-release
            DISTRO=$NAME
            VERSION=$VERSION_ID
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        PLATFORM="macos"
        DISTRO="macOS"
        VERSION=$(sw_vers -productVersion)
        ARCH=$(uname -m)
    elif [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
        PLATFORM="windows"
        DISTRO="Windows"
    else
        error "Unsupported platform: $OSTYPE"
    fi
    
    log "Detected platform: $PLATFORM ($DISTRO $VERSION)"
}

# System requirements check
check_requirements() {
    log "Checking system requirements..."
    
    # Check available memory
    if [[ "$PLATFORM" == "linux" ]]; then
        MEMORY_GB=$(free -g | awk '/^Mem:/{print $2}')
    elif [[ "$PLATFORM" == "macos" ]]; then
        MEMORY_BYTES=$(sysctl -n hw.memsize)
        MEMORY_GB=$((MEMORY_BYTES / 1024 / 1024 / 1024))
    fi
    
    if [ "$MEMORY_GB" -lt 8 ]; then
        warning "System has ${MEMORY_GB}GB RAM. 8GB+ recommended for optimal performance."
    else
        success "Memory check passed: ${MEMORY_GB}GB RAM available"
    fi
    
    # Check available disk space
    AVAILABLE_SPACE=$(df -BG "$HOME" | awk 'NR==2 {print $4}' | sed 's/G//')
    if [ "$AVAILABLE_SPACE" -lt 5 ]; then
        error "Insufficient disk space. Need at least 5GB, have ${AVAILABLE_SPACE}GB"
    else
        success "Disk space check passed: ${AVAILABLE_SPACE}GB available"
    fi
}

# Backup existing installation
backup_existing() {
    log "Checking for existing installation..."
    
    local config_dirs=()
    if [[ "$PLATFORM" == "macos" ]]; then
        config_dirs+=(
            "$HOME/Library/Application Support/GerdsenAI"
            "$HOME/.config/GerdsenAI"
        )
    else
        config_dirs+=(
            "$HOME/.config/GerdsenAI"
            "$HOME/.local/share/GerdsenAI"
        )
    fi
    
    for dir in "${config_dirs[@]}"; do
        if [ -d "$dir" ]; then
            log "Backing up existing configuration: $dir"
            mkdir -p "$BACKUP_DIR"
            cp -r "$dir" "$BACKUP_DIR/" || warning "Failed to backup $dir"
        fi
    done
    
    if [ -d "$BACKUP_DIR" ]; then
        success "Backup created at: $BACKUP_DIR"
    fi
}

# Install system dependencies
install_system_deps() {
    log "Installing system dependencies for $PLATFORM..."
    
    case "$PLATFORM" in
        "linux")
            if command -v apt-get &> /dev/null; then
                # Ubuntu/Debian
                log "Installing dependencies with apt-get..."
                sudo apt-get update || warning "Failed to update package list"
                sudo apt-get install -y \
                    curl \
                    wget \
                    build-essential \
                    libwebkit2gtk-4.0-dev \
                    libssl-dev \
                    libgtk-3-dev \
                    libayatana-appindicator3-dev \
                    librsvg2-dev \
                    git || error "Failed to install system dependencies"
                    
            elif command -v dnf &> /dev/null; then
                # Fedora/RHEL
                log "Installing dependencies with dnf..."
                sudo dnf update -y || warning "Failed to update package list"
                sudo dnf install -y \
                    curl \
                    wget \
                    gcc \
                    gcc-c++ \
                    make \
                    webkit2gtk3-devel \
                    openssl-devel \
                    gtk3-devel \
                    libappindicator-gtk3-devel \
                    librsvg2-devel \
                    git || error "Failed to install system dependencies"
                    
            elif command -v pacman &> /dev/null; then
                # Arch Linux
                log "Installing dependencies with pacman..."
                sudo pacman -Syu --noconfirm || warning "Failed to update package list"
                sudo pacman -S --noconfirm \
                    curl \
                    wget \
                    base-devel \
                    webkit2gtk \
                    openssl \
                    gtk3 \
                    libappindicator-gtk3 \
                    librsvg \
                    git || error "Failed to install system dependencies"
            else
                warning "Unknown Linux distribution. Please install dependencies manually."
            fi
            ;;
            
        "macos")
            log "Installing dependencies with Homebrew..."
            if ! command -v brew &> /dev/null; then
                log "Installing Homebrew..."
                /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)" || error "Failed to install Homebrew"
            fi
            
            brew update || warning "Failed to update Homebrew"
            brew install curl wget git || warning "Failed to install some dependencies"
            ;;
            
        *)
            warning "Automatic dependency installation not supported for $PLATFORM"
            ;;
    esac
    
    success "System dependencies installed"
}

# Install Node.js
install_nodejs() {
    log "Installing Node.js..."
    
    if command -v node &> /dev/null; then
        NODE_VERSION=$(node --version | sed 's/v//')
        NODE_MAJOR=$(echo "$NODE_VERSION" | cut -d. -f1)
        
        if [ "$NODE_MAJOR" -ge 18 ]; then
            success "Node.js $NODE_VERSION already installed"
            return
        else
            warning "Node.js $NODE_VERSION is too old. Need 18+. Installing newer version..."
        fi
    fi
    
    case "$PLATFORM" in
        "linux")
            # Install Node.js 20 LTS
            curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash - || error "Failed to setup Node.js repository"
            sudo apt-get install -y nodejs || error "Failed to install Node.js"
            ;;
            
        "macos")
            brew install node@20 || error "Failed to install Node.js"
            ;;
            
        *)
            error "Please install Node.js 18+ manually from https://nodejs.org/"
            ;;
    esac
    
    # Verify installation
    if command -v node &> /dev/null && command -v npm &> /dev/null; then
        NODE_VERSION=$(node --version)
        NPM_VERSION=$(npm --version)
        success "Node.js $NODE_VERSION and npm $NPM_VERSION installed"
    else
        error "Node.js installation verification failed"
    fi
}

# Install Rust
install_rust() {
    log "Installing Rust..."
    
    if command -v rustc &> /dev/null; then
        RUST_VERSION=$(rustc --version | cut -d' ' -f2)
        success "Rust $RUST_VERSION already installed"
        return
    fi
    
    log "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y || error "Failed to install Rust"
    
    # Source cargo environment
    source "$HOME/.cargo/env" || error "Failed to source Rust environment"
    
    # Verify installation
    if command -v rustc &> /dev/null && command -v cargo &> /dev/null; then
        RUST_VERSION=$(rustc --version)
        success "Rust installed: $RUST_VERSION"
    else
        error "Rust installation verification failed"
    fi
}

# Install Ollama
install_ollama() {
    log "Installing Ollama..."
    
    if command -v ollama &> /dev/null; then
        OLLAMA_VERSION=$(ollama --version)
        success "Ollama already installed: $OLLAMA_VERSION"
        return
    fi
    
    case "$PLATFORM" in
        "linux")
            curl -fsSL https://ollama.ai/install.sh | sh || error "Failed to install Ollama"
            ;;
            
        "macos")
            if command -v brew &> /dev/null; then
                brew install ollama || error "Failed to install Ollama"
            else
                curl -fsSL https://ollama.ai/install.sh | sh || error "Failed to install Ollama"
            fi
            ;;
            
        *)
            error "Please install Ollama manually from https://ollama.ai/"
            ;;
    esac
    
    # Verify installation and start service
    if command -v ollama &> /dev/null; then
        success "Ollama installed successfully"
        
        # Start Ollama service
        log "Starting Ollama service..."
        if [[ "$PLATFORM" == "macos" ]]; then
            # On macOS, Ollama typically starts automatically
            ollama serve &
            OLLAMA_PID=$!
            sleep 3
        else
            # On Linux, start as background service
            nohup ollama serve > /dev/null 2>&1 &
            OLLAMA_PID=$!
            sleep 3
        fi
        
        # Test connection
        if curl -s http://localhost:11434/api/version > /dev/null; then
            success "Ollama service started and accessible"
        else
            warning "Ollama installed but service may not be running correctly"
        fi
    else
        error "Ollama installation verification failed"
    fi
}

# Install Docker (optional, for SearXNG)
install_docker() {
    log "Installing Docker (optional, for web search)..."
    
    if command -v docker &> /dev/null; then
        DOCKER_VERSION=$(docker --version | cut -d' ' -f3 | sed 's/,//')
        success "Docker already installed: $DOCKER_VERSION"
        return
    fi
    
    case "$PLATFORM" in
        "linux")
            # Install Docker using official script
            curl -fsSL https://get.docker.com -o get-docker.sh || warning "Failed to download Docker installer"
            if [ -f get-docker.sh ]; then
                sudo sh get-docker.sh || warning "Failed to install Docker"
                sudo usermod -aG docker "$USER" || warning "Failed to add user to docker group"
                rm get-docker.sh
            fi
            ;;
            
        "macos")
            warning "Please install Docker Desktop manually from https://www.docker.com/products/docker-desktop/"
            warning "Docker is optional - web search will be disabled without it"
            return
            ;;
            
        *)
            warning "Docker installation not supported on this platform"
            return
            ;;
    esac
    
    # Verify installation
    if command -v docker &> /dev/null; then
        success "Docker installed successfully"
        log "Note: You may need to restart your shell or log out/in for Docker permissions to take effect"
    else
        warning "Docker installation may have failed - web search will be disabled"
    fi
}

# Install Python and ChromaDB
install_chromadb() {
    log "Installing Python and ChromaDB..."
    
    # Check for Python
    if command -v python3 &> /dev/null; then
        PYTHON_VERSION=$(python3 --version | cut -d' ' -f2)
        success "Python $PYTHON_VERSION already installed"
    else
        case "$PLATFORM" in
            "linux")
                sudo apt-get install -y python3 python3-pip || warning "Failed to install Python"
                ;;
            "macos")
                brew install python@3.11 || warning "Failed to install Python"
                ;;
            *)
                warning "Please install Python 3.8+ manually"
                return
                ;;
        esac
    fi
    
    # Install ChromaDB
    log "Installing ChromaDB..."
    if command -v pip3 &> /dev/null; then
        pip3 install --user chromadb || warning "Failed to install ChromaDB"
        success "ChromaDB installed"
    else
        warning "pip3 not available - ChromaDB installation skipped"
    fi
}

# Download and install GerdsenAI Socrates
install_application() {
    log "Installing GerdsenAI Socrates..."
    
    cd "$PROJECT_ROOT"
    
    # Install npm dependencies
    log "Installing Node.js dependencies..."
    npm install || error "Failed to install npm dependencies"
    
    # Install development tools if needed
    if ! command -v @tauri-apps/cli &> /dev/null; then
        log "Installing Tauri CLI..."
        npm install -g @tauri-apps/cli || warning "Failed to install Tauri CLI globally"
    fi
    
    # Build the application
    log "Building application..."
    npm run build || error "Failed to build frontend"
    
    # Build Tauri application
    log "Building Tauri application (this may take several minutes)..."
    case "$PLATFORM" in
        "macos")
            if [[ "$ARCH" == "arm64" ]]; then
                npm run tauri:build:macos:silicon || warning "Failed to build for Apple Silicon, trying universal"
                npm run tauri:build:macos || error "Failed to build macOS application"
            else
                npm run tauri:build:macos:intel || warning "Failed to build for Intel, trying universal"
                npm run tauri:build:macos || error "Failed to build macOS application"
            fi
            
            # Install the built application
            if [ -d "src-tauri/target/release/bundle/dmg" ]; then
                DMG_FILE=$(find src-tauri/target/release/bundle/dmg -name "*.dmg" | head -1)
                if [ -n "$DMG_FILE" ]; then
                    log "Installing application from $DMG_FILE"
                    hdiutil attach "$DMG_FILE" -quiet
                    MOUNT_POINT=$(hdiutil info | grep "GerdsenAI" | awk '{print $3}')
                    if [ -n "$MOUNT_POINT" ]; then
                        cp -r "$MOUNT_POINT/GerdsenAI Socrates.app" /Applications/ || error "Failed to install application"
                        hdiutil detach "$MOUNT_POINT" -quiet
                        success "Application installed to /Applications/"
                    fi
                fi
            fi
            ;;
            
        "linux")
            npm run tauri build || error "Failed to build Linux application"
            
            # Install the built AppImage
            if [ -d "src-tauri/target/release/bundle/appimage" ]; then
                APPIMAGE_FILE=$(find src-tauri/target/release/bundle/appimage -name "*.AppImage" | head -1)
                if [ -n "$APPIMAGE_FILE" ]; then
                    log "Installing AppImage: $APPIMAGE_FILE"
                    mkdir -p "$HOME/.local/bin"
                    cp "$APPIMAGE_FILE" "$HOME/.local/bin/gerdsenai-socrates" || error "Failed to install AppImage"
                    chmod +x "$HOME/.local/bin/gerdsenai-socrates"
                    
                    # Create desktop entry
                    create_desktop_entry
                    success "Application installed to $HOME/.local/bin/"
                fi
            fi
            ;;
            
        *)
            error "Unsupported platform for application build: $PLATFORM"
            ;;
    esac
    
    success "GerdsenAI Socrates installed successfully"
}

# Create desktop entry for Linux
create_desktop_entry() {
    log "Creating desktop entry..."
    
    DESKTOP_FILE="$HOME/.local/share/applications/gerdsenai-socrates.desktop"
    mkdir -p "$(dirname "$DESKTOP_FILE")"
    
    cat > "$DESKTOP_FILE" << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=GerdsenAI Socrates
Comment=Advanced AI Coding Assistant
Exec=$HOME/.local/bin/gerdsenai-socrates
Icon=gerdsenai-socrates
Terminal=false
Categories=Development;IDE;
EOF
    
    chmod +x "$DESKTOP_FILE"
    success "Desktop entry created"
}

# Download and install AI models
install_models() {
    log "Installing recommended AI models..."
    
    if ! command -v ollama &> /dev/null; then
        warning "Ollama not available - skipping model installation"
        return
    fi
    
    # Test Ollama connection
    if ! curl -s http://localhost:11434/api/version > /dev/null; then
        warning "Ollama service not accessible - skipping model installation"
        return
    fi
    
    local models=("qwen2.5-coder:7b" "codellama:7b")
    
    for model in "${models[@]}"; do
        log "Installing model: $model (this may take several minutes)..."
        ollama pull "$model" || warning "Failed to install model: $model"
    done
    
    # Verify models
    log "Verifying installed models..."
    ollama list || warning "Failed to list models"
    
    success "Model installation completed"
}

# Setup SearXNG (optional)
setup_searxng() {
    log "Setting up SearXNG web search (optional)..."
    
    if ! command -v docker &> /dev/null; then
        warning "Docker not available - skipping SearXNG setup"
        return
    fi
    
    cd "$PROJECT_ROOT"
    
    if [ -d "docker/searxng" ]; then
        cd docker/searxng
        
        log "Starting SearXNG services..."
        if [ -f "start-searxng.sh" ]; then
            chmod +x start-searxng.sh
            ./start-searxng.sh || warning "Failed to start SearXNG services"
            
            # Test SearXNG
            sleep 10
            if curl -s "http://localhost:8080/search?q=test&format=json" > /dev/null; then
                success "SearXNG web search configured and running"
            else
                warning "SearXNG may not be running correctly"
            fi
        else
            warning "SearXNG start script not found"
        fi
    else
        warning "SearXNG configuration not found"
    fi
}

# Final configuration and testing
final_setup() {
    log "Performing final setup and testing..."
    
    # Create configuration directories
    case "$PLATFORM" in
        "macos")
            CONFIG_DIR="$HOME/Library/Application Support/GerdsenAI/Socrates"
            ;;
        *)
            CONFIG_DIR="$HOME/.config/GerdsenAI/Socrates"
            ;;
    esac
    
    mkdir -p "$CONFIG_DIR"
    
    # Create basic configuration
    cat > "$CONFIG_DIR/config.json" << EOF
{
    "version": "1.0.0",
    "firstRun": true,
    "theme": "system",
    "services": {
        "ollama": {
            "enabled": true,
            "url": "http://localhost:11434"
        },
        "searxng": {
            "enabled": $(command -v docker &> /dev/null && echo true || echo false),
            "url": "http://localhost:8080"
        },
        "chromadb": {
            "enabled": $(python3 -c "import chromadb" 2>/dev/null && echo true || echo false)
        }
    }
}
EOF
    
    success "Configuration created at: $CONFIG_DIR"
    
    # Test services
    log "Testing service connections..."
    
    # Test Ollama
    if curl -s http://localhost:11434/api/version > /dev/null; then
        success "✓ Ollama service is accessible"
    else
        warning "✗ Ollama service is not accessible"
    fi
    
    # Test SearXNG
    if curl -s http://localhost:8080/search?q=test > /dev/null; then
        success "✓ SearXNG web search is accessible"
    else
        warning "✗ SearXNG web search is not accessible (optional)"
    fi
    
    # Test ChromaDB
    if python3 -c "import chromadb" 2>/dev/null; then
        success "✓ ChromaDB is available"
    else
        warning "✗ ChromaDB is not available (optional)"
    fi
}

# Cleanup function
cleanup() {
    log "Cleaning up temporary files..."
    
    # Kill background processes if they were started by this script
    if [ -n "$OLLAMA_PID" ]; then
        kill "$OLLAMA_PID" 2>/dev/null || true
    fi
    
    # Remove temporary files
    rm -f get-docker.sh 2>/dev/null || true
    
    success "Cleanup completed"
}

# Main installation function
main() {
    echo "🚀 GerdsenAI Socrates - Simplified Installer"
    echo "==========================================="
    echo
    
    log "Starting installation process..."
    log "Installation log: $INSTALL_LOG"
    
    # Set up cleanup trap
    trap cleanup EXIT
    
    # Run installation steps
    detect_platform
    check_requirements
    backup_existing
    install_system_deps
    install_nodejs
    install_rust
    install_ollama
    install_docker
    install_chromadb
    install_application
    install_models
    setup_searxng
    final_setup
    
    echo
    echo "🎉 Installation completed successfully!"
    echo
    echo "📋 Summary:"
    echo "  • Platform: $PLATFORM ($DISTRO $VERSION)"
    echo "  • Node.js: $(node --version 2>/dev/null || echo 'Not available')"
    echo "  • Rust: $(rustc --version 2>/dev/null | cut -d' ' -f2 || echo 'Not available')"
    echo "  • Ollama: $(ollama --version 2>/dev/null || echo 'Not available')"
    echo "  • Docker: $(docker --version 2>/dev/null | cut -d' ' -f3 | sed 's/,//' || echo 'Not available')"
    echo
    echo "🚀 To start using GerdsenAI Socrates:"
    
    case "$PLATFORM" in
        "macos")
            echo "  • Open Applications folder and launch 'GerdsenAI Socrates'"
            echo "  • Or use Spotlight search (Cmd+Space) and type 'GerdsenAI'"
            ;;
        "linux")
            echo "  • Run: gerdsenai-socrates"
            echo "  • Or find 'GerdsenAI Socrates' in your applications menu"
            ;;
    esac
    
    echo
    echo "📚 Documentation:"
    echo "  • User Manual: COMPREHENSIVE_USER_MANUAL.md"
    echo "  • Troubleshooting: TROUBLESHOOTING_GUIDE.md"
    echo "  • Installation log: $INSTALL_LOG"
    
    if [ -d "$BACKUP_DIR" ]; then
        echo "  • Backup of previous installation: $BACKUP_DIR"
    fi
    
    echo
    echo "🔧 If you encounter issues:"
    echo "  • Check the troubleshooting guide"
    echo "  • Review the installation log"
    echo "  • Visit: https://github.com/GerdsenAI/GerdsenAI_AutoCoder/issues"
    echo
    
    success "Installation completed! 🎉"
}

# Run main function
main "$@"