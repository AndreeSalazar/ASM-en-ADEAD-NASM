@echo off
REM ========================================
REM   ADead - Compilando versión TINY
REM   Optimización máxima de tamaño
REM ========================================

setlocal enabledelayedexpansion

set SCRIPT_DIR=%~dp0
set ASM_FILE=%SCRIPT_DIR%test_simple.asm
set OBJ_FILE=%SCRIPT_DIR%test_simple.obj
set EXE_FILE=%SCRIPT_DIR%test_simple_tiny.exe
set EXE_FILE_UPX=%SCRIPT_DIR%test_simple_tiny_upx.exe

echo.
echo ========================================
echo   ADead - Build TINY (Optimización Máxima)
echo ========================================
echo.

REM Verificar que NASM está disponible
where nasm >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: NASM no encontrado. Por favor instala NASM y agregalo al PATH.
    pause
    exit /b 1
)

REM Paso 1: Ensamblar con NASM
echo [1/4] Ensamblando %ASM_FILE%...
if not exist "%ASM_FILE%" (
    echo ERROR: Archivo .asm no encontrado: %ASM_FILE%
    pause
    exit /b 1
)

nasm -f win64 "%ASM_FILE%" -o "%OBJ_FILE%"
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Fallo al ensamblar con NASM
    echo Verifica que el archivo .asm sea válido y que NASM esté correctamente instalado.
    pause
    exit /b 1
)

if not exist "%OBJ_FILE%" (
    echo ERROR: Archivo .obj no fue generado: %OBJ_FILE%
    pause
    exit /b 1
)

echo    OK: %OBJ_FILE% generado

REM Verificar tamaño del .obj
for %%A in ("%OBJ_FILE%") do set OBJ_SIZE=%%~zA
if !OBJ_SIZE! EQU 0 (
    echo ERROR: El archivo .obj generado está vacío
    pause
    exit /b 1
)
set /a OBJ_SIZE_KB=!OBJ_SIZE!/1024
echo    Tamaño .obj: !OBJ_SIZE_KB! KB
echo.

REM Paso 2: Intentar linkear con GCC (más optimizado)
where gcc >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo [2/4] Linkeando con GCC (flags optimizados)...
    gcc -nostdlib -s -Wl,--strip-all,--gc-sections,--file-alignment=16,--section-alignment=16,--no-seh "%OBJ_FILE%" -lkernel32 -o "%EXE_FILE%"
    if %ERRORLEVEL% EQU 0 (
        echo    OK: %EXE_FILE% generado con GCC
        goto :check_size
    )
)

REM Paso 2b: Intentar con Zig (alternativa)
where zig >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo [2/4] Linkeando con Zig (flags optimizados)...
    zig build-exe -target x86_64-windows-gnu -O ReleaseSmall -fstrip -fsingle-threaded -fno-unwind-tables -lc -femit-bin="%EXE_FILE%" "%OBJ_FILE%"
    if %ERRORLEVEL% EQU 0 (
        echo    OK: %EXE_FILE% generado con Zig
        goto :check_size
    )
)

REM Paso 2c: Intentar con link.exe (Microsoft)
where link.exe >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo [2/4] Linkeando con Microsoft Linker...
    link "%OBJ_FILE%" kernel32.lib /subsystem:console /entry:main /opt:ref /opt:icf /align:16 /nodefaultlib /out:"%EXE_FILE%"
    if %ERRORLEVEL% EQU 0 (
        echo    OK: %EXE_FILE% generado con Microsoft Linker
        goto :check_size
    )
)

echo ERROR: No se encontró ningún linker disponible (GCC, Zig o link.exe)
pause
exit /b 1

:check_size
REM Verificar que el .exe fue generado
if not exist "%EXE_FILE%" (
    echo ERROR: Archivo .exe no fue generado: %EXE_FILE%
    pause
    exit /b 1
)

REM Verificar tamaño del .exe
for %%A in ("%EXE_FILE%") do set EXE_SIZE=%%~zA
if !EXE_SIZE! EQU 0 (
    echo ERROR: El archivo .exe generado está vacío
    pause
    exit /b 1
)
set /a EXE_SIZE_KB=!EXE_SIZE!/1024
echo    Tamaño .exe (sin UPX): !EXE_SIZE_KB! KB
echo.

REM Paso 3: UPX (compresión extrema) - Opcional
where upx >nul 2>&1
if %ERRORLEVEL% EQU 0 (
    echo [3/4] Comprimiendo con UPX...
    upx --best --lzma "%EXE_FILE%" -o "%EXE_FILE_UPX%" >nul 2>&1
    if %ERRORLEVEL% EQU 0 (
        for %%A in ("%EXE_FILE_UPX%") do set UPX_SIZE=%%~zA
        set /a UPX_SIZE_KB=!UPX_SIZE!/1024
        echo    OK: %EXE_FILE_UPX% generado
        echo    Tamaño .exe (con UPX): !UPX_SIZE_KB! KB
        echo.
        echo    Reducción: !EXE_SIZE_KB! KB -^> !UPX_SIZE_KB! KB (^-%((!EXE_SIZE_KB! - !UPX_SIZE_KB!) * 100 / !EXE_SIZE_KB!)^%)
    ) else (
        echo    ADVERTENCIA: UPX falló, continuando sin compresión
    )
) else (
    echo [3/4] UPX no encontrado, saltando compresión
    echo    (Instala UPX para compresión adicional: https://upx.github.io/)
)
echo.

REM Paso 4: Ejecutar y verificar
echo [4/4] Ejecutando programa...
echo.
echo ========================================
echo   SALIDA DEL PROGRAMA:
echo ========================================
"%EXE_FILE%"
set EXIT_CODE=%ERRORLEVEL%
echo.
echo ========================================
echo   RESULTADOS FINALES:
echo ========================================
echo    Tamaño .asm: (verificar manualmente)
echo    Tamaño .obj: !OBJ_SIZE_KB! KB
echo    Tamaño .exe: !EXE_SIZE_KB! KB
if defined UPX_SIZE_KB (
    echo    Tamaño .exe (UPX): !UPX_SIZE_KB! KB
)
echo    Código de salida: !EXIT_CODE!
echo.
echo ========================================
echo   OBJETIVO ALCANZADO:
echo ========================================
if !EXE_SIZE_KB! LEQ 15 (
    echo    ✅ TAMAÑO OPTIMIZADO: !EXE_SIZE_KB! KB ^< 15 KB
) else (
    echo    ⚠️  Tamaño: !EXE_SIZE_KB! KB (objetivo: ^< 15 KB)
)
if defined UPX_SIZE_KB (
    if !UPX_SIZE_KB! LEQ 10 (
        echo    ✅ TAMAÑO CON UPX: !UPX_SIZE_KB! KB ^< 10 KB
    ) else (
        echo    ⚠️  Tamaño con UPX: !UPX_SIZE_KB! KB (objetivo: ^< 10 KB)
    )
)
echo.
pause

