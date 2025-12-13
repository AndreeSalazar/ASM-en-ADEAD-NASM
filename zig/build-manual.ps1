# Script para compilar biblioteca Zig manualmente (solución temporal para Zig 0.16.0)
# Uso: .\build-manual.ps1

Write-Host "🔨 Compilando biblioteca Zig manualmente..." -ForegroundColor Yellow

# Crear directorio de salida
New-Item -ItemType Directory -Force -Path "zig-out/lib" | Out-Null

# Compilar biblioteca estática directamente desde expr_parser.zig
# (main.zig tiene referencias a módulos que no existen actualmente)
zig build-lib src/expr_parser.zig --name adead_zig --library c -fno-strip -O Debug

# Verificar si se compiló
if (Test-Path "libadead_zig.a") {
    # Linux: renombrar a .lib para compatibilidad
    Copy-Item "libadead_zig.a" -Destination "zig-out/lib/adead_zig.lib" -Force
    Write-Host "✅ Biblioteca compilada: zig-out/lib/adead_zig.lib" -ForegroundColor Green
} elseif (Test-Path "adead_zig.lib") {
    # Windows: copiar directamente
    Copy-Item "adead_zig.lib" -Destination "zig-out/lib/adead_zig.lib" -Force
    Write-Host "✅ Biblioteca compilada: zig-out/lib/adead_zig.lib" -ForegroundColor Green
} else {
    Write-Host "❌ Error: No se encontró biblioteca compilada" -ForegroundColor Red
    exit 1
}

# Limpiar archivos temporales
Remove-Item -ErrorAction SilentlyContinue "libadead_zig.a", "adead_zig.lib"

Write-Host "`n✨ Listo para compilar Rust con: cargo build" -ForegroundColor Cyan

