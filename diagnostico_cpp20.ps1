# Script de diagnóstico para C++20
Write-Host "=== Diagnostico C++20 para ADead ===" -ForegroundColor Cyan
Write-Host ""

# Inicializar variables
$cpp20_works = $false
$format_works = $false
$ranges_works = $false

# 1. Verificar versión de g++
Write-Host "[1/5] Verificando versión de g++..." -ForegroundColor Yellow
$gpp_path = "C:\msys64\mingw64\bin\g++.exe"
if (Test-Path $gpp_path) {
    $version = & $gpp_path --version 2>&1 | Select-Object -First 1
    Write-Host "  ✓ $version" -ForegroundColor Green
} else {
    Write-Host "  ✗ g++ no encontrado" -ForegroundColor Red
    exit 1
}

# 2. Crear archivo de prueba C++20 simple
Write-Host ""
Write-Host "[2/5] Creando archivo de prueba C++20..." -ForegroundColor Yellow
$test_file = Join-Path $env:TEMP "cpp20_test.cpp"
$test_code = '#include <iostream>
#if __cplusplus >= 202002L
int main() { return 0; }
#else
#error "C++20 no soportado"
#endif
'
Set-Content -Path $test_file -Value $test_code -Encoding UTF8
Write-Host "  ✓ Archivo creado: $test_file" -ForegroundColor Green

# 3. Intentar compilar con C++20
Write-Host ""
Write-Host "[3/5] Compilando con -std=c++20..." -ForegroundColor Yellow
$obj_file = Join-Path $env:TEMP "cpp20_test.o"
$compile_output = & $gpp_path -std=c++20 -c $test_file -o $obj_file 2>&1
$exit_code = $LASTEXITCODE

if ($exit_code -eq 0) {
    Write-Host "  ✓ Compilación exitosa con C++20" -ForegroundColor Green
    $cpp20_works = $true
} else {
    Write-Host "  ✗ Error al compilar con C++20" -ForegroundColor Red
    Write-Host "  Salida del compilador:" -ForegroundColor Yellow
    $compile_output | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
    $cpp20_works = $false
}

# 4. Verificar si std::format está disponible (requiere libstdc++ actualizado)
Write-Host ""
Write-Host "[4/5] Verificando std::format (requiere libstdc++ actualizado)..." -ForegroundColor Yellow
$format_test_file = Join-Path $env:TEMP "format_test.cpp"
$format_test_code = '#include <format>
#include <iostream>
int main() {
    std::cout << std::format("Test: {}\n", 42);
    return 0;
}
'
Set-Content -Path $format_test_file -Value $format_test_code -Encoding UTF8

$format_obj = Join-Path $env:TEMP "format_test.o"
$format_output = & $gpp_path -std=c++20 -c $format_test_file -o $format_obj 2>&1
$format_exit_code = $LASTEXITCODE

if ($format_exit_code -eq 0) {
    Write-Host "  ✓ std::format disponible" -ForegroundColor Green
    $format_works = $true
} else {
    Write-Host "  ⚠ std::format no disponible (puede requerir libstdc++ más reciente)" -ForegroundColor Yellow
    Write-Host "  Detalles:" -ForegroundColor Gray
    $format_output | Select-Object -First 3 | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
    $format_works = $false
}

# 5. Verificar std::ranges
Write-Host ""
Write-Host "[5/5] Verificando std::ranges..." -ForegroundColor Yellow
$ranges_test_file = Join-Path $env:TEMP "ranges_test.cpp"
$ranges_test_code = '#include <ranges>
#include <vector>
#include <algorithm>
int main() {
    std::vector<int> v = {3, 1, 4, 1, 5};
    std::ranges::sort(v);
    return 0;
}
'
Set-Content -Path $ranges_test_file -Value $ranges_test_code -Encoding UTF8

$ranges_obj = Join-Path $env:TEMP "ranges_test.o"
$ranges_output = & $gpp_path -std=c++20 -c $ranges_test_file -o $ranges_obj 2>&1
$ranges_exit_code = $LASTEXITCODE

if ($ranges_exit_code -eq 0) {
    Write-Host "  ✓ std::ranges disponible" -ForegroundColor Green
    $ranges_works = $true
} else {
    Write-Host "  ⚠ std::ranges no disponible" -ForegroundColor Yellow
    Write-Host "  Detalles:" -ForegroundColor Gray
    $ranges_output | Select-Object -First 3 | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
    $ranges_works = $false
}

# Resumen
Write-Host ""
Write-Host "=== RESUMEN ===" -ForegroundColor Cyan
Write-Host "  C++20 básico: $(if ($cpp20_works) { '✓ Soportado' } else { '✗ No soportado' })" -ForegroundColor $(if ($cpp20_works) { 'Green' } else { 'Red' })
Write-Host "  std::format:  $(if ($format_works) { '✓ Disponible' } else { '⚠ No disponible' })" -ForegroundColor $(if ($format_works) { 'Green' } else { 'Yellow' })
Write-Host "  std::ranges:  $(if ($ranges_works) { '✓ Disponible' } else { '⚠ No disponible' })" -ForegroundColor $(if ($ranges_works) { 'Green' } else { 'Yellow' })

Write-Host ""
if (-not $cpp20_works) {
    Write-Host "⚠️  PROBLEMA: C++20 básico no funciona" -ForegroundColor Red
    Write-Host "   Solución: Actualizar MSYS2 y g++" -ForegroundColor Yellow
    Write-Host "   Comando: pacman -Syu mingw-w64-x86_64-gcc" -ForegroundColor Gray
} elseif (-not $format_works -or -not $ranges_works) {
    Write-Host "⚠️  PROBLEMA: Algunas features de C++20 no están disponibles" -ForegroundColor Yellow
    Write-Host "   Esto puede ser porque:" -ForegroundColor Yellow
    Write-Host "   1. libstdc++ necesita actualización" -ForegroundColor Gray
    Write-Host "   2. MSYS2 necesita actualización completa" -ForegroundColor Gray
    Write-Host "   Solución: Actualizar MSYS2 completamente" -ForegroundColor Yellow
    Write-Host "   Comando: pacman -Syu" -ForegroundColor Gray
} else {
    Write-Host "OK: Todo funciona correctamente!" -ForegroundColor Green
    Write-Host "   ADead deberia usar C++20 automaticamente" -ForegroundColor Green
}

# Limpiar archivos temporales
Remove-Item $test_file -ErrorAction SilentlyContinue
Remove-Item $obj_file -ErrorAction SilentlyContinue
Remove-Item $format_test_file -ErrorAction SilentlyContinue
Remove-Item $format_obj -ErrorAction SilentlyContinue
Remove-Item $ranges_test_file -ErrorAction SilentlyContinue
Remove-Item $ranges_obj -ErrorAction SilentlyContinue

