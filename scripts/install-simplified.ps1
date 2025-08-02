# GerdsenAI Socrates - Simplified Windows Installer
# Copyright (c) 2025 GerdsenAI. All rights reserved.
# This script automates the complete installation process on Windows

param(
    [switch]$SkipPrerequisites,
    [switch]$SkipModels,
    [switch]$SkipDocker,
    [string]$InstallPath = "$env:LOCALAPPDATA\GerdsenAI\Socrates"
)

# Requires PowerShell 5.1 or later and Administrator privileges
#Requires -Version 5.1
#Requires -RunAsAdministrator

# Script configuration
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$ScriptDir = $PSScriptRoot
$ProjectRoot = Split-Path $ScriptDir -Parent
$LogFile = "$env:TEMP\gerdsenai-install.log"
$BackupDir = "$env:USERPROFILE\.gerdsenai-backup-$(Get-Date -Format 'yyyyMMdd_HHmmss')"

# Function definitions
function Write-Log {
    param([string]$Message, [string]$Level = "Info")
    
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logEntry = "[$timestamp] [$Level] $Message"
    
    switch ($Level) {
        "Success" { Write-Host "✅ $Message" -ForegroundColor Green }
        "Warning" { Write-Host "⚠️  $Message" -ForegroundColor Yellow }
        "Error" { Write-Host "❌ $Message" -ForegroundColor Red }
        default { Write-Host "🔍 $Message" -ForegroundColor Blue }
    }
    
    $logEntry | Out-File -FilePath $LogFile -Append -Encoding UTF8
}

function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Test-SystemRequirements {
    Write-Log "Checking system requirements..."
    
    # Check Windows version
    $osVersion = [System.Environment]::OSVersion.Version
    if ($osVersion.Major -lt 10) {
        Write-Log "Windows 10 or later is required. Current version: $($osVersion)" "Error"
        exit 1
    }
    Write-Log "Windows version check passed: $($osVersion)" "Success"
    
    # Check available memory
    $memory = Get-CimInstance -ClassName Win32_ComputerSystem
    $memoryGB = [math]::Round($memory.TotalPhysicalMemory / 1GB, 1)
    
    if ($memoryGB -lt 8) {
        Write-Log "System has ${memoryGB}GB RAM. 8GB+ recommended for optimal performance." "Warning"
    } else {
        Write-Log "Memory check passed: ${memoryGB}GB RAM available" "Success"
    }
    
    # Check available disk space
    $drive = Get-PSDrive -Name $env:SystemDrive.Replace(':', '')
    $freeSpaceGB = [math]::Round($drive.Free / 1GB, 1)
    
    if ($freeSpaceGB -lt 5) {
        Write-Log "Insufficient disk space. Need at least 5GB, have ${freeSpaceGB}GB" "Error"
        exit 1
    } else {
        Write-Log "Disk space check passed: ${freeSpaceGB}GB available" "Success"
    }
}

function Backup-ExistingInstallation {
    Write-Log "Checking for existing installation..."
    
    $configPaths = @(
        "$env:APPDATA\GerdsenAI",
        "$env:LOCALAPPDATA\GerdsenAI"
    )
    
    $backupCreated = $false
    foreach ($path in $configPaths) {
        if (Test-Path $path) {
            Write-Log "Backing up existing configuration: $path"
            if (-not $backupCreated) {
                New-Item -ItemType Directory -Path $BackupDir -Force | Out-Null
                $backupCreated = $true
            }
            Copy-Item -Path $path -Destination $BackupDir -Recurse -Force
        }
    }
    
    if ($backupCreated) {
        Write-Log "Backup created at: $BackupDir" "Success"
    }
}

function Install-Chocolatey {
    if (Get-Command choco -ErrorAction SilentlyContinue) {
        Write-Log "Chocolatey already installed" "Success"
        return
    }
    
    Write-Log "Installing Chocolatey package manager..."
    Set-ExecutionPolicy Bypass -Scope Process -Force
    [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
    Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
    
    # Refresh environment variables
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "User") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "Machine")
    
    if (Get-Command choco -ErrorAction SilentlyContinue) {
        Write-Log "Chocolatey installed successfully" "Success"
    } else {
        Write-Log "Chocolatey installation failed" "Error"
        exit 1
    }
}

function Install-NodeJS {
    Write-Log "Installing Node.js..."
    
    # Check if Node.js is already installed
    try {
        $nodeVersion = & node --version 2>$null
        if ($nodeVersion -match "v(\d+)\.") {
            $majorVersion = [int]$matches[1]
            if ($majorVersion -ge 18) {
                Write-Log "Node.js $nodeVersion already installed" "Success"
                return
            } else {
                Write-Log "Node.js $nodeVersion is too old. Need 18+. Installing newer version..." "Warning"
            }
        }
    } catch {
        Write-Log "Node.js not found, installing..."
    }
    
    # Install Node.js via Chocolatey
    choco install nodejs --version=20.11.0 -y
    
    # Refresh environment variables
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "User") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "Machine")
    
    # Verify installation
    try {
        $nodeVersion = & node --version
        $npmVersion = & npm --version
        Write-Log "Node.js $nodeVersion and npm $npmVersion installed" "Success"
    } catch {
        Write-Log "Node.js installation verification failed" "Error"
        exit 1
    }
}

function Install-Rust {
    Write-Log "Installing Rust..."
    
    # Check if Rust is already installed
    try {
        $rustVersion = & rustc --version 2>$null
        Write-Log "Rust already installed: $rustVersion" "Success"
        return
    } catch {
        Write-Log "Rust not found, installing..."
    }
    
    # Download and install Rust
    $rustupInit = "$env:TEMP\rustup-init.exe"
    Write-Log "Downloading Rust installer..."
    Invoke-WebRequest -Uri "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe" -OutFile $rustupInit
    
    Write-Log "Installing Rust (this may take several minutes)..."
    & $rustupInit -y --default-toolchain stable
    
    # Refresh environment variables
    $env:Path += ";$env:USERPROFILE\.cargo\bin"
    
    # Verify installation
    try {
        $rustVersion = & rustc --version
        Write-Log "Rust installed: $rustVersion" "Success"
    } catch {
        Write-Log "Rust installation verification failed" "Error"
        exit 1
    }
    
    # Clean up
    Remove-Item $rustupInit -Force -ErrorAction SilentlyContinue
}

function Install-Ollama {
    Write-Log "Installing Ollama..."
    
    # Check if Ollama is already installed
    try {
        $ollamaVersion = & ollama version 2>$null
        Write-Log "Ollama already installed: $ollamaVersion" "Success"
        return
    } catch {
        Write-Log "Ollama not found, installing..."
    }
    
    # Download and install Ollama
    $ollamaInstaller = "$env:TEMP\OllamaSetup.exe"
    Write-Log "Downloading Ollama installer..."
    Invoke-WebRequest -Uri "https://ollama.com/download/OllamaSetup.exe" -OutFile $ollamaInstaller
    
    Write-Log "Installing Ollama..."
    Start-Process -FilePath $ollamaInstaller -ArgumentList "/S" -Wait
    
    # Refresh environment variables
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "User") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "Machine")
    
    # Verify installation and start service
    try {
        Write-Log "Starting Ollama service..."
        Start-Process -FilePath "ollama" -ArgumentList "serve" -WindowStyle Hidden
        Start-Sleep -Seconds 5
        
        # Test connection
        $response = Invoke-RestMethod -Uri "http://localhost:11434/api/version" -ErrorAction SilentlyContinue
        if ($response) {
            Write-Log "Ollama service started and accessible" "Success"
        } else {
            Write-Log "Ollama installed but service may not be running correctly" "Warning"
        }
    } catch {
        Write-Log "Ollama installation verification failed" "Error"
        exit 1
    }
    
    # Clean up
    Remove-Item $ollamaInstaller -Force -ErrorAction SilentlyContinue
}

function Install-Docker {
    if ($SkipDocker) {
        Write-Log "Skipping Docker installation as requested"
        return
    }
    
    Write-Log "Installing Docker Desktop (optional for web search)..."
    
    # Check if Docker is already installed
    try {
        $dockerVersion = & docker --version 2>$null
        Write-Log "Docker already installed: $dockerVersion" "Success"
        return
    } catch {
        Write-Log "Docker not found, installing..."
    }
    
    # Install Docker Desktop via Chocolatey
    try {
        choco install docker-desktop -y
        Write-Log "Docker Desktop installed successfully" "Success"
        Write-Log "Note: Docker Desktop requires a restart to complete installation" "Warning"
    } catch {
        Write-Log "Docker installation failed - web search will be disabled" "Warning"
    }
}

function Install-Python {
    Write-Log "Installing Python..."
    
    # Check if Python is already installed
    try {
        $pythonVersion = & python --version 2>$null
        Write-Log "Python already installed: $pythonVersion" "Success"
    } catch {
        Write-Log "Installing Python via Chocolatey..."
        choco install python3 -y
        
        # Refresh environment variables
        $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "User") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "Machine")
    }
    
    # Install ChromaDB
    Write-Log "Installing ChromaDB..."
    try {
        & python -m pip install --user chromadb
        Write-Log "ChromaDB installed" "Success"
    } catch {
        Write-Log "ChromaDB installation failed - RAG features may be limited" "Warning"
    }
}

function Install-Application {
    Write-Log "Installing GerdsenAI Socrates..."
    
    Set-Location $ProjectRoot
    
    # Install npm dependencies
    Write-Log "Installing Node.js dependencies..."
    & npm install
    if ($LASTEXITCODE -ne 0) {
        Write-Log "Failed to install npm dependencies" "Error"
        exit 1
    }
    
    # Install Tauri CLI if needed
    try {
        & npx @tauri-apps/cli --version | Out-Null
    } catch {
        Write-Log "Installing Tauri CLI..."
        & npm install -g @tauri-apps/cli
    }
    
    # Build the application
    Write-Log "Building application..."
    & npm run build
    if ($LASTEXITCODE -ne 0) {
        Write-Log "Failed to build frontend" "Error"
        exit 1
    }
    
    # Build Tauri application
    Write-Log "Building Tauri application (this may take several minutes)..."
    & npm run tauri build
    if ($LASTEXITCODE -ne 0) {
        Write-Log "Failed to build Tauri application" "Error"
        exit 1
    }
    
    # Install the built application
    $msiPath = Get-ChildItem -Path "src-tauri\target\release\bundle\msi" -Filter "*.msi" | Select-Object -First 1
    if ($msiPath) {
        Write-Log "Installing application from $($msiPath.FullName)"
        Start-Process -FilePath "msiexec.exe" -ArgumentList "/i", "`"$($msiPath.FullName)`"", "/quiet" -Wait
        Write-Log "Application installed successfully" "Success"
    } else {
        Write-Log "MSI installer not found" "Error"
        exit 1
    }
}

function Install-Models {
    if ($SkipModels) {
        Write-Log "Skipping model installation as requested"
        return
    }
    
    Write-Log "Installing recommended AI models..."
    
    # Test Ollama connection
    try {
        $response = Invoke-RestMethod -Uri "http://localhost:11434/api/version" -ErrorAction Stop
        Write-Log "Ollama service is accessible" "Success"
    } catch {
        Write-Log "Ollama service not accessible - skipping model installation" "Warning"
        return
    }
    
    $models = @("qwen2.5-coder:7b", "codellama:7b")
    
    foreach ($model in $models) {
        Write-Log "Installing model: $model (this may take several minutes)..."
        try {
            & ollama pull $model
            if ($LASTEXITCODE -eq 0) {
                Write-Log "Model $model installed successfully" "Success"
            } else {
                Write-Log "Failed to install model: $model" "Warning"
            }
        } catch {
            Write-Log "Failed to install model: $model" "Warning"
        }
    }
    
    # Verify models
    Write-Log "Verifying installed models..."
    try {
        & ollama list
    } catch {
        Write-Log "Failed to list models" "Warning"
    }
}

function Setup-SearXNG {
    if ($SkipDocker) {
        Write-Log "Skipping SearXNG setup (Docker not installed)"
        return
    }
    
    Write-Log "Setting up SearXNG web search (optional)..."
    
    # Check if Docker is available
    try {
        & docker --version | Out-Null
    } catch {
        Write-Log "Docker not available - skipping SearXNG setup" "Warning"
        return
    }
    
    $searxngPath = Join-Path $ProjectRoot "docker\searxng"
    if (Test-Path $searxngPath) {
        Set-Location $searxngPath
        
        Write-Log "Starting SearXNG services..."
        if (Test-Path "start-searxng.bat") {
            & .\start-searxng.bat
            
            # Test SearXNG
            Start-Sleep -Seconds 15
            try {
                $response = Invoke-RestMethod -Uri "http://localhost:8080/search?q=test&format=json" -ErrorAction Stop
                Write-Log "SearXNG web search configured and running" "Success"
            } catch {
                Write-Log "SearXNG may not be running correctly" "Warning"
            }
        } else {
            Write-Log "SearXNG start script not found" "Warning"
        }
        
        Set-Location $ProjectRoot
    } else {
        Write-Log "SearXNG configuration not found" "Warning"
    }
}

function Set-FinalConfiguration {
    Write-Log "Performing final setup and testing..."
    
    # Create configuration directory
    $configDir = "$env:APPDATA\GerdsenAI\Socrates"
    New-Item -ItemType Directory -Path $configDir -Force | Out-Null
    
    # Create basic configuration
    $dockerAvailable = $false
    try {
        & docker --version | Out-Null
        $dockerAvailable = $true
    } catch {}
    
    $chromaAvailable = $false
    try {
        & python -c "import chromadb" 2>$null
        $chromaAvailable = $true
    } catch {}
    
    $config = @{
        version = "1.0.0"
        firstRun = $true
        theme = "system"
        services = @{
            ollama = @{
                enabled = $true
                url = "http://localhost:11434"
            }
            searxng = @{
                enabled = $dockerAvailable
                url = "http://localhost:8080"
            }
            chromadb = @{
                enabled = $chromaAvailable
            }
        }
    } | ConvertTo-Json -Depth 3
    
    $configFile = Join-Path $configDir "config.json"
    $config | Out-File -FilePath $configFile -Encoding UTF8
    
    Write-Log "Configuration created at: $configDir" "Success"
    
    # Test services
    Write-Log "Testing service connections..."
    
    # Test Ollama
    try {
        $response = Invoke-RestMethod -Uri "http://localhost:11434/api/version" -ErrorAction Stop
        Write-Log "✓ Ollama service is accessible" "Success"
    } catch {
        Write-Log "✗ Ollama service is not accessible" "Warning"
    }
    
    # Test SearXNG
    try {
        $response = Invoke-RestMethod -Uri "http://localhost:8080/search?q=test" -ErrorAction Stop
        Write-Log "✓ SearXNG web search is accessible" "Success"
    } catch {
        Write-Log "✗ SearXNG web search is not accessible (optional)" "Warning"
    }
    
    # Test ChromaDB
    try {
        & python -c "import chromadb" 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Log "✓ ChromaDB is available" "Success"
        } else {
            Write-Log "✗ ChromaDB is not available (optional)" "Warning"
        }
    } catch {
        Write-Log "✗ ChromaDB is not available (optional)" "Warning"
    }
}

function Show-Summary {
    Write-Host ""
    Write-Host "🎉 Installation completed successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "📋 Summary:" -ForegroundColor Cyan
    
    try { $nodeVersion = & node --version 2>$null } catch { $nodeVersion = "Not available" }
    try { $rustVersion = (& rustc --version 2>$null).Split(' ')[1] } catch { $rustVersion = "Not available" }
    try { $ollamaVersion = & ollama version 2>$null } catch { $ollamaVersion = "Not available" }
    try { $dockerVersion = (& docker --version 2>$null).Split(' ')[2].TrimEnd(',') } catch { $dockerVersion = "Not available" }
    
    Write-Host "  • Platform: Windows $(Get-CimInstance Win32_OperatingSystem | Select-Object -ExpandProperty Version)"
    Write-Host "  • Node.js: $nodeVersion"
    Write-Host "  • Rust: $rustVersion"
    Write-Host "  • Ollama: $ollamaVersion"
    Write-Host "  • Docker: $dockerVersion"
    Write-Host ""
    Write-Host "🚀 To start using GerdsenAI Socrates:" -ForegroundColor Cyan
    Write-Host "  • Look for 'GerdsenAI Socrates' in your Start Menu"
    Write-Host "  • Or search for 'GerdsenAI' in Windows Search"
    Write-Host ""
    Write-Host "📚 Documentation:" -ForegroundColor Cyan
    Write-Host "  • User Manual: COMPREHENSIVE_USER_MANUAL.md"
    Write-Host "  • Troubleshooting: TROUBLESHOOTING_GUIDE.md"
    Write-Host "  • Installation log: $LogFile"
    
    if (Test-Path $BackupDir) {
        Write-Host "  • Backup of previous installation: $BackupDir"
    }
    
    Write-Host ""
    Write-Host "🔧 If you encounter issues:" -ForegroundColor Cyan
    Write-Host "  • Check the troubleshooting guide"
    Write-Host "  • Review the installation log"
    Write-Host "  • Visit: https://github.com/GerdsenAI/GerdsenAI_AutoCoder/issues"
    Write-Host ""
}

# Main installation function
function Main {
    Write-Host "🚀 GerdsenAI Socrates - Simplified Windows Installer" -ForegroundColor Green
    Write-Host "==================================================" -ForegroundColor Green
    Write-Host ""
    
    Write-Log "Starting installation process..."
    Write-Log "Installation log: $LogFile"
    
    # Check prerequisites
    if (-not (Test-Administrator)) {
        Write-Log "This script must be run as Administrator" "Error"
        Write-Host "Please right-click on PowerShell and select 'Run as Administrator'" -ForegroundColor Red
        exit 1
    }
    
    # Run installation steps
    try {
        Test-SystemRequirements
        Backup-ExistingInstallation
        
        if (-not $SkipPrerequisites) {
            Install-Chocolatey
            Install-NodeJS
            Install-Rust
            Install-Ollama
            Install-Docker
            Install-Python
        }
        
        Install-Application
        
        if (-not $SkipModels) {
            Install-Models
        }
        
        Setup-SearXNG
        Set-FinalConfiguration
        
        Show-Summary
        
        Write-Log "Installation completed successfully!" "Success"
        
    } catch {
        Write-Log "Installation failed: $($_.Exception.Message)" "Error"
        Write-Host ""
        Write-Host "❌ Installation failed. Check the log file for details: $LogFile" -ForegroundColor Red
        exit 1
    }
}

# Run main function
Main