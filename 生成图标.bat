@echo off
chcp 65001 >nul 2>&1
REM 生成 Tauri 应用图标

echo.
echo ========================================
echo   生成 Tauri 应用图标
echo ========================================
echo.

cd /d "%~dp0"

REM 检查 Tauri CLI
cargo tauri --version >nul 2>&1
if %errorlevel% neq 0 (
    echo [错误] 未找到 Tauri CLI
    echo 请先运行: .\安装TauriCLI.bat
    pause
    exit /b 1
)

echo [方法 1] 使用 Tauri CLI 生成图标
echo.
echo 需要准备一个 512x512 的 PNG 图标文件
echo 然后运行：
echo   cargo tauri icon path/to/your/icon.png
echo.
echo [方法 2] 使用在线工具（推荐）
echo.
echo 1. 访问: https://convertio.co/zh/png-ico/
echo 2. 上传一个 PNG 图片（建议 256x256 或更大）
echo 3. 下载为 ICO 格式
echo 4. 保存到: tauri\icons\icon.ico
echo.
echo [方法 3] 使用其他在线工具
echo   - https://www.icoconverter.com/
echo   - https://cloudconvert.com/png-to-ico
echo.

REM 检查图标目录
if not exist "tauri\icons" (
    mkdir "tauri\icons"
    echo [信息] 已创建图标目录
)

if exist "tauri\icons\icon.ico" (
    echo [成功] 图标文件已存在: tauri\icons\icon.ico
) else (
    echo [提示] 图标文件不存在，请使用上述方法创建
)

echo.
pause

