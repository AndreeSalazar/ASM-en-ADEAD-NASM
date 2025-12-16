# Script para ejecutar todos los tests de arrays en Pruebas
$workspace = "C:\Users\andre\OneDrive\Documentos\ASM-en-ADEAD-NASM"
$compiler = Join-Path $workspace "CORE\rust\target\release\adeadc.exe"
$testDir = Join-Path $workspace "Pruebas"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Tests de Métodos de Arrays - ADead" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$tests = @(
    "test_array_insert",
    "test_array_remove",
    "test_array_index",
    "test_array_count",
    "test_array_sort",
    "test_array_completo"
)

$passed = 0
$failed = 0

foreach ($test in $tests) {
    $testFile = Join-Path $testDir "$test.ad"
    $asmFile = Join-Path $testDir "$test.asm"
    
    if (-not (Test-Path $testFile)) {
        Write-Host "[SKIP] $test.ad - Archivo no encontrado" -ForegroundColor Yellow
        continue
    }
    
    Write-Host "[TEST] $test.ad" -ForegroundColor Cyan
    
    # Compilar
    $output = & $compiler compile $testFile --backend auto -o $asmFile 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        if (Test-Path $asmFile) {
            $content = Get-Content $asmFile -Raw
            if ($content -match "section|\.text|\.globl|main:|\.intel_syntax|push|mov|call") {
                Write-Host "  [PASS] Compilación exitosa: $test.asm" -ForegroundColor Green
                $passed++
            } else {
                Write-Host "  [FAIL] Generó código inválido" -ForegroundColor Red
                $failed++
            }
        } else {
            Write-Host "  [FAIL] Archivo ASM no generado" -ForegroundColor Red
            $failed++
        }
    } else {
        Write-Host "  [FAIL] Error de compilación" -ForegroundColor Red
        Write-Host $output -ForegroundColor Red
        $failed++
    }
    Write-Host ""
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Resumen: $passed/$($tests.Count) tests pasaron" -ForegroundColor $(if ($failed -eq 0) { "Green" } else { "Yellow" })
Write-Host "========================================" -ForegroundColor Cyan

