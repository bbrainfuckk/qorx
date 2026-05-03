@echo off
setlocal

set "QORX_EXE=%~dp0qorx.exe"

if not exist "%QORX_EXE%" (
  echo Qorx CLI launcher could not find qorx.exe next to this file.
  echo Keep this launcher in the same folder as qorx.exe.
  echo.
  pause
  exit /b 1
)

cd /d "%~dp0"
echo Qorx CLI
echo Running: "%QORX_EXE%"
echo.
"%QORX_EXE%" doctor
echo.
echo Try these commands:
echo   qorx.exe --version
echo   qorx.exe index .
echo   qorx.exe strict-answer "what proves Qorx is a language runtime?"
echo.
echo This window stays open for double-click users.
cmd /k
