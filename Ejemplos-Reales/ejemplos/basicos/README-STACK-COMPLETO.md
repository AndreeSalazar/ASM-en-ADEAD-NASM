# 🚀 Stack Completo para Build y Ejecución

**Fecha:** Diciembre 2025  
**Autor:** Eddi Andreé Salazar Matos

## 🎯 Stack Completo Implementado

```
ADead → Parser Manual → C++ Generator → GCC++/Clang++ → Rust Cleaner → ASM → NASM → .obj → Zig/GCC/Clang (linker) → .exe
```

### Componentes del Stack

1. **Parser Manual (Rust)** - Parsea código ADead
2. **C++ Generator (Rust)** - Genera código C++20/C++17
3. **GCC++/Clang++** - Compila C++ → ASM (REQUERIDO)
4. **Rust Cleaner** - Limpia ASM virgen/puro
5. **NASM** - Ensambla ASM → .obj (REQUERIDO)
6. **Zig/GCC/Clang Linker** - Linkea .obj → .exe (Zig recomendado)

---

## 📋 Scripts Disponibles

### ⭐ Script Principal: `BUILD-COMPLETO-STACK.ps1`

**Compila y ejecuta TODOS los tests con el stack completo.**

**Uso:**
```powershell
# Compilar y ejecutar todos los tests
.\BUILD-COMPLETO-STACK.ps1

# Compilar solo tests de strings
.\BUILD-COMPLETO-STACK.ps1 -Filter "test_strings*.ad"

# Compilar sin ejecutar
.\BUILD-COMPLETO-STACK.ps1 -NoExecute

# Modo verbose (mostrar más detalles)
.\BUILD-COMPLETO-STACK.ps1 -Verbose
```

**O usando el wrapper batch:**
```cmd
BUILD-COMPLETO-STACK.bat
EJECUTAR-TODOS-COMPLETO.bat
```

### Script Individual: `BUILD-Y-EJECUTAR.ps1`

**Compila y ejecuta un test individual.**

**Uso:**
```powershell
.\BUILD-Y-EJECUTAR.ps1 test_strings_basico.ad
```

**O usando el wrapper batch:**
```cmd
BUILD-Y-EJECUTAR.bat test_strings_basico.ad
```

---

## 🔍 Detección Automática

El script detecta automáticamente:

### 1. Compilador ADead
Busca en:
- `..\..\..\CORE\rust\target\release\adeadc.exe`
- `CORE\rust\target\release\adeadc.exe`
- PATH (`adeadc.exe`)

### 2. Compilador C++
**Prioridad:**
1. Clang++ (LLVM) - Mejor soporte C++20
2. GCC++ (MSYS2) - Fallback C++17

**Ubicaciones buscadas:**
- `clang++` (PATH)
- `C:\Program Files\LLVM\bin\clang++.exe`
- `g++` (PATH)
- `C:\msys64\mingw64\bin\g++.exe`

### 3. NASM
**Ubicaciones buscadas:**
- `nasm` (PATH)
- `C:\Users\andre\AppData\Local\bin\NASM\nasm.exe`

### 4. Linker
**Prioridad:**
1. Zig (recomendado - más fácil de instalar)
2. GCC++/Clang++ (fallback)

**Ubicaciones Zig:**
- `zig` (PATH)
- `C:\zig-x86_64-windows-0.16.0-dev.1484+d0ba6642b\zig.exe`

---

## 📊 Flujo Paso a Paso

### Paso 1: Compilar ADead → ASM
```powershell
adeadc.exe compile test.ad --backend cpp -o test.asm
```
- Parser Manual parsea código ADead
- C++ Generator genera código C++20/C++17
- GCC++/Clang++ compila C++ → ASM
- Rust Cleaner limpia ASM virgen/puro

### Paso 2: Ensamblar ASM → .obj
```powershell
nasm -f win64 test.asm -o test.obj
```
- NASM ensambla código ASM a objeto

### Paso 3: Linkear .obj → .exe
```powershell
# Con Zig (recomendado)
zig build-exe test.obj -target x86_64-windows -lc -o test.exe

# O con GCC/Clang
g++ test.obj -o test.exe
```
- Linker enlaza objeto a ejecutable

### Paso 4: Ejecutar
```powershell
.\test.exe
```

---

## ✅ Verificación del Stack

Antes de compilar, verifica que todas las herramientas estén disponibles:

```powershell
# Desde el directorio raíz del proyecto
.\VERIFICAR-STACK-COMPLETO.ps1
```

Esto verificará:
- ✅ Compilador ADead
- ✅ Compilador C++ (GCC++/Clang++)
- ✅ NASM
- ✅ Zig/GCC/Clang Linker

---

## 🎯 Ejemplos de Uso

### Ejemplo 1: Compilar y ejecutar un test individual

```powershell
cd Ejemplos-Reales\ejemplos\basicos
.\BUILD-Y-EJECUTAR.ps1 test_strings_basico.ad
```

**Salida esperada:**
```
[1/4] Compilando ADead -> ASM...
[OK] ASM generado: test_strings_basico.asm
[2/4] Ensamblando ASM -> .obj...
[OK] Objeto generado: test_strings_basico.obj
[3/4] Linkeando .obj -> .exe...
[OK] Ejecutable generado: test_strings_basico.exe
[4/4] Ejecutando...
Hello World
[COMPLETADO] Codigo de salida: 0
```

### Ejemplo 2: Compilar todos los tests

```powershell
.\BUILD-COMPLETO-STACK.ps1
```

**Salida esperada:**
```
[1/6] Detectando herramientas...
  [OK] Compilador ADead: C:\...\adeadc.exe
  [OK] Compilador C++: C:\Program Files\LLVM\bin\clang++.exe
  [OK] NASM: nasm
  [OK] Linker: zig

[2/6] Buscando archivos .ad...
  [OK] Encontrados 48 archivos .ad

[3/6] Compilando archivos...
  Compilando: test_strings_basico.ad...
    [OK] ASM generado: test_strings_basico.asm
    [OK] Objeto generado: test_strings_basico.obj
    [OK] Ejecutable generado: test_strings_basico.exe
  ...

[4/6] Resumen de compilacion...
Total archivos: 48
Compilados exitosamente: 45
Fallidos: 3

[5/6] Ejecutables generados...
Ejecutables listos:
  - test_strings_basico.exe (12.5 KB)
  - test_strings_concat.exe (13.2 KB)
  ...
```

### Ejemplo 3: Compilar solo tests de strings

```powershell
.\BUILD-COMPLETO-STACK.ps1 -Filter "test_strings*.ad"
```

### Ejemplo 4: Compilar sin ejecutar (solo build)

```powershell
.\BUILD-COMPLETO-STACK.ps1 -NoExecute
```

---

## 🔧 Solución de Problemas

### Error: "Compilador ADead no encontrado"

**Solución:**
```powershell
cd CORE\rust
cargo build --release
```

### Error: "Compilador C++ no encontrado"

**Solución:**
Instala Clang++ (recomendado) o GCC++:
- **Clang++:** https://llvm.org/builds/
- **GCC++:** Instalar MSYS2 desde https://www.msys2.org/

### Error: "NASM no encontrado"

**Solución:**
Instala NASM desde https://www.nasm.us/ y agrégalo al PATH.

### Error: "Linker no encontrado"

**Solución:**
Instala Zig (recomendado) o usa GCC/Clang:
- **Zig:** https://ziglang.org/download/
- O agrega GCC/Clang al PATH

### Error: "Se generó C++ en lugar de ASM"

**Causa:** El compilador C++ no está disponible o no funciona.

**Solución:**
1. Verifica que Clang++ o GCC++ esté instalado
2. Ejecuta: `.\VERIFICAR-GCC-CLANG.ps1`
3. Asegúrate de que el compilador esté en el PATH

---

## 📈 Estadísticas del Stack

### Tiempos Estimados

| Operación | Tiempo Estimado |
|-----------|----------------|
| Compilar ADead → ASM | 1-3 segundos |
| Ensamblar ASM → .obj | < 1 segundo |
| Linkear .obj → .exe | < 1 segundo |
| **Total por test** | **2-5 segundos** |

### Tamaños Generados

| Archivo | Tamaño Típico |
|---------|---------------|
| `.ad` (fuente) | 100-500 bytes |
| `.asm` (generado) | 1-5 KB |
| `.obj` (objeto) | 2-10 KB |
| `.exe` (ejecutable) | 10-50 KB |

---

## 🎯 Ventajas del Stack Completo

### ✅ Detección Automática
- No necesitas configurar nada manualmente
- El script encuentra todas las herramientas automáticamente
- Funciona con diferentes instalaciones

### ✅ Stack Optimizado
- C++20 cuando está disponible (mejor código)
- C++17 como fallback (compatibilidad)
- ASM virgen/puro (sin overhead)
- Ejecutables pequeños (10-50 KB)

### ✅ Flexibilidad
- Zig como linker (más fácil de instalar)
- GCC/Clang como alternativa
- Funciona con lo que tengas disponible

---

## 📚 Scripts Relacionados

- `BUILD-COMPLETO-STACK.ps1` - Script principal (compilar todos)
- `BUILD-Y-EJECUTAR.ps1` - Script individual (un test)
- `VERIFICAR-STACK-COMPLETO.ps1` - Verificar herramientas
- `ejecutar_con_zig.bat` - Script legacy (todavía funciona)
- `ejecutar_test_simple.bat` - Script legacy (todavía funciona)

---

## ✅ Checklist de Verificación

Antes de usar el stack completo, verifica:

- [ ] Compilador ADead compilado (`CORE\rust\target\release\adeadc.exe`)
- [ ] Compilador C++ instalado (Clang++ o GCC++)
- [ ] NASM instalado y en PATH
- [ ] Zig instalado (opcional pero recomendado)
- [ ] Todas las herramientas verificadas con `VERIFICAR-STACK-COMPLETO.ps1`

---

**Última actualización:** Diciembre 2025













