# Test simple de C++20
$gpp = "C:\msys64\mingw64\bin\g++.exe"

Write-Host "Probando C++20..." -ForegroundColor Cyan

# Test 1: C++20 basico
$test1 = Join-Path $env:TEMP "test1.cpp"
'#include <iostream>
#if __cplusplus >= 202002L
int main() { return 0; }
#else
#error "C++20 not supported"
#endif' | Out-File -FilePath $test1 -Encoding ASCII

$obj1 = Join-Path $env:TEMP "test1.o"
$result1 = & $gpp -std=c++20 -c $test1 -o $obj1 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "OK: C++20 basico funciona" -ForegroundColor Green
    $cpp20_ok = $true
} else {
    Write-Host "ERROR: C++20 basico no funciona" -ForegroundColor Red
    $result1 | ForEach-Object { Write-Host $_ -ForegroundColor Gray }
    $cpp20_ok = $false
}

# Test 2: std::format
$test2 = Join-Path $env:TEMP "test2.cpp"
'#include <format>
#include <iostream>
int main() { std::cout << std::format("Test: {}\n", 42); return 0; }' | Out-File -FilePath $test2 -Encoding ASCII

$obj2 = Join-Path $env:TEMP "test2.o"
$result2 = & $gpp -std=c++20 -c $test2 -o $obj2 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "OK: std::format funciona" -ForegroundColor Green
    $format_ok = $true
} else {
    Write-Host "ADVERTENCIA: std::format no funciona" -ForegroundColor Yellow
    $result2 | Select-Object -First 2 | ForEach-Object { Write-Host $_ -ForegroundColor Gray }
    $format_ok = $false
}

# Test 3: std::ranges
$test3 = Join-Path $env:TEMP "test3.cpp"
'#include <ranges>
#include <vector>
int main() { std::vector<int> v = {3,1,4}; std::ranges::sort(v); return 0; }' | Out-File -FilePath $test3 -Encoding ASCII

$obj3 = Join-Path $env:TEMP "test3.o"
$result3 = & $gpp -std=c++20 -c $test3 -o $obj3 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "OK: std::ranges funciona" -ForegroundColor Green
    $ranges_ok = $true
} else {
    Write-Host "ADVERTENCIA: std::ranges no funciona" -ForegroundColor Yellow
    $result3 | Select-Object -First 2 | ForEach-Object { Write-Host $_ -ForegroundColor Gray }
    $ranges_ok = $false
}

Write-Host ""
Write-Host "RESUMEN:" -ForegroundColor Cyan
Write-Host "  C++20 basico: $(if ($cpp20_ok) { 'OK' } else { 'ERROR' })"
Write-Host "  std::format:  $(if ($format_ok) { 'OK' } else { 'NO DISPONIBLE' })"
Write-Host "  std::ranges:  $(if ($ranges_ok) { 'OK' } else { 'NO DISPONIBLE' })"

if (-not $cpp20_ok) {
    Write-Host ""
    Write-Host "SOLUCION: Actualizar MSYS2" -ForegroundColor Yellow
    Write-Host "  En MSYS2 terminal ejecutar: pacman -Syu mingw-w64-x86_64-gcc" -ForegroundColor Gray
} elseif (-not $format_ok -or -not $ranges_ok) {
    Write-Host ""
    Write-Host "SOLUCION: Actualizar libstdc++" -ForegroundColor Yellow
    Write-Host "  En MSYS2 terminal ejecutar: pacman -Syu" -ForegroundColor Gray
    Write-Host "  Nota: ADead funcionara con C++17 como fallback" -ForegroundColor Gray
}

# Limpiar
Remove-Item $test1, $test2, $test3, $obj1, $obj2, $obj3 -ErrorAction SilentlyContinue

