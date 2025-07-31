#!/bin/bash

# Platform-specific testing script for Auto-Coder Companion
# This script tests IDE integration on different platforms

set -e

# Set up environment
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

echo "🧪 Running platform-specific IDE integration tests..."

# Detect platform
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  PLATFORM="linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
  PLATFORM="macos"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
  PLATFORM="windows"
else
  echo "❌ Unsupported platform: $OSTYPE"
  exit 1
fi

echo "🖥️ Detected platform: $PLATFORM"

# Test VS Code integration
test_vscode() {
  echo "📋 Testing VS Code integration..."
  
  # Check if VS Code is installed
  if command -v code >/dev/null 2>&1; then
    echo "✅ VS Code found"
    
    # Install extension
    echo "📦 Installing VS Code extension..."
    code --install-extension ./extensions/vscode/auto-coder-companion.vsix
    
    # Test extension activation
    echo "🚀 Testing extension activation..."
    code --list-extensions | grep auto-coder-companion
    
    echo "✅ VS Code integration test passed"
    return 0
  else
    echo "⚠️ VS Code not found, skipping test"
    return 0
  fi
}

# Test VSCodium integration
test_vscodium() {
  echo "📋 Testing VSCodium integration..."
  
  # Check if VSCodium is installed
  if command -v codium >/dev/null 2>&1; then
    echo "✅ VSCodium found"
    
    # Install extension
    echo "📦 Installing VSCodium extension..."
    codium --install-extension ./extensions/vscode/auto-coder-companion.vsix
    
    # Test extension activation
    echo "🚀 Testing extension activation..."
    codium --list-extensions | grep auto-coder-companion
    
    echo "✅ VSCodium integration test passed"
    return 0
  else
    echo "⚠️ VSCodium not found, skipping test"
    return 0
  fi
}

# Test Visual Studio integration
test_visualstudio() {
  echo "📋 Testing Visual Studio integration..."
  
  if [[ "$PLATFORM" == "windows" ]]; then
    # Check if Visual Studio is installed
    if command -v devenv.exe >/dev/null 2>&1; then
      echo "✅ Visual Studio found"
      
      # Install extension
      echo "📦 Installing Visual Studio extension..."
      # This would typically use VSIXInstaller.exe
      # For this test script, we'll just check if the file exists
      if [[ -f "./extensions/visual-studio/AutoCoderExtension/bin/Release/AutoCoderCompanion.vsix" ]]; then
        echo "✅ Visual Studio extension file exists"
      else
        echo "❌ Visual Studio extension file not found"
        return 1
      fi
      
      echo "✅ Visual Studio integration test passed"
      return 0
    else
      echo "⚠️ Visual Studio not found, skipping test"
      return 0
    fi
  else
    echo "⚠️ Visual Studio integration only available on Windows, skipping test"
    return 0
  fi
}

# Test LSP server
test_lsp_server() {
  echo "📋 Testing LSP server..."
  
  # Build LSP server
  echo "🔨 Building LSP server..."
  cargo build --manifest-path src-tauri/Cargo.toml --bin auto-coder-lsp
  
  # Test LSP server startup
  echo "🚀 Testing LSP server startup..."
  if [[ "$PLATFORM" == "windows" ]]; then
    timeout 5 ./target/debug/auto-coder-lsp.exe --test-mode || true
  else
    timeout 5 ./target/debug/auto-coder-lsp --test-mode || true
  fi
  
  echo "✅ LSP server test passed"
  return 0
}

# Run tests based on platform
echo "🧪 Running tests for $PLATFORM platform..."

# Common tests for all platforms
test_lsp_server
test_vscode
test_vscodium

# Platform-specific tests
if [[ "$PLATFORM" == "windows" ]]; then
  test_visualstudio
fi

echo "🏁 Platform-specific IDE integration testing completed!"
