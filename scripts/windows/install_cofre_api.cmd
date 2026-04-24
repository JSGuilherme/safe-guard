@echo off
setlocal
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0install_cofre_api.ps1" %*
if errorlevel 1 (
  echo.
  echo Falha na instalacao da API.
  pause
  exit /b 1
)
echo.
echo Instalacao finalizada com sucesso.
pause
