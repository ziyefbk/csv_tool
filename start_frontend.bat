@echo off
REM Start frontend development server

cd /d %~dp0frontend

if not exist "node_modules" (
    echo [ERROR] Dependencies not installed
    echo Please run: setup_gui.bat first
    pause
    exit /b 1
)

echo [START] Frontend development server...
echo Server will be available at: http://localhost:5173
echo Press Ctrl+C to stop
echo.

npm run dev

