@echo off
REM Script batch para ejecutar el build completo con stack completo
REM Uso: BUILD-COMPLETO-STACK.bat [filtro] [opciones]
REM Ejemplos:
REM   BUILD-COMPLETO-STACK.bat
REM   BUILD-COMPLETO-STACK.bat test_strings*.ad
REM   BUILD-COMPLETO-STACK.bat *.ad -NoExecute

set SCRIPT_DIR=%~dp0
cd /d "%SCRIPT_DIR%"

echo ========================================
echo   Build Completo con Stack Completo
echo ========================================
echo.

REM Ejecutar PowerShell con política de ejecución bypass
powershell.exe -ExecutionPolicy Bypass -File "BUILD-COMPLETO-STACK.ps1" %*

if %ERRORLEVEL% neq 0 (
    echo.
    echo ERROR: El script fallo con codigo %ERRORLEVEL%
    pause
    exit /b %ERRORLEVEL%
)

pause













