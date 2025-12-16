@echo off
REM Script batch para compilar y ejecutar un test individual con stack completo
REM Uso: BUILD-Y-EJECUTAR.bat test_strings_basico.ad

if "%~1"=="" (
    echo ERROR: Debes especificar un archivo de test
    echo Uso: BUILD-Y-EJECUTAR.bat test_strings_basico.ad
    exit /b 1
)

set SCRIPT_DIR=%~dp0
cd /d "%SCRIPT_DIR%"

powershell.exe -ExecutionPolicy Bypass -File "BUILD-Y-EJECUTAR.ps1" -TestFile "%~1"

pause








