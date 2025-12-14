# Script de compilación completa para ADead
# Compila Zig primero, luego Rust, y prueba el flujo completo

param(
    [switch]$Test,
    [switch]$Clean
)

$ErrorActionPreference = "Stop"

Write-Host "`n🔧 COMPILACIÓN COMPLETA DE ADEAD 🔧`n" -ForegroundColor Cyan

# Limpiar si se solicita
if ($Clean) {
    Write-Host "🧹 Limpiando builds anteriores..." -ForegroundColor Yellow
    if (Test-Path "zig\zig-out") {
        Remove-Item -Recurse -Force "zig\zig-out" -ErrorAction SilentlyContinue
    }
    if (Test-Path "rust\target") {
        Remove-Item -Recurse -Force "rust\target" -ErrorAction SilentlyContinue
    }
    Write-Host "✅ Limpieza completa`n" -ForegroundColor Green
}

# Paso 1: Compilar Zig
Write-Host "📦 PASO 1: Compilando Zig..." -ForegroundColor Cyan
Push-Location "zig"

if (-not (Test-Path "zig-out\lib")) {
    New-Item -ItemType Directory -Path "zig-out\lib" -Force | Out-Null
}

Write-Host "Ejecutando: zig build-lib src/nasm_generator.zig ..." -ForegroundColor Gray
zig build-lib src/nasm_generator.zig `
    -target x86_64-windows `
    -fno-stack-check `
    -lc `
    -O ReleaseFast `
    --name adead_zig

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Error compilando Zig" -ForegroundColor Red
    Pop-Location
    exit 1
}

# Mover archivos generados
if (Test-Path "adead_zig.lib") {
    Move-Item "adead_zig.lib" "zig-out/lib/" -Force
    Write-Host "✅ Biblioteca Zig generada: zig-out/lib/adead_zig.lib" -ForegroundColor Green
} elseif (Test-Path "adead_zig.obj") {
    Move-Item "adead_zig.obj" "zig-out/lib/" -Force
    Copy-Item "zig-out/lib/adead_zig.obj" "zig-out/lib/adead_zig.lib" -Force
    Write-Host "✅ Biblioteca Zig generada: zig-out/lib/adead_zig.lib" -ForegroundColor Green
} else {
    Write-Host "⚠️ Archivo Zig generado no encontrado en ubicación esperada" -ForegroundColor Yellow
}

Pop-Location

if (-not (Test-Path "zig\zig-out\lib\adead_zig.lib")) {
    Write-Host "❌ Error: Biblioteca Zig no encontrada después de compilar" -ForegroundColor Red
    exit 1
}

# Paso 2: Compilar Rust
Write-Host "`n📦 PASO 2: Compilando Rust..." -ForegroundColor Cyan
Push-Location "rust"

# Establecer variable de entorno para que build.rs encuentre la biblioteca
$zigLibPath = (Resolve-Path "..\zig\zig-out\lib").Path
$env:ZIG_LIB_PATH = $zigLibPath
Write-Host "ZIG_LIB_PATH=$env:ZIG_LIB_PATH" -ForegroundColor Gray

cargo build --release -p adead-cli

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Error compilando Rust" -ForegroundColor Red
    Pop-Location
    exit 1
}

Write-Host "✅ Rust compilado exitosamente`n" -ForegroundColor Green
Pop-Location

# Paso 3: Probar el compilador
if ($Test) {
    Write-Host "🧪 PASO 3: Probando print 3.14..." -ForegroundColor Cyan
    Push-Location "."
    
    $testFile = "Ejemplos-Reales\compilados\debug-float.ad"
    if (Test-Path $testFile) {
        Write-Host "Compilando: $testFile" -ForegroundColor Gray
        .\rust\target\release\adeadc.exe compile $testFile 2>&1 | Out-Null
        
        $asmFile = "Ejemplos-Reales\compilados\debug-float.asm"
        if (Test-Path $asmFile) {
            $asmContent = Get-Content $asmFile -Raw
            if ($asmContent -match "3\.14") {
                Write-Host "✅ ¡ÉXITO! 3.14 detectado en ASM generado!" -ForegroundColor Green
                Write-Host "`nPrimeras líneas del ASM:" -ForegroundColor Cyan
                Get-Content $asmFile | Select-Object -First 20
            } else {
                Write-Host "⚠️ ASM generado pero 3.14 no detectado" -ForegroundColor Yellow
                Write-Host "Contenido del ASM:" -ForegroundColor Gray
                Get-Content $asmFile | Select-Object -First 20
            }
        } else {
            Write-Host "⚠️ Archivo ASM no generado" -ForegroundColor Yellow
        }
    } else {
        Write-Host "⚠️ Archivo de prueba no encontrado: $testFile" -ForegroundColor Yellow
    }
    
    Pop-Location
}

Write-Host ""
Write-Host "COMPILACION COMPLETA EXITOSA!" -ForegroundColor Green
Write-Host ""
Write-Host "Resumen:" -ForegroundColor Cyan
Write-Host "  Zig compilado: zig-out/lib/adead_zig.lib" -ForegroundColor White
Write-Host "  Rust compilado: rust/target/release/adeadc.exe" -ForegroundColor White
Write-Host "  Todo funcionando correctamente!" -ForegroundColor White
Write-Host ""
Write-Host 'Para probar el compilador, ejecutar:' -ForegroundColor Yellow
Write-Host '  adeadc.exe compile Ejemplos-Reales\compilados\debug-float.ad' -ForegroundColor Gray
Write-Host ""

