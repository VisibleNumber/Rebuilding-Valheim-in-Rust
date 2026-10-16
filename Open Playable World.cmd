@echo off
cd /d "%~dp0"
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoProfile -ExecutionPolicy Bypass -File "%~dp0Open-Playable.ps1" %*
set "lab_exit=%errorlevel%"
if not "%lab_exit%"=="0" pause
exit /b %lab_exit%

