@echo off
REM CSV Tool GUI 设置脚本 (Windows)

echo 🚀 开始设置 CSV Tool GUI...

REM 检查 Node.js
where node >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ 未找到 Node.js，请先安装 Node.js 18 或更高版本
    exit /b 1
)

echo ✅ Node.js 版本:
node -v

REM 检查 Rust
where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo ❌ 未找到 Rust，请先安装 Rust
    exit /b 1
)

echo ✅ Rust 版本:
rustc --version

REM 安装前端依赖
echo 📦 安装前端依赖...
cd frontend
call npm install

if %errorlevel% neq 0 (
    echo ❌ 前端依赖安装失败
    exit /b 1
)

echo ✅ 前端依赖安装完成

REM 返回根目录
cd ..

echo.
echo ✨ 设置完成！
echo.
echo 运行开发模式:
echo   cd tauri ^&^& cargo tauri dev
echo.
echo 构建生产版本:
echo   cd tauri ^&^& cargo tauri build

pause

