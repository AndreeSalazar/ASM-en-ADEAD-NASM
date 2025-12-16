# Script PowerShell simplificado para compilar y ejecutar un test individual
# Usa el stack completo detectado automáticamente
# Uso: BUILD-Y-EJECUTAR.ps1 test_strings_basico.ad

param(
    [Parameter(Mandatory=$true)]
    [string]$TestFile
)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Build y Ejecutar: $TestFile" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Obtener directorio del script
$script_dir = Split-Path -Parent $MyInvocation.MyCommand.Path
if (-not $script_dir) {
    $script_dir = $PWD.Path
}
Set-Location $script_dir

# Verificar que el archivo existe (buscar en directorio actual)
$test_file_path = $TestFile
if (-not (Test-Path $test_file_path)) {
    # Intentar solo el nombre del archivo
    $test_file_path = Join-Path $script_dir (Split-Path -Leaf $TestFile)
    if (-not (Test-Path $test_file_path)) {
        Write-Host "[ERROR] Archivo no encontrado: $TestFile" -ForegroundColor Red
        Write-Host "Buscado en: $test_file_path" -ForegroundColor Yellow
        exit 1
    }
}

# Buscar compilador ADead
$adeadc_path = $null
$search_paths = @(
    "..\..\..\CORE\rust\target\release\adeadc.exe",
    "..\..\..\target\release\adeadc.exe",
    "CORE\rust\target\release\adeadc.exe"
)

foreach ($path in $search_paths) {
    if (Test-Path $path) {
        $adeadc_path = (Resolve-Path $path).Path
        break
    }
}

if (-not $adeadc_path) {
    try {
        $test = & adeadc.exe --version 2>&1
        if ($LASTEXITCODE -eq 0) {
            $adeadc_path = "adeadc.exe"
        }
    } catch {}
}

if (-not $adeadc_path) {
    Write-Host "[ERROR] Compilador ADead no encontrado" -ForegroundColor Red
    Write-Host "Compila el proyecto: cd CORE\rust && cargo build --release" -ForegroundColor Yellow
    exit 1
}

# Detectar compilador C++
$cpp_compiler = $null
$cpp_path = $null

# Buscar Clang++ primero (mejor C++20)
$clang_paths = @("clang++", "C:\Program Files\LLVM\bin\clang++.exe")
foreach ($path in $clang_paths) {
    try {
        if (Test-Path $path) {
            $test = & $path --version 2>&1
            if ($LASTEXITCODE -eq 0) {
                $cpp_compiler = "clang++"
                $cpp_path = $path
                break
            }
        } else {
            $test = & $path --version 2>&1
            if ($LASTEXITCODE -eq 0) {
                $cpp_compiler = "clang++"
                $cpp_path = $path
                break
            }
        }
    } catch {}
}

# Si no hay Clang++, buscar GCC++
if (-not $cpp_compiler) {
    $gcc_paths = @("g++", "C:\msys64\mingw64\bin\g++.exe")
    foreach ($path in $gcc_paths) {
        try {
            if (Test-Path $path) {
                $test = & $path --version 2>&1
                if ($LASTEXITCODE -eq 0) {
                    $cpp_compiler = "g++"
                    $cpp_path = $path
                    break
                }
            } else {
                $test = & $path --version 2>&1
                if ($LASTEXITCODE -eq 0) {
                    $cpp_compiler = "g++"
                    $cpp_path = $path
                    break
                }
            }
        } catch {}
    }
}

if (-not $cpp_compiler) {
    Write-Host "[ERROR] Compilador C++ no encontrado" -ForegroundColor Red
    exit 1
}

# Detectar NASM
$nasm_path = "nasm"
try {
    $test = & nasm -v 2>&1
    if ($LASTEXITCODE -ne 0) {
        $nasm_path = "C:\Users\andre\AppData\Local\bin\NASM\nasm.exe"
        if (-not (Test-Path $nasm_path)) {
            throw "NASM no encontrado"
        }
    }
} catch {
    Write-Host "[ERROR] NASM no encontrado" -ForegroundColor Red
    exit 1
}

# Detectar Linker (Zig preferido)
$linker = $null
$linker_path = $null

try {
    $test = & zig version 2>&1
    if ($LASTEXITCODE -eq 0) {
        $linker = "zig"
        $linker_path = "zig"
    }
} catch {
    $zig_paths = @(
        "C:\zig-x86_64-windows-0.16.0-dev.1484+d0ba6642b\zig.exe"
    )
    foreach ($path in $zig_paths) {
        if (Test-Path $path) {
            $linker = "zig"
            $linker_path = $path
            break
        }
    }
}

if (-not $linker) {
    $linker = $cpp_compiler
    $linker_path = $cpp_path
}

Write-Host "[INFO] Stack detectado:" -ForegroundColor Cyan
Write-Host "  ADead: $adeadc_path" -ForegroundColor Gray
Write-Host "  C++: $cpp_path" -ForegroundColor Gray
Write-Host "  NASM: $nasm_path" -ForegroundColor Gray
Write-Host "  Linker: $linker_path" -ForegroundColor Gray
Write-Host ""

# Compilar
$test_name = [System.IO.Path]::GetFileNameWithoutExtension($TestFile)
$asm_file = "$test_name.asm"
$obj_file = "$test_name.obj"
$exe_file = "$test_name.exe"

Write-Host "[1/4] Compilando ADead -> ASM..." -ForegroundColor Yellow
$compile_output = & $adeadc_path compile $test_file_path --backend cpp -o $asm_file 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "[FAIL] Compilacion fallo" -ForegroundColor Red
    $compile_output | Select-Object -First 5 | ForEach-Object { Write-Host "  $_" -ForegroundColor DarkGray }
    exit 1
}

# Verificar si se generó C++ en lugar de ASM
$asm_content = Get-Content $asm_file -Raw -ErrorAction SilentlyContinue
if ($asm_content -match "// Codigo C\+\+ generado|#include") {
    Write-Host "[INFO] Se genero C++ en lugar de ASM, compilando manualmente..." -ForegroundColor Yellow
    
    # El pipeline interno falló, necesitamos compilar C++ → ASM manualmente
    $cpp_file = "$test_name.cpp"
    Copy-Item $asm_file $cpp_file -Force
    
    Write-Host "[1.5/4] Compilando C++ -> ASM manualmente..." -ForegroundColor Yellow
    
    # Detectar estándar C++
    $cpp_std = "-std=c++17"
    try {
        $test_cpp20 = @'
#include <version>
#if __cplusplus >= 202002L
int main() { return 0; }
#else
#error "C++20 not supported"
#endif
'@
        $test_file = Join-Path $env:TEMP "adead_cpp20_test_$PID.cpp"
        $test_obj = Join-Path $env:TEMP "adead_cpp20_test_$PID.o"
        Set-Content -Path $test_file -Value $test_cpp20
        $test_compile = & $cpp_path -std=c++20 -c $test_file -o $test_obj 2>&1
        if ($LASTEXITCODE -eq 0) {
            $cpp_std = "-std=c++20"
        }
        Remove-Item $test_file -ErrorAction SilentlyContinue
        Remove-Item $test_obj -ErrorAction SilentlyContinue
    } catch {
        # Usar C++17 por defecto
    }
    
    # Compilar C++ → ASM (intentar C++20 primero, luego C++17 si falla)
    $cpp_to_asm_output = $null
    $compiled_ok = $false
    
    # Intentar con C++20 primero
    if ($cpp_std -eq "-std=c++20") {
        $cpp_to_asm_output = & $cpp_path -S -std=c++20 -O2 -fno-asynchronous-unwind-tables -fno-exceptions -fno-stack-protector -mno-red-zone -masm=intel -o $asm_file $cpp_file 2>&1
        if ($LASTEXITCODE -eq 0 -and (Test-Path $asm_file)) {
            $compiled_ok = $true
        } else {
            Write-Host "[WARN] C++20 fallo, intentando C++17..." -ForegroundColor Yellow
        }
    }
    
    # Si C++20 falló, intentar C++17
    if (-not $compiled_ok) {
        $cpp_to_asm_output = & $cpp_path -S -std=c++17 -O2 -fno-asynchronous-unwind-tables -fno-exceptions -fno-stack-protector -mno-red-zone -masm=intel -o $asm_file $cpp_file 2>&1
        if ($LASTEXITCODE -eq 0 -and (Test-Path $asm_file)) {
            $compiled_ok = $true
        }
    }
    
    if (-not $compiled_ok) {
        Write-Host "[FAIL] Compilacion C++ -> ASM fallo" -ForegroundColor Red
        $cpp_to_asm_output | Select-Object -First 10 | ForEach-Object { Write-Host "  $_" -ForegroundColor DarkGray }
        Write-Host ""
        Write-Host "Archivo C++ generado guardado en: $cpp_file" -ForegroundColor Yellow
        Write-Host "Puedes compilarlo manualmente con:" -ForegroundColor Yellow
        Write-Host "  $cpp_path -S -std=c++17 -O2 -masm=intel -o $asm_file $cpp_file" -ForegroundColor Gray
        exit 1
    }
    
    # Limpiar archivo C++ temporal
    Remove-Item $cpp_file -ErrorAction SilentlyContinue
    Write-Host "[OK] C++ compilado a ASM: $asm_file" -ForegroundColor Green
} else {
    Write-Host "[OK] ASM generado: $asm_file" -ForegroundColor Green
}

Write-Host "[2/4] Ensamblando ASM -> .obj..." -ForegroundColor Yellow

# Detectar formato ASM (GAS o NASM)
$asm_content_check = Get-Content $asm_file -Raw -ErrorAction SilentlyContinue
$is_gas = $asm_content_check -match "\.def|\.scl|\.type|\.text|\.globl|\.intel_syntax"

if ($is_gas) {
    Write-Host "[INFO] Formato GAS detectado, usando Clang para ensamblar..." -ForegroundColor Yellow
    
    # Usar Clang para ensamblar GAS directamente
    $clang_asm_output = & $cpp_path -c -target x86_64-pc-windows-msvc -o $obj_file $asm_file 2>&1
    
    if ($LASTEXITCODE -ne 0 -or -not (Test-Path $obj_file)) {
        Write-Host "[FAIL] Ensamblado con Clang fallo" -ForegroundColor Red
        $clang_asm_output | Select-Object -First 5 | ForEach-Object { Write-Host "  $_" -ForegroundColor DarkGray }
        
        # Intentar con GAS (as) si está disponible
        try {
            $gas_test = & as --version 2>&1
            if ($LASTEXITCODE -eq 0) {
                Write-Host "[INFO] Intentando con GAS (as)..." -ForegroundColor Yellow
                $gas_output = & as --64 -o $obj_file $asm_file 2>&1
                if ($LASTEXITCODE -ne 0 -or -not (Test-Path $obj_file)) {
                    Write-Host "[FAIL] Ensamblado con GAS tambien fallo" -ForegroundColor Red
                    exit 1
                }
            } else {
                exit 1
            }
        } catch {
            Write-Host "[ERROR] GAS (as) no disponible" -ForegroundColor Red
            exit 1
        }
    }
    Write-Host "[OK] Objeto generado (GAS): $obj_file" -ForegroundColor Green
} else {
    # Formato NASM
    Write-Host "[INFO] Formato NASM detectado, usando NASM..." -ForegroundColor Yellow
    $nasm_output = & $nasm_path -f win64 $asm_file -o $obj_file 2>&1
    if ($LASTEXITCODE -ne 0 -or -not (Test-Path $obj_file)) {
        Write-Host "[FAIL] Ensamblado con NASM fallo" -ForegroundColor Red
        $nasm_output | Select-Object -First 5 | ForEach-Object { Write-Host "  $_" -ForegroundColor DarkGray }
        exit 1
    }
    Write-Host "[OK] Objeto generado (NASM): $obj_file" -ForegroundColor Green
}

Write-Host "[3/4] Linkeando .obj -> .exe..." -ForegroundColor Yellow
if ($linker -eq "zig") {
    # Zig: zig build-exe archivo.obj -target x86_64-windows -lc -o archivo.exe
    $link_output = & $linker_path build-exe -target x86_64-windows -lc $obj_file -o $exe_file 2>&1
    if ($LASTEXITCODE -ne 0) {
        # Intentar sin -lc
        $link_output = & $linker_path build-exe -target x86_64-windows $obj_file -o $exe_file 2>&1
    }
} else {
    # GCC/Clang: g++ archivo.obj -o archivo.exe
    $link_output = & $linker_path $obj_file -o $exe_file 2>&1
}

if ($LASTEXITCODE -ne 0 -or -not (Test-Path $exe_file)) {
    Write-Host "[FAIL] Linkeo fallo" -ForegroundColor Red
    $link_output | Select-Object -First 3 | ForEach-Object { Write-Host "  $_" -ForegroundColor DarkGray }
    exit 1
}
Write-Host "[OK] Ejecutable generado: $exe_file" -ForegroundColor Green

Write-Host "[4/4] Ejecutando..." -ForegroundColor Yellow
Write-Host ""
$exec_output = & ".\$exe_file" 2>&1
$exec_exit = $LASTEXITCODE

if ($exec_output) {
    $exec_output | ForEach-Object { Write-Host $_ -ForegroundColor White }
}

Write-Host ""
Write-Host "[COMPLETADO] Codigo de salida: $exec_exit" -ForegroundColor $(if ($exec_exit -eq 0) { "Green" } else { "Yellow" })

