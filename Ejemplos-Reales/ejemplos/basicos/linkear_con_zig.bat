@echo off
REM Script para linkear objetos .obj con Zig
REM Uso: linkear_con_zig.bat archivo.obj [archivo.exe]

if "%~1"=="" (
    echo ERROR: Debes especificar un archivo .obj
    echo Uso: linkear_con_zig.bat archivo.obj [archivo.exe]
    exit /b 1
)

set OBJ_FILE=%~1
set EXE_FILE=%~2

if "%EXE_FILE%"=="" (
    set EXE_FILE=%~n1.exe
)

echo ========================================
echo   Linkeando con Zig
echo ========================================
echo.
echo Archivo objeto: %OBJ_FILE%
echo Archivo ejecutable: %EXE_FILE%
echo.

REM Verificar Zig
where zig >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo ERROR: Zig no encontrado en PATH
    echo Por favor, agrega Zig al PATH
    pause
    exit /b 1
)

REM Verificar que el archivo .obj existe
if not exist "%OBJ_FILE%" (
    echo ERROR: Archivo no encontrado: %OBJ_FILE%
    pause
    exit /b 1
)

echo Linkeando...
REM Intentar con C runtime primero
zig build-exe "%OBJ_FILE%" -target x86_64-windows -lc -o "%EXE_FILE%"

if %ERRORLEVEL% neq 0 (
    echo Advertencia: Linkeo con -lc fallo, intentando sin...
    zig build-exe "%OBJ_FILE%" -target x86_64-windows -o "%EXE_FILE%"
    if %ERRORLEVEL% neq 0 (
        echo ERROR: Linkeo fallo
        pause
        exit /b 1
    )
)

if exist "%EXE_FILE%" (
    echo.
    echo [OK] Ejecutable generado: %EXE_FILE%
    echo.
    echo Puedes ejecutarlo con:
    echo   %EXE_FILE%
) else (
    echo ERROR: Ejecutable no fue generado
    pause
    exit /b 1
)

pause

