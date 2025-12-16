@echo off
REM Script para compilar manualmente un test
REM Uso: COMPILAR-MANUAL.bat test_strings_basico.ad

if "%~1"=="" (
    echo ERROR: Debes especificar un archivo de test
    echo Uso: COMPILAR-MANUAL.bat test_strings_basico.ad
    exit /b 1
)

set TEST_FILE=%~1
set SCRIPT_DIR=%~dp0
cd /d "%SCRIPT_DIR%"

echo ========================================
echo   Compilacion Manual
echo ========================================
echo.

REM Buscar compilador
call BUSCAR-COMPILADOR.bat >nul 2>&1
if "%COMPILER_PATH%"=="" (
    echo ERROR: Compilador no encontrado. Ejecuta BUSCAR-COMPILADOR.bat primero.
    pause
    exit /b 1
)

echo Compilando %TEST_FILE%...
echo Compilador: %COMPILER_PATH%
echo.

"%COMPILER_PATH%" compile "%TEST_FILE%" --backend auto -o "%TEST_FILE:.ad=.asm%"

if %ERRORLEVEL% neq 0 (
    echo.
    echo ERROR: Compilacion fallo
    pause
    exit /b 1
)

if exist "%TEST_FILE:.ad=.asm%" (
    echo.
    echo [OK] Archivo ASM generado: %TEST_FILE:.ad=.asm%
    echo.
    echo Puedes ensamblarlo con:
    echo   nasm -f win64 %TEST_FILE:.ad=.asm% -o %TEST_FILE:.ad=.obj%
    echo   gcc %TEST_FILE:.ad=.obj% -o %TEST_FILE:.ad=.exe%
) else (
    echo.
    echo ERROR: Archivo ASM no fue generado
)

pause

