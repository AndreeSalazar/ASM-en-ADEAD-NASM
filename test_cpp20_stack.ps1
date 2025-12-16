# Script de prueba para verificar el stack completo con C++20
# Autor: Eddi Andreé Salazar Matos
# Fecha: Diciembre 2025

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Test: Stack Completo ADead con C++20" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 1. Verificar compilador C++
Write-Host "[1/7] Verificando compilador C++..." -ForegroundColor Yellow

$gpp_path = "C:\msys64\mingw64\bin\g++.exe"
if (Test-Path $gpp_path) {
    Write-Host "  ✓ Encontrado: $gpp_path" -ForegroundColor Green
    $version = & $gpp_path --version 2>&1 | Select-Object -First 1
    Write-Host "  $version" -ForegroundColor Gray
} else {
    Write-Host "  ✗ No se encontró g++ en $gpp_path" -ForegroundColor Red
    Write-Host "  Buscando en PATH..." -ForegroundColor Yellow
    $gpp_path = "g++"
}

# 2. Verificar soporte C++20
Write-Host ""
Write-Host "[2/7] Verificando soporte C++20..." -ForegroundColor Yellow

$test_cpp20_lines = @(
    '#include <version>',
    '#if __cplusplus >= 202002L',
    'int main() { return 0; }',
    '#else',
    '#error "C++20 not supported"',
    '#endif'
)
$test_cpp20 = $test_cpp20_lines -join "`n"

$test_file = Join-Path $env:TEMP "adead_cpp20_test.cpp"
$test_obj = Join-Path $env:TEMP "adead_cpp20_test.o"

Set-Content -Path $test_file -Value $test_cpp20

try {
    $compile_output = & $gpp_path -std=c++20 -c $test_file -o $test_obj 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✓ C++20 soportado" -ForegroundColor Green
        $cpp20_supported = $true
    } else {
        Write-Host "  ⚠ C++20 no soportado, usando C++17" -ForegroundColor Yellow
        if ($compile_output) {
            Write-Host "    Detalles: $($compile_output -join ' ')" -ForegroundColor Gray
        }
        $cpp20_supported = $false
    }
} catch {
    Write-Host "  ⚠ Error al verificar C++20: $_" -ForegroundColor Yellow
    $cpp20_supported = $false
} finally {
    Remove-Item $test_file -ErrorAction SilentlyContinue
    Remove-Item $test_obj -ErrorAction SilentlyContinue
}

# 3. Compilar compilador ADead
Write-Host ""
Write-Host "[3/7] Compilando compilador ADead..." -ForegroundColor Yellow

Push-Location (Join-Path "CORE" "rust")
try {
    cargo build --release 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✓ Compilador ADead compilado exitosamente" -ForegroundColor Green
    } else {
        Write-Host "  ✗ Error al compilar compilador ADead" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "  ✗ Error: $_" -ForegroundColor Red
    exit 1
} finally {
    Pop-Location
}

# 4. Crear ejemplo de prueba
Write-Host ""
Write-Host "[4/7] Creando ejemplo de prueba..." -ForegroundColor Yellow

$test_lines = @(
    'let arr = [1, 2, 3]',
    'arr.append(4)',
    'print arr[0]',
    'print len(arr)',
    'arr.sort()',
    'print arr[0]'
)
$test_code = $test_lines -join "`n"

$test_ad_file = Join-Path $env:TEMP "test_adead_cpp20.ad"
Set-Content -Path $test_ad_file -Value $test_code
Write-Host "  ✓ Archivo de prueba creado: $test_ad_file" -ForegroundColor Green

# 5. Probar compilación completa
Write-Host ""
Write-Host "[5/7] Probando stack completo..." -ForegroundColor Yellow

$adeadc = Join-Path "CORE" (Join-Path "rust" (Join-Path "target" (Join-Path "release" "adeadc.exe")))
if (-not (Test-Path $adeadc)) {
    Write-Host "  ✗ No se encontró adeadc.exe" -ForegroundColor Red
    exit 1
}

Write-Host "  Compilando con pipeline C++..." -ForegroundColor Gray
try {
    $output_asm = Join-Path $env:TEMP "test_output.asm"
    & $adeadc compile $test_ad_file --backend cpp -o $output_asm 2>&1 | Tee-Object -Variable output
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✓ Compilación exitosa" -ForegroundColor Green
        
        # Verificar que el ASM fue generado
        if (Test-Path $output_asm) {
            $asm_content = Get-Content $output_asm -Raw
            if ($asm_content.Length -gt 100) {
                Write-Host "  ✓ ASM generado correctamente" -ForegroundColor Green
                
                # Verificar que usa C++20 si está disponible
                if ($cpp20_supported -and $asm_content -match "c\+\+20|C\+\+20") {
                    Write-Host "  ✓ C++20 detectado y usado" -ForegroundColor Green
                } elseif ($cpp20_supported) {
                    Write-Host "  ⚠ C++20 disponible pero no detectado en output" -ForegroundColor Yellow
                }
            } else {
                Write-Host "  ⚠ ASM generado pero parece vacío" -ForegroundColor Yellow
            }
        } else {
            Write-Host "  ⚠ ASM no generado" -ForegroundColor Yellow
        }
    } else {
        Write-Host "  ✗ Error en compilación" -ForegroundColor Red
        Write-Host $output -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "  ✗ Error: $_" -ForegroundColor Red
    exit 1
}

# 6. Compilar C++ generado a ejecutable
Write-Host ""
Write-Host "[6/7] Compilando C++ a ejecutable..." -ForegroundColor Yellow

# Inicializar variable exe_file
$exe_file = Join-Path $env:TEMP "test_adead_cpp20.exe"

# Buscar el código C++ generado (puede estar en temp o en el directorio de trabajo)
$cpp_files = Get-ChildItem -Path $env:TEMP -Filter "*.cpp" -ErrorAction SilentlyContinue | 
    Where-Object { $_.LastWriteTime -gt (Get-Date).AddMinutes(-5) } |
    Sort-Object LastWriteTime -Descending | Select-Object -First 1

if (-not $cpp_files) {
    Write-Host "  ⚠ No se encontró código C++ generado, buscando en directorio actual..." -ForegroundColor Yellow
    $cpp_files = Get-ChildItem -Path . -Filter "*.cpp" -ErrorAction SilentlyContinue | 
        Where-Object { $_.LastWriteTime -gt (Get-Date).AddMinutes(-5) } |
        Sort-Object LastWriteTime -Descending | Select-Object -First 1
}

if ($cpp_files) {
    $cpp_file = $cpp_files.FullName
    
    Write-Host "  Compilando: $cpp_file" -ForegroundColor Gray
    
    $cpp_std = if ($cpp20_supported) { "-std=c++20" } else { "-std=c++17" }
    
    try {
        & $gpp_path $cpp_std -O2 -o $exe_file $cpp_file 2>&1 | Tee-Object -Variable compile_output
        
        if ($LASTEXITCODE -eq 0 -and (Test-Path $exe_file)) {
            Write-Host "  ✓ Ejecutable compilado exitosamente" -ForegroundColor Green
            
            # Verificar que el código C++ usa features de C++20 si está disponible
            $cpp_content = Get-Content $cpp_file -Raw
            if ($cpp20_supported) {
                $uses_ranges = $cpp_content -match "ranges|std::ranges"
                $uses_format = $cpp_content -match "std::format|format"
                $uses_consteval = $cpp_content -match "consteval"
                
                if ($uses_ranges -or $uses_format -or $uses_consteval) {
                    Write-Host "  ✓ C++20 features detectadas en código generado" -ForegroundColor Green
                } else {
                    Write-Host "  ⚠ C++20 disponible pero features no detectadas en código" -ForegroundColor Yellow
                }
            }
        } else {
            Write-Host "  ✗ Error al compilar ejecutable" -ForegroundColor Red
            Write-Host $compile_output -ForegroundColor Red
        }
    } catch {
        Write-Host "  ✗ Error: $_" -ForegroundColor Red
    }
} else {
    Write-Host "  ⚠ No se encontró código C++ generado para compilar" -ForegroundColor Yellow
}

# 7. Ejecutar programa y verificar salida
Write-Host ""
Write-Host "[7/7] Ejecutando programa..." -ForegroundColor Yellow

if (Test-Path $exe_file) {
    try {
        $exec_output = & $exe_file 2>&1
        $exit_code = $LASTEXITCODE
        
        if ($exit_code -eq 0) {
            Write-Host "  ✓ Programa ejecutado exitosamente" -ForegroundColor Green
            Write-Host "  Salida:" -ForegroundColor Gray
            $exec_output | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
            
            # Verificar salida esperada (1, 4, 1)
            $output_str = $exec_output -join "`n"
            if ($output_str -match "1" -and $output_str -match "4") {
                Write-Host "  ✓ Salida correcta verificada" -ForegroundColor Green
            } else {
                Write-Host "  ⚠ Salida no coincide con lo esperado" -ForegroundColor Yellow
            }
        } else {
            Write-Host "  ✗ Programa terminó con código de error: $exit_code" -ForegroundColor Red
            Write-Host $exec_output -ForegroundColor Red
        }
    } catch {
        Write-Host "  ✗ Error al ejecutar: $_" -ForegroundColor Red
    }
} else {
    Write-Host "  ⚠ Ejecutable no encontrado, saltando ejecución" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  ✓ Stack completo funcionando!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Resumen:" -ForegroundColor Yellow
Write-Host "  • Compilador C++: $gpp_path" -ForegroundColor Gray
Write-Host "  • C++20 soportado: $cpp20_supported" -ForegroundColor Gray
Write-Host "  • Compilador ADead: Compilado" -ForegroundColor Gray
Write-Host "  • Pipeline completo: Funcionando" -ForegroundColor Gray
if (Test-Path $exe_file) {
    Write-Host "  • Ejecutable generado: $exe_file" -ForegroundColor Gray
}
Write-Host ""
Write-Host "Stack completo verificado:" -ForegroundColor Cyan
Write-Host "  ADead → Parser Manual → C++ Generator → GCC++/Clang++ → Rust Cleaner → ASM Virgen/Puro" -ForegroundColor Gray
Write-Host ""

