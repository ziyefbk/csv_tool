@echo off
chcp 65001 >nul 2>&1
REM 配置 Cargo 国内镜像源以加速下载

echo.
echo ========================================
echo   配置 Cargo 镜像源
echo ========================================
echo.

REM 创建 .cargo 目录（如果不存在）
if not exist "%USERPROFILE%\.cargo" (
    mkdir "%USERPROFILE%\.cargo"
    echo [信息] 创建 .cargo 目录
)

REM 检查是否已有 config.toml
set CONFIG_FILE=%USERPROFILE%\.cargo\config.toml
if exist "%CONFIG_FILE%" (
    echo [警告] 已存在 config.toml，将备份为 config.toml.bak
    copy "%CONFIG_FILE%" "%CONFIG_FILE%.bak" >nul 2>&1
)

REM 创建新的 config.toml
(
echo [source.crates-io]
echo replace-with = 'ustc'
echo.
echo [source.ustc]
echo registry = "https://mirrors.ustc.edu.cn/crates.io-index"
echo.
echo [net]
echo retry = 10
) > "%CONFIG_FILE%"

if %errorlevel% equ 0 (
    echo [成功] Cargo 镜像源配置完成！
    echo.
    echo 使用的镜像：中科大镜像源
    echo 配置文件位置：%CONFIG_FILE%
    echo.
    echo 现在可以重新运行构建命令：
    echo   .\构建EXE.bat
    echo.
) else (
    echo [错误] 配置失败
    pause
    exit /b 1
)

pause

