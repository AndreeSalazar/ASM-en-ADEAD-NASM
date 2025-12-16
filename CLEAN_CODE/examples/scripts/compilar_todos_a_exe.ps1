# Compilar todos los archivos ASM a ejecutables y comparar
# Usa GAS (as) para compilar ASM de Clang

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "COMPILACION DIFERENCIAL - Todos a EXE" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$ErrorActionPreference = "Continue"

# Archivos a compilar
$archivos = @(
    @{nombre="Sucio"; archivo="test_array_CLANG_dirty.asm"; obj="test_array_dirty.obj"; exe="test_array_dirty.exe"},
    @{nombre="Basico"; archivo="test_array_CLANG_cleaned_basic.asm"; obj="test_array_basic.obj"; exe="test_array_basic.exe"},
    @{nombre="Avanzado"; archivo="test_array_CLANG_cleaned_advanced.asm"; obj="test_array_advanced.obj"; exe="test_array_advanced.exe"},
    @{nombre="Extremo"; archivo="test_array_CLANG_cleaned_extreme.asm"; obj="test_array_extreme.obj"; exe="test_array_extreme.exe"}
)

# Buscar GAS
$gas_path = $null
$gas_paths = @("as", "C:\msys64\mingw64\bin\as.exe", "C:\msys64\usr\bin\as.exe")

foreach ($path in $gas_paths) {
    try {
        $result = & $path --version 2>&1 | Select-Object -First 1
        if ($LASTEXITCODE -eq 0 -or $result) {
            $gas_path = $path
            Write-Host "GAS encontrado: $result" -ForegroundColor Green
            break
        }
    } catch { continue }
}

if (-not $gas_path) {
    Write-Host "ERROR: GAS (as) no encontrado." -ForegroundColor Red
    Write-Host "Instala MSYS2: https://www.msys2.org/" -ForegroundColor Yellow
    Write-Host "O usa: pacman -S mingw-w64-x86_64-binutils" -ForegroundColor Yellow
    exit 1
}

# Buscar GCC
$gcc_path = $null
$gcc_paths = @("gcc", "C:\msys64\mingw64\bin\gcc.exe")

foreach ($path in $gcc_paths) {
    try {
        $result = & $path --version 2>&1 | Select-Object -First 1
        if ($LASTEXITCODE -eq 0 -or $result) {
            $gcc_path = $path
            Write-Host "GCC encontrado: $result" -ForegroundColor Green
            break
        }
    } catch { continue }
}

if (-not $gcc_path) {
    Write-Host "ERROR: GCC no encontrado." -ForegroundColor Red
    exit 1
}

Write-Host ""

# Crear wrapper C simple para enlazar
$wrapper_c = "test_array_wrapper.c"
@"
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

// Declaraciones de funciones del ASM
typedef struct {
    void* data;
    size_t length;
    size_t capacity;
} Array;

extern Array array_new(void);
extern Array array_from_values(size_t count, int64_t* values);
extern void array_append(Array* arr, int64_t value);
extern int64_t array_get(Array* arr, size_t index);
extern void array_set(Array* arr, size_t index, int64_t value);
extern size_t array_len(Array* arr);

int main(void) {
    printf("=== Test Array Functions ===\n");
    
    // Test 1: Crear array desde valores
    int64_t vals[] = {1, 2, 3};
    Array arr = array_from_values(3, vals);
    
    printf("Array length: %zu\n", array_len(&arr));
    printf("Array[0]: %lld\n", array_get(&arr, 0));
    printf("Array[1]: %lld\n", array_get(&arr, 1));
    printf("Array[2]: %lld\n", array_get(&arr, 2));
    
    printf("\n=== Test completado ===\n");
    return 0;
}
"@ | Out-File -FilePath $wrapper_c -Encoding UTF8

Write-Host "Wrapper C creado: $wrapper_c" -ForegroundColor Green
Write-Host ""

# Funcion para obtener tamano
function Get-FileSize {
    param($file)
    if (Test-Path $file) {
        return (Get-Item $file).Length
    }
    return 0
}

# Compilar cada archivo
$resultados = @()

foreach ($arch in $archivos) {
    Write-Host "----------------------------------------" -ForegroundColor Yellow
    Write-Host "Compilando: $($arch.nombre)" -ForegroundColor Yellow
    Write-Host "Archivo: $($arch.archivo)" -ForegroundColor White
    
    if (-not (Test-Path $arch.archivo)) {
        Write-Host "  ADVERTENCIA: Archivo no encontrado" -ForegroundColor Yellow
        continue
    }
    
    # Ensamblar con GAS
    Write-Host "  Ensamblando..." -ForegroundColor Cyan
    $gas_output = & $gas_path --64 -o $arch.obj $arch.archivo 2>&1
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  ERROR al ensamblar:" -ForegroundColor Red
        $gas_output | Select-Object -First 5 | ForEach-Object { Write-Host "    $_" -ForegroundColor Red }
        continue
    }
    
    if (-not (Test-Path $arch.obj)) {
        Write-Host "  ERROR: Objeto no creado" -ForegroundColor Red
        continue
    }
    
    $obj_size = Get-FileSize $arch.obj
    Write-Host "  OK: Objeto creado ($obj_size bytes)" -ForegroundColor Green
    
    # Enlazar con GCC
    Write-Host "  Enlazando..." -ForegroundColor Cyan
    $gcc_output = & $gcc_path -O2 -o $arch.exe $arch.obj $wrapper_c 2>&1
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  ADVERTENCIA: Error al enlazar (puede faltar main o funciones)" -ForegroundColor Yellow
        $gcc_output | Select-Object -First 3 | ForEach-Object { Write-Host "    $_" -ForegroundColor Yellow }
        
        # Intentar enlazar solo el objeto (sin wrapper)
        $gcc_output2 = & $gcc_path -O2 -o $arch.exe $arch.obj 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Host "  No se pudo crear ejecutable" -ForegroundColor Red
            continue
        }
    }
    
    if (Test-Path $arch.exe) {
        $exe_size = Get-FileSize $arch.exe
        Write-Host "  OK: Ejecutable creado ($exe_size bytes)" -ForegroundColor Green
        
        # Ejecutar para verificar
        Write-Host "  Ejecutando..." -ForegroundColor Cyan
        try {
            $output = & ".\$($arch.exe)" 2>&1
            Write-Host "  Salida:" -ForegroundColor White
            $output | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
        } catch {
            Write-Host "  ADVERTENCIA: Error al ejecutar" -ForegroundColor Yellow
        }
        
        $resultados += @{
            Nombre = $arch.nombre
            ASM_Size = (Get-FileSize $arch.archivo)
            OBJ_Size = $obj_size
            EXE_Size = $exe_size
            ASM_Lines = (Get-Content $arch.archivo).Count
        }
    } else {
        Write-Host "  ERROR: Ejecutable no creado" -ForegroundColor Red
    }
    
    Write-Host ""
}

# Mostrar comparacion
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "COMPARACION FINAL" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

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
        Write-Host "Reduccion vs Sucio:" -ForegroundColor Green
        foreach ($r in $resultados) {
            if ($r.Nombre -ne "Sucio") {
                $asm_red = [math]::Round((($sucio.ASM_Size - $r.ASM_Size) / $sucio.ASM_Size) * 100, 1)
                $exe_red = if ($sucio.EXE_Size -gt 0 -and $r.EXE_Size -gt 0) {
                    [math]::Round((($sucio.EXE_Size - $r.EXE_Size) / $sucio.EXE_Size) * 100, 1)
                } else { 0 }
                Write-Host "  $($r.Nombre): ASM -$asm_red%, EXE -$exe_red%" -ForegroundColor Cyan
            }
        }
    }
} else {
    Write-Host "No se pudieron compilar archivos" -ForegroundColor Red
}

Write-Host ""
Write-Host "Archivos generados:" -ForegroundColor Cyan
foreach ($arch in $archivos) {
    if (Test-Path $arch.exe) {
        Write-Host "  - $($arch.exe)" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Completado" -ForegroundColor Green
Write-Host ""

