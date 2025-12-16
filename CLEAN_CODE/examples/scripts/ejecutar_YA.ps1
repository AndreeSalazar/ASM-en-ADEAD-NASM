# Ejecutar YA - Solucion inmediata para ver resultados
# Compila el codigo C original y muestra comparacion

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "EJECUTAR YA - COMPARACION COMPLETA" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Buscar GCC en rutas comunes
$gcc_paths = @(
    "C:\msys64\mingw64\bin\gcc.exe",
    "C:\msys64\usr\bin\gcc.exe",
    "gcc"
)

$gcc_path = $null
foreach ($path in $gcc_paths) {
    if (Test-Path $path -ErrorAction SilentlyContinue) {
        $gcc_path = $path
        break
    }
    try {
        $result = & $path --version 2>&1 | Select-Object -First 1
        if ($result) {
            $gcc_path = $path
            Write-Host "GCC encontrado: $result" -ForegroundColor Green
            break
        }
    } catch { continue }
}

if (-not $gcc_path) {
    Write-Host "ERROR: GCC no encontrado" -ForegroundColor Red
    Write-Host "Instala MSYS2: https://www.msys2.org/" -ForegroundColor Yellow
    exit 1
}

Write-Host ""

# Funciones auxiliares
function Get-FileSize {
    param($file)
    if (Test-Path $file) { return (Get-Item $file).Length }
    return 0
}

function Get-LineCount {
    param($file)
    if (Test-Path $file) { return (Get-Content $file).Count }
    return 0
}

# 1. Compilar codigo C original a ejecutable
Write-Host "1. Compilando codigo C original..." -ForegroundColor Yellow
$c_file = "test_array_original.c"
$exe_original = "test_array_original.exe"

if (Test-Path $c_file) {
    & $gcc_path -O2 -o $exe_original $c_file 2>&1 | Out-Null
    
    if (Test-Path $exe_original) {
        $exe_size = Get-FileSize $exe_original
        Write-Host "   OK: Ejecutable creado ($exe_size bytes)" -ForegroundColor Green
        
        Write-Host "   Ejecutando..." -ForegroundColor Cyan
        Write-Host "   Salida:" -ForegroundColor White
        & ".\$exe_original" 2>&1 | ForEach-Object { Write-Host "     $_" -ForegroundColor Gray }
    } else {
        Write-Host "   ERROR: No se pudo compilar" -ForegroundColor Red
    }
} else {
    Write-Host "   ERROR: Archivo C no encontrado: $c_file" -ForegroundColor Red
}

Write-Host ""

# 2. Generar ASM desde C
Write-Host "2. Generando ASM desde C..." -ForegroundColor Yellow
$asm_generated = "test_array_generated.asm"

if (Test-Path $c_file) {
    & $gcc_path -S -masm=intel -O2 -o $asm_generated $c_file 2>&1 | Out-Null
    
    if (Test-Path $asm_generated) {
        $asm_gen_size = Get-FileSize $asm_generated
        $asm_gen_lines = Get-LineCount $asm_generated
        Write-Host "   OK: ASM generado ($asm_gen_size bytes, $asm_gen_lines lineas)" -ForegroundColor Green
    } else {
        Write-Host "   ADVERTENCIA: No se pudo generar ASM" -ForegroundColor Yellow
    }
}

Write-Host ""

# 3. Mostrar comparacion completa
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "COMPARACION - LOS 4 ELEMENTOS" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$archivos = @(
    @{nombre="Sucio (Clang)"; archivo="test_array_CLANG_dirty.asm"; tipo="ASM"},
    @{nombre="Basico"; archivo="test_array_CLANG_cleaned_basic.asm"; tipo="ASM"},
    @{nombre="Avanzado"; archivo="test_array_CLANG_cleaned_advanced.asm"; tipo="ASM"},
    @{nombre="Extremo"; archivo="test_array_CLANG_cleaned_extreme.asm"; tipo="ASM"},
    @{nombre="Generado (GCC)"; archivo="test_array_generated.asm"; tipo="ASM"}
)

$objetos = @(
    @{nombre="Sucio"; archivo="test_array_CLANG_dirty.obj"; tipo="OBJ"},
    @{nombre="Basico"; archivo="test_array_CLANG_cleaned_basic.obj"; tipo="OBJ"},
    @{nombre="Avanzado"; archivo="test_array_CLANG_cleaned_advanced.obj"; tipo="OBJ"},
    @{nombre="Extremo"; archivo="test_array_CLANG_cleaned_extreme.obj"; tipo="OBJ"}
)

$resultados = @()

# Agregar ejecutable original
if (Test-Path $exe_original) {
    $resultados += @{
        Nombre = "Original (EXE)"
        ASM_Size = 0
        ASM_Lines = 0
        OBJ_Size = 0
        EXE_Size = Get-FileSize $exe_original
        Tipo = "EXE"
    }
}

# Agregar archivos ASM
foreach ($arch in $archivos) {
    if (Test-Path $arch.archivo) {
        $resultados += @{
            Nombre = $arch.nombre
            ASM_Size = Get-FileSize $arch.archivo
            ASM_Lines = Get-LineCount $arch.archivo
            OBJ_Size = 0
            EXE_Size = 0
            Tipo = "ASM"
        }
    }
}

# Agregar objetos
foreach ($obj in $objetos) {
    if (Test-Path $obj.archivo) {
        $resultados += @{
            Nombre = "$($obj.nombre) (OBJ)"
            ASM_Size = 0
            ASM_Lines = 0
            OBJ_Size = Get-FileSize $obj.archivo
            EXE_Size = 0
            Tipo = "OBJ"
        }
    }
}

# Mostrar tabla
Write-Host "+----------------------+----------+----------+----------+----------+" -ForegroundColor White
Write-Host "| Version              | ASM (B)  | Lineas   | OBJ (B)  | EXE (B)  |" -ForegroundColor White
Write-Host "+----------------------+----------+----------+----------+----------+" -ForegroundColor White

foreach ($r in $resultados) {
    $asm_str = if ($r.ASM_Size -gt 0) { $r.ASM_Size.ToString().PadLeft(8) } else { "    N/A" }
    $lines_str = if ($r.ASM_Lines -gt 0) { $r.ASM_Lines.ToString().PadLeft(8) } else { "    N/A" }
    $obj_str = if ($r.OBJ_Size -gt 0) { $r.OBJ_Size.ToString().PadLeft(8) } else { "    N/A" }
    $exe_str = if ($r.EXE_Size -gt 0) { $r.EXE_Size.ToString().PadLeft(8) } else { "    N/A" }
    
    Write-Host "| $($r.Nombre.PadRight(20)) | $asm_str | $lines_str | $obj_str | $exe_str |" -ForegroundColor White
}

Write-Host "+----------------------+----------+----------+----------+----------+" -ForegroundColor White
Write-Host ""

# Calcular reducciones
$sucio_asm = $resultados | Where-Object { $_.Nombre -eq "Sucio (Clang)" } | Select-Object -First 1
$sucio_obj = $resultados | Where-Object { $_.Nombre -eq "Sucio (OBJ)" } | Select-Object -First 1

if ($sucio_asm) {
    Write-Host "REDUCCION vs ASM Sucio:" -ForegroundColor Green
    foreach ($r in $resultados) {
        if ($r.Tipo -eq "ASM" -and $r.Nombre -ne "Sucio (Clang)" -and $r.ASM_Size -gt 0) {
            $red = [math]::Round((($sucio_asm.ASM_Size - $r.ASM_Size) / $sucio_asm.ASM_Size) * 100, 1)
            Write-Host "  $($r.Nombre): -$red% ($($sucio_asm.ASM_Size) -> $($r.ASM_Size) bytes)" -ForegroundColor Cyan
        }
    }
    Write-Host ""
}

if ($sucio_obj) {
    Write-Host "REDUCCION vs OBJ Sucio:" -ForegroundColor Green
    foreach ($r in $resultados) {
        if ($r.Tipo -eq "OBJ" -and $r.Nombre -ne "Sucio (OBJ)" -and $r.OBJ_Size -gt 0) {
            $red = [math]::Round((($sucio_obj.OBJ_Size - $r.OBJ_Size) / $sucio_obj.OBJ_Size) * 100, 1)
            Write-Host "  $($r.Nombre): -$red% ($($sucio_obj.OBJ_Size) -> $($r.OBJ_Size) bytes)" -ForegroundColor Cyan
        }
    }
    Write-Host ""
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "EJECUTABLES DISPONIBLES:" -ForegroundColor Cyan
if (Test-Path $exe_original) {
    Write-Host "  - $exe_original" -ForegroundColor Green
    Write-Host "    Ejecutar: .\$exe_original" -ForegroundColor White
}
Write-Host ""
Write-Host "Completado" -ForegroundColor Green
Write-Host ""

