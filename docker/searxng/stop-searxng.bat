@echo off
REM Stop SearXNG Docker containers (Windows)
REM This script provides a convenient way to stop the SearXNG service

cd /d "%~dp0"

echo Stopping SearXNG development environment...

REM Stop services
docker-compose down
if %errorlevel% neq 0 (
    echo Error: Failed to stop services.
    pause
    exit /b 1
)

echo ✓ SearXNG services stopped successfully!

REM Optional: Remove volumes (uncomment if you want to reset data)
REM echo Removing persistent data volumes...
REM docker-compose down -v
REM echo ✓ Volumes removed!

pause