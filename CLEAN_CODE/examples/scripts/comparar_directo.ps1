# Comparacion directa de archivos ASM (sin compilar)
# Muestra diferencias entre ASM sucio y limpio

Write-Host "Comparacion Directa: ASM Sucio vs ASM Limpio" -ForegroundColor Cyan
Write-Host "==============================================" -ForegroundColor Cyan
Write-Host ""

$dirty_asm = "test_array_CLANG_dirty.asm"
$clean_asm = "test_array_CLANG_cleaned_extreme.asm"

if (-not (Test-Path $dirty_asm)) {
    Write-Host "ERROR: No se encuentra $dirty_asm" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $clean_asm)) {
    Write-Host "ERROR: No se encuentra $clean_asm" -ForegroundColor Red
    exit 1
}

# Obtener tamanos
$dirty_size = (Get-Item $dirty_asm).Length
$clean_size = (Get-Item $clean_asm).Length
$size_reduction = [math]::Round((($dirty_size - $clean_size) / $dirty_size) * 100, 1)

# Obtener lineas
$dirty_lines = (Get-Content $dirty_asm).Count
$clean_lines = (Get-Content $clean_asm).Count
$lines_reduction = [math]::Round((($dirty_lines - $clean_lines) / $dirty_lines) * 100, 1)

# Contar instrucciones reales (aproximado)
$dirty_content = Get-Content $dirty_asm -Raw
$clean_content = Get-Content $clean_asm -Raw

# Contar instrucciones comunes
$dirty_inst = ([regex]::Matches($dirty_content, "\b(mov|call|ret|push|pop|add|sub|jmp|jne|je|cmp|lea|shl|shr)\b")).Count
$clean_inst = ([regex]::Matches($clean_content, "\b(mov|call|ret|push|pop|add|sub|jmp|jne|je|cmp|lea|shl|shr)\b")).Count
$inst_reduction = if ($dirty_inst -gt 0) { 
    [math]::Round((($dirty_inst - $clean_inst) / $dirty_inst) * 100, 1) 
} else { 0 }

# Contar metadatos eliminados
$dirty_metadata = ([regex]::Matches($dirty_content, "\.(def|scl|type|endef|globl|p2align|file|text|section)")).Count
$clean_metadata = ([regex]::Matches($clean_content, "\.(def|scl|type|endef|globl|p2align|file|text|section)")).Count

# Contar comentarios
$dirty_comments = ([regex]::Matches($dirty_content, "#")).Count
$clean_comments = ([regex]::Matches($clean_content, "#")).Count

Write-Host "ESTADISTICAS GENERALES:" -ForegroundColor Yellow
Write-Host "----------------------"
Write-Host "Archivo ASM Sucio:" -ForegroundColor White
Write-Host "  Tamano:     $dirty_size bytes"
Write-Host "  Lineas:     $dirty_lines lineas"
Write-Host "  Instrucciones: ~$dirty_inst instrucciones"
Write-Host "  Metadatos:   $dirty_metadata elementos"
Write-Host "  Comentarios: $dirty_comments comentarios"
Write-Host ""

Write-Host "Archivo ASM Limpio:" -ForegroundColor White
Write-Host "  Tamano:     $clean_size bytes"
Write-Host "  Lineas:     $clean_lines lineas"
Write-Host "  Instrucciones: ~$clean_inst instrucciones"
Write-Host "  Metadatos:   $clean_metadata elementos"
Write-Host "  Comentarios: $clean_comments comentarios"
Write-Host ""

Write-Host "REDUCCION:" -ForegroundColor Green
Write-Host "----------"
Write-Host "  Tamano:     $size_reduction% ($dirty_size -> $clean_size bytes)"
Write-Host "  Lineas:     $lines_reduction% ($dirty_lines -> $clean_lines lineas)"
Write-Host "  Instrucciones: $inst_reduction% (~$dirty_inst -> ~$clean_inst)"
Write-Host "  Metadatos eliminados: $($dirty_metadata - $clean_metadata) elementos"
Write-Host "  Comentarios eliminados: $($dirty_comments - $clean_comments) comentarios"
Write-Host ""

Write-Host "==============================================" -ForegroundColor Cyan
Write-Host "RESUMEN:" -ForegroundColor Cyan
Write-Host "  El ASM limpio es $lines_reduction% mas pequeno en lineas" -ForegroundColor Green
Write-Host "  El ASM limpio es $size_reduction% mas pequeno en tamano" -ForegroundColor Green
Write-Host "  Se eliminaron $($dirty_metadata - $clean_metadata) elementos de metadatos" -ForegroundColor Green
Write-Host "  Se eliminaron $($dirty_comments - $clean_comments) comentarios de debug" -ForegroundColor Green
Write-Host ""

Write-Host "NOTA: El ASM generado por Clang usa sintaxis GAS," -ForegroundColor Yellow
Write-Host "      no NASM. Para compilarlo necesitas usar 'as' (GAS)" -ForegroundColor Yellow
Write-Host "      o convertirlo a formato NASM primero." -ForegroundColor Yellow
Write-Host ""

