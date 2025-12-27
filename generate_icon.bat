@echo off
REM Generate Tauri Application Icon

echo.
echo ========================================
echo   Generate Tauri Application Icon
echo ========================================
echo.

cd /d "%~dp0"

REM Check Tauri CLI
cargo tauri --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Tauri CLI not found
    echo Please run: .\install_tauri_cli.bat first
    pause
    exit /b 1
)

echo [Method 1] Using Tauri CLI to generate icon
echo.
echo You need a 512x512 PNG icon file
echo Then run:
echo   cargo tauri icon path/to/your/icon.png
echo.
echo [Method 2] Using online tool (Recommended)
echo.
echo 1. Visit: https://convertio.co/zh/png-ico/
echo 2. Upload a PNG image (recommended 256x256 or larger)
echo 3. Download as ICO format
echo 4. Save to: tauri\icons\icon.ico
echo.
echo [Method 3] Using other online tools
echo   - https://www.icoconverter.com/
echo   - https://cloudconvert.com/png-to-ico
echo.

REM Check icon directory
if not exist "tauri\icons" (
    mkdir "tauri\icons"
    echo [INFO] Icon directory created
)

if exist "tauri\icons\icon.ico" (
    echo [SUCCESS] Icon file exists: tauri\icons\icon.ico
) else (
    echo [TIP] Icon file does not exist, please create using methods above
)

echo.
pause

