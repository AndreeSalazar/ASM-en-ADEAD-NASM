@echo off
REM Script de diagnóstico para verificar el entorno
echo ========================================
echo   Diagnostico del Entorno de Testing
echo ========================================
echo.

set SCRIPT_DIR=%~dp0
cd /d "%SCRIPT_DIR%"

echo [1/5] Verificando directorio actual...
echo Directorio: %CD%
echo.

echo [2/5] Verificando archivos de test...
if exist "test_strings_basico.ad" (
    echo   [OK] test_strings_basico.ad encontrado
) else (
    echo   [ERROR] test_strings_basico.ad NO encontrado
)
echo.

echo [3/5] Verificando compilador...
set COMPILER_PATH=%CD%\..\..\..\CORE\rust\target\release\adeadc.exe
if exist "%COMPILER_PATH%" (
    echo   [OK] Compilador encontrado: %COMPILER_PATH%
) else (
    echo   [ERROR] Compilador NO encontrado en: %COMPILER_PATH%
    echo   Buscando en ubicaciones alternativas...
    
    REM Intentar path absoluto desde el workspace root
    for %%I in ("%CD%") do set WORKSPACE_ROOT=%%~dpI
    set COMPILER_PATH=%WORKSPACE_ROOT%CORE\rust\target\release\adeadc.exe
    if exist "%COMPILER_PATH%" (
        echo   [OK] Compilador encontrado en: %COMPILER_PATH%
    ) else (
        echo   [ERROR] Compilador tampoco encontrado en: %COMPILER_PATH%
    )
)
echo.

echo [4/5] Verificando herramientas externas...
where nasm >nul 2>&1
if %ERRORLEVEL%==0 (
    echo   [OK] NASM encontrado
    nasm -v
) else (
    echo   [ADVERTENCIA] NASM no encontrado en PATH
)
echo.

where gcc >nul 2>&1
if %ERRORLEVEL%==0 (
    echo   [OK] GCC encontrado
    gcc --version | findstr /C:"gcc"
) else (
    echo   [ADVERTENCIA] GCC no encontrado en PATH
)
echo.

echo [5/5] Verificando PowerShell...
powershell.exe -Command "Write-Host 'PowerShell funciona'" 2>nul
if %ERRORLEVEL%==0 (
    echo   [OK] PowerShell funciona
) else (
    echo   [ERROR] PowerShell no funciona correctamente
)
echo.

echo ========================================
echo   Diagnostico completado
echo ========================================
pause

