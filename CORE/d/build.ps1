# Script de build para módulo D de ADead
# Compila el módulo D a biblioteca estática para enlazar con Rust

Write-Host "=== Compilando módulo D para ADead ===" -ForegroundColor Cyan

# Verificar si DMD está instalado
$dmdCmd = Get-Command dmd -ErrorAction SilentlyContinue
$ldcCmd = Get-Command ldc2 -ErrorAction SilentlyContinue

if (-not $dmdCmd -and -not $ldcCmd) {
    Write-Host "❌ Error: D Language no está instalado" -ForegroundColor Red
    Write-Host "`nPor favor instala DMD o LDC:" -ForegroundColor Yellow
    Write-Host "  - DMD: https://dlang.org/download.html" -ForegroundColor White
    Write-Host "  - LDC: https://github.com/ldc-developers/ldc/releases" -ForegroundColor White
    exit 1
}

$compiler = if ($dmdCmd) { "dmd" } else { "ldc2" }
Write-Host "✅ Usando compilador: $compiler" -ForegroundColor Green

# Crear directorio de salida
New-Item -ItemType Directory -Force -Path "build" | Out-Null

# Compilar con DMD
if ($compiler -eq "dmd") {
    Write-Host "`nCompilando módulos D con DMD..." -ForegroundColor Yellow
    Write-Host "  - adead_metaprog.d" -ForegroundColor Gray
    Write-Host "  - adead_ctfe.d" -ForegroundColor Gray
    
    & dmd -c `
        -of"build/adead_d.obj" `
        -release `
        -O `
        -H `
        -Hd"build" `
        src/adead_metaprog.d src/adead_ctfe.d
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Compilación exitosa" -ForegroundColor Green
        Write-Host "   Objeto: build/adead_d.obj" -ForegroundColor White
        Write-Host "   Headers: build/" -ForegroundColor White
    } else {
        Write-Host "❌ Error en la compilación" -ForegroundColor Red
        exit 1
    }
} else {
    # Compilar con LDC
    Write-Host "`nCompilando módulos D con LDC..." -ForegroundColor Yellow
    Write-Host "  - adead_metaprog.d" -ForegroundColor Gray
    Write-Host "  - adead_ctfe.d" -ForegroundColor Gray
    
    & ldc2 -c `
        -of="build/adead_d.obj" `
        -release `
        -O3 `
        -H `
        -Hd="build" `
        src/adead_metaprog.d src/adead_ctfe.d
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Compilación exitosa" -ForegroundColor Green
        Write-Host "   Objeto: build/adead_d.obj" -ForegroundColor White
    } else {
        Write-Host "❌ Error en la compilación" -ForegroundColor Red
        exit 1
    }
}

Write-Host "`n=== Build completado ===" -ForegroundColor Green
Write-Host "El objeto está en: d/build/adead_d.obj" -ForegroundColor White
Write-Host "Puedes enlazarlo con Rust usando el objeto .obj" -ForegroundColor White

