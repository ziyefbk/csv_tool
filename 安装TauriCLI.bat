@echo off
chcp 65001 >nul
REM 安装 Tauri CLI 工具

echo.
echo ========================================
echo   Tauri CLI 安装工具
echo ========================================
echo.

REM 检查 Rust
where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ 未找到 Rust
    echo.
    echo 请先安装 Rust: https://rustup.rs/
    pause
    exit /b 1
)

echo ✅ 检测到 Rust
echo.

REM 检查是否已安装
cargo tauri --version >nul 2>&1
if %errorlevel% equ 0 (
    echo ✅ Tauri CLI 已安装
    cargo tauri --version
    echo.
    pause
    exit /b 0
)

echo [安装] 正在安装 Tauri CLI...
echo.
echo 这可能需要 5-15 分钟，请耐心等待...
echo.

cargo install tauri-cli --version "^1.5" --locked

if %errorlevel% neq 0 (
    echo.
    echo ❌ 安装失败
    echo.
    echo 可能的解决方案:
    echo 1. 检查网络连接
    echo 2. 确保已安装 Microsoft C++ Build Tools
    echo 3. 尝试使用国内镜像（如果在中国）
    echo.
    echo 手动安装命令:
    echo   cargo install tauri-cli --version "^1.5" --locked
    echo.
    pause
    exit /b 1
)

echo.
echo ✅ 安装成功！
echo.
cargo tauri --version
echo.
pause

