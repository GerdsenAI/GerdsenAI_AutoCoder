@echo off
REM ============================================================================
REM Backward Compatibility Wrapper for install.bat
REM 
REM This script has been moved to scripts/windows/install.bat for better
REM organization. This wrapper ensures backward compatibility for existing
REM documentation and scripts that reference the old location.
REM 
REM For the latest documentation, see: docs/installation/installation-guide.md
REM ============================================================================

echo [INFO] Launching install script from new location...
echo [INFO] Scripts have been reorganized to scripts/windows/ directory
echo.

call "%~dp0scripts\windows\install.bat" %*
exit /b %ERRORLEVEL%
