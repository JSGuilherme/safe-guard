@echo off
setlocal
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0uninstall_cofre_api.ps1" %*
if errorlevel 1 (
  echo.
  echo Falha na desinstalacao da API.
  pause
  exit /b 1
)
echo.
echo Desinstalacao finalizada com sucesso.
pause
