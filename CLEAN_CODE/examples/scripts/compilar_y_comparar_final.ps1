# Compilacion diferencial completa - Compara todos los archivos
# Muestra comparacion de ASM, OBJ y EXE (si se puede crear)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "COMPILACION DIFERENCIAL - 4 ELEMENTOS" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Archivos a procesar
$archivos = @(
    @{nombre="Sucio"; archivo="test_array_CLANG_dirty.asm"; tiene_main=$true},
    @{nombre="Basico"; archivo="test_array_CLANG_cleaned_basic.asm"; tiene_main=$false},
    @{nombre="Avanzado"; archivo="test_array_CLANG_cleaned_advanced.asm"; tiene_main=$false},
    @{nombre="Extremo"; archivo="test_array_CLANG_cleaned_extreme.asm"; tiene_main=$false}
)

# Buscar herramientas
$gas_path = "as"
$gcc_path = "gcc"

if (-not (Get-Command $gas_path -ErrorAction SilentlyContinue)) {
    $gas_path = "C:\msys64\mingw64\bin\as.exe"
}

if (-not (Get-Command $gcc_path -ErrorAction SilentlyContinue)) {
    $gcc_path = "C:\msys64\mingw64\bin\gcc.exe"
}

Write-Host "Herramientas:" -ForegroundColor Yellow
Write-Host "  GAS: $gas_path" -ForegroundColor White
Write-Host "  GCC: $gcc_path" -ForegroundColor White
Write-Host ""

# Funcion para obtener tamano
function Get-FileSize {
    param($file)
    if (Test-Path $file) { return (Get-Item $file).Length }
    return 0
}

# Resultados
$resultados = @()

# Procesar cada archivo
foreach ($arch in $archivos) {
    Write-Host "----------------------------------------" -ForegroundColor Yellow
    Write-Host "Procesando: $($arch.nombre)" -ForegroundColor Yellow
    Write-Host "Archivo: $($arch.archivo)" -ForegroundColor White
    
    if (-not (Test-Path $arch.archivo)) {
        Write-Host "  ERROR: Archivo no encontrado" -ForegroundColor Red
        continue
    }
    
    # Estadisticas del ASM
    $asm_size = Get-FileSize $arch.archivo
    $asm_lines = (Get-Content $arch.archivo).Count
    $asm_content = Get-Content $arch.archivo -Raw
    
    # Contar instrucciones aproximadas
    $inst_count = ([regex]::Matches($asm_content, "\b(mov|call|ret|push|pop|add|sub|jmp|jne|je|cmp|lea|shl|shr|mul|div|xor|and|or)\b")).Count
    
    Write-Host "  ASM: $asm_size bytes, $asm_lines lineas, ~$inst_count instrucciones" -ForegroundColor Cyan
    
    # Compilar a objeto
    $obj_file = $arch.archivo -replace "\.asm$", ".obj"
    Write-Host "  Compilando a objeto..." -ForegroundColor Cyan
    
    $gas_output = & $gas_path --64 -o $obj_file $arch.archivo 2>&1
    
    if ($LASTEXITCODE -eq 0 -and (Test-Path $obj_file)) {
        $obj_size = Get-FileSize $obj_file
        Write-Host "  OK: Objeto creado ($obj_size bytes)" -ForegroundColor Green
        
        # Intentar crear ejecutable
        $exe_file = $arch.archivo -replace "\.asm$", ".exe"
        $exe_size = 0
        $exe_created = $false
        
        if ($arch.tiene_main) {
            # Tiene main, enlazar con librerias de Windows
            Write-Host "  Enlazando (tiene main)..." -ForegroundColor Cyan
            # Intentar con librerias de Windows necesarias
            $gcc_output = & $gcc_path -O2 -o $exe_file $obj_file -lmsvcrt 2>&1
            
            if ($LASTEXITCODE -ne 0 -or -not (Test-Path $exe_file)) {
                # Intentar sin librerias especificas
                $gcc_output = & $gcc_path -O2 -o $exe_file $obj_file 2>&1
            }
            
            if (Test-Path $exe_file) {
                $exe_size = Get-FileSize $exe_file
                $exe_created = $true
                Write-Host "  OK: Ejecutable creado ($exe_size bytes)" -ForegroundColor Green
                
                # Ejecutar
                Write-Host "  Ejecutando..." -ForegroundColor Cyan
                try {
                    $exec_output = & ".\$exe_file" 2>&1
                    if ($exec_output) {
                        $exec_output | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
                    } else {
                        Write-Host "    (Sin salida)" -ForegroundColor Yellow
                    }
                } catch {
                    Write-Host "    (Error al ejecutar: $($_.Exception.Message))" -ForegroundColor Yellow
                }
            } else {
                Write-Host "  ADVERTENCIA: No se pudo crear ejecutable" -ForegroundColor Yellow
                $gcc_output | Select-Object -First 3 | ForEach-Object { Write-Host "    $_" -ForegroundColor Yellow }
            }
        } else {
            Write-Host "  INFO: Solo tiene funciones, no main (no se puede crear EXE standalone)" -ForegroundColor Yellow
        }
        
        $resultados += @{
            Nombre = $arch.nombre
            ASM_Size = $asm_size
            ASM_Lines = $asm_lines
            ASM_Inst = $inst_count
            OBJ_Size = $obj_size
            EXE_Size = $exe_size
            EXE_Created = $exe_created
        }
    } else {
        Write-Host "  ERROR: No se pudo compilar" -ForegroundColor Red
        $gas_output | Select-Object -First 5 | ForEach-Object { Write-Host "    $_" -ForegroundColor Red }
    }
    
    Write-Host ""
}

# Mostrar comparacion
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "COMPARACION FINAL - LOS 4 ELEMENTOS" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

if ($resultados.Count -gt 0) {
    Write-Host "+-----------+----------+----------+----------+----------+----------+" -ForegroundColor White
    Write-Host "| Version   | ASM (B)  | Lineas   | Inst.    | OBJ (B)  | EXE (B)  |" -ForegroundColor White
    Write-Host "+-----------+----------+----------+----------+----------+----------+" -ForegroundColor White
    
    foreach ($r in $resultados) {
        $exe_str = if ($r.EXE_Created) { 
            $r.EXE_Size.ToString().PadLeft(8) 
        } else { 
            "    N/A" 
        }
        Write-Host "| $($r.Nombre.PadRight(9)) | $($r.ASM_Size.ToString().PadLeft(8)) | $($r.ASM_Lines.ToString().PadLeft(8)) | $($r.ASM_Inst.ToString().PadLeft(8)) | $($r.OBJ_Size.ToString().PadLeft(8)) | $exe_str |" -ForegroundColor White
    }
    
    Write-Host "+-----------+----------+----------+----------+----------+----------+" -ForegroundColor White
    Write-Host ""
    
    # Calcular reducciones vs Sucio
    $sucio = $resultados | Where-Object { $_.Nombre -eq "Sucio" } | Select-Object -First 1
    if ($sucio) {
        Write-Host "REDUCCION vs Sucio:" -ForegroundColor Green
        Write-Host ""
        
        foreach ($r in $resultados) {
            if ($r.Nombre -ne "Sucio") {
                $asm_red = [math]::Round((($sucio.ASM_Size - $r.ASM_Size) / $sucio.ASM_Size) * 100, 1)
                $lines_red = [math]::Round((($sucio.ASM_Lines - $r.ASM_Lines) / $sucio.ASM_Lines) * 100, 1)
                $inst_red = if ($sucio.ASM_Inst -gt 0) {
                    [math]::Round((($sucio.ASM_Inst - $r.ASM_Inst) / $sucio.ASM_Inst) * 100, 1)
                } else { 0 }
                $obj_red = if ($sucio.OBJ_Size -gt 0) {
                    [math]::Round((($sucio.OBJ_Size - $r.OBJ_Size) / $sucio.OBJ_Size) * 100, 1)
                } else { 0 }
                
                Write-Host "$($r.Nombre):" -ForegroundColor Cyan
                Write-Host "  ASM:    -$asm_red%  ($($sucio.ASM_Size) -> $($r.ASM_Size) bytes)" -ForegroundColor White
                Write-Host "  Lineas: -$lines_red%  ($($sucio.ASM_Lines) -> $($r.ASM_Lines) lineas)" -ForegroundColor White
                Write-Host "  Inst:   -$inst_red%  (~$($sucio.ASM_Inst) -> ~$($r.ASM_Inst) instrucciones)" -ForegroundColor White
                Write-Host "  OBJ:    -$obj_red%  ($($sucio.OBJ_Size) -> $($r.OBJ_Size) bytes)" -ForegroundColor White
                Write-Host ""
            }
        }
    }
} else {
    Write-Host "No se pudieron procesar archivos" -ForegroundColor Red
}

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Archivos generados:" -ForegroundColor Cyan
foreach ($r in $resultados) {
    $obj_name = "test_array_CLANG_$($r.Nombre.ToLower()).obj"
    $obj_name = $obj_name -replace "sucio", "dirty" -replace "basico", "cleaned_basic" -replace "avanzado", "cleaned_advanced" -replace "extremo", "cleaned_extreme"
    
    if (Test-Path $obj_name) {
        Write-Host "  OBJ: $obj_name ($(Get-FileSize $obj_name) bytes)" -ForegroundColor Green
    }
    
    if ($r.EXE_Created) {
        $exe_name = "test_array_CLANG_$($r.Nombre.ToLower()).exe"
        $exe_name = $exe_name -replace "sucio", "dirty" -replace "basico", "cleaned_basic" -replace "avanzado", "cleaned_advanced" -replace "extremo", "cleaned_extreme"
        if (Test-Path $exe_name) {
            Write-Host "  EXE: $exe_name ($(Get-FileSize $exe_name) bytes)" -ForegroundColor Green
        }
    }
}

Write-Host ""
Write-Host "Completado" -ForegroundColor Green
Write-Host ""

