@echo off
REM Script para ejecutar tests usando Zig como linker
REM Uso: ejecutar_con_zig.bat test_strings_basico.ad

if "%~1"=="" (
    echo ERROR: Debes especificar un archivo de test
    echo Uso: ejecutar_con_zig.bat test_strings_basico.ad
    exit /b 1
)

set TEST_FILE=%~1
set SCRIPT_DIR=%~dp0
cd /d "%SCRIPT_DIR%"

echo ========================================
echo   Ejecutando Test con Zig: %TEST_FILE%
echo ========================================
echo.

REM Buscar compilador ADead en varias ubicaciones posibles
set COMPILER_PATH=
set SEARCH_PATHS=%SCRIPT_DIR%..\..\..\CORE\rust\target\release\adeadc.exe
set SEARCH_PATHS=%SEARCH_PATHS% %SCRIPT_DIR%..\..\..\target\release\adeadc.exe
set SEARCH_PATHS=%SEARCH_PATHS% %SCRIPT_DIR%..\..\..\..\CORE\rust\target\release\adeadc.exe
set SEARCH_PATHS=%SEARCH_PATHS% %CD%\CORE\rust\target\release\adeadc.exe
set SEARCH_PATHS=%SEARCH_PATHS% %CD%\target\release\adeadc.exe

for %%P in (%SEARCH_PATHS%) do (
    if exist "%%P" (
        set COMPILER_PATH=%%P
        goto :found_compiler
    )
)

REM Si no se encuentra, buscar en PATH
where adeadc.exe >nul 2>&1
if %ERRORLEVEL% equ 0 (
    set COMPILER_PATH=adeadc.exe
    goto :found_compiler
)

echo ERROR: Compilador no encontrado
echo Buscado en:
echo   %SCRIPT_DIR%..\..\..\CORE\rust\target\release\adeadc.exe
echo   %SCRIPT_DIR%..\..\..\target\release\adeadc.exe
echo   PATH
echo.
echo Por favor, compila el proyecto primero:
echo   cd CORE\rust
echo   cargo build --release
pause
exit /b 1

:found_compiler
echo [INFO] Compilador encontrado: %COMPILER_PATH%

echo [1/5] Compilando %TEST_FILE%...
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

echo [2/5] Compilacion exitosa: %TEST_FILE:.ad=.asm%
echo.

REM Detectar si el archivo generado es C++ en lugar de ASM
REM Si contiene "// Código C++ generado" o "#include", es C++
findstr /C:"// Código C++ generado" /C:"#include" "%TEST_FILE:.ad=.asm%" >nul 2>&1
if %ERRORLEVEL% equ 0 (
    echo ERROR: El compilador genero codigo C++ en lugar de ASM
    echo Esto significa que la compilacion C++ a ASM fallo
    echo.
    echo Revisando el archivo generado:
    type "%TEST_FILE:.ad=.asm%" | more
    echo.
    echo Por favor, verifica que Clang/GCC este instalado y funcionando
    pause
    exit /b 1
)

REM Detectar formato ASM (GAS o NASM)
REM GAS usa .text, .globl, .intel_syntax
REM NASM usa section .text, global
set IS_GAS=0
findstr /C:".text" /C:".globl" /C:".intel_syntax" "%TEST_FILE:.ad=.asm%" >nul 2>&1
if %ERRORLEVEL% equ 0 (
    set IS_GAS=1
    echo [INFO] Detectado formato GAS (GNU Assembler)
) else (
    echo [INFO] Detectado formato NASM
)

if %IS_GAS% equ 1 (
    REM Usar GAS (as) o Clang para ensamblar formato GAS
    REM Primero intentar con Clang (que puede ensamblar directamente)
    set ASSEMBLER_FOUND=0
    
    REM Buscar Clang en ubicaciones comunes
    set CLANG_PATH=
    where clang >nul 2>&1
    if %ERRORLEVEL% equ 0 (
        set CLANG_PATH=clang
        set ASSEMBLER_FOUND=1
    ) else if exist "C:\Program Files\LLVM\bin\clang.exe" (
        set CLANG_PATH=C:\Program Files\LLVM\bin\clang.exe
        set ASSEMBLER_FOUND=1
    )
    
    if %ASSEMBLER_FOUND% equ 1 (
        echo [3/5] Ensamblando con Clang (formato GAS)...
        "%CLANG_PATH%" -c -target x86_64-pc-windows-msvc -o "%TEST_FILE:.ad=.obj%" "%TEST_FILE:.ad=.asm%"
        if %ERRORLEVEL% equ 0 (
            goto :assembled_ok
        )
    )
    
    REM Si Clang falla, intentar con GAS (as)
    where as >nul 2>&1
    if %ERRORLEVEL% equ 0 (
        echo [3/5] Ensamblando con GAS (as)...
        as --64 -o "%TEST_FILE:.ad=.obj%" "%TEST_FILE:.ad=.asm%"
        if %ERRORLEVEL% equ 0 (
            goto :assembled_ok
        )
    )
    
    echo ERROR: No se pudo ensamblar el formato GAS
    echo Intentado: Clang y GAS (as)
    echo Opciones:
    echo   1. Instalar Clang/LLVM (recomendado)
    echo   2. Instalar MinGW/MSYS2 que incluye GAS
    pause
    exit /b 1
    
    :assembled_ok
) else (
    REM Usar NASM para ensamblar
    where nasm >nul 2>&1
    if %ERRORLEVEL% neq 0 (
        echo ERROR: NASM no encontrado en PATH
        echo Por favor, agrega NASM al PATH o instala NASM
        pause
        exit /b 1
    )
    
    echo [3/5] Ensamblando con NASM...
    nasm -f win64 "%TEST_FILE:.ad=.asm%" -o "%TEST_FILE:.ad=.obj%"
    
    if %ERRORLEVEL% neq 0 (
        echo ERROR: Ensamblado fallo
        pause
        exit /b 1
    )
)

if not exist "%TEST_FILE:.ad=.obj%" (
    echo ERROR: Archivo OBJ no fue generado
    pause
    exit /b 1
)

:assembled_ok
echo [4/5] Ensamblado exitoso: %TEST_FILE:.ad=.obj%
echo.

REM Verificar Zig
where zig >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo ERROR: Zig no encontrado en PATH
    echo Por favor, agrega Zig al PATH
    pause
    exit /b 1
)

echo [5/5] Linkeando con Zig...
REM Zig puede linkear objetos .obj directamente
REM Usamos -lc para linkear con C runtime (necesario para Windows)
zig build-exe "%TEST_FILE:.ad=.obj%" -target x86_64-windows -lc -o "%TEST_FILE:.ad=.exe%"

if %ERRORLEVEL% neq 0 (
    echo ERROR: Linkeo con Zig fallo
    echo Intentando sin -lc...
    zig build-exe "%TEST_FILE:.ad=.obj%" -target x86_64-windows -o "%TEST_FILE:.ad=.exe%"
    if %ERRORLEVEL% neq 0 (
        echo ERROR: Linkeo fallo completamente
        pause
        exit /b 1
    )
)

if not exist "%TEST_FILE:.ad=.exe%" (
    echo ERROR: Ejecutable no fue generado
    pause
    exit /b 1
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

