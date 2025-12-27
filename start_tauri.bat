@echo off
REM Start Tauri application

cd /d %~dp0tauri

if not exist "Cargo.toml" (
    echo [ERROR] Not in tauri directory
    echo Please run from project root
    pause
    exit /b 1
)

REM Check Tauri CLI
cargo tauri --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] cargo-tauri-cli not found
    echo Please run: .\安装TauriCLI.bat first
    pause
    exit /b 1
)

echo [START] Tauri application...
echo Make sure frontend server is running on http://localhost:5173
echo Press Ctrl+C to stop
echo.

cargo tauri dev

