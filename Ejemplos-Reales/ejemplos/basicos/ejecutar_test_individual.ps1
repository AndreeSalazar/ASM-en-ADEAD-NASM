# Script para ejecutar un test individual
# Uso: .\ejecutar_test_individual.ps1 test_strings_basico.ad

param(
    [Parameter(Mandatory=$true)]
    [string]$TestFile
)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Ejecutando Test: $TestFile" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Resolver path del compilador de forma más robusta
$script_dir = Split-Path -Parent $MyInvocation.MyCommand.Path
$workspace_root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $script_dir))

# Intentar múltiples ubicaciones posibles
$possible_paths = @(
    (Join-Path $workspace_root "CORE\rust\target\release\adeadc.exe"),
    (Join-Path $workspace_root "CORE\target\release\adeadc.exe"),
    "..\..\..\CORE\rust\target\release\adeadc.exe",
    "..\..\..\CORE\target\release\adeadc.exe"
)

$compiler_path = $null
foreach ($path in $possible_paths) {
    if (Test-Path $path) {
        $compiler_path = $path
        break
    }
}

$test_path = Join-Path (Get-Location) $TestFile

if (-not (Test-Path $test_path)) {
    Write-Host "ERROR: Archivo no encontrado: $test_path" -ForegroundColor Red
    exit 1
}

if ($null -eq $compiler_path) {
    Write-Host "ERROR: Compilador no encontrado en ninguna ubicacion" -ForegroundColor Red
    Write-Host "Buscado en:" -ForegroundColor Yellow
    foreach ($path in $possible_paths) {
        Write-Host "  $path" -ForegroundColor Gray
    }
    Write-Host ""
    Write-Host "Por favor, compila el proyecto primero:" -ForegroundColor Yellow
    Write-Host "  cd CORE\rust" -ForegroundColor Yellow
    Write-Host "  cargo build --release" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "O si compilaste desde CORE:" -ForegroundColor Yellow
    Write-Host "  cd CORE" -ForegroundColor Yellow
    Write-Host "  cargo build --release" -ForegroundColor Yellow
    exit 1
}

Write-Host "Compilador encontrado en: $compiler_path" -ForegroundColor Green

# Compilar
$asm_file = $TestFile -replace "\.ad$", ".asm"
Write-Host "[1/4] Compilando..." -ForegroundColor Yellow

# Capturar tanto stdout como stderr para mostrar errores
$compile_output = & $compiler_path compile $test_path --backend auto -o $asm_file 2>&1
$compile_exit_code = $LASTEXITCODE

if ($compile_exit_code -ne 0) {
    Write-Host "ERROR: Compilación falló" -ForegroundColor Red
    Write-Host "Detalles del error:" -ForegroundColor Yellow
    $compile_output | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
    exit 1
}

if (-not (Test-Path $asm_file)) {
    Write-Host "ERROR: Archivo ASM no fue generado: $asm_file" -ForegroundColor Red
    Write-Host "Output del compilador:" -ForegroundColor Yellow
    $compile_output | ForEach-Object { Write-Host "  $_" -ForegroundColor Gray }
    exit 1
}

Write-Host "[2/4] Compilación exitosa: $asm_file" -ForegroundColor Green

# Verificar NASM y GCC
$nasm_path = "nasm"
$gcc_path = "gcc"

if (-not (Get-Command $nasm_path -ErrorAction SilentlyContinue)) {
    Write-Host "ADVERTENCIA: NASM no encontrado, no se puede ensamblar" -ForegroundColor Yellow
    Write-Host "Archivo ASM generado: $asm_file" -ForegroundColor Cyan
    Write-Host "Puedes ensamblarlo manualmente con:" -ForegroundColor Yellow
    Write-Host "  nasm -f win64 $asm_file -o $($asm_file -replace '\.asm$', '.obj')" -ForegroundColor Gray
    exit 0
}

if (-not (Get-Command $gcc_path -ErrorAction SilentlyContinue)) {
    Write-Host "ADVERTENCIA: GCC no encontrado, no se puede linkear" -ForegroundColor Yellow
    Write-Host "Archivo ASM generado: $asm_file" -ForegroundColor Cyan
    exit 0
}

# Ensamblar
$obj_file = $asm_file -replace "\.asm$", ".obj"
Write-Host "[3/4] Ensamblando..." -ForegroundColor Yellow
$nasm_output = & $nasm_path -f win64 $asm_file -o $obj_file 2>&1

if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Ensamblado falló" -ForegroundColor Red
    Write-Host "Detalles del error:" -ForegroundColor Yellow
    $nasm_output | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
    exit 1
}

if (-not (Test-Path $obj_file)) {
    Write-Host "ERROR: Archivo objeto no fue generado: $obj_file" -ForegroundColor Red
    exit 1
}

Write-Host "[4/4] Ensamblado exitoso: $obj_file" -ForegroundColor Green

# Linkear
$exe_file = $TestFile -replace "\.ad$", ".exe"
Write-Host "[5/5] Linkeando..." -ForegroundColor Yellow
$gcc_output = & $gcc_path $obj_file -o $exe_file 2>&1

if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Linkeo falló" -ForegroundColor Red
    Write-Host "Detalles del error:" -ForegroundColor Yellow
    $gcc_output | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
    exit 1
}

if (-not (Test-Path $exe_file)) {
    Write-Host "ERROR: Ejecutable no fue generado: $exe_file" -ForegroundColor Red
    exit 1
}

Write-Host "[5/5] Linkeo exitoso: $exe_file" -ForegroundColor Green

# Ejecutar
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Ejecutando programa..." -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$exec_output = & ".\$exe_file" 2>&1
$exec_exit_code = $LASTEXITCODE

# Mostrar output
if ($exec_output) {
    $exec_output | ForEach-Object { Write-Host $_ -ForegroundColor White }
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Programa completado (exit code: $exec_exit_code)" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

