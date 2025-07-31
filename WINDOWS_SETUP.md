# CSE-Icon AutoCoder - Windows 11 Setup Guide

## System Requirements

- Windows 11 (recommended) or Windows 10
- 8GB RAM minimum (16GB recommended)
- 2GB free disk space
- Processor with 4+ cores
- Microsoft WebView2 Runtime
- Visual Studio Build Tools 2022
- Ollama installed (https://ollama.ai/download)

## Installation Instructions

### Automatic Installation

1. Right-click on `INSTALL_DEPENDENCIES.BAT` and select "Run as administrator"
2. Follow the on-screen instructions to install all required dependencies
3. Once dependencies are installed, run `START_APPLICATION.BAT` to launch the application

### Manual Installation

If the automatic installation fails, follow these steps to manually install the required dependencies:

1. **Install Node.js 20.x**
   - Download from: https://nodejs.org/
   - Select the LTS version (20.x or newer)
   - Run the installer and follow the on-screen instructions

2. **Install Rust**
   - Download from: https://www.rust-lang.org/tools/install
   - Run `rustup-init.exe` and follow the on-screen instructions
   - Choose the default installation options

3. **Install Visual Studio Build Tools**
   - Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
   - Select "Desktop development with C++" workload
   - Complete the installation

4. **Install WebView2 Runtime**
   - Download from: https://developer.microsoft.com/en-us/microsoft-edge/webview2/
   - Run the installer and follow the on-screen instructions

5. **Install Ollama**
   - Download from: https://ollama.ai/download
   - Run the installer and follow the on-screen instructions

6. **Install Project Dependencies**
   - Open Command Prompt in the project directory
   - Run `npm install`
   - Run `cargo install tauri-cli`

## Building the Application

### Development Mode

1. Open Command Prompt in the project directory
2. Run `npm run tauri dev`

### Production Build

1. Open Command Prompt in the project directory
2. Run `npm run tauri build`
3. The installer will be created in `.\target\release\bundle\msi\`

## Creating an Installer

1. Right-click on `GENERATE_INSTALLER.BAT` and select "Run as administrator"
2. Follow the on-screen instructions
3. The installer will be created in `.\installer\CSE-Icon_AutoCoder_Setup.msi`
4. A ZIP archive will also be created at `.\CSE-Icon_AutoCoder_Installer.zip`

## Troubleshooting

### Common Issues

1. **"Node.js not found" error**
   - Make sure Node.js is installed and added to your PATH
   - Try restarting your computer after installation

2. **"Rust not found" error**
   - Make sure Rust is installed and added to your PATH
   - Try running `rustup update` to ensure you have the latest version

3. **Build fails with WebView2 errors**
   - Make sure WebView2 Runtime is installed
   - Try reinstalling WebView2 Runtime

4. **"Ollama is not running" error**
   - Make sure Ollama is installed and running
   - Try starting Ollama manually before running the application

5. **"Visual Studio Build Tools not found" error**
   - Make sure Visual Studio Build Tools are installed with the "Desktop development with C++" workload
   - Try restarting your computer after installation

### Getting Help

If you encounter any issues not covered in this guide, please visit:
https://cse-icon.com/support
