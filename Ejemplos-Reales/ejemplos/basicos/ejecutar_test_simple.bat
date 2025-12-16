@echo off
REM Script simplificado para ejecutar un test
REM Uso: ejecutar_test_simple.bat test_strings_basico.ad

if "%~1"=="" (
    echo ERROR: Debes especificar un archivo de test
    echo Uso: ejecutar_test_simple.bat test_strings_basico.ad
    exit /b 1
)

set TEST_FILE=%~1
set SCRIPT_DIR=%~dp0
cd /d "%SCRIPT_DIR%"

echo ========================================
echo   Ejecutando Test: %TEST_FILE%
echo ========================================
echo.

REM Buscar compilador en múltiples ubicaciones
set COMPILER_PATH=%SCRIPT_DIR%..\..\..\CORE\rust\target\release\adeadc.exe
if not exist "%COMPILER_PATH%" (
    REM Intentar desde CORE directamente
    set COMPILER_PATH=%SCRIPT_DIR%..\..\..\CORE\target\release\adeadc.exe
    if not exist "%COMPILER_PATH%" (
        REM Intentar path absoluto desde workspace root
        for %%I in ("%SCRIPT_DIR%") do set WORKSPACE_ROOT=%%~dpI
        set COMPILER_PATH=%WORKSPACE_ROOT%CORE\rust\target\release\adeadc.exe
        if not exist "%COMPILER_PATH%" (
            set COMPILER_PATH=%WORKSPACE_ROOT%CORE\target\release\adeadc.exe
            if not exist "%COMPILER_PATH%" (
                echo ERROR: Compilador no encontrado en ninguna ubicacion
                echo Buscado en:
                echo   %SCRIPT_DIR%..\..\..\CORE\rust\target\release\adeadc.exe
                echo   %SCRIPT_DIR%..\..\..\CORE\target\release\adeadc.exe
                echo   %WORKSPACE_ROOT%CORE\rust\target\release\adeadc.exe
                echo   %WORKSPACE_ROOT%CORE\target\release\adeadc.exe
                echo.
                echo Por favor, compila el proyecto primero:
                echo   cd CORE\rust
                echo   cargo build --release
                echo.
                echo O si compilaste desde CORE:
                echo   cd CORE
                echo   cargo build --release
                pause
                exit /b 1
            )
        )
    )
)
echo Compilador encontrado en: %COMPILER_PATH%

echo [1/4] Compilando %TEST_FILE%...
"%COMPILER_PATH%" compile "%TEST_FILE%" --backend auto -o "%TEST_FILE:.ad=.asm%"

if %ERRORLEVEL% neq 0 (
    echo ERROR: Compilacion fallo
    pause
    exit /b 1
)

if not exist "%TEST_FILE:.ad=.asm%" (
    echo ERROR: Archivo ASM no fue generado
    pause
    exit /b 1
)

echo [2/4] Compilacion exitosa: %TEST_FILE:.ad=.asm%
echo.

REM Verificar NASM
where nasm >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo ADVERTENCIA: NASM no encontrado, no se puede ensamblar
    echo Archivo ASM generado: %TEST_FILE:.ad=.asm%
    pause
    exit /b 0
)

echo [3/4] Ensamblando...
nasm -f win64 "%TEST_FILE:.ad=.asm%" -o "%TEST_FILE:.ad=.obj%"

if %ERRORLEVEL% neq 0 (
    echo ERROR: Ensamblado fallo
    pause
    exit /b 1
)

echo [4/4] Ensamblado exitoso: %TEST_FILE:.ad=.obj%
echo.

REM Verificar GCC o Zig para linkear
set LINKER_FOUND=0
set LINKER_CMD=

REM Intentar GCC primero
where gcc >nul 2>&1
if %ERRORLEVEL%==0 (
    set LINKER_FOUND=1
    set LINKER_CMD=gcc "%TEST_FILE:.ad=.obj%" -o "%TEST_FILE:.ad=.exe%"
    echo [5/5] Linkeando con GCC...
) else (
    REM Intentar Zig como alternativa
    where zig >nul 2>&1
    if %ERRORLEVEL%==0 (
        set LINKER_FOUND=1
        set LINKER_CMD=zig build-exe "%TEST_FILE:.ad=.obj%" -target x86_64-windows -lc -o "%TEST_FILE:.ad=.exe%"
        echo [5/5] Linkeando con Zig...
    )
)

if %LINKER_FOUND%==0 (
    echo ADVERTENCIA: Ni GCC ni Zig encontrados, no se puede linkear
    echo Archivo OBJ generado: %TEST_FILE:.ad=.obj%
    echo.
    echo Opciones:
    echo   1. Instalar GCC (MinGW-w64)
    echo   2. Usar Zig (ya instalado, agregar al PATH)
    echo   3. Linkear manualmente con: zig build-exe %TEST_FILE:.ad=.obj% -target x86_64-windows -lc
    pause
    exit /b 0
)

REM Ejecutar comando de linker
%LINKER_CMD%

if %ERRORLEVEL% neq 0 (
    REM Si Zig falla con -lc, intentar sin
    if "%LINKER_CMD:zig=%" neq "%LINKER_CMD%" (
        echo Advertencia: Linkeo con -lc fallo, intentando sin...
        zig build-exe "%TEST_FILE:.ad=.obj%" -target x86_64-windows -o "%TEST_FILE:.ad=.exe%"
        if %ERRORLEVEL% neq 0 (
            echo ERROR: Linkeo fallo completamente
            pause
            exit /b 1
        )
    ) else (
        echo ERROR: Linkeo fallo
        pause
        exit /b 1
    )
)

echo [5/5] Linkeo exitoso: %TEST_FILE:.ad=.exe%
echo.

echo ========================================
echo   Ejecutando programa...
echo ========================================
echo.

"%TEST_FILE:.ad=.exe%"

echo.
echo ========================================
echo   Programa completado
echo ========================================
pause

