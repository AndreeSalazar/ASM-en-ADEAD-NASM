# ADead - Build TINY con Zig (PowerShell)
# Optimización máxima de tamaño

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$asmFile = Join-Path $scriptDir "test_simple.asm"
$objFile = Join-Path $scriptDir "test_simple.obj"
$exeFile = Join-Path $scriptDir "test_simple_tiny_zig.exe"
$exeFileUpx = Join-Path $scriptDir "test_simple_tiny_zig_upx.exe"

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  ADead - Build TINY (Zig Optimizado)" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# Verificar NASM
if (-not (Get-Command nasm -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: NASM no encontrado" -ForegroundColor Red
    exit 1
}

# Verificar Zig
if (-not (Get-Command zig -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: Zig no encontrado" -ForegroundColor Red
    exit 1
}

# Paso 1: Ensamblar
Write-Host "[1/4] Ensamblando..." -ForegroundColor Yellow
& nasm -f win64 $asmFile -o $objFile
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Fallo al ensamblar" -ForegroundColor Red
    exit 1
}

$objSize = (Get-Item $objFile).Length
$objSizeKB = [math]::Round($objSize / 1KB, 2)
Write-Host "  OK: $objFile generado ($objSizeKB KB)`n" -ForegroundColor Green

# Paso 2: Linkear con Zig (flags optimizados)
Write-Host "[2/4] Linkeando con Zig (flags optimizados)..." -ForegroundColor Yellow
& zig build-exe `
    -target x86_64-windows-gnu `
    -O ReleaseSmall `
    -fstrip `
    -fsingle-threaded `
    -fno-unwind-tables `
    -lc `
    -femit-bin=$exeFile `
    $objFile

if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Fallo al linkear con Zig" -ForegroundColor Red
    exit 1
}

$exeSize = (Get-Item $exeFile).Length
$exeSizeKB = [math]::Round($exeSize / 1KB, 2)
Write-Host "  OK: $exeFile generado ($exeSizeKB KB)`n" -ForegroundColor Green

# Paso 3: UPX (opcional)
if (Get-Command upx -ErrorAction SilentlyContinue) {
    Write-Host "[3/4] Comprimiendo con UPX..." -ForegroundColor Yellow
    & upx --best --lzma $exeFile -o $exeFileUpx 2>&1 | Out-Null
    
    if ($LASTEXITCODE -eq 0 -and (Test-Path $exeFileUpx)) {
        $upxSize = (Get-Item $exeFileUpx).Length
        $upxSizeKB = [math]::Round($upxSize / 1KB, 2)
        $reduction = [math]::Round((($exeSize - $upxSize) / $exeSize) * 100, 1)
        Write-Host "  OK: $exeFileUpx generado ($upxSizeKB KB)" -ForegroundColor Green
        Write-Host "  Reducción: $reduction%`n" -ForegroundColor Cyan
    } else {
        Write-Host "  ADVERTENCIA: UPX falló, continuando sin compresión`n" -ForegroundColor Yellow
    }
} else {
    Write-Host "[3/4] UPX no encontrado, saltando compresión`n" -ForegroundColor Yellow
}

# Paso 4: Ejecutar
Write-Host "[4/4] Ejecutando programa...`n" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  SALIDA DEL PROGRAMA:" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

$output = & $exeFile 2>&1
Write-Host $output -ForegroundColor White
$exitCode = $LASTEXITCODE

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  RESULTADOS FINALES:" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Tamaño .obj: $objSizeKB KB" -ForegroundColor Yellow
Write-Host "  Tamaño .exe: $exeSizeKB KB" -ForegroundColor Yellow
if (Test-Path $exeFileUpx) {
    Write-Host "  Tamaño .exe (UPX): $upxSizeKB KB" -ForegroundColor Yellow
}
Write-Host "  Código de salida: $exitCode" -ForegroundColor Yellow
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  OBJETIVO ALCANZADO:" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

if ($exeSizeKB -le 15) {
    Write-Host "  ✅ TAMAÑO OPTIMIZADO: $exeSizeKB KB < 15 KB" -ForegroundColor Green
} else {
    Write-Host "  ⚠️  Tamaño: $exeSizeKB KB (objetivo: < 15 KB)" -ForegroundColor Yellow
}

if (Test-Path $exeFileUpx) {
    if ($upxSizeKB -le 10) {
        Write-Host "  ✅ TAMAÑO CON UPX: $upxSizeKB KB < 10 KB" -ForegroundColor Green
    } else {
        Write-Host "  ⚠️  Tamaño con UPX: $upxSizeKB KB (objetivo: < 10 KB)" -ForegroundColor Yellow
    }
}

Write-Host ""

