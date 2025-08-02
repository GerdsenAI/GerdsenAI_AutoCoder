@echo off
REM GerdsenAI Socrates - Installer Generation Script
REM Copyright (c) 2025 GerdsenAI. All rights reserved.

echo ===================================================
echo GerdsenAI Socrates - Installer Generation Script
echo ===================================================
echo.

REM Check for administrative privileges
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] This script requires administrative privileges.
    echo Please right-click on this file and select "Run as administrator".
    echo.
    pause
    exit /b 1
)

REM Check for required tools
echo [INFO] Checking for required tools...

REM Check for Node.js
where node >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] Node.js not found. Please run INSTALL_DEPENDENCIES.BAT first.
    echo.
    pause
    exit /b 1
)

REM Check for Rust
where rustc >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] Rust not found. Please run INSTALL_DEPENDENCIES.BAT first.
    echo.
    pause
    exit /b 1
)

REM Check for Tauri CLI
where cargo >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] Cargo not found. Please run INSTALL_DEPENDENCIES.BAT first.
    echo.
    pause
    exit /b 1
)

REM Check if package.json exists
if not exist "package.json" (
    echo [ERROR] package.json not found. Make sure you're running this script from the project root directory.
    echo.
    pause
    exit /b 1
)

echo [SUCCESS] All required tools found.
echo.

REM Clean previous builds
echo [INFO] Cleaning previous builds...

if exist ".\target" (
    echo [INFO] Removing old build artifacts...
    rmdir /s /q ".\target"
    
    if %errorLevel% neq 0 (
        echo [WARNING] Failed to remove old build artifacts. Continuing anyway...
    ) else (
        echo [SUCCESS] Old build artifacts removed.
    )
)

if exist ".\dist" (
    echo [INFO] Removing old distribution files...
    rmdir /s /q ".\dist"
    
    if %errorLevel% neq 0 (
        echo [WARNING] Failed to remove old distribution files. Continuing anyway...
    ) else (
        echo [SUCCESS] Old distribution files removed.
    )
)

echo.

REM Build the application
echo [INFO] Building the application...
echo [INFO] This may take several minutes. Please be patient.
echo.

call npm run tauri build

if %errorLevel% neq 0 (
    echo [ERROR] Failed to build the application.
    echo Please check the error messages above and fix any issues.
    echo.
    pause
    exit /b 1
)

echo [SUCCESS] Application built successfully.
echo.

REM Check if the installer was created
if not exist ".\target\release\bundle\msi\GerdsenAI_Socrates_*.msi" (
    echo [ERROR] Installer not found. Build process may have failed.
    echo.
    pause
    exit /b 1
)

REM Create output directory
if not exist ".\installer" (
    mkdir ".\installer"
    
    if %errorLevel% neq 0 (
        echo [ERROR] Failed to create installer directory.
        echo.
        pause
        exit /b 1
    )
)

REM Copy installer to output directory
echo [INFO] Copying installer to output directory...

for %%f in (".\target\release\bundle\msi\GerdsenAI_Socrates_*.msi") do (
    copy "%%f" ".\installer\GerdsenAI_Socrates_Setup.msi"
    
    if %errorLevel% neq 0 (
        echo [ERROR] Failed to copy installer.
        echo.
        pause
        exit /b 1
    )
    
    echo [SUCCESS] Installer copied to .\installer\GerdsenAI_Socrates_Setup.msi
)

REM Create README file in installer directory
echo [INFO] Creating README file...

echo GerdsenAI Socrates Installer> ".\installer\README.txt"
echo ===========================>> ".\installer\README.txt"
echo.>> ".\installer\README.txt"
echo Installation Instructions:>> ".\installer\README.txt"
echo 1. Right-click on GerdsenAI_Socrates_Setup.msi and select "Install">> ".\installer\README.txt"
echo 2. Follow the on-screen instructions to complete the installation>> ".\installer\README.txt"
echo 3. Launch GerdsenAI Socrates from the Start Menu>> ".\installer\README.txt"
echo.>> ".\installer\README.txt"
echo Requirements:>> ".\installer\README.txt"
echo - Windows 11 (recommended) or Windows 10>> ".\installer\README.txt"
echo - 8GB RAM minimum>> ".\installer\README.txt"
echo - 2GB free disk space>> ".\installer\README.txt"
echo - Ollama installed (https://ollama.ai/download)>> ".\installer\README.txt"
echo.>> ".\installer\README.txt"
echo For support, please visit: https://gerdsenai.com/support>> ".\installer\README.txt"

echo [SUCCESS] README file created.
echo.

REM Create ZIP archive
echo [INFO] Creating ZIP archive of installer...

where powershell >nul 2>&1
if %errorLevel% neq 0 (
    echo [WARNING] PowerShell not found. Skipping ZIP archive creation.
) else (
    powershell -Command "& {Compress-Archive -Path '.\installer\*' -DestinationPath '.\GerdsenAI_Socrates_Installer.zip' -Force}"
    
    if %errorLevel% neq 0 (
        echo [WARNING] Failed to create ZIP archive. You can manually zip the installer directory.
    ) else (
        echo [SUCCESS] ZIP archive created: GerdsenAI_Socrates_Installer.zip
    )
)

echo.
echo ===================================================
echo [SUCCESS] Installer generation completed successfully!
echo.
echo Installer location: .\installer\GerdsenAI_Socrates_Setup.msi
echo ZIP archive: .\GerdsenAI_Socrates_Installer.zip
echo.
echo You can distribute this installer to users for easy installation.
echo ===================================================
echo.

pause
