# Build Script Solo Zig - PowerShell (Windows)

Write-Host "Compilando Zig..." -ForegroundColor Yellow

if (Get-Command zig -ErrorAction SilentlyContinue) {
    Set-Location zig
    zig build
    if ($LASTEXITCODE -eq 0) {
        Write-Host "OK: Zig compilado exitosamente" -ForegroundColor Green
        Write-Host "   Biblioteca: zig/zig-out/lib/libadead_zig.a" -ForegroundColor Cyan
    } else {
        Write-Host "ERROR: Error compilando Zig" -ForegroundColor Red
        Set-Location ..
        exit 1
    }
    Set-Location ..
} else {
    Write-Host "ERROR: Zig no esta instalado o no esta en PATH" -ForegroundColor Red
    Write-Host "   Instala Zig desde: https://ziglang.org/download/" -ForegroundColor Yellow
    exit 1
}
