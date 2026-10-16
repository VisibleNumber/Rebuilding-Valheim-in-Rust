@echo off
cd /d "%~dp0"
if not exist "test-game\valheim_Data\Managed\ValheimLab.Diagnostic.dll" (
 echo The test copy is not prepared. See README.md.
 pause
 exit /b 1
)
echo Opening your separate Valheim test copy.
echo Create a NEW character and LOCAL world. F8 stops or starts recording.
echo Reports are saved in the recordings folder.
tasklist /FI "IMAGENAME eq valheim.exe" /NH | findstr /I "valheim.exe" >nul
if not errorlevel 1 (
 echo Valheim is already running. Close it when ready to try the separate test copy.
 pause
 exit /b 1
)
:chooseLog
set "labLog=%~dp0evidence\manual-%RANDOM%-%RANDOM%.log"
if exist "%labLog%" goto chooseLog
start "" /D "%~dp0test-game" "%~dp0test-game\valheim.exe" -savedir "%~dp0test-saves" -logFile "%labLog%" -lab-record
