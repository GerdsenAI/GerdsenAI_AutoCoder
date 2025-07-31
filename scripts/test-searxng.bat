@echo off
REM SearXNG Integration Test Runner (Windows)
REM This script sets up and runs comprehensive SearXNG integration tests

setlocal enabledelayedexpansion

set SCRIPT_DIR=%~dp0
set PROJECT_ROOT=%SCRIPT_DIR%..
set DOCKER_DIR=%PROJECT_ROOT%\docker\searxng

echo 🔍 SearXNG Integration Test Runner
echo ==================================

REM Check if Docker is available
docker --version >nul 2>&1
if !errorlevel! neq 0 (
    echo ❌ Error: Docker is not installed or not in PATH
    pause
    exit /b 1
)

docker info >nul 2>&1
if !errorlevel! neq 0 (
    echo ❌ Error: Docker is not running
    pause
    exit /b 1
)

REM Parse command line arguments
set stop_after_tests=false
set skip_start=false

:parse_args
if "%~1"=="" goto args_done
if "%~1"=="--stop-after" (
    set stop_after_tests=true
    shift
    goto parse_args
)
if "%~1"=="--skip-start" (
    set skip_start=true
    shift
    goto parse_args
)
if "%~1"=="--help" (
    echo Usage: %0 [OPTIONS]
    echo.
    echo Options:
    echo   --stop-after    Stop SearXNG containers after tests complete
    echo   --skip-start    Skip starting SearXNG ^(assume it's already running^)
    echo   --help          Show this help message
    echo.
    pause
    exit /b 0
)
echo Unknown option: %~1
echo Use --help for usage information
pause
exit /b 1

:args_done

REM Function to check if SearXNG is healthy
:check_searxng_health
set max_attempts=30
set attempt=0

echo ⏳ Waiting for SearXNG to become healthy...

:health_loop
if !attempt! geq !max_attempts! goto health_failed

curl -f -s http://localhost:8080/healthz >nul 2>&1
if !errorlevel! equ 0 (
    echo ✅ SearXNG is healthy and ready!
    goto health_success
)

set /a attempt=!attempt!+1
echo    Attempt !attempt!/!max_attempts! - waiting...
timeout /t 2 /nobreak >nul
goto health_loop

:health_failed
echo ❌ SearXNG failed to become healthy within 60 seconds
exit /b 1

:health_success
goto :eof

REM Function to start SearXNG if not running
:start_searxng
echo 🐳 Checking SearXNG Docker containers...

cd /d "%DOCKER_DIR%"

REM Check if containers are already running
for /f "delims=" %%i in ('docker-compose ps -q searxng 2^>nul') do (
    for /f "delims=" %%j in ('docker inspect --format="{{.State.Status}}" %%i 2^>nul') do (
        if "%%j"=="running" (
            echo ✅ SearXNG containers are already running
            call :check_searxng_health
            if !errorlevel! neq 0 (
                echo 🔄 Restarting SearXNG containers...
                docker-compose restart
                call :check_searxng_health
            )
            goto start_done
        )
    )
)

echo 🚀 Starting SearXNG containers...
docker-compose up -d
if !errorlevel! neq 0 (
    echo ❌ Failed to start SearXNG containers
    pause
    exit /b 1
)

call :check_searxng_health
if !errorlevel! neq 0 exit /b 1

:start_done
goto :eof

REM Function to run tests
:run_tests
echo 🧪 Running SearXNG Integration Tests...

cd /d "%PROJECT_ROOT%\src-tauri"

REM Set test environment variables
set RUST_TEST_THREADS=1
set RUST_LOG=debug

echo.
echo 📋 Running Integration Tests...
cargo test searxng_integration_tests --lib -- --nocapture
if !errorlevel! neq 0 set test_failed=true

echo.
echo ⚡ Running Performance Tests...
cargo test searxng_performance_tests --lib -- --nocapture
if !errorlevel! neq 0 set test_failed=true

echo.
echo 🏥 Running Health Tests...
cargo test searxng_health_tests --lib -- --nocapture
if !errorlevel! neq 0 set test_failed=true

echo.
echo 🔍 Running All SearXNG Tests...
cargo test searxng --lib -- --nocapture
if !errorlevel! neq 0 set test_failed=true

if defined test_failed (
    exit /b 1
) else (
    exit /b 0
)

REM Function to clean up
:cleanup
echo.
echo 🧹 Cleanup options:
echo   To stop SearXNG containers: cd %DOCKER_DIR% ^&^& docker-compose down
echo   To remove data volumes: cd %DOCKER_DIR% ^&^& docker-compose down -v
echo   To view logs: cd %DOCKER_DIR% ^&^& docker-compose logs
goto :eof

REM Main execution
set test_failed=

REM Start SearXNG if needed
if "%skip_start%"=="false" (
    call :start_searxng
    if !errorlevel! neq 0 exit /b 1
) else (
    echo ⏭️  Skipping SearXNG startup (--skip-start specified)
    call :check_searxng_health
    if !errorlevel! neq 0 (
        echo ❌ SearXNG is not healthy. Remove --skip-start to auto-start.
        pause
        exit /b 1
    )
)

REM Run tests
call :run_tests
set test_result=!errorlevel!

if !test_result! equ 0 (
    echo.
    echo ✅ All SearXNG tests passed successfully!
) else (
    echo.
    echo ❌ Some SearXNG tests failed!
)

REM Stop containers if requested
if "%stop_after_tests%"=="true" (
    echo.
    echo 🛑 Stopping SearXNG containers...
    cd /d "%DOCKER_DIR%"
    docker-compose down
)

call :cleanup
pause
exit /b !test_result!