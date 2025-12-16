# Script de diagnostico completo
$ErrorActionPreference = "Stop"

$workspace = "C:\Users\andre\OneDrive\Documentos\ASM-en-ADEAD-NASM"
Set-Location $workspace

Write-Host "`n=== DIAGNOSTICO COMPLETO ===" -ForegroundColor Cyan
Write-Host ""

# 1. Verificar compilador
$compiler_path = Join-Path $workspace "CORE\rust\target\release\adeadc.exe"
Write-Host "[1/6] Verificando compilador..." -ForegroundColor Yellow
if (Test-Path $compiler_path) {
    Write-Host "  [OK] Compilador encontrado: $compiler_path" -ForegroundColor Green
} else {
    Write-Host "  [ERROR] Compilador NO encontrado" -ForegroundColor Red
    exit 1
}

# 2. Verificar Zig
Write-Host "`n[2/6] Verificando Zig..." -ForegroundColor Yellow
$zig_check = & zig version 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "  [OK] Zig encontrado: $($zig_check[0])" -ForegroundColor Green
} else {
    Write-Host "  [ERROR] Zig NO encontrado" -ForegroundColor Red
    exit 1
}

# 3. Verificar NASM
Write-Host "`n[3/6] Verificando NASM..." -ForegroundColor Yellow
$nasm_check = & nasm --version 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "  [OK] NASM encontrado: $($nasm_check[0])" -ForegroundColor Green
} else {
    Write-Host "  [ERROR] NASM NO encontrado" -ForegroundColor Red
    exit 1
}

# 4. Verificar archivo de test
$test_file = Join-Path $workspace "Pruebas Zig Linker\test_simple.ad"
Write-Host "`n[4/6] Verificando archivo de test..." -ForegroundColor Yellow
if (Test-Path $test_file) {
    Write-Host "  [OK] Archivo encontrado: $test_file" -ForegroundColor Green
    Write-Host "  Contenido:" -ForegroundColor Gray
    Get-Content $test_file | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
} else {
    Write-Host "  [ERROR] Archivo NO encontrado" -ForegroundColor Red
    exit 1
}

# 5. Compilar y linkear
Write-Host "`n[5/6] Compilando y linkeando..." -ForegroundColor Yellow
$build_output = & $compiler_path build $test_file --linker zig --backend nasm 2>&1
$build_exit = $LASTEXITCODE

if ($build_exit -ne 0) {
    Write-Host "  [ERROR] Build fallo" -ForegroundColor Red
    $build_output | ForEach-Object { Write-Host "    $_" -ForegroundColor Red }
    exit 1
}

Write-Host "  [OK] Build exitoso" -ForegroundColor Green
$build_output | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }

# 6. Buscar ejecutable
Write-Host "`n[6/6] Buscando ejecutable..." -ForegroundColor Yellow
$exe_expected = $test_file -replace "\.ad$", ".exe"
$exe_found = $false

# Buscar en ubicacion esperada
if (Test-Path $exe_expected) {
    Write-Host "  [OK] Ejecutable encontrado en ubicacion esperada: $exe_expected" -ForegroundColor Green
    $exe_file = $exe_expected
    $exe_found = $true
} else {
    Write-Host "  [WARN] Ejecutable NO encontrado en: $exe_expected" -ForegroundColor Yellow
    
    # Buscar en directorio del test
    $test_dir = Split-Path -Parent $test_file
    $exe_in_dir = Join-Path $test_dir (Split-Path -Leaf $exe_expected)
    if (Test-Path $exe_in_dir) {
        Write-Host "  [OK] Ejecutable encontrado en directorio del test: $exe_in_dir" -ForegroundColor Green
        $exe_file = $exe_in_dir
        $exe_found = $true
    } else {
        # Buscar en todo el workspace
        Write-Host "  [INFO] Buscando en todo el workspace..." -ForegroundColor Yellow
        $found_exes = Get-ChildItem -Path $workspace -Filter "test_simple.exe" -Recurse -ErrorAction SilentlyContinue | Where-Object { $_.LastWriteTime -gt (Get-Date).AddMinutes(-5) }
        if ($found_exes) {
            $exe_file = $found_exes[0].FullName
            Write-Host "  [OK] Ejecutable encontrado: $exe_file" -ForegroundColor Green
            $exe_found = $true
        } else {
            Write-Host "  [ERROR] Ejecutable NO encontrado en ninguna ubicacion" -ForegroundColor Red
        }
    }
}

if ($exe_found) {
    Write-Host "`n=== Ejecutando programa ===" -ForegroundColor Cyan
    $exe_path = Resolve-Path $exe_file -ErrorAction SilentlyContinue
    if (-not $exe_path) {
        $exe_path = $exe_file
    }
    
    Write-Host "Ejecutando: $exe_path" -ForegroundColor Yellow
    try {
        $exec_output = & $exe_path 2>&1
        $exec_exit = $LASTEXITCODE
        
        Write-Host "`nSalida del programa:" -ForegroundColor Cyan
        if ($exec_output) {
            $exec_output | ForEach-Object { Write-Host "  $_" -ForegroundColor White }
        } else {
            Write-Host "  (sin salida visible)" -ForegroundColor Gray
        }
        
        Write-Host "`nCodigo de salida: $exec_exit" -ForegroundColor $(if ($exec_exit -eq 0) { "Green" } else { "Red" })
    } catch {
        Write-Host "ERROR al ejecutar: $_" -ForegroundColor Red
    }
}

Write-Host "`n=== DIAGNOSTICO COMPLETADO ===" -ForegroundColor Green
