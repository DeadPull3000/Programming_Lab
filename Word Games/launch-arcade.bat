@echo off
setlocal
echo ============================================================
echo         A Cozy Word Game Arcade — Desktop Launcher
echo ============================================================
echo.

cd /d "%~dp0"

echo [1/3] Building & Starting Native Rust Backend Server...
start "Cozy Arcade Rust Server" /b cargo run -p arcade-server

timeout /t 2 /nobreak >nul

echo [2/3] Starting Desktop GUI Server...
cd desktop
start "Cozy Arcade Frontend" /b npm run dev -- --host 127.0.0.1 --port 5173

timeout /t 3 /nobreak >nul

echo [3/3] Opening Cozy Arcade Window...
start "" msedge --app=http://127.0.0.1:5173 || start "" chrome --app=http://127.0.0.1:5173 || start http://127.0.0.1:5173

echo.
echo Cozy Word Game Arcade is running!
echo Close this terminal or stop the processes when finished.
pause
