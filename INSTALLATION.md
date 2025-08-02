# GerdsenAI Socrates - Installation Guide

This guide provides detailed instructions for installing and setting up the GerdsenAI Socrates on Windows 11.

## System Requirements

- **Operating System**: Windows 11 (64-bit)
- **Processor**: 4-core CPU or better
- **Memory**: 8GB RAM minimum (16GB recommended)
- **Storage**: 2GB available space
- **Additional**: Internet connection for initial setup and model downloads

## Prerequisites

1. **Ollama**: GerdsenAI Socrates requires Ollama to be installed for AI model functionality.
   - Download Ollama from [https://ollama.ai/download](https://ollama.ai/download)
   - Install and verify it's running (you should see the Ollama icon in your system tray)

2. **Visual Studio Code** or **Visual Studio**: For optimal IDE integration
   - Download VS Code from [https://code.visualstudio.com/](https://code.visualstudio.com/)
   - Or Visual Studio from [https://visualstudio.microsoft.com/](https://visualstudio.microsoft.com/)

## Installation Options

### Option 1: Automated Installation (Recommended)

1. Download the GerdsenAI Socrates installer from the official website
2. Right-click on `GerdsenAI_Socrates_Setup.exe` and select "Run as administrator"
3. Follow the on-screen instructions to complete the installation
4. Launch GerdsenAI Socrates from the Start menu or desktop shortcut

### Option 2: Using the Setup Script

1. Extract the GerdsenAI Socrates zip file to a location of your choice
2. Right-click on `WINDOWS_SETUP.BAT` and select "Run as administrator"
3. The script will automatically:
   - Install required dependencies
   - Configure the application
   - Create desktop and Start menu shortcuts
   - Set up IDE integration

### Option 3: Manual Installation

1. Extract the GerdsenAI Socrates zip file to a location of your choice
2. Open Command Prompt as administrator
3. Navigate to the extracted directory:
   ```
   cd path\to\GerdsenAI_Socrates
   ```
4. Run the following commands:
   ```
   npm install
   npm run tauri build
   ```
5. The installer will be generated in the `target\release\bundle\msi\` directory
6. Run the installer and follow the on-screen instructions

## Post-Installation Setup

1. **First Launch**: When you first launch GerdsenAI Socrates, it will:
   - Detect your installed IDEs
   - Ask for permission to integrate with them
   - Download the default Ollama model (if not already present)

2. **IDE Integration**:
   - For VS Code: The application will automatically dock to the right side
   - For Visual Studio: The application will automatically dock to the right side
   - You can undock/redock using the dock/undock button in the top-right corner

3. **Model Selection**:
   - Click on the model dropdown to select your preferred Ollama model
   - The application will download the model if it's not already available locally

## Troubleshooting

- **Application doesn't start**: Verify that Ollama is running and accessible
- **IDE integration not working**: Restart your IDE and the GerdsenAI Socrates application
- **Models not loading**: Check your internet connection and Ollama configuration

## Uninstallation

1. Open Windows Settings
2. Go to Apps > Installed Apps
3. Find "GerdsenAI Socrates" in the list
4. Click on the three dots and select "Uninstall"
5. Follow the on-screen instructions

Alternatively, you can use the provided uninstaller in the installation directory.

## Support

For additional help, please visit our support website or contact our technical support team.
