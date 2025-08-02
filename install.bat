@echo off
REM GerdsenAI Socrates - Windows Installer Wrapper
REM Copyright (c) 2025 GerdsenAI. All rights reserved.

setlocal enabledelayedexpansion

echo.
echo ╔══════════════════════════════════════════════════════════════╗
echo ║                                                              ║
echo ║                   🚀 GerdsenAI Socrates                     ║
echo ║                 Advanced AI Coding Assistant                ║
echo ║                                                              ║
echo ║                    Windows Installer                        ║
echo ║                                                              ║
echo ╚══════════════════════════════════════════════════════════════╝
echo.

REM Check for administrative privileges
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] This installer requires administrative privileges.
    echo.
    echo Please right-click on this file and select "Run as administrator"
    echo or run PowerShell as Administrator and execute:
    echo.
    echo   PowerShell -ExecutionPolicy Bypass -File scripts\install-simplified.ps1
    echo.
    pause
    exit /b 1
)

REM Check for PowerShell
powershell -Command "Get-Host" >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] PowerShell is required but not available.
    echo Please install PowerShell 5.1 or later.
    echo.
    pause
    exit /b 1
)

echo [INFO] Checking PowerShell execution policy...
for /f "tokens=*" %%i in ('powershell -Command "Get-ExecutionPolicy"') do set EXEC_POLICY=%%i

if "%EXEC_POLICY%"=="Restricted" (
    echo [WARNING] PowerShell execution policy is Restricted.
    echo This installer needs to temporarily change the execution policy.
    echo.
    set /p CONFIRM="Allow execution policy change? (Y/N): "
    if /i "!CONFIRM!" neq "Y" (
        echo Installation cancelled by user.
        pause
        exit /b 1
    )
)

echo.
echo [INFO] Starting PowerShell installer...
echo.

REM Run the PowerShell installer
powershell -ExecutionPolicy Bypass -File "%~dp0scripts\install-simplified.ps1"

if %errorLevel% neq 0 (
    echo.
    echo [ERROR] Installation failed. Check the log file for details.
    echo Log location: %TEMP%\gerdsenai-install.log
    echo.
    echo For help:
    echo   • Check TROUBLESHOOTING_GUIDE.md
    echo   • Visit: https://github.com/GerdsenAI/GerdsenAI_AutoCoder/issues
    echo.
    pause
    exit /b 1
)

echo.
echo ╔══════════════════════════════════════════════════════════════╗
echo ║                                                              ║
echo ║                    🎉 Installation Complete!                ║
echo ║                                                              ║
echo ╚══════════════════════════════════════════════════════════════╝
echo.
echo 🚀 To start using GerdsenAI Socrates:
echo   • Look for 'GerdsenAI Socrates' in your Start Menu
echo   • Or search for 'GerdsenAI' in Windows Search
echo.
echo 📚 Documentation:
echo   • User Manual: COMPREHENSIVE_USER_MANUAL.md
echo   • Troubleshooting: TROUBLESHOOTING_GUIDE.md
echo.
echo Happy coding with GerdsenAI Socrates! 🚀
echo.
pause