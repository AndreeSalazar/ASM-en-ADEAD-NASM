# Script para ver la relacion entre archivos de forma visual

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "RELACION DE ARCHIVOS - FLUJO COMPLETO" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Funciones auxiliares
function Get-FileSize {
    param($file)
    if (Test-Path $file) { 
        $size = (Get-Item $file).Length
        if ($size -gt 1024) {
            return "$([math]::Round($size/1KB, 1)) KB"
        } else {
            return "$size bytes"
        }
    }
    return "N/A"
}

function Get-LineCount {
    param($file)
    if (Test-Path $file) { return (Get-Content $file).Count }
    return 0
}

# 1. Código fuente
Write-Host "1. CODIGO FUENTE ORIGINAL" -ForegroundColor Yellow
Write-Host "   " -NoNewline
Write-Host "CODIGO/test_array_original.c" -ForegroundColor Green
$c_size = Get-FileSize "CODIGO\test_array_original.c"
$c_lines = Get-LineCount "CODIGO\test_array_original.c"
Write-Host "   Tamaño: $c_size, Líneas: $c_lines" -ForegroundColor White
Write-Host ""

# 2. Ejecutables
Write-Host "2. EJECUTABLES (EXE/)" -ForegroundColor Yellow
Write-Host "   Generados desde: CODIGO/test_array_original.c" -ForegroundColor Gray
Write-Host "   Comando: gcc -O2 -o test_array_funcional.exe test_array_original.c" -ForegroundColor Gray
Write-Host ""

$exe_files = Get-ChildItem "EXE\*.exe" -ErrorAction SilentlyContinue
if ($exe_files) {
    foreach ($exe in $exe_files) {
        Write-Host "   " -NoNewline
        Write-Host "EXE/$($exe.Name)" -ForegroundColor Green
        Write-Host "   Tamaño: $(Get-FileSize $exe.FullName)" -ForegroundColor White
        Write-Host "   Propósito: Ejecutable funcional que demuestra que el código funciona" -ForegroundColor Cyan
        Write-Host "   Salida: 1, 2, 3" -ForegroundColor Gray
        Write-Host ""
    }
} else {
    Write-Host "   (No hay ejecutables)" -ForegroundColor Yellow
}

# 3. ASM
Write-Host "3. ARCHIVOS ASM (ASM/)" -ForegroundColor Yellow
Write-Host "   Flujo:" -ForegroundColor Gray
Write-Host "   CODIGO/test_array_original.c → [GCC -S] → ASM/dirty.asm" -ForegroundColor Gray
Write-Host "   ASM/dirty.asm → [CLEAN_CODE] → ASM/basic.asm, advanced.asm, extreme.asm" -ForegroundColor Gray
Write-Host ""

$asm_files = @(
    @{nombre="dirty.asm"; descripcion="ASM sucio (original de Clang/GCC)"},
    @{nombre="basic.asm"; descripcion="ASM limpio - nivel básico"},
    @{nombre="advanced.asm"; descripcion="ASM limpio - nivel avanzado"},
    @{nombre="extreme.asm"; descripcion="ASM limpio - nivel extremo"}
)

foreach ($asm in $asm_files) {
    $file = "ASM\$($asm.nombre)"
    if (Test-Path $file) {
        Write-Host "   " -NoNewline
        Write-Host "ASM/$($asm.nombre)" -ForegroundColor Green
        Write-Host "   Tamaño: $(Get-FileSize $file), Líneas: $(Get-LineCount $file)" -ForegroundColor White
        Write-Host "   Descripción: $($asm.descripcion)" -ForegroundColor Cyan
        
        # Calcular reducción vs sucio
        if ($asm.nombre -ne "dirty.asm") {
            $dirty_file = "ASM\dirty.asm"
            if (Test-Path $dirty_file) {
                $dirty_size = (Get-Item $dirty_file).Length
                $current_size = (Get-Item $file).Length
                $reduction = [math]::Round((($dirty_size - $current_size) / $dirty_size) * 100, 1)
                Write-Host "   Reducción vs sucio: -$reduction%" -ForegroundColor Yellow
            }
        }
        Write-Host ""
    }
}

# 4. Objetos
Write-Host "4. OBJETOS COMPILADOS (OBJ/)" -ForegroundColor Yellow
Write-Host "   Generados desde: Archivos ASM usando GAS (as --64)" -ForegroundColor Gray
Write-Host ""

$obj_files = @(
    @{nombre="dirty.obj"; origen="dirty.asm"},
    @{nombre="basic.obj"; origen="basic.asm"},
    @{nombre="advanced.obj"; origen="advanced.asm"},
    @{nombre="extreme.obj"; origen="extreme.asm"}
)

foreach ($obj in $obj_files) {
    $file = "OBJ\$($obj.nombre)"
    if (Test-Path $file) {
        Write-Host "   " -NoNewline
        Write-Host "OBJ/$($obj.nombre)" -ForegroundColor Green
        Write-Host "   Origen: ASM/$($obj.origen)" -ForegroundColor Gray
        Write-Host "   Tamaño: $(Get-FileSize $file)" -ForegroundColor White
        
        # Calcular reducción vs sucio
        if ($obj.nombre -ne "dirty.obj") {
            $dirty_obj = "OBJ\dirty.obj"
            if (Test-Path $dirty_obj) {
                $dirty_size = (Get-Item $dirty_obj).Length
                $current_size = (Get-Item $file).Length
                $reduction = [math]::Round((($dirty_size - $current_size) / $dirty_size) * 100, 1)
                Write-Host "   Reducción vs sucio: -$reduction%" -ForegroundColor Yellow
            }
        }
        Write-Host ""
    }
}

# 5. Diagrama de flujo
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "DIAGRAMA DE FLUJO" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "CODIGO/test_array_original.c" -ForegroundColor Green
Write-Host "    │" -ForegroundColor White
Write-Host "    ├──→ [gcc -O2] ──→ EXE/test_array_funcional.exe" -ForegroundColor Cyan
Write-Host "    │" -ForegroundColor White
Write-Host "    └──→ [gcc -S] ──→ ASM/dirty.asm" -ForegroundColor Cyan
Write-Host "            │" -ForegroundColor White
Write-Host "            ├──→ [CLEAN_CODE básico] ──→ ASM/basic.asm ──→ [GAS] ──→ OBJ/basic.obj" -ForegroundColor Cyan
Write-Host "            ├──→ [CLEAN_CODE avanzado] ──→ ASM/advanced.asm ──→ [GAS] ──→ OBJ/advanced.obj" -ForegroundColor Cyan
Write-Host "            └──→ [CLEAN_CODE extremo] ──→ ASM/extreme.asm ──→ [GAS] ──→ OBJ/extreme.obj" -ForegroundColor Cyan
Write-Host ""

# 6. Resumen de relaciones
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "RESUMEN DE RELACIONES" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "Ejecutables (EXE/):" -ForegroundColor Yellow
Write-Host "  └──→ Generados desde: CODIGO/test_array_original.c" -ForegroundColor White
Write-Host "  └──→ Propósito: Demostrar que el código funciona" -ForegroundColor White
Write-Host ""

Write-Host "ASM Sucio (ASM/dirty.asm):" -ForegroundColor Yellow
Write-Host "  └──→ Generado desde: CODIGO/test_array_original.c" -ForegroundColor White
Write-Host "  └──→ Genera: OBJ/dirty.obj" -ForegroundColor White
Write-Host ""

Write-Host "ASM Limpios (ASM/basic.asm, advanced.asm, extreme.asm):" -ForegroundColor Yellow
Write-Host "  └──→ Generados desde: ASM/dirty.asm (usando CLEAN_CODE)" -ForegroundColor White
Write-Host "  └──→ Generan: OBJ/basic.obj, OBJ/advanced.obj, OBJ/extreme.obj" -ForegroundColor White
Write-Host ""

Write-Host "Objetos (OBJ/):" -ForegroundColor Yellow
Write-Host "  └──→ Generados desde: Archivos ASM (usando GAS)" -ForegroundColor White
Write-Host "  └──→ Propósito: Demostrar reducción de tamaño en código compilado" -ForegroundColor White
Write-Host ""

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Para comparar todo:" -ForegroundColor Yellow
Write-Host "  .\comparar.ps1" -ForegroundColor Cyan
Write-Host ""

