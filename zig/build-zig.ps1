# Script de compilación directa para Zig
# Alternativa a build.zig cuando hay problemas con la API

Write-Host "`n🔧 Compilando Zig directamente..." -ForegroundColor Cyan

# Limpiar compilaciones anteriores
if (Test-Path "zig-out\lib") {
    Remove-Item -Recurse -Force "zig-out\lib" -ErrorAction SilentlyContinue
}

# Crear directorio de salida
New-Item -ItemType Directory -Path "zig-out\lib" -Force | Out-Null

Write-Host "Compilando src/main.zig como biblioteca..." -ForegroundColor Yellow

# Compilar main.zig que re-exporta expr_parser
zig build-lib src/main.zig -target x86_64-windows -fno-stack-check -lc -O ReleaseFast --name adead_zig

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Error compilando Zig" -ForegroundColor Red
    exit 1
}

# Mover archivos generados
if (Test-Path "adead_zig.lib") {
    Move-Item "adead_zig.lib" "zig-out/lib/" -Force
    Write-Host "✅ Zig compilado exitosamente!" -ForegroundColor Green
    Write-Host "📁 Biblioteca: zig-out/lib/adead_zig.lib" -ForegroundColor Cyan
    exit 0
}

if (Test-Path "adead_zig.obj") {
    Move-Item "adead_zig.obj" "zig-out/lib/" -Force
    Copy-Item "zig-out/lib/adead_zig.obj" "zig-out/lib/adead_zig.lib" -Force
    Write-Host "✅ Zig compilado exitosamente!" -ForegroundColor Green
    Write-Host "📁 Biblioteca: zig-out/lib/adead_zig.lib" -ForegroundColor Cyan
    exit 0
}

Write-Host "⚠️ Archivo generado no encontrado" -ForegroundColor Yellow
exit 1
