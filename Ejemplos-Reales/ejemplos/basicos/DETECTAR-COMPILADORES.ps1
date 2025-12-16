# Script PowerShell para detectar todos los compiladores C/C++ instalados
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Deteccion de Compiladores C/C++" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Función para verificar comando
function Test-Compiler {
    param([string]$Name, [string]$VersionFlag = "--version")
    
    $cmd = Get-Command $Name -ErrorAction SilentlyContinue
    if ($cmd) {
        Write-Host "[OK] $Name encontrado" -ForegroundColor Green
        Write-Host "     Ubicacion: $($cmd.Source)" -ForegroundColor Gray
        try {
            $version = & $Name $VersionFlag 2>&1 | Select-Object -First 3
            Write-Host "     Version:" -ForegroundColor Gray
            $version | ForEach-Object { Write-Host "       $_" -ForegroundColor DarkGray }
        } catch {
            Write-Host "     (No se pudo obtener version)" -ForegroundColor Yellow
        }
        return $true
    } else {
        Write-Host "[NO] $Name no encontrado" -ForegroundColor Red
        return $false
    }
}

# Verificar compiladores comunes
Write-Host "=== Compiladores C/C++ ===" -ForegroundColor Yellow
$found = @()

if (Test-Compiler "gcc") { $found += "gcc" }
if (Test-Compiler "g++") { $found += "g++" }
if (Test-Compiler "clang") { $found += "clang" }
if (Test-Compiler "clang++") { $found += "clang++" }
if (Test-Compiler "cl" "-?") { $found += "cl (MSVC)" }
Write-Host ""

# Verificar linkers
Write-Host "=== Linkers ===" -ForegroundColor Yellow
if (Test-Compiler "ld") { $found += "ld" }
if (Test-Compiler "link") { $found += "link (MSVC)" }
Write-Host ""

# Verificar ensambladores
Write-Host "=== Ensambladores ===" -ForegroundColor Yellow
if (Test-Compiler "nasm" "-v") { $found += "nasm" }
if (Test-Compiler "yasm" "-v") { $found += "yasm" }
Write-Host ""

# Buscar en ubicaciones comunes
Write-Host "=== Buscando en ubicaciones comunes ===" -ForegroundColor Yellow

$searchPaths = @(
    "C:\Program Files\LLVM",
    "C:\Program Files (x86)\LLVM",
    "C:\Program Files\mingw-w64",
    "C:\Program Files (x86)\mingw-w64",
    "C:\MinGW",
    "C:\TDM-GCC-64",
    "C:\msys64\mingw64",
    "$env:LOCALAPPDATA\Programs\mingw",
    "$env:USERPROFILE\mingw",
    "$env:USERPROFILE\AppData\Local\Programs\mingw"
)

foreach ($path in $searchPaths) {
    if (Test-Path $path) {
        Write-Host "[OK] Directorio encontrado: $path" -ForegroundColor Green
        
        # Buscar gcc.exe
        $gcc = Get-ChildItem -Path $path -Filter "gcc.exe" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($gcc) {
            Write-Host "     gcc.exe: $($gcc.FullName)" -ForegroundColor Gray
        }
        
        # Buscar clang.exe
        $clang = Get-ChildItem -Path $path -Filter "clang.exe" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($clang) {
            Write-Host "     clang.exe: $($clang.FullName)" -ForegroundColor Gray
        }
    }
}

Write-Host ""

# Buscar Visual Studio
Write-Host "=== Visual Studio ===" -ForegroundColor Yellow
$vsPaths = @(
    "C:\Program Files\Microsoft Visual Studio",
    "C:\Program Files (x86)\Microsoft Visual Studio"
)

foreach ($vsPath in $vsPaths) {
    if (Test-Path $vsPath) {
        $vsVersions = Get-ChildItem -Path $vsPath -Directory -ErrorAction SilentlyContinue
        foreach ($vsVersion in $vsVersions) {
            Write-Host "[OK] Visual Studio encontrado: $($vsVersion.FullName)" -ForegroundColor Green
            
            # Buscar MSVC compiler
            $clPath = Get-ChildItem -Path $vsVersion.FullName -Filter "cl.exe" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
            if ($clPath) {
                Write-Host "     cl.exe: $($clPath.FullName)" -ForegroundColor Gray
            }
        }
    }
}

Write-Host ""

# Resumen
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Resumen" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
if ($found.Count -gt 0) {
    Write-Host "Compiladores encontrados:" -ForegroundColor Green
    $found | ForEach-Object { Write-Host "  - $_" -ForegroundColor White }
} else {
    Write-Host "No se encontraron compiladores en PATH" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Recomendaciones:" -ForegroundColor Yellow
    Write-Host "  1. Instalar MinGW-w64: https://www.mingw-w64.org/" -ForegroundColor Gray
    Write-Host "  2. Instalar LLVM/Clang: https://llvm.org/builds/" -ForegroundColor Gray
    Write-Host "  3. Instalar Visual Studio Build Tools" -ForegroundColor Gray
}

