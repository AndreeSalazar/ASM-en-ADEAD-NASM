@echo off
REM Script batch para verificar compilación
echo ========================================
echo   Verificación de Compilación
echo ========================================
echo.

set SCRIPT_DIR=%~dp0
cd /d "%SCRIPT_DIR%"

REM Ejecutar PowerShell con política de ejecución bypass
powershell.exe -ExecutionPolicy Bypass -File "verificar_compilacion.ps1"

