@echo off
REM GerdsenAI Socrates - Application Startup Script
REM Copyright (c) 2025 GerdsenAI. All rights reserved.

echo ===================================================
echo GerdsenAI Socrates - Application Startup Script
echo ===================================================
echo.

REM Check if Ollama is running
echo [INFO] Checking if Ollama is running...

where ollama >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] Ollama is not installed. Please install Ollama first.
    echo [INFO] Visit: https://ollama.ai/download
    echo.
    
    set /p OPEN_BROWSER="Would you like to open the Ollama download page? (y/n): "
    if /i "%OPEN_BROWSER%"=="y" (
        start https://ollama.ai/download
    )
    
    echo [INFO] After installing Ollama, please run this script again.
    echo.
    pause
    exit /b 1
)

REM Try to connect to Ollama API
powershell -Command "& {try { $response = Invoke-WebRequest -Uri 'http://localhost:11434/api/version' -TimeoutSec 5 -ErrorAction Stop; exit 0 } catch { exit 1 }}"

if %errorLevel% neq 0 (
    echo [WARNING] Ollama is not running.
    echo [INFO] Starting Ollama...
    
    start "" ollama serve
    
    REM Wait for Ollama to start
    echo [INFO] Waiting for Ollama to start...
    
    set /a ATTEMPTS=0
    :WAIT_LOOP
    if %ATTEMPTS% geq 30 (
        echo [ERROR] Failed to start Ollama. Please start it manually.
        echo.
        set /p CONTINUE="Continue without Ollama? (y/n): "
        if /i not "%CONTINUE%"=="y" (
            echo [INFO] Startup aborted.
            pause
            exit /b 1
        )
        goto :CONTINUE_STARTUP
    )
    
    timeout /t 1 /nobreak >nul
    
    powershell -Command "& {try { $response = Invoke-WebRequest -Uri 'http://localhost:11434/api/version' -TimeoutSec 5 -ErrorAction Stop; exit 0 } catch { exit 1 }}"
    
    if %errorLevel% equ 0 (
        echo [SUCCESS] Ollama started successfully.
        goto :CONTINUE_STARTUP
    )
    
    set /a ATTEMPTS+=1
    goto :WAIT_LOOP
) else (
    echo [SUCCESS] Ollama is running.
)

:CONTINUE_STARTUP

REM Check if development or production mode
if exist ".\target\release\GerdsenAI_Socrates.exe" (
    set MODE=production
    goto :PRODUCTION_MODE
) else if exist ".\target\release\bundle" (
    set MODE=production
    goto :PRODUCTION_MODE
) else (
    set MODE=development
    goto :DEVELOPMENT_MODE
)

:DEVELOPMENT_MODE
echo [INFO] Starting GerdsenAI Socrates in development mode...
echo.

REM Check if package.json exists
if not exist "package.json" (
    echo [ERROR] package.json not found. Make sure you're running this script from the project root directory.
    echo.
    pause
    exit /b 1
)

REM Check if node_modules exists
if not exist "node_modules" (
    echo [WARNING] node_modules not found. Installing dependencies...
    call npm install
    
    if %errorLevel% neq 0 (
        echo [ERROR] Failed to install dependencies.
        echo.
        pause
        exit /b 1
    )
)

REM Start the application
call npm run tauri dev

if %errorLevel% neq 0 (
    echo [ERROR] Failed to start the application in development mode.
    echo.
    pause
    exit /b 1
)

goto :END

:PRODUCTION_MODE
echo [INFO] Starting GerdsenAI Socrates in production mode...
echo.

REM Check if the built application exists
if exist ".\target\release\GerdsenAI_Socrates.exe" (
    start "" ".\target\release\GerdsenAI_Socrates.exe"
) else if exist ".\target\release\bundle\msi\GerdsenAI_Socrates_*.msi" (
    echo [INFO] Found MSI installer. Please install the application first:
    for %%f in (".\target\release\bundle\msi\GerdsenAI_Socrates_*.msi") do (
        echo [INFO] %%f
    )
    echo.
    
    set /p INSTALL="Would you like to run the installer now? (y/n): "
    if /i "%INSTALL%"=="y" (
        for %%f in (".\target\release\bundle\msi\GerdsenAI_Socrates_*.msi") do (
            start "" "%%f"
        )
    )
    
    pause
    exit /b 0
) else (
    echo [ERROR] Application not built. Please build it first:
    echo [INFO] npm run tauri build
    echo.
    
    set /p BUILD="Would you like to build the application now? (y/n): "
    if /i "%BUILD%"=="y" (
        call npm run tauri build
        
        if %errorLevel% neq 0 (
            echo [ERROR] Failed to build the application.
            echo.
            pause
            exit /b 1
        )
        
        echo [SUCCESS] Application built successfully.
        echo [INFO] Please run this script again to start the application.
    )
    
    pause
    exit /b 0
)

:END
echo.
echo [INFO] GerdsenAI Socrates startup completed.
echo.
pause
