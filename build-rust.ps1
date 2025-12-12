# Build Script Solo Rust - PowerShell (Windows)
# Nota: Rust automaticamente linkea Zig si esta compilado

Write-Host "Compilando Rust..." -ForegroundColor Yellow

if (Get-Command cargo -ErrorAction SilentlyContinue) {
    cargo build --release
    if ($LASTEXITCODE -eq 0) {
        Write-Host "OK: Rust compilado exitosamente" -ForegroundColor Green
        Write-Host "   Binario: target/release/adeadc.exe" -ForegroundColor Cyan
    } else {
        Write-Host "ERROR: Error compilando Rust" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "ERROR: Cargo (Rust) no esta instalado o no esta en PATH" -ForegroundColor Red
    Write-Host "   Instala Rust desde: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}
