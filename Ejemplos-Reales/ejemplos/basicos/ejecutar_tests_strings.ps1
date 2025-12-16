# Script PowerShell para ejecutar todos los tests de strings
# Ejecuta cada test y muestra el resultado

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Tests de Strings Avanzados - ADead" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$tests = @(
    "test_strings_basico.ad",
    "test_strings_concat.ad",
    "test_strings_slice.ad",
    "test_strings_upper.ad",
    "test_strings_lower.ad",
    "test_strings_len.ad",
    "test_strings_completo.ad",
    "test_strings_concatenacion_multiple.ad",
    "test_strings_slicing_avanzado.ad",
    "test_strings_metodos_combinados.ad",
    "test_strings_len_completo.ad",
    "test_strings_operaciones_complejas.ad",
    "test_strings_variables.ad",
    "test_strings_print_expresiones.ad",
    "test_strings_comparacion.ad"
)

# Resolver path del compilador de forma más robusta
$script_dir = Split-Path -Parent $MyInvocation.MyCommand.Path
$workspace_root = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $script_dir))
$compiler_path = Join-Path $workspace_root "CORE\rust\target\release\adeadc.exe"

# Si no existe, intentar path relativo
if (-not (Test-Path $compiler_path)) {
    $compiler_path = "..\..\..\CORE\rust\target\release\adeadc.exe"
}

$test_dir = Get-Location

# Verificar que el compilador existe
if (-not (Test-Path $compiler_path)) {
    Write-Host "ERROR: Compilador no encontrado en: $compiler_path" -ForegroundColor Red
    Write-Host "Por favor, compila el proyecto primero:" -ForegroundColor Yellow
    Write-Host "  cd CORE\rust" -ForegroundColor Yellow
    Write-Host "  cargo build --release" -ForegroundColor Yellow
    exit 1
}

$passed = 0
$failed = 0
$total = $tests.Count

foreach ($test in $tests) {
    $test_path = Join-Path $test_dir $test
    
    if (-not (Test-Path $test_path)) {
        Write-Host "[SKIP] $test - No encontrado" -ForegroundColor Yellow
        continue
    }
    
    Write-Host "[TEST] $test" -ForegroundColor White
    
    # Compilar
    $asm_file = $test -replace "\.ad$", ".asm"
    $exe_file = $test -replace "\.ad$", "_cpp.exe"
    
    try {
        # Compilar con backend auto (selecciona automáticamente el mejor)
        $compile_output = & $compiler_path compile $test_path --backend auto -o $asm_file 2>&1
        $compile_exit_code = $LASTEXITCODE
        
        if ($compile_exit_code -ne 0) {
            Write-Host "  [FAIL] Compilación falló" -ForegroundColor Red
            Write-Host "  Detalles:" -ForegroundColor Yellow
            $compile_output | Select-Object -First 5 | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
            $failed++
            continue
        }
        
        if (-not (Test-Path $asm_file)) {
            Write-Host "  [FAIL] Archivo ASM no generado" -ForegroundColor Red
            $failed++
            continue
        }
        
        # Compilar ASM a EXE (si existe nasm y gcc)
        $nasm_path = "nasm"
        $gcc_path = "gcc"
        
        if ((Get-Command $nasm_path -ErrorAction SilentlyContinue) -and 
            (Get-Command $gcc_path -ErrorAction SilentlyContinue)) {
            
            # Convertir ASM a objeto
            $obj_file = $asm_file -replace "\.asm$", ".obj"
            $nasm_output = & $nasm_path -f win64 $asm_file -o $obj_file 2>&1
            
            if ($LASTEXITCODE -eq 0 -and (Test-Path $obj_file)) {
                # Linkear
                $gcc_output = & $gcc_path $obj_file -o $exe_file 2>&1
                
                if ($LASTEXITCODE -eq 0 -and (Test-Path $exe_file)) {
                    # Ejecutar
                    $output = & ".\$exe_file" 2>&1
                    Write-Host "  [OUTPUT]" -ForegroundColor Green
                    if ($output) {
                        $output | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
                    } else {
                        Write-Host "    (sin output)" -ForegroundColor DarkGray
                    }
                    Write-Host "  [PASS]" -ForegroundColor Green
                    $passed++
                } else {
                    Write-Host "  [FAIL] Linkeo falló" -ForegroundColor Red
                    $gcc_output | Select-Object -First 3 | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
                    $failed++
                }
            } else {
                Write-Host "  [FAIL] Ensamblado falló" -ForegroundColor Red
                $nasm_output | Select-Object -First 3 | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
                $failed++
            }
        } else {
            Write-Host "  [SKIP] NASM/GCC no encontrado, solo compilación" -ForegroundColor Yellow
            Write-Host "  [PASS] Compilación exitosa: $asm_file" -ForegroundColor Green
            $passed++
        }
    } catch {
        Write-Host "  [FAIL] Error: $_" -ForegroundColor Red
        Write-Host "  Stack trace: $($_.ScriptStackTrace)" -ForegroundColor DarkGray
        $failed++
    }
    
    Write-Host ""
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Resumen: $passed/$total tests pasaron" -ForegroundColor Cyan
if ($failed -gt 0) {
    Write-Host "  Fallaron: $failed" -ForegroundColor Red
}
Write-Host "========================================" -ForegroundColor Cyan

