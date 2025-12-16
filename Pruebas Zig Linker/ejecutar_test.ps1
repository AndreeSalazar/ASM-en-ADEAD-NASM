# Script para probar linking con Zig
$ErrorActionPreference = "Stop"

$workspace = "C:\Users\andre\OneDrive\Documentos\ASM-en-ADEAD-NASM"
Set-Location $workspace

Write-Host "`n=== PRUEBA: Linking con Zig ===" -ForegroundColor Cyan
Write-Host ""

# Buscar compilador
$compiler_path = Join-Path $workspace "CORE\rust\target\release\adeadc.exe"
if (-not (Test-Path $compiler_path)) {
    Write-Host "ERROR: Compilador no encontrado en: $compiler_path" -ForegroundColor Red
    Write-Host "Por favor compila primero: cargo build --release --bin adeadc" -ForegroundColor Yellow
    exit 1
}

Write-Host "Compilador encontrado: $compiler_path" -ForegroundColor Green

# Test file
$test_file = Join-Path $workspace "Pruebas Zig Linker\test_simple.ad"
if (-not (Test-Path $test_file)) {
    Write-Host "ERROR: Archivo de test no encontrado: $test_file" -ForegroundColor Red
    exit 1
}

Write-Host "`n[1/3] Compilando y linkeando con Zig..." -ForegroundColor Yellow
Write-Host "   Usando backend NASM directo (no requiere compilador C++)" -ForegroundColor Gray
$build_output = & $compiler_path build $test_file --linker zig --backend nasm 2>&1
$build_exit = $LASTEXITCODE

if ($build_exit -ne 0) {
    Write-Host "ERROR: Build falló" -ForegroundColor Red
    $build_output | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
    exit 1
}

$build_output | ForEach-Object { Write-Host "  $_" -ForegroundColor Gray }

# Verificar ejecutable
$exe_file = $test_file -replace "\.ad$", ".exe"
if (-not (Test-Path $exe_file)) {
    Write-Host "ERROR: Ejecutable no fue generado: $exe_file" -ForegroundColor Red
    Write-Host "Verificando si existe en otra ubicación..." -ForegroundColor Yellow
    
    # Intentar buscar el ejecutable en el directorio del test
    $test_dir = Split-Path -Parent $test_file
    $exe_in_dir = Join-Path $test_dir (Split-Path -Leaf $exe_file)
    if (Test-Path $exe_in_dir) {
        $exe_file = $exe_in_dir
        Write-Host "Ejecutable encontrado en: $exe_file" -ForegroundColor Green
    } else {
        exit 1
    }
}

Write-Host "`n[2/3] Ejecutando programa..." -ForegroundColor Yellow

# Verificar que el ejecutable existe
if (-not (Test-Path $exe_file)) {
    Write-Host "ERROR: Ejecutable no encontrado: $exe_file" -ForegroundColor Red
    exit 1
}

# Obtener la ruta absoluta y normalizar
$exe_path = Resolve-Path $exe_file -ErrorAction SilentlyContinue
if (-not $exe_path) {
    $exe_path = $exe_file
}

# Ejecutar el programa (usar ruta absoluta o relativa correctamente)
try {
    $exec_output = & $exe_path 2>&1
    $exec_exit = $LASTEXITCODE
} catch {
    Write-Host "ERROR al ejecutar: $_" -ForegroundColor Red
    exit 1
}

if ($exec_output) {
    Write-Host "Salida:" -ForegroundColor Cyan
    $exec_output | ForEach-Object { Write-Host "  $_" -ForegroundColor White }
}

Write-Host "`n[3/3] Código de salida: $exec_exit" -ForegroundColor $(if ($exec_exit -eq 0) { "Green" } else { "Yellow" })

Write-Host "`n=== ✅ PRUEBA COMPLETADA ===" -ForegroundColor Green
Write-Host "Ejecutable: $exe_file" -ForegroundColor Gray

