@echo off
REM Script batch para ejecutar todos los tests
echo ========================================
echo   Tests de Strings Avanzados - ADead
echo ========================================
echo.

set SCRIPT_DIR=%~dp0
cd /d "%SCRIPT_DIR%"

REM Ejecutar PowerShell con política de ejecución bypass
powershell.exe -ExecutionPolicy Bypass -File "ejecutar_tests_strings.ps1"

