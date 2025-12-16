# Script para compilar y comparar ASM sucio vs limpio
# PowerShell script para Windows

Write-Host "Compilando ASM Sucio vs Limpio..." -ForegroundColor Cyan
Write-Host ""

$ErrorActionPreference = "Stop"

# Rutas
$dirty_asm = "test_array_CLANG_dirty.asm"
$clean_asm = "test_array_CLANG_cleaned_extreme.asm"
$dirty_obj = "test_array_dirty.obj"
$clean_obj = "test_array_clean.obj"
$dirty_exe = "test_array_dirty.exe"
$clean_exe = "test_array_clean.exe"

# Verificar que NASM esta disponible
try {
    $nasm_version = nasm -v 2>&1
    Write-Host "NASM encontrado: $nasm_version" -ForegroundColor Green
} catch {
    Write-Host "ERROR: NASM no encontrado. Instalalo desde https://www.nasm.us/" -ForegroundColor Red
    exit 1
}

# Buscar GCC en rutas comunes
$gcc_path = $null
$gcc_paths = @(
    "gcc",
    "C:\msys64\mingw64\bin\gcc.exe",
    "C:\msys64\usr\bin\gcc.exe",
    "C:\MinGW\bin\gcc.exe",
    "C:\Program Files\mingw-w64\bin\gcc.exe"
)

foreach ($path in $gcc_paths) {
    try {
        $result = & $path --version 2>&1 | Select-Object -First 1
        if ($LASTEXITCODE -eq 0 -or $result) {
            $gcc_path = $path
            Write-Host "GCC encontrado: $result" -ForegroundColor Green
            break
        }
    } catch {
        continue
    }
}

if (-not $gcc_path) {
    Write-Host "ADVERTENCIA: GCC no encontrado. Solo compararemos archivos .obj" -ForegroundColor Yellow
    Write-Host "Para crear ejecutables, instala GCC desde MSYS2 o MinGW" -ForegroundColor Yellow
}

Write-Host ""

# Funcion para obtener tamano de archivo
function Get-FileSize {
    param($file)
    if (Test-Path $file) {
        $size = (Get-Item $file).Length
        return $size
    }
    return 0
}

# Compilar ASM sucio
Write-Host "Compilando ASM Sucio..." -ForegroundColor Yellow
Write-Host "Ensamblando: $dirty_asm -> $dirty_obj"
nasm -f win64 -o $dirty_obj $dirty_asm
if ($LASTEXITCODE -ne 0) {
    Write-Host "Error al ensamblar $dirty_asm" -ForegroundColor Red
    exit 1
}
Write-Host "OK: Objeto creado" -ForegroundColor Green

Write-Host ""

# Compilar ASM limpio
Write-Host "Compilando ASM Limpio..." -ForegroundColor Yellow
Write-Host "Ensamblando: $clean_asm -> $clean_obj"
nasm -f win64 -o $clean_obj $clean_asm
if ($LASTEXITCODE -ne 0) {
    Write-Host "Error al ensamblar $clean_asm" -ForegroundColor Red
    exit 1
}
Write-Host "OK: Objeto creado" -ForegroundColor Green

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "COMPARACION DE RESULTADOS" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Comparar tamanos
$dirty_asm_size = Get-FileSize $dirty_asm
$clean_asm_size = Get-FileSize $clean_asm
$dirty_obj_size = Get-FileSize $dirty_obj
$clean_obj_size = Get-FileSize $clean_obj

# Comparar lineas de codigo
$dirty_lines = (Get-Content $dirty_asm).Count
$clean_lines = (Get-Content $clean_asm).Count
$lines_reduction = if ($dirty_lines -gt 0) { 
    [math]::Round((($dirty_lines - $clean_lines) / $dirty_lines) * 100, 1) 
} else { 0 }

$asm_reduction = if ($dirty_asm_size -gt 0) { 
    [math]::Round((($dirty_asm_size - $clean_asm_size) / $dirty_asm_size) * 100, 1) 
} else { 0 }

$obj_reduction = if ($dirty_obj_size -gt 0 -and $clean_obj_size -gt 0) { 
    [math]::Round((($dirty_obj_size - $clean_obj_size) / $dirty_obj_size) * 100, 1) 
} else { 0 }

Write-Host "Archivo ASM (.asm):" -ForegroundColor White
Write-Host "  Sucio:  $dirty_asm_size bytes ($dirty_lines lineas)"
Write-Host "  Limpio: $clean_asm_size bytes ($clean_lines lineas)"
Write-Host "  Reduccion: $asm_reduction% (lineas: $lines_reduction%)"
Write-Host ""

Write-Host "Archivo Objeto (.obj):" -ForegroundColor White
Write-Host "  Sucio:  $dirty_obj_size bytes"
Write-Host "  Limpio: $clean_obj_size bytes"
Write-Host "  Reduccion: $obj_reduction%"
Write-Host ""

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Comparacion completada" -ForegroundColor Green
Write-Host ""
Write-Host "Archivos generados:" -ForegroundColor Cyan
Write-Host "  - $dirty_obj"
Write-Host "  - $clean_obj"
Write-Host ""
