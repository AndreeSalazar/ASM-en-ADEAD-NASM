# 🔍 Investigación Completa del Stack: GCC/Clang, Zig Linker y NASM

**Fecha:** Diciembre 2025  
**Autor:** Eddi Andreé Salazar Matos

## 🎯 Objetivo

Investigar y documentar completamente el stack de herramientas:
1. **GCC/Clang** - Para compilación C++ → ASM (REQUERIDO)
2. **Zig** - SOLO para linking (OPCIONAL pero recomendado)
3. **NASM** - Para ensamblar ASM → .obj (REQUERIDO)

---

## 📊 Estado Actual del Sistema

### ✅ Herramientas Instaladas (Confirmadas)

Según la imagen del PATH y documentación:

1. **NASM** ✅
   - Ubicación: `C:\Users\andre\AppData\Local\bin\NASM`
   - En PATH: ✅ Sí
   - Versión: 3.01 (según RESUMEN-ENTORNO.md)
   - Estado: Funcionando correctamente

2. **Zig** ✅
   - Ubicación 1: `C:\zig-x86_64-windows-0.16.0-dev.1484+d0ba6642b`
   - Ubicación 2: `C:\Users\andre\AppData\Local\Microsoft\WinGet\Packages\zig.zig_Microsoft.Winget.Source_8wekyb3d8bbwe\zig-x86_64-windows-0.14.1`
   - En PATH: ✅ Sí (múltiples versiones)
   - Estado: Funcionando correctamente
   - **Uso:** SOLO para linking, NO para compilación

3. **GCC/Clang** ⚠️
   - Estado según RESUMEN-ENTORNO.md: ❌ No disponible
   - Estado según usuario: ✅ Disponible en su PC
   - **Necesita verificación**

---

## 🔍 Investigación: GCC/Clang

### Ubicaciones Comunes donde ADead Busca GCC/Clang

Según el código en `CORE/rust/crates/adead-parser/src/pipeline_selector.rs`:

#### Compiladores C++ Buscados (en orden de preferencia):

1. **En PATH:**
   - `clang++`
   - `g++`
   - `clang`
   - `gcc`

2. **Ubicaciones comunes de Windows:**
   - `C:\msys64\mingw64\bin\g++.exe`
   - `C:\msys64\clang64\bin\clang++.exe`
   - `C:\Program Files\LLVM\bin\clang++.exe`
   - `C:\msys64\mingw64\bin\gcc.exe`
   - `C:\msys64\clang64\bin\clang.exe`

3. **Ubicaciones adicionales (según c_compiler.rs):**
   - `C:\msys64\usr\bin\gcc.exe`
   - `C:\mingw64\bin\gcc.exe`
   - `C:\mingw\bin\gcc.exe`
   - `C:\Program Files\mingw-w64\x86_64-8.1.0-posix-seh-rt_v6-rev0\mingw64\bin\gcc.exe`
   - `C:\Program Files\LLVM\bin\clang.exe`
   - `C:\Program Files (x86)\LLVM\bin\clang.exe`

### Cómo ADead Detecta GCC/Clang

**Función:** `find_cpp_compiler_for_pipeline()`

**Proceso:**
1. Busca compiladores en PATH primero
2. Verifica que respondan a `--version`
3. Prueba rutas absolutas comunes
4. Verifica soporte C++20 si está disponible
5. Usa C++17 como fallback si C++20 no está disponible

**Código relevante:**
```rust
// Buscar compilador que funcione (preferir C++20)
let mut cpp20_compiler: Option<String> = None;
let mut cpp17_compiler: Option<String> = None;

for compiler in compilers_to_try {
    // Verificar si existe
    let compiler_exists = if Path::new(&compiler).exists() {
        true
    } else if compiler.contains("++") || compiler.contains("clang") || compiler.contains("gcc") {
        // Verificar que respondan a --version
        Command::new(&compiler).arg("--version").output().is_ok()
    } else {
        false
    };
    
    if compiler_exists {
        // Verificar soporte C++20 primero (preferido)
        if check_cpp20_support(&compiler) {
            cpp20_compiler = Some(compiler.clone());
        } else if cpp17_compiler.is_none() {
            cpp17_compiler = Some(compiler);
        }
    }
}

// Retornar C++20 si está disponible (preferido), sino C++17
cpp20_compiler.or(cpp17_compiler)
```

### Verificación de C++20

**Función:** `check_cpp20_support(compiler: &str)`

**Proceso:**
1. Crea archivo temporal de prueba C++20
2. Intenta compilar con `-std=c++20`
3. Retorna `true` si compila exitosamente

**Código de prueba:**
```cpp
#include <version>
#if __cplusplus >= 202002L
int main() { return 0; }
#else
#error "C++20 not supported"
#endif
```

### Script de Verificación para GCC/Clang

**Crear script:** `VERIFICAR-GCC-CLANG.ps1`

```powershell
# Script para verificar GCC/Clang en el sistema
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Verificación de GCC/Clang" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Lista de compiladores a buscar
$compilers = @(
    @{Name="clang++"; Path="clang++"},
    @{Name="g++"; Path="g++"},
    @{Name="clang"; Path="clang"},
    @{Name="gcc"; Path="gcc"},
    @{Name="g++.exe (MSYS2)"; Path="C:\msys64\mingw64\bin\g++.exe"},
    @{Name="clang++.exe (MSYS2)"; Path="C:\msys64\clang64\bin\clang++.exe"},
    @{Name="clang++.exe (LLVM)"; Path="C:\Program Files\LLVM\bin\clang++.exe"},
    @{Name="gcc.exe (MSYS2)"; Path="C:\msys64\mingw64\bin\gcc.exe"},
    @{Name="clang.exe (MSYS2)"; Path="C:\msys64\clang64\bin\clang.exe"},
    @{Name="clang.exe (LLVM)"; Path="C:\Program Files\LLVM\bin\clang.exe"}
)

$found_compilers = @()

foreach ($compiler in $compilers) {
    Write-Host "Buscando: $($compiler.Name)..." -ForegroundColor Yellow
    
    if (Test-Path $compiler.Path) {
        Write-Host "  ✓ Encontrado: $($compiler.Path)" -ForegroundColor Green
        
        # Verificar versión
        try {
            $version = & $compiler.Path --version 2>&1 | Select-Object -First 1
            Write-Host "  Versión: $version" -ForegroundColor Gray
            
            # Verificar si es C++ o C
            $is_cpp = $compiler.Path -match "\+\+|clang\+\+|g\+\+"
            
            # Verificar soporte C++20 (solo para compiladores C++)
            if ($is_cpp) {
                Write-Host "  Verificando soporte C++20..." -ForegroundColor Yellow
                
                $test_cpp20 = @"
#include <version>
#if __cplusplus >= 202002L
int main() { return 0; }
#else
#error "C++20 not supported"
#endif
"@
                
                $test_file = Join-Path $env:TEMP "adead_cpp20_test.cpp"
                $test_obj = Join-Path $env:TEMP "adead_cpp20_test.o"
                
                Set-Content -Path $test_file -Value $test_cpp20
                
                try {
                    & $compiler.Path -std=c++20 -c $test_file -o $test_obj 2>&1 | Out-Null
                    if ($LASTEXITCODE -eq 0) {
                        Write-Host "  ✓ C++20 soportado" -ForegroundColor Green
                        $found_compilers += @{
                            Name = $compiler.Name
                            Path = $compiler.Path
                            Cpp20 = $true
                        }
                    } else {
                        Write-Host "  ⚠ Solo C++17 soportado" -ForegroundColor Yellow
                        $found_compilers += @{
                            Name = $compiler.Name
                            Path = $compiler.Path
                            Cpp20 = $false
                        }
                    }
                } catch {
                    Write-Host "  ⚠ Error al verificar C++20: $_" -ForegroundColor Yellow
                } finally {
                    Remove-Item $test_file -ErrorAction SilentlyContinue
                    Remove-Item $test_obj -ErrorAction SilentlyContinue
                }
            } else {
                # Compilador C, no C++
                $found_compilers += @{
                    Name = $compiler.Name
                    Path = $compiler.Path
                    Cpp20 = $null
                }
            }
        } catch {
            Write-Host "  ⚠ Error al obtener versión: $_" -ForegroundColor Yellow
        }
    } else {
        Write-Host "  ✗ No encontrado" -ForegroundColor Red
    }
    Write-Host ""
}

# Resumen
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Resumen" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

if ($found_compilers.Count -eq 0) {
    Write-Host "❌ No se encontraron compiladores GCC/Clang" -ForegroundColor Red
    Write-Host ""
    Write-Host "Opciones:" -ForegroundColor Yellow
    Write-Host "  1. Instalar MSYS2: https://www.msys2.org/" -ForegroundColor Gray
    Write-Host "  2. Instalar LLVM/Clang: https://llvm.org/builds/" -ForegroundColor Gray
    Write-Host "  3. Instalar MinGW-w64: https://www.mingw-w64.org/" -ForegroundColor Gray
} else {
    Write-Host "✅ Compiladores encontrados:" -ForegroundColor Green
    foreach ($compiler in $found_compilers) {
        $cpp20_status = if ($compiler.Cpp20 -eq $true) {
            "C++20 ✅"
        } elseif ($compiler.Cpp20 -eq $false) {
            "C++17 ⚠️"
        } else {
            "C (no C++)"
        }
        Write-Host "  • $($compiler.Name): $($compiler.Path) [$cpp20_status]" -ForegroundColor Gray
    }
    
    # Recomendar mejor compilador
    $best = $found_compilers | Where-Object { $_.Cpp20 -eq $true } | Select-Object -First 1
    if ($best) {
        Write-Host ""
        Write-Host "⭐ Compilador recomendado (C++20): $($best.Name)" -ForegroundColor Green
        Write-Host "   Ruta: $($best.Path)" -ForegroundColor Gray
    } else {
        $best = $found_compilers | Select-Object -First 1
        Write-Host ""
        Write-Host "⭐ Compilador disponible: $($best.Name)" -ForegroundColor Yellow
        Write-Host "   Ruta: $($best.Path)" -ForegroundColor Gray
        Write-Host "   Nota: Solo soporta C++17, no C++20" -ForegroundColor Yellow
    }
}

Write-Host ""
```

---

## 🔍 Investigación: Zig como Linker SOLO

### ⚠️ IMPORTANTE: Zig NO es Compilador C++

**Zig NO puede:**
- ❌ Compilar código C++ → ASM
- ❌ Reemplazar a GCC/Clang en la etapa de compilación
- ❌ Compilar código C++ directamente

**Zig SÍ puede:**
- ✅ Linkear objetos `.obj` → `.exe`
- ✅ Reemplazar a GCC/Clang en la etapa de linking
- ✅ Funcionar como linker alternativo

### Cómo Usar Zig SOLO como Linker

**Flujo completo:**
```
ADead → Parser Manual → C++ Generator → GCC++/Clang++ → ASM → NASM → .obj → Zig (linker) → .exe
```

**Comando Zig para linking:**
```bash
zig build-exe archivo.obj -target x86_64-windows -lc -o archivo.exe
```

**Parámetros:**
- `build-exe`: Construir ejecutable
- `archivo.obj`: Archivo objeto a linkear (generado por NASM)
- `-target x86_64-windows`: Target Windows x86_64
- `-lc`: Linkear con C runtime (necesario para Windows)
- `-o archivo.exe`: Archivo de salida

### Verificación de Zig como Linker

**Script:** `VERIFICAR-ZIG-LINKER.ps1`

```powershell
# Script para verificar Zig como linker
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Verificación de Zig como Linker" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Buscar Zig en PATH
$zig_paths = @(
    "zig",
    "C:\zig-x86_64-windows-0.16.0-dev.1484+d0ba6642b\zig.exe",
    "C:\Users\andre\AppData\Local\Microsoft\WinGet\Packages\zig.zig_Microsoft.Winget.Source_8wekyb3d8bbwe\zig-x86_64-windows-0.14.1\zig.exe"
)

$zig_found = $null

foreach ($path in $zig_paths) {
    Write-Host "Buscando Zig: $path..." -ForegroundColor Yellow
    
    if (Test-Path $path) {
        Write-Host "  ✓ Encontrado: $path" -ForegroundColor Green
        
        try {
            $version = & $path version 2>&1
            Write-Host "  Versión: $version" -ForegroundColor Gray
            $zig_found = $path
            break
        } catch {
            Write-Host "  ⚠ Error al obtener versión: $_" -ForegroundColor Yellow
        }
    } else {
        Write-Host "  ✗ No encontrado" -ForegroundColor Red
    }
}

Write-Host ""

if (-not $zig_found) {
    Write-Host "❌ Zig no encontrado" -ForegroundColor Red
    Write-Host ""
    Write-Host "Instalar Zig:" -ForegroundColor Yellow
    Write-Host "  1. Descargar: https://ziglang.org/download/" -ForegroundColor Gray
    Write-Host "  2. Agregar al PATH" -ForegroundColor Gray
    Write-Host "  3. O usar WinGet: winget install zig.zig" -ForegroundColor Gray
} else {
    Write-Host "✅ Zig encontrado y funcionando" -ForegroundColor Green
    Write-Host ""
    Write-Host "Uso como linker:" -ForegroundColor Cyan
    Write-Host "  zig build-exe archivo.obj -target x86_64-windows -lc -o archivo.exe" -ForegroundColor White
    Write-Host ""
    Write-Host "⚠️ IMPORTANTE:" -ForegroundColor Yellow
    Write-Host "  • Zig SOLO puede linkear, NO compilar C++" -ForegroundColor Yellow
    Write-Host "  • GCC/Clang sigue siendo necesario para C++ → ASM" -ForegroundColor Yellow
    Write-Host "  • Zig reemplaza SOLO el linker, no el compilador" -ForegroundColor Yellow
}

Write-Host ""
```

---

## 🔍 Investigación: NASM

### Estado Actual

**NASM instalado:**
- Ubicación: `C:\Users\andre\AppData\Local\bin\NASM`
- En PATH: ✅ Sí
- Versión: 3.01

### Cómo Usar NASM

**Comando básico:**
```bash
nasm -f win64 archivo.asm -o archivo.obj
```

**Parámetros:**
- `-f win64`: Formato Windows 64-bit
- `archivo.asm`: Archivo ASM de entrada (generado por ADead)
- `-o archivo.obj`: Archivo objeto de salida

### Verificación de NASM

**Script:** `VERIFICAR-NASM.ps1`

```powershell
# Script para verificar NASM
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Verificación de NASM" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Buscar NASM
$nasm_paths = @(
    "nasm",
    "C:\Users\andre\AppData\Local\bin\NASM\nasm.exe"
)

$nasm_found = $null

foreach ($path in $nasm_paths) {
    Write-Host "Buscando NASM: $path..." -ForegroundColor Yellow
    
    if (Test-Path $path) {
        Write-Host "  ✓ Encontrado: $path" -ForegroundColor Green
        
        try {
            $version = & $path -v 2>&1
            Write-Host "  Versión: $version" -ForegroundColor Gray
            $nasm_found = $path
            break
        } catch {
            Write-Host "  ⚠ Error al obtener versión: $_" -ForegroundColor Yellow
        }
    } else {
        # Intentar ejecutar desde PATH
        try {
            $version = & $path -v 2>&1
            if ($LASTEXITCODE -eq 0) {
                Write-Host "  ✓ Encontrado en PATH: $path" -ForegroundColor Green
                Write-Host "  Versión: $version" -ForegroundColor Gray
                $nasm_found = $path
                break
            }
        } catch {
            Write-Host "  ✗ No encontrado" -ForegroundColor Red
        }
    }
}

Write-Host ""

if (-not $nasm_found) {
    Write-Host "❌ NASM no encontrado" -ForegroundColor Red
    Write-Host ""
    Write-Host "Instalar NASM:" -ForegroundColor Yellow
    Write-Host "  1. Descargar: https://www.nasm.us/" -ForegroundColor Gray
    Write-Host "  2. Agregar al PATH" -ForegroundColor Gray
} else {
    Write-Host "✅ NASM encontrado y funcionando" -ForegroundColor Green
    Write-Host ""
    Write-Host "Uso:" -ForegroundColor Cyan
    Write-Host "  nasm -f win64 archivo.asm -o archivo.obj" -ForegroundColor White
}

Write-Host ""
```

---

## 📋 Stack Completo Verificado

### Flujo Paso a Paso

```
┌─────────────────────────────────────────┐
│  1. ADead Source (.ad)                 │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│  2. Parser Manual (Rust)               │
│     • Parsea código ADead              │
│     • Genera AST interno                │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│  3. C++ Generator (Rust)               │
│     • AST → Código C++20/C++17         │
│     • std::vector, RAII, etc.          │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│  4. GCC++/Clang++ (REQUERIDO)          │
│     • C++ → ASM optimizado             │
│     • Detección automática C++20/C++17 │
│     • ⚠️ NO puede ser reemplazado       │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│  5. Rust Cleaner                        │
│     • Limpia ASM virgen/puro            │
│     • Elimina overhead                  │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│  6. NASM (REQUERIDO)                    │
│     • ASM → .obj                        │
│     • Formato win64                     │
│     • ✅ Ya instalado en tu sistema     │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│  7. Zig Linker (OPCIONAL)               │
│     • .obj → .exe                       │
│     • Alternativa a GCC/Clang linker   │
│     • ✅ Ya instalado en tu sistema     │
│     • ⚠️ SOLO para linking, NO compila  │
└──────────────┬──────────────────────────┘
               │
               ▼
         ⚡ Ejecutable .exe ⚡
```

### Resumen de Herramientas

| Herramienta | Estado | Ubicación | Uso |
|-------------|--------|-----------|-----|
| **GCC/Clang** | ⚠️ Necesita verificación | Varias ubicaciones posibles | Compilar C++ → ASM (REQUERIDO) |
| **NASM** | ✅ Instalado | `C:\Users\andre\AppData\Local\bin\NASM` | Ensamblar ASM → .obj (REQUERIDO) |
| **Zig** | ✅ Instalado | Múltiples ubicaciones | Linkear .obj → .exe (OPCIONAL) |

---

## 🚀 Script de Verificación Completa

**Crear:** `VERIFICAR-STACK-COMPLETO.ps1`

Este script verifica todas las herramientas del stack:

```powershell
# Script completo de verificación del stack
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Verificación Completa del Stack" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 1. Verificar GCC/Clang
Write-Host "[1/3] Verificando GCC/Clang..." -ForegroundColor Yellow
# ... código de verificación GCC/Clang ...

# 2. Verificar NASM
Write-Host "[2/3] Verificando NASM..." -ForegroundColor Yellow
# ... código de verificación NASM ...

# 3. Verificar Zig
Write-Host "[3/3] Verificando Zig..." -ForegroundColor Yellow
# ... código de verificación Zig ...

# Resumen final
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Resumen Final" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Stack completo:" -ForegroundColor Cyan
Write-Host "  ADead → Parser Manual → C++ Generator → GCC++/Clang++ → Rust Cleaner → ASM → NASM → .obj → Zig/GCC/Clang (linker) → .exe" -ForegroundColor Gray
Write-Host ""
```

---

## ✅ Conclusión

### Estado Actual

1. **NASM** ✅ - Instalado y funcionando
2. **Zig** ✅ - Instalado y funcionando (para linking)
3. **GCC/Clang** ⚠️ - Necesita verificación

### Próximos Pasos

1. Ejecutar script de verificación de GCC/Clang
2. Confirmar ubicación exacta de GCC/Clang
3. Verificar soporte C++20/C++17
4. Actualizar documentación con ubicaciones exactas

### Notas Importantes

- ⚠️ **GCC/Clang es REQUERIDO** para compilar C++ → ASM
- ⚠️ **Zig NO puede reemplazar** a GCC/Clang en compilación
- ✅ **Zig puede reemplazar** a GCC/Clang en linking
- ✅ **NASM está listo** para ensamblar ASM → .obj

---

**Última actualización:** Diciembre 2025

