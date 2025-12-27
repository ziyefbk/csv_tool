@echo off
REM CSV Tool GUI - Build EXE Script

REM Enable delayed variable expansion
setlocal enabledelayedexpansion

REM Change to script directory (project root)
cd /d "%~dp0"
if %errorlevel% neq 0 (
    echo [ERROR] Cannot change to script directory
    echo Script path: %~dp0
    pause
    exit /b 1
)

echo.
echo ========================================
echo   CSV Tool GUI - Build EXE
echo ========================================
echo.
echo [INFO] Current directory: %CD%
echo.

REM Check Rust
rustc --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Rust not found
    echo Please install Rust: https://rustup.rs/
    pause
    exit /b 1
)

REM Check Node.js
node --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Node.js not found
    echo Please install Node.js: https://nodejs.org/
    pause
    exit /b 1
)

echo [CHECK] Dependencies OK
echo.

REM Check frontend dependencies
if not exist "frontend\node_modules" (
    echo [WARN] Frontend dependencies not installed, installing...
    cd frontend
    call npm install
    if %errorlevel% neq 0 (
        echo [ERROR] Frontend dependency installation failed
        cd ..
        pause
        exit /b 1
    )
    cd ..
    echo [SUCCESS] Frontend dependencies installed
    echo.
)

REM Check Tauri CLI
cargo tauri --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [WARN] Tauri CLI not installed, installing...
    echo This may take a few minutes...
    cargo install tauri-cli --version "^1.5" --locked
    if %errorlevel% neq 0 (
        echo [ERROR] Tauri CLI installation failed
        echo.
        echo Possible reasons:
        echo 1. Rust version too old, run: .\更新Rust.bat
        echo 2. Network issue, check connection
        pause
        exit /b 1
    )
    echo [SUCCESS] Tauri CLI installed
    echo.
)

echo ========================================
echo   Starting build...
echo ========================================
echo.
echo Note: First build may take 10-30 minutes
echo Please wait...
echo.

REM Check directories
if not exist "frontend" (
    echo [ERROR] frontend directory not found
    echo Please ensure you are running this script from project root
    pause
    exit /b 1
)

if not exist "tauri" (
    echo [ERROR] tauri directory not found
    echo Please ensure you are running this script from project root
    pause
    exit /b 1
)

echo [STEP 1/2] Building frontend...
cd frontend
if %errorlevel% neq 0 (
    echo [ERROR] Cannot change to frontend directory
    pause
    exit /b 1
)
call npm run build
set FRONTEND_BUILD_RESULT=%errorlevel%
cd ..
if %FRONTEND_BUILD_RESULT% neq 0 (
    echo.
    echo [ERROR] Frontend build failed
    echo Error code: %FRONTEND_BUILD_RESULT%
    echo.
    pause
    exit /b 1
)

echo.
echo [STEP 2/2] Building Tauri app (Release mode)...
echo This may take a few minutes...
echo.

REM Check icon file
if not exist "tauri\icons\icon.ico" (
    echo [WARNING] Icon file not found: tauri\icons\icon.ico
    echo.
    echo Tauri Windows build requires icon file!
    echo Please create icon file first:
    echo 1. Visit: https://convertio.co/zh/png-ico/
    echo 2. Upload PNG image and download as ICO format
    echo 3. Save to: tauri\icons\icon.ico
    echo.
    echo Or run: .\generate_icon.bat (or .\生成图标.bat)
    echo.
    echo Continuing build anyway (may fail)...
    echo.
)

cd tauri
if %errorlevel% neq 0 (
    echo [ERROR] Cannot change to tauri directory
    echo Current directory: %CD%
    pause
    exit /b 1
)
cargo tauri build
set BUILD_RESULT=%errorlevel%
cd ..

if %BUILD_RESULT% neq 0 (
    echo.
    echo ========================================
    echo   [ERROR] Build failed
    echo ========================================
    echo.
    echo Common issues:
    echo 1. Missing icon file: tauri\icons\icon.ico
    echo 2. Missing Visual C++ Build Tools
    echo    Download: https://visualstudio.microsoft.com/visual-cpp-build-tools/
    echo 3. Insufficient disk space (need at least 5GB)
    echo 4. Network issue causing dependency download failure
    echo.
    pause
    exit /b 1
)

echo.
echo ========================================
echo   Build completed!
echo ========================================
echo.

if exist "tauri\target\release\csv-tool.exe" (
    echo EXE file location:
    echo   tauri\target\release\csv-tool.exe
    echo.
) else (
    echo [WARNING] EXE file not found, check build output
    echo.
)

if exist "tauri\target\release\bundle\msi\*.msi" (
    echo Installer location:
    for %%f in (tauri\target\release\bundle\msi\*.msi) do echo   %%f
    echo.
) else (
    echo [INFO] Installer not found
    echo.
)

echo You can double-click the EXE file to run!
echo.
pause
