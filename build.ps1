# Build script for ADead (PowerShell)

Write-Host "Building ADead..." -ForegroundColor Green

# Build in release mode
cargo build --release -p adead-cli

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "Build successful!" -ForegroundColor Green
Write-Host "Binary location: target/release/adeadc.exe" -ForegroundColor Cyan

# Optionally compile an example
if ($args.Count -gt 0 -and $args[0] -eq "--test") {
    Write-Host "`nCompiling example..." -ForegroundColor Yellow
    $example = if ($args.Count -gt 1) { $args[1] } else { "examples/hello.ad" }
    .\target\release\adeadc.exe compile $example -o test.asm
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Example compiled to test.asm" -ForegroundColor Green
    }
}

