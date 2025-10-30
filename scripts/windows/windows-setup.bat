@echo off
echo Script execution started. Press any key to see the title...
pause
REM GerdsenAI Socrates Windows 11 Setup Script

REM Copyright (c) 2025 GerdsenAI. All rights reserved.

echo ===================================================
echo GerdsenAI Socrates - Windows 11 Setup Script
echo ===================================================
echo.

REM Check for administrative privileges
net session >nul 2>&1
set ADMIN_CHECK_ERR_LEVEL=%errorLevel%
echo DEBUG: Admin check errorlevel from 'net session' is %ADMIN_CHECK_ERR_LEVEL%
if %ADMIN_CHECK_ERR_LEVEL% neq 0 (
    echo INFO: Entering admin rights error block.
    echo Error: Administrative privileges are required. (Code: %ADMIN_CHECK_ERR_LEVEL%)
    echo Please right-click this .bat file and select "Run as administrator".
    echo.
    pause
    exit /b 1
)
echo INFO: Administrative privileges check passed. Press any key to continue...
pause

echo Checking system requirements...
echo.

REM Check Windows version
ver | findstr /i "11\." >nul
if %errorLevel% neq 0 (
    echo Warning: This application is optimized for Windows 11.
    echo You may experience issues on older Windows versions.
    echo.
    timeout /t 5
)

REM Check for Node.js
where node >nul 2>&1
if %errorLevel% neq 0 (
    echo Node.js not found. Installing Node.js...
    echo.
    
    REM Download and install Node.js
    powershell -Command "& {Invoke-WebRequest -Uri 'https://nodejs.org/dist/v20.18.0/node-v20.18.0-x64.msi' -OutFile 'node-installer.msi'}"
    if %errorLevel% neq 0 (
        echo Error: Failed to download Node.js installer.
        if exist node-installer.msi del node-installer.msi
        pause
        exit /b 1
    )
    start /wait msiexec /i node-installer.msi /quiet /norestart
    if %errorLevel% neq 0 (
        echo Error: Node.js installation failed.
        if exist node-installer.msi del node-installer.msi
        pause
        exit /b 1
    )
    del node-installer.msi
    echo Node.js v20.18.0 installed successfully.
    echo.
) else (
    echo Node.js is already installed.
    echo.
)

REM Check for Rust
rustc --version >nul 2>&1
if %errorLevel% neq 0 (
    echo Rust not found. Installing Rust...
    echo.
    
    REM Download and install Rust
    powershell -Command "& {Invoke-WebRequest -Uri 'https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe' -OutFile 'rustup-init.exe'}"
    if %errorLevel% neq 0 (
        echo Error: Failed to download Rust installer.
        if exist rustup-init.exe del rustup-init.exe
        pause
        exit /b 1
    )
    start /wait rustup-init.exe -y --default-toolchain stable --profile minimal
    if %errorLevel% neq 0 (
        echo Error: Rust installation failed.
        if exist rustup-init.exe del rustup-init.exe
        pause
        exit /b 1
    )
    del rustup-init.exe
    echo Rust (stable toolchain, minimal profile) installed successfully.
    echo.
) else (
    echo Rust is already installed.
    echo.
)

REM Check for Ollama
where ollama >nul 2>&1
if %errorLevel% neq 0 (
    echo Ollama not found. Please install Ollama from https://ollama.ai/download
    echo After installing Ollama, please run this setup script again.
    echo.
    start https://ollama.ai/download
    pause
    exit /b 1
) else (
    echo Ollama is already installed.
    echo.
)

REM Navigate to the application directory for npm commands
pushd "%~dp0..\GerdsenAI_Socrates"
if %errorLevel% neq 0 (
    echo Error: Could not navigate to the application directory.
    echo Please ensure the script is in the correct location relative to the AutoCoder folder.
    echo.
    pause
    exit /b 1
)

echo.
echo Correcting Vite version in package.json if necessary...
powershell -Command "& {
    $packageJsonPath = 'package.json';
    if (Test-Path $packageJsonPath) {
        $content = Get-Content $packageJsonPath -Raw;
        if ($content -match '\""vite\"": \""\^?6\.') {
            echo 'Incorrect Vite version found in package.json. Updating to ^5.3.5...';
            $updatedContent = $content -replace '\""vite\"": \""\^?6\.\d+\.\d+\""', '\""vite\"": \""^5.3.5\""';
            Set-Content -Path $packageJsonPath -Value $updatedContent -Encoding UTF8;
            if ($LASTEXITCODE -ne 0) {
                Write-Error 'Error: Failed to update Vite version in package.json.';
                exit 1;
            }
            echo 'Vite version updated in package.json.';
        } else {
            echo 'Vite version in package.json appears correct or does not match ^6.x.x pattern.';
        }
    } else {
        Write-Error ('Error: package.json not found in ' + (Get-Location).Path);
        exit 1;
    }
}"
if %errorLevel% neq 0 (
    echo Error: PowerShell script to update package.json failed. See PowerShell errors above.
    popd
    pause
    exit /b 1
)
echo.

REM Clean install dependencies
echo Cleaning previous installation...
if exist node_modules (
    rd /s /q node_modules
)
if exist package-lock.json (
    del package-lock.json
)

echo Installing dependencies...
call npm install
if %errorLevel% neq 0 (
    echo Error: Failed to install dependencies.
    echo Please check your internet connection and try again.
    echo.
    popd
    pause
    exit /b 1
)
echo Dependencies installed successfully.
echo.

REM Audit and fix dependencies
echo Auditing dependencies...
call npm audit fix
if %errorLevel% neq 0 (
    echo Warning: Some vulnerabilities could not be automatically fixed.
    echo Continuing with build...
    echo.
)

REM Build the application
echo Building GerdsenAI Socrates...
call npm run tauri build
if %errorLevel% neq 0 (
    echo Error: Failed to build the application.
    echo.
    popd
    pause
    exit /b 1
)
echo Application built successfully.
echo.

REM Return to the original directory
popd

REM Create shortcuts
echo Creating shortcuts...
powershell -Command "& {$WshShell = New-Object -ComObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut([Environment]::GetFolderPath('Desktop') + '\GerdsenAI Socrates.lnk'); $Shortcut.TargetPath = '%~dp0..\GerdsenAI_Socrates\src-tauri\target\release\GerdsenAI_Socrates.exe'; $Shortcut.Save()}"
powershell -Command "& {$WshShell = New-Object -ComObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut([Environment]::GetFolderPath('StartMenu') + '\Programs\GerdsenAI Socrates.lnk'); $Shortcut.TargetPath = '%~dp0..\GerdsenAI_Socrates\src-tauri\target\release\GerdsenAI_Socrates.exe'; $Shortcut.Save()}"
echo Shortcuts created successfully.
echo.

REM Configure IDE integration
echo Configuring IDE integration...

REM Check for VS Code
where code >nul 2>&1
if %errorLevel% equ 0 (
    echo Visual Studio Code detected. Configuring integration...
    
    REM Create VS Code extension directory if it doesn't exist
    if not exist "%USERPROFILE%\.vscode\extensions\gerdsenai-socrates" (
        mkdir "%USERPROFILE%\.vscode\extensions\gerdsenai-socrates"
    )
    
    REM Copy necessary files for VS Code integration
    if exist "%~dp0vscode-integration\*" (
        xcopy /E /I /Y "%~dp0vscode-integration\*" "%USERPROFILE%\.vscode\extensions\gerdsenai-socrates"
        echo VS Code integration configured successfully.
    ) else (
        echo VS Code integration files not found. Skipping...
    )
    echo.
)

REM Check for Visual Studio
reg query "HKLM\SOFTWARE\Microsoft\VisualStudio\17.0" >nul 2>&1
if %errorLevel% equ 0 (
    echo Visual Studio detected. Configuring integration...
    
    REM Create Visual Studio extension directory if it doesn't exist
    if not exist "%USERPROFILE%\Documents\Visual Studio 2022\Extensions\GerdsenAI Socrates" (
        mkdir "%USERPROFILE%\Documents\Visual Studio 2022\Extensions\GerdsenAI Socrates"
    )
    
    REM Copy necessary files for Visual Studio integration
    if exist "%~dp0vs-integration\*" (
        xcopy /E /I /Y "%~dp0vs-integration\*" "%USERPROFILE%\Documents\Visual Studio 2022\Extensions\GerdsenAI Socrates"
        echo Visual Studio integration configured successfully.
    ) else (
        echo Visual Studio integration files not found. Skipping...
    )
    echo.
)

echo ===================================================
echo GerdsenAI Socrates setup completed successfully!
echo.
echo You can now launch the application from:
echo  - Desktop shortcut
echo  - Start menu
echo  - Visual Studio Code (if installed)
echo  - Visual Studio (if installed)
echo ===================================================
echo.

pause
