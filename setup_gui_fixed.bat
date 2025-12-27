@echo off
chcp 65001 >nul 2>&1
REM CSV Tool GUI Setup Script for Windows (Fixed)

echo.
echo ========================================
echo   CSV Tool GUI - Windows Setup
echo ========================================
echo.

REM Check Node.js
echo [1/4] Checking Node.js...
where node >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Node.js not found
    echo.
    echo Please install Node.js 18 or higher:
    echo https://nodejs.org/
    echo.
    pause
    exit /b 1
)

for /f "tokens=1 delims=v" %%i in ('node -v 2^>nul') do set NODE_VERSION=%%i
echo [OK] Node.js version: %NODE_VERSION%

REM Check Rust
echo.
echo [2/4] Checking Rust...
where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Rust not found
    echo.
    echo Please install Rust:
    echo https://rustup.rs/
    echo.
    echo Download rustup-init.exe and run it
    echo.
    pause
    exit /b 1
)

for /f "tokens=1" %%i in ('rustc --version 2^>nul') do set RUST_VERSION=%%i
echo [OK] Rust version: %RUST_VERSION%

REM Check and install cargo-tauri-cli
echo.
echo [3/4] Checking Tauri CLI...
cargo tauri --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [WARNING] cargo-tauri-cli not found, installing...
    cargo install tauri-cli --version "^1.5" --locked
    if %errorlevel% neq 0 (
        echo [ERROR] Tauri CLI installation failed
        echo.
        echo Please install manually:
        echo   cargo install tauri-cli --version "^1.5" --locked
        echo.
        pause
        exit /b 1
    )
    echo [OK] Tauri CLI installed
) else (
    echo [OK] Tauri CLI already installed
)

REM Check frontend directory
if not exist "frontend" (
    echo [ERROR] frontend directory not found
    echo Please run this script from project root
    pause
    exit /b 1
)

REM Install frontend dependencies
echo.
echo [4/4] Installing frontend dependencies...
echo This may take a few minutes, please wait...
echo.

REM Change to frontend directory and install
pushd frontend
call npm install 2>nul
set INSTALL_RESULT=%errorlevel%
popd

REM Check installation result
if %INSTALL_RESULT% neq 0 (
    echo.
    echo [ERROR] Frontend dependencies installation failed
    echo.
    echo Possible solutions:
    echo 1. Check network connection
    echo 2. Try clearing cache: npm cache clean --force
    echo 3. Use mirror (if in China): npm config set registry https://registry.npmmirror.com
    echo.
    pause
    exit /b 1
)

echo.
echo [OK] Frontend dependencies installed successfully

echo.
echo ========================================
echo   Setup Complete!
echo ========================================
echo.
echo Next steps:
echo.
echo 1. Build EXE file:
echo    Run: .\构建EXE.bat
echo.
echo 2. Or run in development mode:
echo    Run: .\运行GUI.bat
echo.
echo Tips:
echo - First build will download Rust dependencies, takes longer
echo - Make sure Microsoft C++ Build Tools are installed
echo - See docs\WINDOWS_GUI_GUIDE.md for detailed help
echo.
pause

