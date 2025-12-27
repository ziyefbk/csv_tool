@echo off
chcp 65001 >nul 2>&1
REM 清理项目构建产物以释放磁盘空间

echo.
echo ========================================
echo   清理构建产物
echo ========================================
echo.

REM 切换到脚本所在目录
cd /d "%~dp0"

echo [信息] 当前目录: %CD%
echo.
echo 将清理以下目录：
echo   1. target/ (Rust构建产物)
echo   2. frontend\node_modules/ (前端依赖)
echo   3. frontend\dist/ (前端构建产物)
echo   4. tauri\target/ (Tauri构建产物)
echo.
set /p confirm="确认清理？(Y/N): "
if /i not "%confirm%"=="Y" (
    echo 已取消
    pause
    exit /b 0
)

echo.
echo [步骤 1/4] 清理 Rust 构建产物...
if exist "target" (
    cargo clean
    if %errorlevel% equ 0 (
        echo [成功] Rust 构建产物已清理
    ) else (
        echo [警告] cargo clean 失败，尝试手动删除...
        if exist "target" (
            rd /s /q target
            echo [成功] target 目录已删除
        )
    )
) else (
    echo [跳过] target 目录不存在
)

echo.
echo [步骤 2/4] 清理前端依赖...
if exist "frontend\node_modules" (
    rd /s /q "frontend\node_modules"
    if %errorlevel% equ 0 (
        echo [成功] 前端依赖已删除
    ) else (
        echo [错误] 删除失败，可能正在使用中
    )
) else (
    echo [跳过] frontend\node_modules 不存在
)

echo.
echo [步骤 3/4] 清理前端构建产物...
if exist "frontend\dist" (
    rd /s /q "frontend\dist"
    if %errorlevel% equ 0 (
        echo [成功] 前端构建产物已删除
    ) else (
        echo [错误] 删除失败，可能正在使用中
    )
) else (
    echo [跳过] frontend\dist 不存在
)

echo.
echo [步骤 4/4] 清理 Tauri 构建产物...
if exist "tauri\target" (
    cd tauri
    cargo clean
    cd ..
    if %errorlevel% equ 0 (
        echo [成功] Tauri 构建产物已清理
    ) else (
        echo [警告] cargo clean 失败，尝试手动删除...
        if exist "tauri\target" (
            rd /s /q "tauri\target"
            echo [成功] tauri\target 目录已删除
        )
    )
) else (
    echo [跳过] tauri\target 不存在
)

echo.
echo ========================================
echo   清理完成！
echo ========================================
echo.
echo 提示：需要重新构建时运行：
echo   .\构建EXE.bat
echo.
pause

