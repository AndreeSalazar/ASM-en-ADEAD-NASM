# Script de Verificación de C++20 para ADead
# Verifica si tienes C++20 disponible y configurado correctamente

Write-Host "🔍 Verificando instalación de C++20 para ADead..." -ForegroundColor Cyan
Write-Host ""

$found = $false
$cpp20_available = $false
$compiler_path = ""

# Lista de compiladores a verificar
$compilers = @(
    "g++",
    "clang++",
    "C:\msys64\mingw64\bin\g++.exe",
    "C:\msys64\clang64\bin\clang++.exe",
    "C:\Program Files\LLVM\bin\clang++.exe"
)

Write-Host "Buscando compiladores C++..." -ForegroundColor Yellow

foreach ($compiler in $compilers) {
    if (Test-Path $compiler -ErrorAction SilentlyContinue) {
        Write-Host "  ✅ Encontrado: $compiler" -ForegroundColor Green
        
        # Verificar versión
        try {
            $version_output = & $compiler --version 2>&1 | Out-String
            Write-Host "     Versión: $($version_output.Split("`n")[0])" -ForegroundColor Gray
            
            # Verificar soporte C++20
            $test_file = [System.IO.Path]::GetTempFileName() + ".cpp"
            $test_code = @"
#include <version>
#if __cplusplus >= 202002L
// C++20 disponible
#endif
"@
            Set-Content -Path $test_file -Value $test_code
            
            $obj_file = $test_file -replace '\.cpp$', '.o'
            
            $compile_result = & $compiler -std=c++20 -x c++ -c $test_file -o $obj_file 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                Write-Host "     ✅ C++20 soportado!" -ForegroundColor Green
                $cpp20_available = $true
                $compiler_path = $compiler
                $found = $true
                Remove-Item $test_file -ErrorAction SilentlyContinue
                Remove-Item $obj_file -ErrorAction SilentlyContinue
                break
            } else {
                Write-Host "     ⚠️  C++20 no disponible, pero C++17 sí" -ForegroundColor Yellow
                $compiler_path = $compiler
                $found = $true
                Remove-Item $test_file -ErrorAction SilentlyContinue
                Remove-Item $obj_file -ErrorAction SilentlyContinue
            }
        } catch {
            Write-Host "     ❌ Error al verificar: $_" -ForegroundColor Red
        }
    } else {
        Write-Host "  ❌ No encontrado: $compiler" -ForegroundColor DarkGray
    }
}

Write-Host ""
Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Cyan

if ($found) {
    if ($cpp20_available) {
        Write-Host "✅ ¡C++20 está disponible y funcionando!" -ForegroundColor Green
        Write-Host ""
        Write-Host "Compilador: $compiler_path" -ForegroundColor White
        Write-Host "El sistema ADead usará C++20 automáticamente." -ForegroundColor White
    } else {
        Write-Host "⚠️  C++17 disponible, pero C++20 no está soportado" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "Compilador: $compiler_path" -ForegroundColor White
        Write-Host "El sistema ADead usará C++17 automáticamente." -ForegroundColor White
        Write-Host ""
        Write-Host "Para obtener C++20:" -ForegroundColor Yellow
        Write-Host "  Windows: Actualiza MSYS2 y g++ a versión 10+" -ForegroundColor Gray
        Write-Host "  Linux: Instala g++-12 o más reciente" -ForegroundColor Gray
    }
} else {
    Write-Host "❌ No se encontró ningún compilador C++" -ForegroundColor Red
    Write-Host ""
    Write-Host "Para instalar:" -ForegroundColor Yellow
    Write-Host "  Windows: Instala MSYS2 desde https://www.msys2.org/" -ForegroundColor Gray
    Write-Host "  Linux: sudo apt-get install g++-12" -ForegroundColor Gray
}

Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Cyan


