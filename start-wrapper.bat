@echo off
REM ============================================================================
REM Backward Compatibility Wrapper for START_APPLICATION.BAT
REM 
REM This script has been moved to scripts/windows/start-application.bat for
REM better organization. This wrapper ensures backward compatibility.
REM 
REM For the latest documentation, see: docs/installation/installation-guide.md
REM ============================================================================

echo [INFO] Launching application start script from new location...
echo [INFO] Scripts have been reorganized to scripts/windows/ directory
echo.

call "%~dp0scripts\windows\start-application.bat" %*
exit /b %ERRORLEVEL%
