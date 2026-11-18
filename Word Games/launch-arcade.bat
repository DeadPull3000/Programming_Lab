@echo off
setlocal
powershell.exe -NoProfile -File "%~dp0launch-arcade.ps1"
if errorlevel 1 pause
