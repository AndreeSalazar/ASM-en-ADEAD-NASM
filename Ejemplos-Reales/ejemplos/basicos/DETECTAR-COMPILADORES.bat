@echo off
REM Script batch para detectar compiladores
echo ========================================
echo   Deteccion de Compiladores C/C++
echo ========================================
echo.

echo Ejecutando PowerShell para deteccion completa...
powershell.exe -ExecutionPolicy Bypass -File "%~dp0DETECTAR-COMPILADORES.ps1"

pause

