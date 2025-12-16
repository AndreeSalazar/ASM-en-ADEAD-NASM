# Solución: Error "No se pudo compilar C++ a ASM"

## Problema

El error ocurre porque:
1. El backend `auto` detecta código simple (sin strings avanzados)
2. Usa el pipeline C++ que requiere GCC++/Clang++
3. No hay compilador C++ disponible en el sistema
4. El pipeline falla al intentar compilar C++ a ASM

## Solución

**Usar el backend `nasm` o `direct`** que genera NASM directamente sin necesidad de compilador C++:

```powershell
.\adeadc.ps1 build test.ad --linker zig --backend nasm
```

O:

```powershell
.\adeadc.ps1 build test.ad --linker zig --backend direct
```

## Comparación de Backends

### `auto` (por defecto)
- ✅ Detecta automáticamente el mejor pipeline
- ❌ Requiere compilador C++ para código simple
- ✅ Usa NASM directo para strings avanzados

### `nasm` o `direct` (recomendado)
- ✅ Genera NASM directamente
- ✅ No requiere compilador C++
- ✅ Funciona siempre
- ✅ Más rápido (sin pasos intermedios)

### `cpp`
- ❌ Requiere compilador C++ (GCC++/Clang++)
- ✅ Útil si necesitas optimizaciones de C++

## Ejemplo Corregido

**Antes (falla):**
```powershell
.\adeadc.ps1 build test.ad --linker zig --backend auto
# Error: No se pudo compilar C++ a ASM
```

**Después (funciona):**
```powershell
.\adeadc.ps1 build test.ad --linker zig --backend nasm
# ✅ Compila, ensambla y linkea correctamente
```

## Recomendación

**Para la mayoría de casos, usa `--backend nasm`** porque:
- No requiere compiladores externos
- Es más rápido
- Genera código NASM puro y limpio
- Funciona siempre

---

**Fecha:** Diciembre 2025

