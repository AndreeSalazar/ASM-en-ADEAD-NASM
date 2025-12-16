# Script para ejecutar todos los tests de funciones
$workspace = "C:\Users\andre\OneDrive\Documentos\ASM-en-ADEAD-NASM"
$compiler = Join-Path $workspace "CORE\rust\target\release\adeadc.exe"
$testDir = Join-Path $workspace "New Test 1"

Write-Host "`n=== Tests de Funciones Completas ===" -ForegroundColor Cyan
Write-Host "Compilador: $compiler" -ForegroundColor Gray
Write-Host "Directorio: $testDir`n" -ForegroundColor Gray

$tests = @(
    "test_funcion_simple.ad",
    "test_funcion_multi_param.ad",
    "test_funcion_recursiva.ad",
    "test_funcion_return_multiple.ad",
    "test_funcion_completa.ad"
)

$results = @()

foreach ($test in $tests) {
    $testPath = Join-Path $testDir $test
    $asmPath = Join-Path $testDir ($test -replace '\.ad$', '.asm')
    
    Write-Host "`n--- Compilando: $test ---" -ForegroundColor Yellow
    
    # Compilar ADead → NASM
    & $compiler compile $testPath --backend nasm -o $asmPath 2>&1 | Out-String
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Compilación exitosa" -ForegroundColor Green
        
        # Verificar que el ASM generado existe y tiene contenido
        if (Test-Path $asmPath) {
            $asmContent = Get-Content $asmPath -Raw
            $lines = ($asmContent -split "`n").Count
            
            Write-Host "   ASM generado: $lines líneas" -ForegroundColor Gray
            
            # Verificar características importantes
            $hasFunction = $asmContent -match "fn_"
            $hasPrologue = $asmContent -match "push rbp" -and $asmContent -match "push rbx"
            $hasEpilogue = $asmContent -match "pop r15" -and $asmContent -match "leave"
            $hasReturn = $asmContent -match "return" -or $asmContent -match "mov rax"
            $hasShadowSpace = $asmContent -match "sub rsp, 32"
            
            Write-Host "   Verificaciones:" -ForegroundColor Gray
            Write-Host "     - Función generada: $(if ($hasFunction) { '✅' } else { '❌' })" -ForegroundColor $(if ($hasFunction) { 'Green' } else { 'Red' })
            Write-Host "     - Prologue ABI-safe: $(if ($hasPrologue) { '✅' } else { '❌' })" -ForegroundColor $(if ($hasPrologue) { 'Green' } else { 'Red' })
            Write-Host "     - Epilogue ABI-safe: $(if ($hasEpilogue) { '✅' } else { '❌' })" -ForegroundColor $(if ($hasEpilogue) { 'Green' } else { 'Red' })
            Write-Host "     - Return statement: $(if ($hasReturn) { '✅' } else { '❌' })" -ForegroundColor $(if ($hasReturn) { 'Green' } else { 'Red' })
            Write-Host "     - Shadow space: $(if ($hasShadowSpace) { '✅' } else { '❌' })" -ForegroundColor $(if ($hasShadowSpace) { 'Green' } else { 'Red' })
            
            $results += @{
                Test = $test
                Status = "✅ OK"
                Lines = $lines
                HasFunction = $hasFunction
                HasPrologue = $hasPrologue
                HasEpilogue = $hasEpilogue
                HasReturn = $hasReturn
                HasShadowSpace = $hasShadowSpace
            }
        } else {
            Write-Host "❌ ASM no generado" -ForegroundColor Red
            $results += @{
                Test = $test
                Status = "❌ ERROR"
                Lines = 0
            }
        }
    } else {
        Write-Host "❌ Error en compilación" -ForegroundColor Red
        $results += @{
            Test = $test
            Status = "❌ ERROR"
            Lines = 0
        }
    }
}

Write-Host "`n=== Resumen ===" -ForegroundColor Cyan
foreach ($result in $results) {
    Write-Host "$($result.Test): $($result.Status)" -ForegroundColor $(if ($result.Status -eq "✅ OK") { 'Green' } else { 'Red' })
    if ($result.Lines -gt 0) {
        Write-Host "   Líneas: $($result.Lines)" -ForegroundColor Gray
    }
}

$successCount = ($results | Where-Object { $_.Status -eq "✅ OK" }).Count
Write-Host "`n✅ Tests exitosos: $successCount/$($results.Count)" -ForegroundColor Green

