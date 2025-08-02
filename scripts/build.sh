#!/bin/bash

# Cross-platform build script for Auto-Coder Companion
# This script builds the application for all supported platforms

set -e

# Set up environment
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

echo "🚀 Building GerdsenAI Socrates..."

# Check for required tools
echo "🔍 Checking for required tools..."

if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js 18+ and try again."
    exit 1
fi

if ! command -v npm &> /dev/null; then
    echo "❌ npm is not installed. Please install npm and try again."
    exit 1
fi

if ! command -v rustc &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust 1.70+ and try again."
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo is not installed. Please install Rust 1.70+ and try again."
    exit 1
fi

# Install dependencies
echo "📦 Installing dependencies..."
npm install

# Build frontend
echo "🔨 Building frontend..."
npm run build

# Build Tauri app
echo "🔨 Building Tauri app..."

# Detect platform and build accordingly
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "🍎 macOS detected - building universal binary..."
    
    # Check if universal targets are installed
    if ! rustup target list --installed | grep -q "aarch64-apple-darwin"; then
        echo "📦 Installing Apple Silicon target..."
        rustup target add aarch64-apple-darwin
    fi
    
    if ! rustup target list --installed | grep -q "x86_64-apple-darwin"; then
        echo "📦 Installing Intel target..."
        rustup target add x86_64-apple-darwin
    fi
    
    # Build universal binary
    npm run tauri:build:macos
    echo "✅ Universal macOS binary created (supports Apple Silicon + Intel)"
else
    echo "🖥️ Non-macOS platform detected - building standard binary..."
    npm run tauri build
fi

# Build LSP server
echo "🔨 Building LSP server..."
cd "$PROJECT_ROOT/src-tauri"
cargo build --release --bin auto-coder-lsp

# Build VS Code extension
echo "🔨 Building VS Code extension..."
cd "$PROJECT_ROOT/extensions/vscode"
npm install
npm run package

# Build VSCodium extension (same as VS Code but with different target)
echo "🔨 Building VSCodium extension..."
cd "$PROJECT_ROOT/extensions/vscode"
npm run package -- --target vscodium

# Build Visual Studio extension
echo "🔨 Building Visual Studio extension..."
cd "$PROJECT_ROOT/extensions/visual-studio/AutoCoderExtension"
if command -v dotnet &> /dev/null; then
    dotnet build -c Release
else
    echo "⚠️ .NET SDK not found, skipping Visual Studio extension build"
fi

echo "✅ Build completed successfully!"
echo "📁 Output files can be found in:"

if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "  - macOS App Bundle: $PROJECT_ROOT/src-tauri/target/universal-apple-darwin/release/bundle/macos/"
    echo "  - macOS DMG Installer: $PROJECT_ROOT/src-tauri/target/universal-apple-darwin/release/bundle/dmg/"
else
    echo "  - Tauri app: $PROJECT_ROOT/src-tauri/target/release"
fi

echo "  - LSP Server: $PROJECT_ROOT/src-tauri/target/release/auto-coder-lsp"
echo "  - VS Code extension: $PROJECT_ROOT/extensions/vscode/gerdsenai-socrates.vsix"
echo "  - Visual Studio extension: $PROJECT_ROOT/extensions/visual-studio/AutoCoderExtension/bin/Release"
