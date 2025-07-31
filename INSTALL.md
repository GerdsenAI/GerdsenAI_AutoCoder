# Installation Guide for Auto-Coder Companion

This guide provides detailed instructions for installing Auto-Coder Companion on different platforms and integrating it with supported IDEs.

## System Requirements

- **Operating Systems**:
  - Windows 10/11 (64-bit)
  - macOS 15+ (Intel and Apple Silicon)
  - Linux (Ubuntu 20.04+, Debian 11+, Fedora 35+)
- **Hardware**:
  - 4GB RAM minimum (8GB+ recommended)
  - 2GB free disk space
  - x86_64 or ARM64 processor
- **Software Dependencies**:
  - Ollama installed and running
  - SearXNG instance (optional)

## Standalone Application Installation

### Windows

1. Download the `Auto-Coder-Companion_x.x.x_x64_windows.msi` installer
2. Double-click the installer and follow the on-screen instructions
3. Launch Auto-Coder Companion from the Start menu

### macOS

1. Download the `Auto-Coder-Companion_x.x.x_universal.dmg` disk image
2. Open the disk image and drag Auto-Coder Companion to the Applications folder
3. Right-click the app and select "Open" (required for first launch due to security)
4. Follow any security prompts to allow the application to run

### Linux

#### Debian/Ubuntu (DEB package)

```bash
# Install the DEB package
sudo apt install ./auto-coder-companion_x.x.x_amd64.deb

# Launch from terminal
auto-coder-companion

# Or find in application menu
```

#### AppImage

```bash
# Make the AppImage executable
chmod +x auto-coder-companion_x.x.x_amd64.AppImage

# Run the AppImage
./auto-coder-companion_x.x.x_amd64.AppImage
```

## IDE Extension Installation

### VS Code

1. Open VS Code
2. Go to Extensions (Ctrl+Shift+X or Cmd+Shift+X on macOS)
3. Click the "..." menu in the Extensions panel
4. Select "Install from VSIX..."
5. Navigate to and select the `auto-coder-companion.vsix` file
6. Restart VS Code when prompted

### VSCodium

1. Open VSCodium
2. Go to Extensions (Ctrl+Shift+X or Cmd+Shift+X on macOS)
3. Click the "..." menu in the Extensions panel
4. Select "Install from VSIX..."
5. Navigate to and select the `auto-coder-companion.vsix` file
6. Restart VSCodium when prompted

### Visual Studio 2022+

1. Open Visual Studio
2. Go to Extensions → Manage Extensions
3. Click on "Install from VSIX..."
4. Navigate to and select the `AutoCoderCompanion.vsix` file
5. Restart Visual Studio when prompted

## Configuring Ollama Connection

Auto-Coder Companion connects to Ollama by default at `http://localhost:11434`. To use a different Ollama instance:

1. Open Auto-Coder Companion
2. Click on the model selector dropdown
3. Click "Custom Ollama URL"
4. Enter the URL of your Ollama instance
5. Click "Connect"

## Configuring SearXNG Connection

To configure a custom SearXNG instance:

1. Open Auto-Coder Companion
2. Go to Settings (gear icon)
3. Navigate to the "Search" tab
4. Enter your SearXNG instance URL
5. Click "Save"

## Troubleshooting

### Connection Issues

If Auto-Coder Companion cannot connect to Ollama:

1. Ensure Ollama is running (`ollama serve` in terminal)
2. Check firewall settings if using a remote Ollama instance
3. Verify the correct URL in the connection settings

### IDE Extension Not Loading

If the IDE extension doesn't appear:

1. Check the Extensions panel to ensure it's installed
2. Restart the IDE
3. Check the extension logs for errors:
   - VS Code/VSCodium: Help → Toggle Developer Tools
   - Visual Studio: Help → Microsoft Visual Studio Extension Logs

### Performance Issues

If experiencing slow performance:

1. Try a smaller Ollama model
2. Ensure your system meets the minimum requirements
3. Close other resource-intensive applications

## Uninstallation

### Windows

1. Open Control Panel → Programs → Uninstall a program
2. Select Auto-Coder Companion and click Uninstall

### macOS

1. Drag Auto-Coder Companion from Applications to Trash
2. Empty Trash

### Linux

#### Debian/Ubuntu

```bash
sudo apt remove auto-coder-companion
```

#### AppImage

Simply delete the AppImage file.

### IDE Extensions

- VS Code/VSCodium: Go to Extensions, find Auto-Coder Companion, click the gear icon, and select "Uninstall"
- Visual Studio: Go to Extensions → Manage Extensions, find Auto-Coder Companion, and click "Uninstall"
