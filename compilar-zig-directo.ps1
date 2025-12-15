# Script para compilar usando Zig directamente
# Flujo: ADead → Zig → NASM → .exe

param(
    [Parameter(Mandatory=$true)]
    [string]$InputFile
)

$ErrorActionPreference = "Stop"

Write-Host "=== Compilación con Zig Directo ===" -ForegroundColor Cyan

# Verificar archivo de entrada
if (-not (Test-Path $InputFile)) {
    Write-Host "❌ Archivo no encontrado: $InputFile" -ForegroundColor Red
    exit 1
}

# Leer código fuente
$source = Get-Content $InputFile -Raw
$sourceName = [System.IO.Path]::GetFileNameWithoutExtension($InputFile)
$sourceDir = [System.IO.Path]::GetDirectoryName($InputFile)

# Crear archivo temporal para Zig
$tempFile = "$env:TEMP\adead_temp_$(Get-Random).ad"
$source | Out-File -FilePath $tempFile -Encoding utf8 -NoNewline

Write-Host "`n📄 Código fuente:" -ForegroundColor Yellow
Write-Host ($source.Substring(0, [Math]::Min(200, $source.Length))) -ForegroundColor Gray

# Verificar que Zig está compilado
if (-not (Test-Path "zig\adead_zig.lib")) {
    Write-Host "`n⚠️ Zig library no encontrada, compilando..." -ForegroundColor Yellow
    Push-Location zig
    & ".\build-zig.ps1"
    Pop-Location
    
    if (-not (Test-Path "zig\adead_zig.lib")) {
        Write-Host "❌ No se pudo compilar Zig library" -ForegroundColor Red
        exit 1
    }
}

# Usar el compilador Rust que llama a Zig
if (-not (Test-Path "target\release\adeadc.exe")) {
    Write-Host "`n⚠️ Compilador no encontrado, compilando..." -ForegroundColor Yellow
    cargo build --release
}

Write-Host "`n⚡ Compilando con Zig directo..." -ForegroundColor Cyan
$outputFile = Join-Path $sourceDir "$sourceName.asm"

# El compilador ahora usa Zig primero para while loops
& ".\target\release\adeadc.exe" compile $InputFile -o $outputFile

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n✅ Compilación exitosa: $outputFile" -ForegroundColor Green
    
    # Ensamblar y enlazar
    Write-Host "`n🔨 Ensamblando..." -ForegroundColor Cyan
    $objFile = Join-Path $sourceDir "$sourceName.obj"
    & nasm -f win64 $outputFile -o $objFile
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Ensamblado: $objFile" -ForegroundColor Green
        
        Write-Host "`n🔗 Enlazando..." -ForegroundColor Cyan
        $exeFile = Join-Path $sourceDir "$sourceName.exe"
        & gcc $objFile -o $exeFile -nostdlib -lkernel32 -Wl,--entry=main -Wl,--subsystem=console
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ Ejecutable creado: $exeFile" -ForegroundColor Green
            Write-Host "`n🚀 Ejecutando..." -ForegroundColor Cyan
            Write-Host ("─" * 60) -ForegroundColor Gray
            & $exeFile
            Write-Host ("─" * 60) -ForegroundColor Gray
        }
    }
}

# Limpiar
Remove-Item $tempFile -ErrorAction SilentlyContinue

