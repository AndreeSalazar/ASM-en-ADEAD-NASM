# Crear ejecutables desde los objetos compilados
# Usa el codigo C original como base para enlazar

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "CREANDO EJECUTABLES - LOS 4 ELEMENTOS" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$gcc_path = "C:\msys64\mingw64\bin\gcc.exe"

# Objetos compilados
$objetos = @(
    @{nombre="Sucio"; obj="test_array_CLANG_dirty.obj"; exe="test_array_dirty.exe"},
    @{nombre="Basico"; obj="test_array_CLANG_cleaned_basic.obj"; exe="test_array_basic.exe"},
    @{nombre="Avanzado"; obj="test_array_CLANG_cleaned_advanced.obj"; exe="test_array_advanced.exe"},
    @{nombre="Extremo"; obj="test_array_CLANG_cleaned_extreme.obj"; exe="test_array_extreme.exe"}
)

# Crear main simple que use las funciones
$main_code = @"
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

typedef struct {
    int64_t* data;
    size_t length;
    size_t capacity;
} Array;

// Declaraciones externas (definidas en ASM)
extern Array array_new(void);
extern Array array_from_values(size_t count, int64_t* values);
extern void array_append(Array* arr, int64_t value);
extern int64_t array_get(Array* arr, size_t index);
extern void array_set(Array* arr, size_t index, int64_t value);
extern size_t array_len(Array* arr);

int main(void) {
    printf("=== Test Array Functions ===\n");
    
    // Test: Crear array desde valores
    int64_t vals[] = {1, 2, 3};
    Array arr = array_from_values(3, vals);
    
    printf("Array length: %zu\n", array_len(&arr));
    printf("Array[0]: %lld\n", array_get(&arr, 0));
    printf("Array[1]: %lld\n", array_get(&arr, 1));
    printf("Array[2]: %lld\n", array_get(&arr, 2));
    
    printf("\n=== Test completado ===\n");
    return 0;
}
"@

$main_file = "main_test.c"
$main_code | Out-File -FilePath $main_file -Encoding UTF8
Write-Host "Main test creado: $main_file" -ForegroundColor Green
Write-Host ""

function Get-FileSize {
    param($file)
    if (Test-Path $file) { return (Get-Item $file).Length }
    return 0
}

$resultados = @()

foreach ($obj_info in $objetos) {
    Write-Host "----------------------------------------" -ForegroundColor Yellow
    Write-Host "Creando ejecutable: $($obj_info.nombre)" -ForegroundColor Yellow
    
    if (-not (Test-Path $obj_info.obj)) {
        Write-Host "  ERROR: Objeto no encontrado: $($obj_info.obj)" -ForegroundColor Red
        continue
    }
    
    $obj_size = Get-FileSize $obj_info.obj
    Write-Host "  Objeto: $($obj_info.obj) ($obj_size bytes)" -ForegroundColor Cyan
    
    # Intentar enlazar
    Write-Host "  Enlazando..." -ForegroundColor Cyan
    
    # Para el sucio, usar directamente (tiene main)
    if ($obj_info.nombre -eq "Sucio") {
        $gcc_output = & $gcc_path -O2 -o $obj_info.exe $obj_info.obj 2>&1
    } else {
        # Para los limpios, enlazar con main
        $gcc_output = & $gcc_path -O2 -o $obj_info.exe $obj_info.obj $main_file 2>&1
    }
    
    if (Test-Path $obj_info.exe) {
        $exe_size = Get-FileSize $obj_info.exe
        Write-Host "  OK: Ejecutable creado ($exe_size bytes)" -ForegroundColor Green
        
        # Ejecutar
        Write-Host "  Ejecutando..." -ForegroundColor Cyan
        try {
            $output = & ".\$($obj_info.exe)" 2>&1
            if ($output) {
                $output | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
            } else {
                Write-Host "    (Sin salida)" -ForegroundColor Yellow
            }
        } catch {
            Write-Host "    Error: $($_.Exception.Message)" -ForegroundColor Yellow
        }
        
        $resultados += @{
            Nombre = $obj_info.nombre
            OBJ_Size = $obj_size
            EXE_Size = $exe_size
            EXE_Created = $true
        }
    } else {
        Write-Host "  ADVERTENCIA: No se pudo crear ejecutable" -ForegroundColor Yellow
        $gcc_output | Select-Object -First 3 | ForEach-Object { Write-Host "    $_" -ForegroundColor Yellow }
        
        $resultados += @{
            Nombre = $obj_info.nombre
            OBJ_Size = $obj_size
            EXE_Size = 0
            EXE_Created = $false
        }
    }
    
    Write-Host ""
}

# Mostrar resultados
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "RESULTADOS FINALES - LOS 4 ELEMENTOS" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

if ($resultados.Count -gt 0) {
    Write-Host "+-----------+----------+----------+----------+" -ForegroundColor White
    Write-Host "| Version   | OBJ (B)  | EXE (B)  | Estado   |" -ForegroundColor White
    Write-Host "+-----------+----------+----------+----------+" -ForegroundColor White
    
    foreach ($r in $resultados) {
        $exe_str = if ($r.EXE_Created) { 
            $r.EXE_Size.ToString().PadLeft(8) 
        } else { 
            "    N/A" 
        }
        $estado = if ($r.EXE_Created) { "   OK" } else { "Error" }
        Write-Host "| $($r.Nombre.PadRight(9)) | $($r.OBJ_Size.ToString().PadLeft(8)) | $exe_str | $estado |" -ForegroundColor White
    }
    
    Write-Host "+-----------+----------+----------+----------+" -ForegroundColor White
    Write-Host ""
    
    # Calcular reducciones
    $sucio = $resultados | Where-Object { $_.Nombre -eq "Sucio" } | Select-Object -First 1
    if ($sucio) {
        Write-Host "REDUCCION vs Sucio:" -ForegroundColor Green
        foreach ($r in $resultados) {
            if ($r.Nombre -ne "Sucio" -and $r.EXE_Created -and $sucio.EXE_Created) {
                $exe_red = [math]::Round((($sucio.EXE_Size - $r.EXE_Size) / $sucio.EXE_Size) * 100, 1)
                Write-Host "  $($r.Nombre): EXE -$exe_red% ($($sucio.EXE_Size) -> $($r.EXE_Size) bytes)" -ForegroundColor Cyan
            }
        }
    }
}

Write-Host ""
Write-Host "Archivos ejecutables generados:" -ForegroundColor Cyan
foreach ($r in $resultados) {
    if ($r.EXE_Created) {
        $exe_name = "test_array_$($r.Nombre.ToLower()).exe"
        Write-Host "  - $exe_name ($($r.EXE_Size) bytes)" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "Completado" -ForegroundColor Green
Write-Host ""

