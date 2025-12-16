# Script PowerShell para compilar y ejecutar todos los tests con el stack completo
# Stack: ADead → Parser Manual → C++ Generator → GCC++/Clang++ → Rust Cleaner → ASM → NASM → .obj → Zig/GCC/Clang (linker) → .exe
# Autor: Eddi Andreé Salazar Matos
# Fecha: Diciembre 2025

param(
    [string]$Filter = "*.ad",
    [switch]$NoExecute = $false,
    [switch]$Verbose = $false
)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Build Completo con Stack Completo" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Stack:" -ForegroundColor Cyan
Write-Host "  ADead -> Parser Manual -> C++ Generator -> GCC++/Clang++ -> Rust Cleaner -> ASM -> NASM -> .obj -> Zig/GCC/Clang (linker) -> .exe" -ForegroundColor Gray
Write-Host ""

# Obtener directorio del script
$script_dir = Split-Path -Parent $MyInvocation.MyCommand.Path
if (-not $script_dir) {
    $script_dir = $PWD.Path
}
Set-Location $script_dir

# ========================================
# 1. DETECTAR HERRAMIENTAS
# ========================================

Write-Host "[1/6] Detectando herramientas..." -ForegroundColor Yellow
Write-Host ""

# Buscar compilador ADead
$adeadc_path = $null
$search_paths = @(
    "..\..\..\CORE\rust\target\release\adeadc.exe",
    "..\..\..\target\release\adeadc.exe",
    "CORE\rust\target\release\adeadc.exe",
    "target\release\adeadc.exe"
)

foreach ($path in $search_paths) {
    if (Test-Path $path) {
        $adeadc_path = (Resolve-Path $path).Path
        break
    }
}

# Buscar en PATH
if (-not $adeadc_path) {
    try {
        $adeadc_test = & adeadc.exe --version 2>&1
        if ($LASTEXITCODE -eq 0) {
            $adeadc_path = "adeadc.exe"
        }
    } catch {
        # No encontrado
    }
}

if (-not $adeadc_path) {
    Write-Host "[ERROR] Compilador ADead no encontrado" -ForegroundColor Red
    Write-Host "Por favor, compila el proyecto primero:" -ForegroundColor Yellow
    Write-Host "  cd CORE\rust" -ForegroundColor Gray
    Write-Host "  cargo build --release" -ForegroundColor Gray
    exit 1
}

Write-Host "  [OK] Compilador ADead: $adeadc_path" -ForegroundColor Green

# Buscar compilador C++ (Clang++ o GCC++)
$cpp_compiler = $null
$cpp_compiler_path = $null

# Priorizar Clang++ (mejor soporte C++20)
$clang_paths = @(
    "clang++",
    "C:\Program Files\LLVM\bin\clang++.exe",
    "C:\msys64\clang64\bin\clang++.exe"
)

foreach ($path in $clang_paths) {
    try {
        if (Test-Path $path) {
            $test_output = & $path --version 2>&1 | Select-Object -First 1
            if ($LASTEXITCODE -eq 0) {
                $cpp_compiler = "clang++"
                $cpp_compiler_path = $path
                break
            }
        } else {
            $test_output = & $path --version 2>&1 | Select-Object -First 1
            if ($LASTEXITCODE -eq 0) {
                $cpp_compiler = "clang++"
                $cpp_compiler_path = $path
                break
            }
        }
    } catch {
        continue
    }
}

# Si no hay Clang++, buscar GCC++
if (-not $cpp_compiler) {
    $gcc_paths = @(
        "g++",
        "C:\msys64\mingw64\bin\g++.exe"
    )
    
    foreach ($path in $gcc_paths) {
        try {
            if (Test-Path $path) {
                $test_output = & $path --version 2>&1 | Select-Object -First 1
                if ($LASTEXITCODE -eq 0) {
                    $cpp_compiler = "g++"
                    $cpp_compiler_path = $path
                    break
                }
            } else {
                $test_output = & $path --version 2>&1 | Select-Object -First 1
                if ($LASTEXITCODE -eq 0) {
                    $cpp_compiler = "g++"
                    $cpp_compiler_path = $path
                    break
                }
            }
        } catch {
            continue
        }
    }
}

if (-not $cpp_compiler) {
    Write-Host "[ERROR] Compilador C++ no encontrado (GCC++ o Clang++)" -ForegroundColor Red
    Write-Host "Por favor, instala GCC++ o Clang++:" -ForegroundColor Yellow
    Write-Host "  - MSYS2: https://www.msys2.org/" -ForegroundColor Gray
    Write-Host "  - LLVM/Clang: https://llvm.org/builds/" -ForegroundColor Gray
    exit 1
}

Write-Host "  [OK] Compilador C++: $cpp_compiler_path" -ForegroundColor Green

# Buscar NASM
$nasm_path = $null
try {
    $nasm_test = & nasm -v 2>&1
    if ($LASTEXITCODE -eq 0) {
        $nasm_path = "nasm"
    }
} catch {
    # Buscar en ubicación conocida
    $known_nasm = "C:\Users\andre\AppData\Local\bin\NASM\nasm.exe"
    if (Test-Path $known_nasm) {
        $nasm_path = $known_nasm
    }
}

if (-not $nasm_path) {
    Write-Host "[ERROR] NASM no encontrado" -ForegroundColor Red
    Write-Host "Por favor, instala NASM: https://www.nasm.us/" -ForegroundColor Yellow
    exit 1
}

Write-Host "  [OK] NASM: $nasm_path" -ForegroundColor Green

# Buscar Linker (Zig o GCC/Clang)
$linker = $null
$linker_path = $null

# Priorizar Zig (más fácil de usar)
try {
    $zig_test = & zig version 2>&1
    if ($LASTEXITCODE -eq 0) {
        $linker = "zig"
        $linker_path = "zig"
    }
} catch {
    # Buscar en ubicaciones conocidas
    $zig_paths = @(
        "C:\zig-x86_64-windows-0.16.0-dev.1484+d0ba6642b\zig.exe",
        "C:\Users\andre\AppData\Local\Microsoft\WinGet\Packages\zig.zig_Microsoft.Winget.Source_8wekyb3d8bbwe\zig-x86_64-windows-0.14.1\zig.exe"
    )
    
    foreach ($path in $zig_paths) {
        if (Test-Path $path) {
            $linker = "zig"
            $linker_path = $path
            break
        }
    }
}

# Si no hay Zig, usar GCC/Clang como linker
if (-not $linker) {
    if ($cpp_compiler -eq "clang++") {
        $linker = "clang++"
        $linker_path = $cpp_compiler_path
    } else {
        $linker = "g++"
        $linker_path = $cpp_compiler_path
    }
}

Write-Host "  [OK] Linker: $linker_path" -ForegroundColor Green
Write-Host ""

# ========================================
# 2. ENCONTRAR ARCHIVOS .ad
# ========================================

Write-Host "[2/6] Buscando archivos .ad..." -ForegroundColor Yellow

$ad_files = Get-ChildItem -Path . -Filter $Filter | Where-Object { 
    $_.Name -notmatch "^test-import" -and 
    $_.Name -notmatch "^utils" -and 
    $_.Name -notmatch "^math" -and
    $_.Name -ne "main-import.ad"
} | Sort-Object Name

if ($ad_files.Count -eq 0) {
    Write-Host "[ERROR] No se encontraron archivos .ad" -ForegroundColor Red
    exit 1
}

Write-Host "  [OK] Encontrados $($ad_files.Count) archivos .ad" -ForegroundColor Green
Write-Host ""

# ========================================
# 3. COMPILAR CADA ARCHIVO
# ========================================

Write-Host "[3/6] Compilando archivos..." -ForegroundColor Yellow
Write-Host ""

$results = @()
$compiled = 0
$failed = 0

foreach ($ad_file in $ad_files) {
    $test_name = $ad_file.BaseName
    $asm_file = "$test_name.asm"
    $obj_file = "$test_name.obj"
    $exe_file = "$test_name.exe"
    
    Write-Host "  Compilando: $($ad_file.Name)..." -ForegroundColor Gray
    
    # Paso 1: Compilar ADead → ASM
    try {
        $compile_output = & $adeadc_path compile $ad_file.FullName --backend cpp -o $asm_file 2>&1
        $compile_exit = $LASTEXITCODE
        
        if ($compile_exit -ne 0) {
            Write-Host "    [FAIL] Compilacion ADead fallo" -ForegroundColor Red
            if ($Verbose) {
                $compile_output | Select-Object -First 3 | ForEach-Object { Write-Host "      $_" -ForegroundColor DarkGray }
            }
            $results += @{
                Name = $test_name
                Status = "FAIL"
                Error = "Compilacion ADead fallo"
            }
            $failed++
            continue
        }
        
        if (-not (Test-Path $asm_file)) {
            Write-Host "    [FAIL] Archivo ASM no generado" -ForegroundColor Red
            $results += @{
                Name = $test_name
                Status = "FAIL"
                Error = "ASM no generado"
            }
            $failed++
            continue
        }
        
        # Verificar que es ASM y no C++
        $asm_content = Get-Content $asm_file -Raw -ErrorAction SilentlyContinue
        if ($asm_content -match "// Codigo C\+\+ generado|#include") {
            Write-Host "    [FAIL] Se genero C++ en lugar de ASM (compilador C++ no encontrado)" -ForegroundColor Red
            $results += @{
                Name = $test_name
                Status = "FAIL"
                Error = "C++ generado en lugar de ASM"
            }
            $failed++
            continue
        }
        
        Write-Host "    [OK] ASM generado: $asm_file" -ForegroundColor Green
        
        # Paso 2: Ensamblar ASM → .obj con NASM
        try {
            $nasm_output = & $nasm_path -f win64 $asm_file -o $obj_file 2>&1
            $nasm_exit = $LASTEXITCODE
            
            if ($nasm_exit -ne 0 -or -not (Test-Path $obj_file)) {
                Write-Host "    [FAIL] Ensamblado con NASM fallo" -ForegroundColor Red
                if ($Verbose) {
                    $nasm_output | Select-Object -First 3 | ForEach-Object { Write-Host "      $_" -ForegroundColor DarkGray }
                }
                $results += @{
                    Name = $test_name
                    Status = "FAIL"
                    Error = "Ensamblado fallo"
                }
                $failed++
                continue
            }
            
            Write-Host "    [OK] Objeto generado: $obj_file" -ForegroundColor Green
            
            # Paso 3: Linkear .obj → .exe
            try {
                if ($linker -eq "zig") {
                    $link_output = & $linker_path build-exe $obj_file -target x86_64-windows -lc -o $exe_file 2>&1
                    $link_exit = $LASTEXITCODE
                    
                    # Si falla con -lc, intentar sin
                    if ($link_exit -ne 0) {
                        $link_output = & $linker_path build-exe $obj_file -target x86_64-windows -o $exe_file 2>&1
                        $link_exit = $LASTEXITCODE
                    }
                } else {
                    # Usar GCC/Clang como linker
                    $link_output = & $linker_path $obj_file -o $exe_file 2>&1
                    $link_exit = $LASTEXITCODE
                }
                
                if ($link_exit -ne 0 -or -not (Test-Path $exe_file)) {
                    Write-Host "    [FAIL] Linkeo fallo" -ForegroundColor Red
                    if ($Verbose) {
                        $link_output | Select-Object -First 3 | ForEach-Object { Write-Host "      $_" -ForegroundColor DarkGray }
                    }
                    $results += @{
                        Name = $test_name
                        Status = "FAIL"
                        Error = "Linkeo fallo"
                    }
                    $failed++
                    continue
                }
                
                Write-Host "    [OK] Ejecutable generado: $exe_file" -ForegroundColor Green
                
                # Paso 4: Ejecutar (si no está deshabilitado)
                if (-not $NoExecute) {
                    try {
                        $exec_output = & ".\$exe_file" 2>&1
                        $exec_exit = $LASTEXITCODE
                        
                        $results += @{
                            Name = $test_name
                            Status = "PASS"
                            ExitCode = $exec_exit
                            Output = $exec_output
                        }
                        $compiled++
                        
                        if ($Verbose -and $exec_output) {
                            Write-Host "    [OUTPUT]" -ForegroundColor Cyan
                            $exec_output | ForEach-Object { Write-Host "      $_" -ForegroundColor DarkGray }
                        }
                    } catch {
                        Write-Host "    [WARN] Ejecucion fallo: $_" -ForegroundColor Yellow
                        $results += @{
                            Name = $test_name
                            Status = "PASS_BUILD"
                            Error = "Ejecucion fallo"
                        }
                        $compiled++
                    }
                } else {
                    $results += @{
                        Name = $test_name
                        Status = "PASS_BUILD"
                    }
                    $compiled++
                }
                
            } catch {
                Write-Host "    [FAIL] Error en linkeo: $_" -ForegroundColor Red
                $results += @{
                    Name = $test_name
                    Status = "FAIL"
                    Error = "Error en linkeo: $_"
                }
                $failed++
            }
        } catch {
            Write-Host "    [FAIL] Error en ensamblado: $_" -ForegroundColor Red
            $results += @{
                Name = $test_name
                Status = "FAIL"
                Error = "Error en ensamblado: $_"
            }
            $failed++
        }
    } catch {
        Write-Host "    [FAIL] Error en compilacion: $_" -ForegroundColor Red
        $results += @{
            Name = $test_name
            Status = "FAIL"
            Error = "Error en compilacion: $_"
        }
        $failed++
    }
    
    Write-Host ""
}

# ========================================
# 4. RESUMEN
# ========================================

Write-Host "[4/6] Resumen de compilacion..." -ForegroundColor Yellow
Write-Host ""

$total = $ad_files.Count
$passed_build = ($results | Where-Object { $_.Status -eq "PASS" -or $_.Status -eq "PASS_BUILD" }).Count

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Resultados" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Total archivos: $total" -ForegroundColor White
Write-Host "Compilados exitosamente: $passed_build" -ForegroundColor Green
Write-Host "Fallidos: $failed" -ForegroundColor $(if ($failed -eq 0) { "Green" } else { "Red" })
Write-Host ""

if ($failed -gt 0) {
    Write-Host "Archivos que fallaron:" -ForegroundColor Yellow
    foreach ($result in $results) {
        if ($result.Status -eq "FAIL") {
            Write-Host "  - $($result.Name): $($result.Error)" -ForegroundColor Red
        }
    }
    Write-Host ""
}

# ========================================
# 5. LISTAR EJECUTABLES GENERADOS
# ========================================

Write-Host "[5/6] Ejecutables generados..." -ForegroundColor Yellow
Write-Host ""

$exe_files = Get-ChildItem -Path . -Filter "*.exe" | Where-Object { 
    $_.LastWriteTime -gt (Get-Date).AddMinutes(-10) 
} | Sort-Object Name

if ($exe_files.Count -gt 0) {
    Write-Host "Ejecutables listos:" -ForegroundColor Green
    foreach ($exe in $exe_files) {
        $size_kb = [math]::Round($exe.Length / 1KB, 2)
        Write-Host "  - $($exe.Name) ($size_kb KB)" -ForegroundColor Gray
    }
} else {
    Write-Host "No se generaron ejecutables" -ForegroundColor Yellow
}

Write-Host ""

# ========================================
# 6. INFORMACION DEL STACK USADO
# ========================================

Write-Host "[6/6] Stack utilizado:" -ForegroundColor Yellow
Write-Host ""
Write-Host "Compilador ADead: $adeadc_path" -ForegroundColor Gray
Write-Host "Compilador C++: $cpp_compiler_path" -ForegroundColor Gray
Write-Host "Ensamblador: $nasm_path" -ForegroundColor Gray
Write-Host "Linker: $linker_path" -ForegroundColor Gray
Write-Host ""

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Build completo finalizado!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

if (-not $NoExecute) {
    Write-Host "Para ejecutar un test individual:" -ForegroundColor Cyan
    Write-Host "  .\test_strings_basico.exe" -ForegroundColor Gray
    Write-Host ""
}













