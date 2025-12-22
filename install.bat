@echo off
REM CloudLab CLI Installer for Windows
REM Author: Sakib Dalal
REM GitHub: https://github.com/Sakib-Dalal
REM Run this file by double-clicking or from Command Prompt

title CloudLab Installer

echo.
echo    _____ _                 _ _           _     
echo   / ____^| ^|               ^| ^| ^|         ^| ^|    
echo  ^| ^|    ^| ^| ___  _   _  __^| ^| ^|     __ _^| ^|__  
echo  ^| ^|    ^| ^|/ _ \^| ^| ^| ^|/ _` ^| ^|    / _` ^| '_ \ 
echo  ^| ^|____^| ^| (_) ^| ^|_^| ^| (_^| ^| ^|___^| (_^| ^| ^|_) ^|
echo   \_____^|_^|\___/ \__,_^|\__,_^|______\__,_^|_.__/ 
echo.
echo   CloudLab CLI Installer for Windows
echo   Author: Sakib Dalal
echo   GitHub: https://github.com/Sakib-Dalal
echo.

REM Check if running as administrator
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [WARNING] Not running as Administrator.
    echo [INFO] Some features may require elevated privileges.
    echo.
)

REM Check if PowerShell is available
where powershell >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] PowerShell not found!
    echo Please install PowerShell or use install.ps1 directly.
    pause
    exit /b 1
)

echo [INFO] Starting PowerShell installer...
echo.

REM Run PowerShell installer
powershell -ExecutionPolicy Bypass -File "%~dp0install.ps1"

if %errorLevel% neq 0 (
    echo.
    echo [ERROR] Installation failed!
    echo Please run install.ps1 manually in PowerShell as Administrator.
    pause
    exit /b 1
)

echo.
echo [SUCCESS] Installation complete!
echo.
echo Quick Start:
echo   1. Open a NEW Command Prompt or PowerShell
echo   2. Run: cloudlab init
echo   3. Run: cloudlab install all
echo   4. Run: cloudlab start all
echo   5. Open: http://localhost:3000
echo.

pause