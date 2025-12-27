@echo off
chcp 65001 >nul 2>&1
REM CSV Tool GUI Quick Start Script

echo.
echo ========================================
echo   CSV Tool GUI - Quick Start
echo ========================================
echo.

REM Check if dependencies are installed
if not exist "frontend\node_modules" (
    echo [WARNING] Frontend dependencies not installed
    echo.
    echo Running setup script...
    call setup_gui.bat
    if %errorlevel% neq 0 (
        pause
        exit /b 1
    )
    echo.
)

REM Check Rust version
for /f "tokens=2" %%i in ('rustc --version 2^>nul') do set RUST_VERSION=%%i
if defined RUST_VERSION (
    echo [CHECK] Rust version: %RUST_VERSION%
) else (
    echo [ERROR] Rust not found
    pause
    exit /b 1
)

REM Check Tauri CLI
cargo tauri --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [WARNING] cargo-tauri-cli not found
    echo.
    echo Installing Tauri CLI...
    echo This may take several minutes...
    cargo install tauri-cli --version "^1.5" --locked
    if %errorlevel% neq 0 (
        echo.
        echo [ERROR] Tauri CLI installation failed
        echo.
        echo Possible causes:
        echo 1. Rust version too old, run: .\更新Rust.bat
        echo 2. Network issue, check your connection
        echo.
        echo Manual install:
        echo   cargo install tauri-cli --version "^1.5" --locked
        echo.
        pause
        exit /b 1
    )
    echo [SUCCESS] Tauri CLI installed
    echo.
)

REM Check Tauri directory
if not exist "tauri" (
    echo [ERROR] tauri directory not found
    echo Please run this script from project root
    pause
    exit /b 1
)

echo ========================================
echo   Recommended: Two-Step Startup
echo ========================================
echo.
echo Step 1: Start frontend server (first window)
echo   cd frontend
echo   npm run dev
echo.
echo Step 2: Start Tauri (second window)
echo   cd tauri
echo   cargo tauri dev
echo.
echo Press any key to try auto-start (may be unstable)
pause >nul

echo.
echo [STEP 1] Starting frontend server...
start "CSV Tool Frontend" cmd /k "cd /d %~dp0frontend && npm run dev"

echo [WAIT] Waiting for frontend server to start (5 seconds)...
timeout /t 5 /nobreak >nul

echo [STEP 2] Starting Tauri application...
echo.
cd tauri
cargo tauri dev

if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Startup failed
    echo.
    echo Please use two-step startup method:
    echo 1. Open first PowerShell, run: cd frontend ^&^& npm run dev
    echo 2. Open second PowerShell, run: cd tauri ^&^& cargo tauri dev
    echo.
    echo See: 启动说明.md for details
    echo.
    pause
)

cd ..
