# Build Script para ADead - PowerShell (Windows)
# Compila Zig y luego Rust que linkea Zig

Write-Host "Compilando ADead (Rust + Zig)..." -ForegroundColor Cyan
Write-Host ""

# Paso 1: Compilar Zig (opcional por ahora)
Write-Host "Paso 1: Compilando Zig (opcional)..." -ForegroundColor Yellow
Set-Location zig
if (Get-Command zig -ErrorAction SilentlyContinue) {
    zig build
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ADVERTENCIA: Zig fallo al compilar, pero continuamos..." -ForegroundColor Yellow
        Write-Host "   Nota: Rust puede compilar sin Zig por ahora" -ForegroundColor Yellow
        Write-Host "   Ver: docs/ZIG-VERSION-REQUIREMENTS.md" -ForegroundColor Yellow
    } else {
        Write-Host "OK: Zig compilado exitosamente" -ForegroundColor Green
    }
} else {
    Write-Host "ADVERTENCIA: Zig no esta instalado o no esta en PATH" -ForegroundColor Yellow
    Write-Host "   Instala Zig desde: https://ziglang.org/download/" -ForegroundColor Yellow
    Write-Host "   Rust puede compilar sin Zig por ahora" -ForegroundColor Yellow
}
Set-Location ..

Write-Host ""

# Paso 2: Compilar Rust (linkea Zig automaticamente)
Write-Host "Paso 2: Compilando Rust..." -ForegroundColor Yellow
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Error compilando Rust" -ForegroundColor Red
        exit 1
    }
    Write-Host "OK: Rust compilado exitosamente" -ForegroundColor Green
} else {
    Write-Host "ERROR: Cargo (Rust) no esta instalado o no esta en PATH" -ForegroundColor Red
    Write-Host "   Instala Rust desde: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "Compilacion completa!" -ForegroundColor Green
Write-Host "   Binario: target/release/adeadc.exe" -ForegroundColor Cyan
