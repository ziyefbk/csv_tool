@echo off
chcp 65001 >nul
REM 更新 Rust 工具链

echo.
echo ========================================
echo   Rust 工具链更新工具
echo ========================================
echo.

echo [检查] 当前 Rust 版本...
rustc --version
cargo --version
echo.

echo [更新] 正在更新 Rust 工具链...
echo 这可能需要几分钟时间，请耐心等待...
echo.

rustup update stable

if %errorlevel% neq 0 (
    echo.
    echo [错误] 更新失败
    echo.
    echo 请手动运行: rustup update stable
    echo.
    pause
    exit /b 1
)

echo.
echo [成功] Rust 工具链已更新
echo.
rustc --version
cargo --version
echo.
echo 现在可以重新运行应用了
echo.
pause

