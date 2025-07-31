@echo off
REM Start SearXNG Docker containers for development (Windows)
REM This script provides a convenient way to start the SearXNG service

setlocal enabledelayedexpansion

cd /d "%~dp0"

echo Starting SearXNG development environment...

REM Check if Docker is running
docker info >nul 2>&1
if !errorlevel! neq 0 (
    echo Error: Docker is not running. Please start Docker first.
    pause
    exit /b 1
)

REM Pull latest images
echo Pulling latest SearXNG and Redis images...
docker-compose pull
if !errorlevel! neq 0 (
    echo Error: Failed to pull Docker images.
    pause
    exit /b 1
)

REM Start services
echo Starting SearXNG and Redis services...
docker-compose up -d
if !errorlevel! neq 0 (
    echo Error: Failed to start services.
    pause
    exit /b 1
)

REM Wait for services to be healthy
echo Waiting for services to become healthy...
set timeout=60
set elapsed=0

:check_health
if !elapsed! geq !timeout! goto timeout_reached

REM Check if SearXNG is healthy
for /f "delims=" %%i in ('docker-compose ps -q searxng') do (
    for /f "delims=" %%j in ('docker inspect --format="{{.State.Health.Status}}" %%i 2^>nul') do (
        if "%%j"=="healthy" (
            echo ✓ SearXNG is healthy and ready!
            echo ✓ SearXNG is available at: http://localhost:8080
            echo ✓ Health check endpoint: http://localhost:8080/healthz
            pause
            exit /b 0
        )
    )
)

echo Waiting for SearXNG to become healthy... (!elapsed!/!timeout!s)
timeout /t 5 /nobreak >nul
set /a elapsed=!elapsed!+5
goto check_health

:timeout_reached
echo Warning: SearXNG may not be fully ready yet. Check with: docker-compose logs searxng
echo Access SearXNG at: http://localhost:8080
pause