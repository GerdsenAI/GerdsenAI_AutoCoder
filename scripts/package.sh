#!/bin/bash

# Platform-specific packaging script for Auto-Coder Companion
# This script creates platform-specific installers and packages

set -e

# Set up environment
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

echo "📦 Packaging Auto-Coder Companion..."

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

# Build the application first
echo "🔨 Building application..."
"$SCRIPT_DIR/build.sh"

# Create output directory
OUTPUT_DIR="$PROJECT_ROOT/dist/packages"
mkdir -p "$OUTPUT_DIR"

# Package based on platform
case "$PLATFORM" in
    linux)
        echo "📦 Creating Linux packages..."
        
        # Create DEB package
        echo "📦 Creating DEB package..."
        cp -r "$PROJECT_ROOT/src-tauri/target/release/bundle/deb/"*.deb "$OUTPUT_DIR/"
        
        # Create AppImage
        echo "📦 Creating AppImage..."
        cp -r "$PROJECT_ROOT/src-tauri/target/release/bundle/appimage/"*.AppImage "$OUTPUT_DIR/"
        
        # Create RPM package if available
        if [ -d "$PROJECT_ROOT/src-tauri/target/release/bundle/rpm" ]; then
            echo "📦 Creating RPM package..."
            cp -r "$PROJECT_ROOT/src-tauri/target/release/bundle/rpm/"*.rpm "$OUTPUT_DIR/"
        fi
        ;;
        
    macos)
        echo "📦 Creating macOS packages..."
        
        # Create DMG
        echo "📦 Creating DMG..."
        cp -r "$PROJECT_ROOT/src-tauri/target/release/bundle/dmg/"*.dmg "$OUTPUT_DIR/"
        
        # Create app bundle
        echo "📦 Creating app bundle..."
        cp -r "$PROJECT_ROOT/src-tauri/target/release/bundle/macos" "$OUTPUT_DIR/Auto-Coder Companion.app"
        ;;
        
    windows)
        echo "📦 Creating Windows packages..."
        
        # Create MSI installer
        echo "📦 Creating MSI installer..."
        cp -r "$PROJECT_ROOT/src-tauri/target/release/bundle/msi/"*.msi "$OUTPUT_DIR/"
        
        # Create portable EXE
        echo "📦 Creating portable EXE..."
        mkdir -p "$OUTPUT_DIR/portable"
        cp "$PROJECT_ROOT/src-tauri/target/release/auto-coder-companion.exe" "$OUTPUT_DIR/portable/"
        cp -r "$PROJECT_ROOT/src-tauri/target/release/resources" "$OUTPUT_DIR/portable/"
        ;;
        
    *)
        echo "❌ Unsupported platform for packaging: $PLATFORM"
        exit 1
        ;;
esac

# Package IDE extensions
echo "📦 Packaging IDE extensions..."
mkdir -p "$OUTPUT_DIR/extensions"

# VS Code extension
if [ -f "$PROJECT_ROOT/extensions/vscode/auto-coder-companion.vsix" ]; then
    echo "📦 Copying VS Code extension..."
    cp "$PROJECT_ROOT/extensions/vscode/auto-coder-companion.vsix" "$OUTPUT_DIR/extensions/"
fi

# VSCodium extension
if [ -f "$PROJECT_ROOT/extensions/vscode/auto-coder-companion-vscodium.vsix" ]; then
    echo "📦 Copying VSCodium extension..."
    cp "$PROJECT_ROOT/extensions/vscode/auto-coder-companion-vscodium.vsix" "$OUTPUT_DIR/extensions/"
fi

# Visual Studio extension
if [ -f "$PROJECT_ROOT/extensions/visual-studio/AutoCoderExtension/bin/Release/AutoCoderCompanion.vsix" ]; then
    echo "📦 Copying Visual Studio extension..."
    cp "$PROJECT_ROOT/extensions/visual-studio/AutoCoderExtension/bin/Release/AutoCoderCompanion.vsix" "$OUTPUT_DIR/extensions/"
fi

# Copy documentation
echo "📦 Copying documentation..."
mkdir -p "$OUTPUT_DIR/docs"
cp "$PROJECT_ROOT/README.md" "$OUTPUT_DIR/docs/"
cp "$PROJECT_ROOT/INSTALL.md" "$OUTPUT_DIR/docs/"
cp "$PROJECT_ROOT/USAGE.md" "$OUTPUT_DIR/docs/"

echo "✅ Packaging completed successfully!"
echo "📁 Packages can be found in: $OUTPUT_DIR"
