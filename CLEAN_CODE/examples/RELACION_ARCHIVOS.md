# 🔗 Relación de Archivos - Flujo Completo

## 📊 Flujo de Generación

```
CODIGO/test_array_original.c
    │
    ├─→ [GCC compila] ──→ EXE/test_array_funcional.exe
    │                        (Ejecutable funcional)
    │
    └─→ [GCC -S genera ASM] ──→ ASM/dirty.asm
                                    (ASM sucio de Clang/GCC)
                                    │
                                    ├─→ [CLEAN_CODE básico] ──→ ASM/basic.asm
                                    │                              │
                                    │                              └─→ [GAS compila] ──→ OBJ/basic.obj
                                    │
                                    ├─→ [CLEAN_CODE avanzado] ──→ ASM/advanced.asm
                                    │                               │
                                    │                               └─→ [GAS compila] ──→ OBJ/advanced.obj
                                    │
                                    └─→ [CLEAN_CODE extremo] ──→ ASM/extreme.asm
                                                                   │
                                                                   └─→ [GAS compila] ──→ OBJ/extreme.obj
```

## 📁 Archivos y su Origen

### 1. Código Fuente Original
**Archivo:** `CODIGO/test_array_original.c`
- **Origen:** Código C original que implementa arrays
- **Genera:** 
  - Ejecutables (EXE/)
  - ASM sucio (ASM/dirty.asm)

### 2. Ejecutables (EXE/)
**Archivos:**
- `test_array_funcional.exe` (258 KB)
- `test_array_original.exe` (258 KB) - *Duplicado, mismo archivo*

**Origen:** Compilado directamente desde `CODIGO/test_array_original.c`
```bash
gcc -O2 -o test_array_funcional.exe test_array_original.c
```

**Propósito:** Demostrar que el código funciona correctamente
**Salida:** `1`, `2`, `3` (imprime los elementos del array)

---

### 3. ASM Sucio (ASM/dirty.asm)
**Archivo:** `ASM/dirty.asm` (4,249 bytes, 204 líneas)

**Origen:** Generado por Clang/GCC desde el código C
```bash
gcc -S -masm=intel -O2 test_array_original.c -o dirty.asm
```

**Contiene:**
- Metadatos GAS (`.def`, `.scl`, `.type`, `.endef`)
- Comentarios de debug
- Código ASM sin optimizar
- Funciones: `array_new`, `array_from_values`, `array_append`, `array_get`, `array_set`, `array_len`, `main`

**Genera:** Objetos compilados (OBJ/dirty.obj)

---

### 4. ASM Limpios (ASM/)
**Archivos:**
- `ASM/basic.asm` (582 bytes, 28 líneas) - Limpieza básica
- `ASM/advanced.asm` (582 bytes, 28 líneas) - Limpieza avanzada
- `ASM/extreme.asm` (531 bytes, 26 líneas) - Limpieza extrema

**Origen:** Generados por CLEAN_CODE desde `ASM/dirty.asm`
```rust
// CLEAN_CODE limpia el ASM sucio
let cleaned = clean_asm(dirty_asm, OptimizationLevel::Extreme);
```

**Contiene:**
- Solo código esencial
- Sin metadatos innecesarios
- Sin comentarios de debug
- Código optimizado

**Genera:** Objetos compilados (OBJ/basic.obj, OBJ/advanced.obj, OBJ/extreme.obj)

---

### 5. Objetos Compilados (OBJ/)
**Archivos:**
- `OBJ/dirty.obj` (1,669 bytes) - Del ASM sucio
- `OBJ/basic.obj` (428 bytes) - Del ASM básico
- `OBJ/advanced.obj` (428 bytes) - Del ASM avanzado
- `OBJ/extreme.obj` (428 bytes) - Del ASM extremo

**Origen:** Compilados con GAS desde los archivos ASM
```bash
as --64 -o dirty.obj dirty.asm
as --64 -o basic.obj basic.asm
# etc...
```

**Propósito:** Demostrar la reducción de tamaño en código compilado

---

## 🔍 Comparación Visual

### Tamaños de Archivos

```
Código C Original:
  CODIGO/test_array_original.c ──→ ~1.6 KB

Ejecutables:
  EXE/test_array_funcional.exe ──→ 258 KB (funcional)

ASM:
  ASM/dirty.asm ──→ 4,249 bytes (sucio)
  ASM/basic.asm ──→   582 bytes (limpio básico)    [-86.3%]
  ASM/advanced.asm ──→ 582 bytes (limpio avanzado) [-86.3%]
  ASM/extreme.asm ──→  531 bytes (limpio extremo) [-87.5%]

Objetos:
  OBJ/dirty.obj ──→ 1,669 bytes (sucio)
  OBJ/basic.obj ──→   428 bytes (limpio) [-74.4%]
  OBJ/advanced.obj ──→ 428 bytes (limpio) [-74.4%]
  OBJ/extreme.obj ──→  428 bytes (limpio) [-74.4%]
```

## 📋 Tabla de Relaciones

| Archivo | Origen | Genera | Tamaño | Propósito |
|---------|--------|-------|--------|-----------|
| `CODIGO/test_array_original.c` | Original | EXE, ASM | 1.6 KB | Código fuente |
| `EXE/test_array_funcional.exe` | C → GCC | - | 258 KB | Ejecutable funcional |
| `ASM/dirty.asm` | C → GCC -S | OBJ | 4,249 B | ASM sin limpiar |
| `ASM/basic.asm` | dirty.asm → CLEAN_CODE | OBJ | 582 B | ASM limpio básico |
| `ASM/advanced.asm` | dirty.asm → CLEAN_CODE | OBJ | 582 B | ASM limpio avanzado |
| `ASM/extreme.asm` | dirty.asm → CLEAN_CODE | OBJ | 531 B | ASM limpio extremo |
| `OBJ/dirty.obj` | dirty.asm → GAS | - | 1,669 B | Objeto sucio |
| `OBJ/basic.obj` | basic.asm → GAS | - | 428 B | Objeto básico |
| `OBJ/advanced.obj` | advanced.asm → GAS | - | 428 B | Objeto avanzado |
| `OBJ/extreme.obj` | extreme.asm → GAS | - | 428 B | Objeto extremo |

## 🎯 Cómo Comparar Fácilmente

### Opción 1: Usar el Script
```powershell
.\comparar.ps1
```
Muestra automáticamente todas las comparaciones.

### Opción 2: Comparación Manual

**ASM:**
```powershell
# Ver tamaños
Get-ChildItem ASM\*.asm | Format-Table Name, Length

# Ver líneas
Get-ChildItem ASM\*.asm | ForEach-Object { 
    "$($_.Name): $((Get-Content $_.FullName).Count) líneas" 
}
```

**OBJ:**
```powershell
Get-ChildItem OBJ\*.obj | Format-Table Name, Length
```

**EXE:**
```powershell
Get-ChildItem EXE\*.exe | Format-Table Name, Length
```

## 🔗 Resumen de Relaciones

```
test_array_original.c
    │
    ├──→ test_array_funcional.exe (ejecutable funcional)
    │
    └──→ dirty.asm (ASM sucio)
            │
            ├──→ basic.asm → basic.obj
            ├──→ advanced.asm → advanced.obj
            └──→ extreme.asm → extreme.obj
```

## ✅ Conclusión

**Los ejecutables en EXE/** son el resultado final compilado desde el código C original.  
**Los ASM en ASM/** muestran cómo CLEAN_CODE reduce el código.  
**Los OBJ en OBJ/** demuestran la reducción en código compilado.

**Para comparar:** Usa `.\comparar.ps1` - muestra todo automáticamente.

---

**Última actualización:** Diciembre 2025

