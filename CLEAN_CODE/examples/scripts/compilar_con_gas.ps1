# Compilar ASM con GAS (GNU Assembler) si esta disponible
# El ASM de Clang usa sintaxis GAS, no NASM

Write-Host "Compilando ASM con GAS (GNU Assembler)..." -ForegroundColor Cyan
Write-Host ""

$dirty_asm = "test_array_CLANG_dirty.asm"
$clean_asm = "test_array_CLANG_cleaned_extreme.asm"
$dirty_obj = "test_array_dirty_gas.obj"
$clean_obj = "test_array_clean_gas.obj"
$dirty_exe = "test_array_dirty_gas.exe"
$clean_exe = "test_array_clean_gas.exe"

# Buscar GAS (as) en rutas comunes
$gas_path = $null
$gas_paths = @(
    "as",
    "C:\msys64\mingw64\bin\as.exe",
    "C:\msys64\usr\bin\as.exe",
    "C:\MinGW\bin\as.exe"
)

foreach ($path in $gas_paths) {
    try {
        $result = & $path --version 2>&1 | Select-Object -First 1
        if ($LASTEXITCODE -eq 0 -or $result) {
            $gas_path = $path
            Write-Host "GAS encontrado: $result" -ForegroundColor Green
            break
        }
    } catch {
        continue
    }
}

if (-not $gas_path) {
    Write-Host "ADVERTENCIA: GAS (as) no encontrado." -ForegroundColor Yellow
    Write-Host "El ASM de Clang requiere GAS para compilar." -ForegroundColor Yellow
    Write-Host "Instala MSYS2 o MinGW para obtener GAS." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Para comparar sin compilar, ejecuta:" -ForegroundColor Cyan
    Write-Host "  .\comparar_directo.ps1" -ForegroundColor White
    exit 0
}

# Buscar GCC para enlazar
$gcc_path = $null
$gcc_paths = @(
    "gcc",
    "C:\msys64\mingw64\bin\gcc.exe"
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
    Write-Host "ADVERTENCIA: GCC no encontrado para enlazar." -ForegroundColor Yellow
    Write-Host "Solo se crearan archivos objeto (.obj)" -ForegroundColor Yellow
}

Write-Host ""

# Funcion para obtener tamano
function Get-FileSize {
    param($file)
    if (Test-Path $file) {
        return (Get-Item $file).Length
    }
    return 0
}

# Compilar ASM sucio
Write-Host "Compilando ASM Sucio..." -ForegroundColor Yellow
Write-Host "Ensamblando: $dirty_asm -> $dirty_obj"
& $gas_path --64 -o $dirty_obj $dirty_asm 2>&1 | Out-Null

if ($LASTEXITCODE -eq 0 -and (Test-Path $dirty_obj)) {
    Write-Host "OK: Objeto creado ($(Get-FileSize $dirty_obj) bytes)" -ForegroundColor Green
    
    if ($gcc_path) {
        Write-Host "Enlazando: $dirty_obj -> $dirty_exe"
        & $gcc_path -O2 -o $dirty_exe $dirty_obj 2>&1 | Out-Null
        if ($LASTEXITCODE -eq 0 -and (Test-Path $dirty_exe)) {
            Write-Host "OK: Ejecutable creado ($(Get-FileSize $dirty_exe) bytes)" -ForegroundColor Green
        }
    }
} else {
    Write-Host "Error al ensamblar (puede tener dependencias faltantes)" -ForegroundColor Yellow
}

Write-Host ""

# Compilar ASM limpio
Write-Host "Compilando ASM Limpio..." -ForegroundColor Yellow
Write-Host "Ensamblando: $clean_asm -> $clean_obj"
& $gas_path --64 -o $clean_obj $clean_asm 2>&1 | Out-Null

if ($LASTEXITCODE -eq 0 -and (Test-Path $clean_obj)) {
    Write-Host "OK: Objeto creado ($(Get-FileSize $clean_obj) bytes)" -ForegroundColor Green
    
    if ($gcc_path) {
        Write-Host "Enlazando: $clean_obj -> $clean_exe"
        & $gcc_path -O2 -o $clean_exe $clean_obj 2>&1 | Out-Null
        if ($LASTEXITCODE -eq 0 -and (Test-Path $clean_exe)) {
            Write-Host "OK: Ejecutable creado ($(Get-FileSize $clean_exe) bytes)" -ForegroundColor Green
        }
    }
} else {
    Write-Host "Error al ensamblar (puede tener dependencias faltantes)" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "COMPARACION DE ARCHIVOS OBJETO" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$dirty_obj_size = Get-FileSize $dirty_obj
$clean_obj_size = Get-FileSize $clean_obj
$dirty_exe_size = Get-FileSize $dirty_exe
$clean_exe_size = Get-FileSize $clean_exe

if ($dirty_obj_size -gt 0 -and $clean_obj_size -gt 0) {
    $obj_reduction = [math]::Round((($dirty_obj_size - $clean_obj_size) / $dirty_obj_size) * 100, 1)
    Write-Host "Archivo Objeto (.obj):" -ForegroundColor White
    Write-Host "  Sucio:  $dirty_obj_size bytes"
    Write-Host "  Limpio: $clean_obj_size bytes"
    Write-Host "  Reduccion: $obj_reduction%"
    Write-Host ""
}

if ($dirty_exe_size -gt 0 -and $clean_exe_size -gt 0) {
    $exe_reduction = [math]::Round((($dirty_exe_size - $clean_exe_size) / $dirty_exe_size) * 100, 1)
    Write-Host "Ejecutable (.exe):" -ForegroundColor White
    Write-Host "  Sucio:  $dirty_exe_size bytes"
    Write-Host "  Limpio: $clean_exe_size bytes"
    Write-Host "  Reduccion: $exe_reduction%"
    Write-Host ""
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Comparacion completada" -ForegroundColor Green
Write-Host ""

