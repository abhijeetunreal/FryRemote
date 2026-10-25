@echo off
title FryRemote Launcher
cd /d "%~dp0"
set "PATH=C:\Users\abhij\.cargo\bin;%PATH%"
if not exist "%~dp0target\debug\tvremote-desktop.exe" (
    echo Building FryRemote...
    cargo build --bin tvremote-desktop
)
echo Starting FryRemote Desktop App...
start "" "%~dp0target\debug\tvremote-desktop.exe"
