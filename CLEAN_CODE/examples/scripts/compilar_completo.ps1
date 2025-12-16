# Compilar archivos ASM completos a ejecutables
# El archivo sucio tiene main completo, los limpios son fragmentos

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "COMPILACION DIFERENCIAL COMPLETA" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$ErrorActionPreference = "Continue"

# Buscar herramientas
$gas_path = $null
$gcc_path = $null

# Buscar GAS
foreach ($path in @("as", "C:\msys64\mingw64\bin\as.exe")) {
    try {
        $result = & $path --version 2>&1 | Select-Object -First 1
        if ($LASTEXITCODE -eq 0 -or $result) {
            $gas_path = $path
            Write-Host "GAS encontrado: $result" -ForegroundColor Green
            break
        }
    } catch { continue }
}

# Buscar GCC
foreach ($path in @("gcc", "C:\msys64\mingw64\bin\gcc.exe")) {
    try {
        $result = & $path --version 2>&1 | Select-Object -First 1
        if ($LASTEXITCODE -eq 0 -or $result) {
            $gcc_path = $path
            Write-Host "GCC encontrado: $result" -ForegroundColor Green
            break
        }
    } catch { continue }
}

if (-not $gas_path -or -not $gcc_path) {
    Write-Host "ERROR: GAS o GCC no encontrados" -ForegroundColor Red
    exit 1
}

Write-Host ""

# Funcion para obtener tamano
function Get-FileSize {
    param($file)
    if (Test-Path $file) { return (Get-Item $file).Length }
    return 0
}

# Compilar archivo sucio (tiene main completo)
Write-Host "1. Compilando ASM SUCIO (con main)..." -ForegroundColor Yellow
$dirty_asm = "test_array_CLANG_dirty.asm"
$dirty_obj = "test_array_dirty.obj"
$dirty_exe = "test_array_dirty.exe"

if (Test-Path $dirty_asm) {
    Write-Host "   Ensamblando..." -ForegroundColor Cyan
    & $gas_path --64 -o $dirty_obj $dirty_asm 2>&1 | Out-Null
    
    if (Test-Path $dirty_obj) {
        Write-Host "   OK: Objeto creado ($(Get-FileSize $dirty_obj) bytes)" -ForegroundColor Green
        
        Write-Host "   Enlazando..." -ForegroundColor Cyan
        & $gcc_path -O2 -o $dirty_exe $dirty_obj 2>&1 | Out-Null
        
        if (Test-Path $dirty_exe) {
            Write-Host "   OK: Ejecutable creado ($(Get-FileSize $dirty_exe) bytes)" -ForegroundColor Green
            Write-Host "   Ejecutando..." -ForegroundColor Cyan
            & ".\$dirty_exe" 2>&1 | ForEach-Object { Write-Host "     $_" -ForegroundColor Gray }
        }
    }
} else {
    Write-Host "   ERROR: Archivo no encontrado" -ForegroundColor Red
}

Write-Host ""

# Para los archivos limpios, necesitamos crear un main simple
# ya que solo tienen fragmentos de funciones
$main_simple = @"
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

typedef struct {
    void* data;
    size_t length;
    size_t capacity;
} Array;

extern Array array_new(void);

int main(void) {
    printf("Test: Array functions\n");
    Array arr = array_new();
    printf("Array created successfully\n");
    return 0;
}
"@

$main_file = "main_simple.c"
$main_simple | Out-File -FilePath $main_file -Encoding UTF8

# Compilar archivos limpios (solo tienen funciones, no main)
$archivos_limpios = @(
    @{nombre="Basico"; archivo="test_array_CLANG_cleaned_basic.asm"; exe="test_array_basic.exe"},
    @{nombre="Avanzado"; archivo="test_array_CLANG_cleaned_advanced.asm"; exe="test_array_advanced.exe"},
    @{nombre="Extremo"; archivo="test_array_CLANG_cleaned_extreme.asm"; exe="test_array_extreme.exe"}
)

$resultados = @()

foreach ($arch in $archivos_limpios) {
    Write-Host "$($archivos_limpios.IndexOf($arch) + 2). Compilando ASM $($arch.nombre)..." -ForegroundColor Yellow
    
    if (-not (Test-Path $arch.archivo)) {
        Write-Host "   ERROR: Archivo no encontrado" -ForegroundColor Red
        continue
    }
    
    $obj_file = $arch.archivo -replace "\.asm$", ".obj"
    
    Write-Host "   Ensamblando..." -ForegroundColor Cyan
    & $gas_path --64 -o $obj_file $arch.archivo 2>&1 | Out-Null
    
    if (Test-Path $obj_file) {
        $obj_size = Get-FileSize $obj_file
        Write-Host "   OK: Objeto creado ($obj_size bytes)" -ForegroundColor Green
        
        Write-Host "   Enlazando con main simple..." -ForegroundColor Cyan
        & $gcc_path -O2 -o $arch.exe $obj_file $main_file 2>&1 | Out-Null
        
        if (Test-Path $arch.exe) {
            $exe_size = Get-FileSize $arch.exe
            Write-Host "   OK: Ejecutable creado ($exe_size bytes)" -ForegroundColor Green
            
            Write-Host "   Ejecutando..." -ForegroundColor Cyan
            try {
                $output = & ".\$($arch.exe)" 2>&1
                $output | ForEach-Object { Write-Host "     $_" -ForegroundColor Gray }
            } catch {
                Write-Host "     (Error al ejecutar)" -ForegroundColor Yellow
            }
            
            $resultados += @{
                Nombre = $arch.nombre
                ASM_Size = (Get-FileSize $arch.archivo)
                OBJ_Size = $obj_size
                EXE_Size = $exe_size
                ASM_Lines = (Get-Content $arch.archivo).Count
            }
        } else {
            Write-Host "   ADVERTENCIA: No se pudo crear ejecutable" -ForegroundColor Yellow
        }
    } else {
        Write-Host "   ERROR: No se pudo crear objeto" -ForegroundColor Red
    }
    
    Write-Host ""
}

# Mostrar comparacion
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "COMPARACION FINAL" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Agregar resultado del sucio
if (Test-Path $dirty_exe) {
    $resultados = @(
        @{
            Nombre = "Sucio"
            ASM_Size = (Get-FileSize $dirty_asm)
            OBJ_Size = (Get-FileSize $dirty_obj)
            EXE_Size = (Get-FileSize $dirty_exe)
            ASM_Lines = (Get-Content $dirty_asm).Count
        }
    ) + $resultados
}

if ($resultados.Count -gt 0) {
    Write-Host "┌───────────┬──────────┬──────────┬──────────┬──────────┐" -ForegroundColor White
    Write-Host "│ Version   │ ASM (B)  │ OBJ (B)  │ EXE (B)  │ Lineas   │" -ForegroundColor White
    Write-Host "├───────────┼──────────┼──────────┼──────────┼──────────┤" -ForegroundColor White
    
    foreach ($r in $resultados) {
        Write-Host "│ $($r.Nombre.PadRight(9)) │ $($r.ASM_Size.ToString().PadLeft(8)) │ $($r.OBJ_Size.ToString().PadLeft(8)) │ $($r.EXE_Size.ToString().PadLeft(8)) │ $($r.ASM_Lines.ToString().PadLeft(8)) │" -ForegroundColor White
    }
    
    Write-Host "└───────────┴──────────┴──────────┴──────────┴──────────┘" -ForegroundColor White
    Write-Host ""
    
    # Calcular reducciones
    $sucio = $resultados | Where-Object { $_.Nombre -eq "Sucio" } | Select-Object -First 1
    if ($sucio) {
        Write-Host "REDUCCION vs Sucio:" -ForegroundColor Green
        foreach ($r in $resultados) {
            if ($r.Nombre -ne "Sucio") {
                $asm_red = [math]::Round((($sucio.ASM_Size - $r.ASM_Size) / $sucio.ASM_Size) * 100, 1)
                $obj_red = if ($sucio.OBJ_Size -gt 0) {
                    [math]::Round((($sucio.OBJ_Size - $r.OBJ_Size) / $sucio.OBJ_Size) * 100, 1)
                } else { 0 }
                $exe_red = if ($sucio.EXE_Size -gt 0 -and $r.EXE_Size -gt 0) {
                    [math]::Round((($sucio.EXE_Size - $r.EXE_Size) / $sucio.EXE_Size) * 100, 1)
                } else { 0 }
                $lines_red = [math]::Round((($sucio.ASM_Lines - $r.ASM_Lines) / $sucio.ASM_Lines) * 100, 1)
                
                Write-Host "  $($r.Nombre):" -ForegroundColor Cyan
                Write-Host "    ASM:   -$asm_red% ($($sucio.ASM_Size) -> $($r.ASM_Size) bytes)" -ForegroundColor White
                Write-Host "    OBJ:   -$obj_red% ($($sucio.OBJ_Size) -> $($r.OBJ_Size) bytes)" -ForegroundColor White
                Write-Host "    EXE:   -$exe_red% ($($sucio.EXE_Size) -> $($r.EXE_Size) bytes)" -ForegroundColor White
                Write-Host "    Lineas: -$lines_red% ($($sucio.ASM_Lines) -> $($r.ASM_Lines) lineas)" -ForegroundColor White
                Write-Host ""
            }
        }
    }
} else {
    Write-Host "No se pudieron compilar archivos" -ForegroundColor Red
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Archivos ejecutables generados:" -ForegroundColor Cyan
foreach ($r in $resultados) {
    $exe_name = "test_array_$($r.Nombre.ToLower()).exe"
    if (Test-Path $exe_name) {
        Write-Host "  - $exe_name ($(Get-FileSize $exe_name) bytes)" -ForegroundColor Green
    }
}
if (Test-Path $dirty_exe) {
    Write-Host "  - $dirty_exe ($(Get-FileSize $dirty_exe) bytes)" -ForegroundColor Green
}

Write-Host ""
Write-Host "Completado" -ForegroundColor Green
Write-Host ""

