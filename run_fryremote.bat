@echo off
title FryRemote Launcher
cd /d "%~dp0"
set "PATH=C:\Users\abhij\.cargo\bin;%PATH%"
echo Starting FryRemote Desktop App...
start "" "%~dp0target\debug\tvremote-desktop.exe"
exit
