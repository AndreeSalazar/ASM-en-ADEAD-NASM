# Script para ejecutar todos los tests de strings avanzados
$workspace = "C:\Users\andre\OneDrive\Documentos\ASM-en-ADEAD-NASM"
$compiler = Join-Path $workspace "CORE\rust\target\release\adeadc.exe"
$testDir = Join-Path $workspace "Pruebas Avanzadas"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Tests de Strings Avanzados - ADead" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$tests = @(
    "test_string_concat",
    "test_string_slice",
    "test_string_upper",
    "test_string_lower",
    "test_string_len",
    "test_string_completo"
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
    
    # Compilar con backend auto (debería usar NASM directo)
    $output = & $compiler compile $testFile --backend auto -o $asmFile 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        if (Test-Path $asmFile) {
            $content = Get-Content $asmFile -Raw
            if ($content -match "section|\.text|\.globl|main:|\.intel_syntax|push|mov|call|string_concat|string_slice|string_upper|string_lower") {
                Write-Host "  [PASS] Compilación exitosa: $test.asm" -ForegroundColor Green
                Write-Host "         Tamaño: $($content.Length) caracteres" -ForegroundColor Gray
                $passed++
            } else {
                Write-Host "  [FAIL] Generó código inválido o no contiene funciones de strings" -ForegroundColor Red
                $failed++
            }
        } else {
            Write-Host "  [FAIL] Archivo ASM no generado" -ForegroundColor Red
            $failed++
        }
    } else {
        Write-Host "  [FAIL] Error de compilación" -ForegroundColor Red
        Write-Host $output -ForegroundColor Red | Select-Object -First 5
        $failed++
    }
    Write-Host ""
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Resumen: $passed/$($tests.Count) tests pasaron" -ForegroundColor $(if ($failed -eq 0) { "Green" } else { "Yellow" })
Write-Host "========================================" -ForegroundColor Cyan

