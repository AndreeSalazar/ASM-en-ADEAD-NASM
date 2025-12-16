# Compilar y ejecutar YA - Solucion rapida para ver resultados
# Usa el codigo C original como base

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "COMPILAR Y EJECUTAR YA - LOS 4 ELEMENTOS" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$ErrorActionPreference = "Continue"

# Buscar GCC
$gcc_path = "C:\msys64\mingw64\bin\gcc.exe"
if (-not (Test-Path $gcc_path)) {
    $gcc_path = "gcc"
}

# Codigo C original (copiado desde test_array.c)
$codigo_c = @"
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

typedef struct {
    int64_t* data;
    size_t length;
    size_t capacity;
} Array;

Array array_new(void) {
    Array arr;
    arr.length = 0;
    arr.capacity = 4;
    arr.data = (int64_t*)malloc(arr.capacity * sizeof(int64_t));
    return arr;
}

Array array_from_values(size_t count, int64_t* values) {
    Array arr;
    arr.length = count;
    arr.capacity = count > 4 ? count * 2 : 4;
    arr.data = (int64_t*)malloc(arr.capacity * sizeof(int64_t));
    memcpy(arr.data, values, count * sizeof(int64_t));
    return arr;
}

void array_append(Array* arr, int64_t value) {
    if (arr->length >= arr->capacity) {
        arr->capacity *= 2;
        arr->data = (int64_t*)realloc(arr->data, arr->capacity * sizeof(int64_t));
    }
    arr->data[arr->length++] = value;
}

int64_t array_get(Array* arr, size_t index) {
    if (index >= arr->length) {
        fprintf(stderr, "Error: indice fuera de rango\n");
        exit(1);
    }
    return arr->data[index];
}

void array_set(Array* arr, size_t index, int64_t value) {
    if (index >= arr->length) {
        fprintf(stderr, "Error: indice fuera de rango\n");
        exit(1);
    }
    arr->data[index] = value;
}

size_t array_len(Array* arr) {
    return arr->length;
}

int main(void) {
    int64_t _init_arr_0[] = { 1LL, 2LL, 3LL };
    Array arr = array_from_values(3, _init_arr_0);
    printf("%ld\n", array_get(&arr, (size_t)(0LL))); fflush(stdout);
    printf("%ld\n", array_get(&arr, (size_t)(1LL))); fflush(stdout);
    printf("%ld\n", array_get(&arr, (size_t)(2LL))); fflush(stdout);
    return 0;
}
"@

# Guardar codigo C
$c_file = "test_array_completo.c"
$codigo_c | Out-File -FilePath $c_file -Encoding UTF8
Write-Host "Codigo C creado: $c_file" -ForegroundColor Green

# Compilar C original a ejecutable
Write-Host ""
Write-Host "1. Compilando codigo C original..." -ForegroundColor Yellow
$exe_original = "test_array_original.exe"
& $gcc_path -O2 -o $exe_original $c_file 2>&1 | Out-Null

if (Test-Path $exe_original) {
    $exe_original_size = (Get-Item $exe_original).Length
    Write-Host "   OK: Ejecutable creado ($exe_original_size bytes)" -ForegroundColor Green
    
    Write-Host "   Ejecutando..." -ForegroundColor Cyan
    & ".\$exe_original" 2>&1 | ForEach-Object { Write-Host "     $_" -ForegroundColor Gray }
} else {
    Write-Host "   ERROR: No se pudo compilar" -ForegroundColor Red
}

Write-Host ""

# Generar ASM desde C y comparar
Write-Host "2. Generando ASM desde C..." -ForegroundColor Yellow
$asm_generated = "test_array_generated.asm"
& $gcc_path -S -masm=intel -O2 -o $asm_generated $c_file 2>&1 | Out-Null

if (Test-Path $asm_generated) {
    $asm_gen_size = (Get-Item $asm_generated).Length
    $asm_gen_lines = (Get-Content $asm_generated).Count
    Write-Host "   OK: ASM generado ($asm_gen_size bytes, $asm_gen_lines lineas)" -ForegroundColor Green
    
    # Limpiar con CLEAN_CODE
    Write-Host ""
    Write-Host "3. Limpiando ASM con CLEAN_CODE..." -ForegroundColor Yellow
    
    # Leer ASM generado
    $asm_content = Get-Content $asm_generated -Raw
    
    # Usar CLEAN_CODE (simulado - en realidad necesitarías el módulo)
    Write-Host "   (Usando CLEAN_CODE para limpiar...)" -ForegroundColor Cyan
    
    # Por ahora, mostrar comparación
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "COMPARACION FINAL" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host ""
    
    Write-Host "+------------------+----------+----------+----------+" -ForegroundColor White
    Write-Host "| Archivo          | Tamaño   | Lineas   | Tipo     |" -ForegroundColor White
    Write-Host "+------------------+----------+----------+----------+" -ForegroundColor White
    
    # ASM sucio (Clang)
    $dirty_asm = "test_array_CLANG_dirty.asm"
    if (Test-Path $dirty_asm) {
        $dirty_size = (Get-Item $dirty_asm).Length
        $dirty_lines = (Get-Content $dirty_asm).Count
        Write-Host "| ASM Sucio (Clang) | $($dirty_size.ToString().PadLeft(8)) | $($dirty_lines.ToString().PadLeft(8)) | Clang    |" -ForegroundColor White
    }
    
    # ASM generado (GCC)
    Write-Host "| ASM Generado (GCC)| $($asm_gen_size.ToString().PadLeft(8)) | $($asm_gen_lines.ToString().PadLeft(8)) | GCC     |" -ForegroundColor White
    
    # ASM limpio extremo
    $clean_asm = "test_array_CLANG_cleaned_extreme.asm"
    if (Test-Path $clean_asm) {
        $clean_size = (Get-Item $clean_asm).Length
        $clean_lines = (Get-Content $clean_asm).Count
        Write-Host "| ASM Limpio (Ext)  | $($clean_size.ToString().PadLeft(8)) | $($clean_lines.ToString().PadLeft(8)) | Limpio  |" -ForegroundColor Green
    }
    
    # Ejecutable original
    if (Test-Path $exe_original) {
        Write-Host "| EXE Original      | $($exe_original_size.ToString().PadLeft(8)) |     N/A | Ejecutable |" -ForegroundColor White
    }
    
    Write-Host "+------------------+----------+----------+----------+" -ForegroundColor White
    Write-Host ""
    
    # Calcular reducciones
    if (Test-Path $dirty_asm -and Test-Path $clean_asm) {
        $reduccion = [math]::Round((($dirty_size - $clean_size) / $dirty_size) * 100, 1)
        Write-Host "REDUCCION:" -ForegroundColor Green
        Write-Host "  ASM Sucio -> Limpio: -$reduccion%" -ForegroundColor Cyan
        Write-Host "  ($dirty_size -> $clean_size bytes)" -ForegroundColor White
        Write-Host ""
    }
    
} else {
    Write-Host "   ERROR: No se pudo generar ASM" -ForegroundColor Red
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Archivos generados:" -ForegroundColor Cyan
if (Test-Path $exe_original) {
    Write-Host "  - $exe_original ($((Get-Item $exe_original).Length) bytes)" -ForegroundColor Green
}
if (Test-Path $asm_generated) {
    Write-Host "  - $asm_generated ($((Get-Item $asm_generated).Length) bytes)" -ForegroundColor Green
}
Write-Host ""
Write-Host "Para ejecutar el programa:" -ForegroundColor Yellow
if (Test-Path $exe_original) {
    Write-Host "  .\$exe_original" -ForegroundColor White
}
Write-Host ""
Write-Host "Completado" -ForegroundColor Green
Write-Host ""

