# Wrapper script para adeadc
# Uso: .\adeadc.ps1 build test.ad --linker zig

$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
$exePath = Join-Path $scriptPath "CORE\rust\target\release\adeadc.exe"

if (-not (Test-Path $exePath)) {
    Write-Host "ERROR: adeadc.exe no encontrado en: $exePath" -ForegroundColor Red
    Write-Host "Por favor compila primero: cargo build --release --bin adeadc" -ForegroundColor Yellow
    exit 1
}

& $exePath $args

