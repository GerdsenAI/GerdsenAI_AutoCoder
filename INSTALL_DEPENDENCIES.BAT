@echo off
REM CSE-Icon AutoCoder - Dependency Installation Script
REM Copyright (c) 2025 CSE-Icon Software. All rights reserved.

echo ===================================================
echo CSE-Icon AutoCoder - Dependency Installation Script
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

echo [INFO] Checking system requirements...
echo.

REM Check Windows version
ver | findstr /i "11\." >nul
if %errorLevel% neq 0 (
    echo [WARNING] This application is optimized for Windows 11.
    echo You may experience issues on older Windows versions.
    echo.
    timeout /t 5
)

REM Check CPU cores
for /f "tokens=2 delims==" %%i in ('wmic cpu get NumberOfCores /value') do set CPU_CORES=%%i
if %CPU_CORES% LSS 4 (
    echo [WARNING] Your system has less than the recommended 4 CPU cores (%CPU_CORES% detected).
    echo The application may run slower than expected.
    echo.
    set /p CONTINUE="Continue anyway? (y/n): "
    if /i not "%CONTINUE%"=="y" (
        echo [INFO] Installation aborted by user.
        exit /b 0
    )
) else (
    echo [SUCCESS] CPU check passed (%CPU_CORES% cores detected).
)

REM Check memory
for /f "tokens=2 delims==" %%i in ('wmic ComputerSystem get TotalPhysicalMemory /value') do set TOTAL_MEM_B=%%i
set /a TOTAL_MEM_GB=%TOTAL_MEM_B:~0,-9%/1024

if %TOTAL_MEM_GB% LSS 8 (
    echo [WARNING] Your system has less than the recommended 8GB of RAM (%TOTAL_MEM_GB% GB detected).
    echo The application may run slower than expected or crash with large models.
    echo.
    set /p CONTINUE="Continue anyway? (y/n): "
    if /i not "%CONTINUE%"=="y" (
        echo [INFO] Installation aborted by user.
        exit /b 0
    )
) else (
    echo [SUCCESS] Memory check passed (%TOTAL_MEM_GB% GB detected).
)

REM Check disk space
for /f "tokens=3" %%i in ('dir /-c 2^>nul ^| findstr "bytes free"') do set FREE_SPACE=%%i
set FREE_SPACE=%FREE_SPACE:,=%
set /a FREE_SPACE_GB=%FREE_SPACE:~0,-9%/1024

if %FREE_SPACE_GB% LSS 2 (
    echo [ERROR] You have less than the required 2GB of free disk space (%FREE_SPACE_GB% GB available).
    echo Please free up some disk space and try again.
    echo.
    pause
    exit /b 1
) else (
    echo [SUCCESS] Disk space check passed (%FREE_SPACE_GB% GB available).
)

echo [SUCCESS] System requirements check completed.
echo.

REM Check for Node.js
where node >nul 2>&1
if %errorLevel% neq 0 (
    echo [INFO] Node.js not found. Installing Node.js...
    echo.
    
    REM Download and install Node.js
    echo [INFO] Downloading Node.js installer...
    powershell -Command "& {Invoke-WebRequest -Uri 'https://nodejs.org/dist/v20.18.0/node-v20.18.0-x64.msi' -OutFile 'node-installer.msi'}"
    
    if %errorLevel% neq 0 (
        echo [ERROR] Failed to download Node.js installer.
        echo Please check your internet connection and try again.
        echo.
        pause
        exit /b 1
    )
    
    echo [INFO] Installing Node.js...
    start /wait msiexec /i node-installer.msi /quiet /norestart
    
    if %errorLevel% neq 0 (
        echo [ERROR] Failed to install Node.js.
        echo Please install it manually from https://nodejs.org/
        echo.
        pause
        exit /b 1
    )
    
    del node-installer.msi
    
    echo [SUCCESS] Node.js installed successfully.
    echo.
    
    REM Refresh environment variables
    echo [INFO] Refreshing environment variables...
    call refreshenv.cmd >nul 2>&1 || (
        echo [WARNING] Could not refresh environment variables automatically.
        echo Please close and reopen this command prompt after installation.
        echo.
    )
) else (
    for /f "tokens=*" %%i in ('node --version') do set NODE_VERSION=%%i
    echo [SUCCESS] Node.js is already installed (version %NODE_VERSION%).
    echo.
    
    REM Check if version is at least 20.x
    set NODE_MAJOR=%NODE_VERSION:~1,2%
    if %NODE_MAJOR% LSS 20 (
        echo [WARNING] Node.js version 20.x or higher is recommended. You have %NODE_VERSION%.
        set /p UPGRADE="Would you like to upgrade Node.js? (y/n): "
        if /i "%UPGRADE%"=="y" (
            echo [INFO] Downloading Node.js installer...
            powershell -Command "& {Invoke-WebRequest -Uri 'https://nodejs.org/dist/v20.18.0/node-v20.18.0-x64.msi' -OutFile 'node-installer.msi'}"
            
            if %errorLevel% neq 0 (
                echo [ERROR] Failed to download Node.js installer.
                echo Please check your internet connection and try again.
                echo.
                pause
                exit /b 1
            )
            
            echo [INFO] Installing Node.js...
            start /wait msiexec /i node-installer.msi /quiet /norestart
            
            if %errorLevel% neq 0 (
                echo [ERROR] Failed to install Node.js.
                echo Please install it manually from https://nodejs.org/
                echo.
                pause
                exit /b 1
            )
            
            del node-installer.msi
            
            echo [SUCCESS] Node.js upgraded successfully.
            echo.
            
            REM Refresh environment variables
            echo [INFO] Refreshing environment variables...
            call refreshenv.cmd >nul 2>&1 || (
                echo [WARNING] Could not refresh environment variables automatically.
                echo Please close and reopen this command prompt after installation.
                echo.
            )
        )
    )
)

REM Check for Rust
rustc --version >nul 2>&1
if %errorLevel% neq 0 (
    echo [INFO] Rust not found. Installing Rust...
    echo.
    
    REM Download and install Rust
    echo [INFO] Downloading Rust installer...
    powershell -Command "& {Invoke-WebRequest -Uri 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe' -OutFile 'rustup-init.exe'}"
    
    if %errorLevel% neq 0 (
        echo [ERROR] Failed to download Rust installer.
        echo Please check your internet connection and try again.
        echo.
        pause
        exit /b 1
    )
    
    echo [INFO] Installing Rust...
    start /wait rustup-init.exe -y --default-toolchain stable --profile minimal
    
    if %errorLevel% neq 0 (
        echo [ERROR] Failed to install Rust.
        echo Please install it manually from https://www.rust-lang.org/tools/install
        echo.
        pause
        exit /b 1
    )
    
    del rustup-init.exe
    
    echo [SUCCESS] Rust installed successfully.
    echo.
    
    REM Refresh environment variables
    echo [INFO] Refreshing environment variables...
    call refreshenv.cmd >nul 2>&1 || (
        echo [WARNING] Could not refresh environment variables automatically.
        echo Please close and reopen this command prompt after installation.
        echo.
    )
) else (
    for /f "tokens=2" %%i in ('rustc --version') do set RUST_VERSION=%%i
    echo [SUCCESS] Rust is already installed (version %RUST_VERSION%).
    echo.
)

REM Check for Visual Studio Build Tools
where cl.exe >nul 2>&1
if %errorLevel% neq 0 (
    echo [INFO] Visual Studio Build Tools not found. Installing...
    echo.
    
    REM Download and install Visual Studio Build Tools
    echo [INFO] Downloading Visual Studio Build Tools installer...
    powershell -Command "& {Invoke-WebRequest -Uri 'https://aka.ms/vs/17/release/vs_buildtools.exe' -OutFile 'vs_buildtools.exe'}"
    
    if %errorLevel% neq 0 (
        echo [ERROR] Failed to download Visual Studio Build Tools installer.
        echo Please check your internet connection and try again.
        echo.
        pause
        exit /b 1
    )
    
    echo [INFO] Installing Visual Studio Build Tools...
    start /wait vs_buildtools.exe --quiet --wait --norestart --nocache ^
        --installPath "%ProgramFiles(x86)%\Microsoft Visual Studio\2022\BuildTools" ^
        --add Microsoft.VisualStudio.Workload.VCTools ^
        --includeRecommended
    
    if %errorLevel% neq 0 (
        echo [ERROR] Failed to install Visual Studio Build Tools.
        echo Please install it manually from https://visualstudio.microsoft.com/visual-cpp-build-tools/
        echo.
        pause
        exit /b 1
    )
    
    del vs_buildtools.exe
    
    echo [SUCCESS] Visual Studio Build Tools installed successfully.
    echo.
    
    REM Refresh environment variables
    echo [INFO] Refreshing environment variables...
    call refreshenv.cmd >nul 2>&1 || (
        echo [WARNING] Could not refresh environment variables automatically.
        echo Please close and reopen this command prompt after installation.
        echo.
    )
) else (
    echo [SUCCESS] Visual Studio Build Tools are already installed.
    echo.
)

REM Check for WebView2
reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" >nul 2>&1
if %errorLevel% neq 0 (
    echo [INFO] WebView2 Runtime not found. Installing...
    echo.
    
    REM Download and install WebView2
    echo [INFO] Downloading WebView2 installer...
    powershell -Command "& {Invoke-WebRequest -Uri 'https://go.microsoft.com/fwlink/p/?LinkId=2124703' -OutFile 'MicrosoftEdgeWebview2Setup.exe'}"
    
    if %errorLevel% neq 0 (
        echo [ERROR] Failed to download WebView2 installer.
        echo Please check your internet connection and try again.
        echo.
        pause
        exit /b 1
    )
    
    echo [INFO] Installing WebView2...
    start /wait MicrosoftEdgeWebview2Setup.exe /silent /install
    
    if %errorLevel% neq 0 (
        echo [ERROR] Failed to install WebView2.
        echo Please install it manually from https://developer.microsoft.com/en-us/microsoft-edge/webview2/
        echo.
        pause
        exit /b 1
    )
    
    del MicrosoftEdgeWebview2Setup.exe
    
    echo [SUCCESS] WebView2 installed successfully.
    echo.
) else (
    echo [SUCCESS] WebView2 Runtime is already installed.
    echo.
)

REM Check for Ollama
where ollama >nul 2>&1
if %errorLevel% neq 0 (
    echo [WARNING] Ollama not found. The application requires Ollama to function properly.
    echo Please install Ollama from: https://ollama.ai/download
    echo.
    
    set /p OPEN_BROWSER="Would you like to open the Ollama download page? (y/n): "
    if /i "%OPEN_BROWSER%"=="y" (
        start https://ollama.ai/download
    )
    
    echo [INFO] After installing Ollama, please run this script again.
    echo.
    pause
    exit /b 1
) else (
    echo [SUCCESS] Ollama is already installed.
    echo.
)

REM Install npm dependencies
echo [INFO] Installing npm dependencies...
echo.

REM Check if package.json exists
if not exist "package.json" (
    echo [ERROR] package.json not found. Make sure you're running this script from the project root directory.
    echo.
    pause
    exit /b 1
)

REM Install dependencies
call npm install

if %errorLevel% neq 0 (
    echo [ERROR] Failed to install npm dependencies.
    echo Please check your internet connection and try again.
    echo.
    pause
    exit /b 1
)

echo [SUCCESS] npm dependencies installed successfully.
echo.

REM Install Tauri CLI
echo [INFO] Installing Tauri CLI...
echo.

call cargo install tauri-cli

if %errorLevel% neq 0 (
    echo [ERROR] Failed to install Tauri CLI.
    echo Please check your Rust installation and try again.
    echo.
    pause
    exit /b 1
)

echo [SUCCESS] Tauri CLI installed successfully.
echo.

echo ===================================================
echo [SUCCESS] All dependencies installed successfully!
echo [INFO] You can now build the application using: npm run tauri build
echo ===================================================
echo.

pause
