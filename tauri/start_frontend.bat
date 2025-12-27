@echo off
cd /d %~dp0..\frontend
if exist node_modules (
    npm run dev
) else (
    echo [错误] 前端依赖未安装，请先运行 setup_gui.bat
    pause
    exit /b 1
)

