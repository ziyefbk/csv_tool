@echo off
chcp 65001 >nul 2>&1
REM Force update Rust toolchain

echo.
echo ========================================
echo   Force Update Rust Toolchain
echo ========================================
echo.

echo [STEP 1] Current versions:
rustc --version
cargo --version
echo.

echo [STEP 2] Updating rustup...
rustup self update
echo.

echo [STEP 3] Installing latest stable...
rustup install stable
echo.

echo [STEP 4] Setting default toolchain...
rustup default stable
echo.

echo [STEP 5] Updated versions:
rustc --version
cargo --version
echo.

echo [STEP 6] Cleaning cargo cache...
cargo clean
echo.

echo ========================================
echo   Update Complete!
echo ========================================
echo.
echo Please restart PowerShell and try again
echo.
pause

