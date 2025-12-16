@echo off
REM Script para compilar y ejecutar TODOS los tests con el stack completo
REM Stack: ADead -> C++ Generator -> GCC++/Clang++ -> Rust Cleaner -> ASM -> NASM -> .obj -> Zig/GCC/Clang (linker) -> .exe

set SCRIPT_DIR=%~dp0
cd /d "%SCRIPT_DIR%"

echo ========================================
echo   Compilando y Ejecutando TODOS los Tests
echo   Stack Completo
echo ========================================
echo.

REM Ejecutar build completo y ejecutar todos
powershell.exe -ExecutionPolicy Bypass -File "BUILD-COMPLETO-STACK.ps1" -Verbose

pause













