@echo off
REM Script para verificar todas las herramientas disponibles
echo ========================================
echo   Verificacion de Herramientas
echo ========================================
echo.

echo [1/6] Verificando NASM...
where nasm >nul 2>&1
if %ERRORLEVEL%==0 (
    echo   [OK] NASM encontrado
    nasm -v
) else (
    echo   [ERROR] NASM no encontrado
    echo   Buscando en ubicaciones comunes...
    if exist "C:\Users\andre\AppData\Local\bin\NASM\nasm.exe" (
        echo   [OK] Encontrado en: C:\Users\andre\AppData\Local\bin\NASM\nasm.exe
    ) else (
        echo   [ERROR] NASM no encontrado en ninguna ubicacion
    )
)
echo.

echo [2/6] Verificando Zig...
where zig >nul 2>&1
if %ERRORLEVEL%==0 (
    echo   [OK] Zig encontrado
    zig version
) else (
    echo   [ERROR] Zig no encontrado
    echo   Buscando en ubicaciones comunes...
    if exist "C:\zig-x86_64-windows-0.16.0-dev.1484+d0ba6642b\zig.exe" (
        echo   [OK] Encontrado en: C:\zig-x86_64-windows-0.16.0-dev.1484+d0ba6642b\zig.exe
        echo   NOTA: Agrega esta ruta al PATH para usar Zig
    ) else (
        echo   [ERROR] Zig no encontrado
    )
)
echo.

echo [3/6] Verificando Rust/Cargo...
where cargo >nul 2>&1
if %ERRORLEVEL%==0 (
    echo   [OK] Cargo encontrado
    cargo --version
) else (
    echo   [ERROR] Cargo no encontrado
)
echo.

echo [4/6] Verificando Compilador ADead...
set COMPILER_PATH=%CD%\..\..\..\CORE\rust\target\release\adeadc.exe
if exist "%COMPILER_PATH%" (
    echo   [OK] Compilador encontrado: %COMPILER_PATH%
) else (
    set COMPILER_PATH=%CD%\..\..\..\CORE\target\release\adeadc.exe
    if exist "%COMPILER_PATH%" (
        echo   [OK] Compilador encontrado: %COMPILER_PATH%
    ) else (
        echo   [ERROR] Compilador no encontrado
        echo   Ejecuta: cd CORE\rust ^&^& cargo build --release
    )
)
echo.

echo [5/6] Verificando GCC...
where gcc >nul 2>&1
if %ERRORLEVEL%==0 (
    echo   [OK] GCC encontrado
    gcc --version | findstr /C:"gcc"
) else (
    echo   [NO] GCC no encontrado (no es necesario si usas Zig)
)
echo.

echo [6/6] Verificando Clang...
where clang >nul 2>&1
if %ERRORLEVEL%==0 (
    echo   [OK] Clang encontrado
    clang --version | findstr /C:"clang"
) else (
    echo   [NO] Clang no encontrado (no es necesario si usas Zig)
)
echo.

echo ========================================
echo   Resumen
echo ========================================
echo.
echo Herramientas necesarias para compilar tests:
echo   [REQUERIDO] NASM - Ensamblador
echo   [REQUERIDO] Zig o GCC - Linker
echo   [REQUERIDO] Compilador ADead
echo.
echo Si tienes NASM y Zig, puedes usar:
echo   ejecutar_con_zig.bat test_strings_basico.ad
echo.

pause

