# GerdsenAI Socrates - Windows 11 Setup Guide

## System Requirements

- **Windows 11** (recommended) or Windows 10 version 1903+ 
- **8GB RAM minimum** (16GB+ recommended for optimal AI performance)
- **4GB free disk space** (for application + AI models)
- **Processor with 4+ cores** (8+ cores recommended for parallel processing)
- **Internet connection** for initial setup and AI model downloads
- **Microsoft WebView2 Runtime** (usually pre-installed on Windows 11)
- **Visual Studio Build Tools 2022** or Visual Studio Community 2022
- **Ollama** (https://ollama.ai/download) - Required for AI functionality

## Installation Instructions

### ⚡ Quick Start (Recommended)

**For most users, this one-click setup will handle everything:**

1. **Right-click** on `scripts/windows/install-dependencies.bat` and select **"Run as administrator"** (or use the root wrapper `install-wrapper.bat`)
2. **Wait** for the automatic installation to complete (5-15 minutes)
3. **Run** `scripts/windows/start-application.bat` to launch GerdsenAI Socrates (or use the root wrapper `start-wrapper.bat`)
4. **Verify** setup by running `node scripts/verify-setup.js` in the project directory

> 💡 **Tip**: The installer will automatically detect missing dependencies and install them. You'll see green checkmarks for successful installations.

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
   - Open Command Prompt **as Administrator** in the project directory
   - Run `npm install`
   - If you see permission errors, run `npm config set prefix %APPDATA%\npm`

---

## 🔧 Real-World Troubleshooting

### Common Installation Issues

#### "Ollama not found" or "Connection refused to localhost:11434"

**Problem**: The AI functionality shows "Ollama Not Running" error.

**Solutions**:
1. **Install Ollama**: Download from https://ollama.ai/download
2. **Start Ollama**: 
   - Open Command Prompt and run: `ollama serve`
   - Or restart the Ollama desktop application
3. **Install a model**: Run `ollama pull llama3.2` (recommended starter model)
4. **Check firewall**: Ensure Windows Firewall allows Ollama on port 11434

#### "npm install" fails with permission errors

**Problem**: `EACCES` or `permission denied` errors during installation.

**Solutions**:
1. **Run as Administrator**: Right-click Command Prompt → "Run as administrator"
2. **Fix npm permissions**: 
   ```bash
   npm config set prefix %APPDATA%\npm
   npm config set cache %APPDATA%\npm-cache
   ```
3. **Clear npm cache**: `npm cache clean --force`

#### "Visual Studio Build Tools required" during npm install

**Problem**: Native modules fail to build due to missing C++ build tools.

**Solutions**:
1. **Install Build Tools**: Download Visual Studio Build Tools 2022
2. **Select workload**: Choose "Desktop development with C++"
3. **Alternative**: Install Visual Studio Community 2022 (includes build tools)
4. **Restart** Command Prompt after installation

#### "Port 3000 already in use" when starting development

**Problem**: Development server can't start due to port conflicts.

**Solutions**:
1. **Find conflicting process**: Run `netstat -ano | findstr :3000`
2. **Kill process**: Run `taskkill /PID <process-id> /F`
3. **Use different port**: Run `npm run dev -- --port 3001`
4. **Check for other dev servers**: Stop other React/Node.js applications

#### Application starts but shows blank screen

**Problem**: GerdsenAI Socrates opens but displays a white/blank screen.

**Solutions**:
1. **Check WebView2**: Update Microsoft Edge or install WebView2 Runtime
2. **Clear cache**: Delete `%APPDATA%\GerdsenAI Socrates` folder
3. **Run in compatibility mode**: Right-click .exe → Properties → Compatibility → Windows 10
4. **Check antivirus**: Temporarily disable real-time protection

#### "ChromaDB connection failed" error

**Problem**: Document management features don't work.

**Solutions**:
1. **Check Python**: Ensure Python 3.8+ is installed
2. **Install ChromaDB**: Run `pip install chromadb`
3. **Start ChromaDB**: Run `chroma run --host localhost --port 8000`
4. **Check port**: Ensure port 8000 is not blocked by firewall

### Performance Optimization

#### Slow AI responses or timeouts

**Solutions**:
1. **Upgrade RAM**: 16GB+ recommended for large models
2. **Use smaller models**: Try `ollama pull llama3.2:1b` for faster responses
3. **Close other applications**: Free up system resources
4. **Check model loading**: Run `ollama list` to see loaded models

#### High CPU usage during development

**Solutions**:
1. **Disable hot reload**: Add `FAST_REFRESH=false` to your .env file  
2. **Limit TypeScript checking**: Set `"skipLibCheck": true` in tsconfig.json
3. **Use production build**: Run `npm run tauri build` for optimized performance

---

## 🎯 Quick Verification Checklist

After installation, verify everything works:

- [ ] **Node.js**: `node --version` shows 20.19+ or 22.12+
- [ ] **Rust**: `rustc --version` shows recent version
- [ ] **Ollama**: Visit http://localhost:11434 in browser (should show Ollama API)
- [ ] **Project**: `npm run dev` starts without errors
- [ ] **AI Chat**: Can send messages and receive responses
- [ ] **Search**: Web search returns results (if SearXNG installed)
- [ ] **Documents**: Can upload and search documents (if ChromaDB installed)

**Run automatic verification**: `node scripts/verify-setup.js`

---

## 📞 Getting Help

If you're still experiencing issues:

1. **Check logs**: Look in the terminal/console for specific error messages
2. **Run verification**: Use `node scripts/verify-setup.js` for detailed diagnosis
3. **Update everything**: Ensure all dependencies are at latest versions
4. **Community support**: Search for similar issues in the project repository
5. **Report bugs**: Create an issue with your system details and error logs

### System Information for Bug Reports

When reporting issues, include:
- Windows version: `winver`
- Node.js version: `node --version`
- npm version: `npm --version`
- Rust version: `rustc --version`
- Error messages and logs
- Steps to reproduce the issue
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

1. Right-click on `scripts/windows/generate-installer.bat` and select "Run as administrator"
2. Follow the on-screen instructions
3. The installer will be created in `.\installer\GerdsenAI_Socrates_Setup.msi`
4. A ZIP archive will also be created at `.\GerdsenAI_Socrates_Installer.zip`

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
https://gerdsenai.com/support
