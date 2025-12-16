# Script para verificar que todos los tests compilan correctamente
# No ejecuta los programas, solo verifica que la compilación funciona

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Verificación de Compilación" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Resolver path del compilador de forma más robusta
$script_dir = Split-Path -Parent $MyInvocation.MyCommand.Path
$workspace_root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $script_dir))
$compiler_path = Join-Path $workspace_root "CORE\rust\target\release\adeadc.exe"

# Si no existe, intentar path relativo
if (-not (Test-Path $compiler_path)) {
    $compiler_path = "..\..\..\CORE\rust\target\release\adeadc.exe"
}

$test_dir = Get-Location

if (-not (Test-Path $compiler_path)) {
    Write-Host "ERROR: Compilador no encontrado en: $compiler_path" -ForegroundColor Red
    Write-Host "Por favor, compila el proyecto primero:" -ForegroundColor Yellow
    Write-Host "  cd CORE\rust" -ForegroundColor Yellow
    Write-Host "  cargo build --release" -ForegroundColor Yellow
    exit 1
}

$tests = Get-ChildItem -Path $test_dir -Filter "test_strings_*.ad"

$passed = 0
$failed = 0
$total = $tests.Count

foreach ($test in $tests) {
    Write-Host "[VERIFY] $($test.Name)" -ForegroundColor White
    
    $asm_file = $test.Name -replace "\.ad$", ".asm"
    
    try {
        $compile_output = & $compiler_path compile $test.FullName --backend auto -o $asm_file 2>&1
        $compile_exit_code = $LASTEXITCODE
        
        if ($compile_exit_code -eq 0) {
            if (Test-Path $asm_file) {
                $file_size = (Get-Item $asm_file).Length
                Write-Host "  [PASS] Compilación exitosa ($file_size bytes)" -ForegroundColor Green
                $passed++
            } else {
                Write-Host "  [FAIL] Archivo ASM no generado" -ForegroundColor Red
                Write-Host "  Output:" -ForegroundColor Yellow
                $compile_output | Select-Object -First 3 | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
                $failed++
            }
        } else {
            Write-Host "  [FAIL] Error en compilación" -ForegroundColor Red
            Write-Host "  Detalles:" -ForegroundColor Yellow
            $compile_output | Select-Object -First 5 | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
            $failed++
        }
    } catch {
        Write-Host "  [FAIL] Error: $_" -ForegroundColor Red
        Write-Host "  Stack trace: $($_.ScriptStackTrace)" -ForegroundColor DarkGray
        $failed++
    }
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Resumen: $passed/$total tests compilaron correctamente" -ForegroundColor Cyan
if ($failed -gt 0) {
    Write-Host "  Fallaron: $failed" -ForegroundColor Red
}
Write-Host "========================================" -ForegroundColor Cyan

