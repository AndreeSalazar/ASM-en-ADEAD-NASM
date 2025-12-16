@echo off
REM Script para buscar el compilador en todas las ubicaciones posibles
echo ========================================
echo   Buscando Compilador ADead
echo ========================================
echo.

set SCRIPT_DIR=%~dp0
set FOUND=0

REM Buscar en múltiples ubicaciones
echo Buscando en:
echo.

REM 1. Desde script_dir hacia arriba
set PATH1=%SCRIPT_DIR%..\..\..\CORE\rust\target\release\adeadc.exe
if exist "%PATH1%" (
    echo [OK] Encontrado: %PATH1%
    set COMPILER_PATH=%PATH1%
    set FOUND=1
    goto :found
)

REM 2. CORE directamente
set PATH2=%SCRIPT_DIR%..\..\..\CORE\target\release\adeadc.exe
if exist "%PATH2%" (
    echo [OK] Encontrado: %PATH2%
    set COMPILER_PATH=%PATH2%
    set FOUND=1
    goto :found
)

REM 3. Buscar recursivamente desde el workspace root
for %%I in ("%SCRIPT_DIR%") do set WORKSPACE_ROOT=%%~dpI
set PATH3=%WORKSPACE_ROOT%CORE\rust\target\release\adeadc.exe
if exist "%PATH3%" (
    echo [OK] Encontrado: %PATH3%
    set COMPILER_PATH=%PATH3%
    set FOUND=1
    goto :found
)

REM 4. CORE directamente desde root
set PATH4=%WORKSPACE_ROOT%CORE\target\release\adeadc.exe
if exist "%PATH4%" (
    echo [OK] Encontrado: %PATH4%
    set COMPILER_PATH=%PATH4%
    set FOUND=1
    goto :found
)

REM 5. Buscar en cualquier target\release
echo Buscando recursivamente...
for /r "%WORKSPACE_ROOT%" %%F in (adeadc.exe) do (
    if exist "%%F" (
        echo [OK] Encontrado: %%F
        set COMPILER_PATH=%%F
        set FOUND=1
        goto :found
    )
)

:found
if %FOUND%==0 (
    echo [ERROR] Compilador NO encontrado
    echo.
    echo Ubicaciones verificadas:
    echo   %PATH1%
    echo   %PATH2%
    echo   %PATH3%
    echo   %PATH4%
    echo.
    echo Por favor, compila el proyecto:
    echo   cd CORE\rust
    echo   cargo build --release
    echo.
    echo O verifica que el ejecutable se llama adeadc.exe
) else (
    echo.
    echo ========================================
    echo   Compilador encontrado!
    echo ========================================
    echo Ubicacion: %COMPILER_PATH%
    echo.
    echo Puedes usar esta ruta en los scripts o ejecutar:
    echo   "%COMPILER_PATH%" compile test_strings_basico.ad --backend auto -o test.asm
)

pause

