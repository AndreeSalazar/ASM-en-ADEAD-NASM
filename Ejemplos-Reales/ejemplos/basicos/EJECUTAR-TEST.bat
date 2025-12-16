@echo off
REM Script batch alternativo para ejecutar tests
REM Uso: EJECUTAR-TEST.bat test_strings_basico.ad

if "%~1"=="" (
    echo ERROR: Debes especificar un archivo de test
    echo Uso: EJECUTAR-TEST.bat test_strings_basico.ad
    exit /b 1
)

set TEST_FILE=%~1
set SCRIPT_DIR=%~dp0
cd /d "%SCRIPT_DIR%"

echo ========================================
echo   Ejecutando Test: %TEST_FILE%
echo ========================================
echo.

REM Ejecutar PowerShell con política de ejecución bypass
powershell.exe -ExecutionPolicy Bypass -File "ejecutar_test_individual.ps1" "%TEST_FILE%"

