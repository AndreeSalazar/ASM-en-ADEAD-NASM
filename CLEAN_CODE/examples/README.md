# 📊 CLEAN_CODE - Ejemplos y Comparación

## 🗂️ Estructura de Carpetas

```
examples/
├── README.md              # Este archivo (guía principal)
├── comparar.ps1           # Script único para comparar TODO
│
├── ASM/                   # Archivos ASM (sucio y limpios)
│   ├── dirty.asm          # ASM original de Clang (sucio)
│   ├── basic.asm          # ASM limpio - nivel básico
│   ├── advanced.asm       # ASM limpio - nivel avanzado
│   └── extreme.asm        # ASM limpio - nivel extremo
│
├── OBJ/                   # Objetos compilados (.obj)
│   ├── dirty.obj          # Objeto del ASM sucio
│   ├── basic.obj          # Objeto del ASM básico
│   ├── advanced.obj       # Objeto del ASM avanzado
│   └── extreme.obj         # Objeto del ASM extremo
│
├── EXE/                   # Ejecutables
│   └── test_array_funcional.exe  # Ejecutable funcional (compilado desde C)
│
├── CODIGO/                # Código fuente C
│   └── test_array_original.c     # Código C original
│
├── docs/                  # Documentación
│   ├── RESULTADOS.md      # Resultados de la comparación
│   ├── IDEAS.md           # Ideas para mejoras futuras
│   └── SOLUCION.md        # Soluciones independientes
│
└── scripts/               # Scripts adicionales (opcionales)
    └── (scripts de desarrollo)
```

## 🚀 Uso Rápido

### Comparar los 4 elementos (Sucio vs Limpios)

```powershell
.\comparar.ps1
```

Este script:
- ✅ Compila el código C original a ejecutable
- ✅ Ejecuta el programa y muestra resultados
- ✅ Compara los 4 archivos ASM (Sucio, Básico, Avanzado, Extremo)
- ✅ Compara los objetos compilados
- ✅ Muestra reducciones y estadísticas

### Ver relación entre archivos

```powershell
.\ver_relacion.ps1
```

Este script muestra:
- ✅ De dónde viene cada archivo
- ✅ Qué genera cada archivo
- ✅ Diagrama de flujo completo
- ✅ Relaciones entre todos los archivos

## 📈 Resultados Esperados

### Reducción en ASM:
- **Sucio:** 4,249 bytes, 204 líneas
- **Básico:** 582 bytes, 28 líneas (**-86.3%**)
- **Avanzado:** 582 bytes, 28 líneas (**-86.3%**)
- **Extremo:** 531 bytes, 26 líneas (**-87.5%**)

### Reducción en Objetos:
- **Sucio:** 1,669 bytes
- **Limpios:** 428 bytes cada uno (**-74.4%**)

## 📁 Archivos Principales

### CODIGO (Carpeta `CODIGO/`)
- `test_array_original.c` - **Código fuente original**
  - Genera: Ejecutables (EXE/) y ASM sucio (ASM/dirty.asm)

### EXE (Carpeta `EXE/`)
- `test_array_funcional.exe` - **Ejecutable funcional**
  - Origen: Compilado desde `CODIGO/test_array_original.c`
  - Propósito: Demostrar que el código funciona (salida: 1, 2, 3)
  - Tamaño: 258 KB

### ASM (Carpeta `ASM/`)
- `dirty.asm` - **ASM sucio** (original de Clang/GCC)
  - Origen: Generado desde `CODIGO/test_array_original.c`
  - Tamaño: 4,249 bytes, 204 líneas
  - Genera: `OBJ/dirty.obj`
- `basic.asm` - ASM limpio con optimizaciones básicas
  - Origen: `ASM/dirty.asm` → CLEAN_CODE básico
  - Tamaño: 582 bytes, 28 líneas (-86.3%)
- `advanced.asm` - ASM limpio con optimizaciones avanzadas
  - Origen: `ASM/dirty.asm` → CLEAN_CODE avanzado
  - Tamaño: 582 bytes, 28 líneas (-86.3%)
- `extreme.asm` - ASM limpio con optimizaciones extremas
  - Origen: `ASM/dirty.asm` → CLEAN_CODE extremo
  - Tamaño: 531 bytes, 26 líneas (-87.5%)

### OBJ (Carpeta `OBJ/`)
- Objetos compilados desde los archivos ASM usando GAS
- `dirty.obj` - 1,669 bytes
- `basic.obj`, `advanced.obj`, `extreme.obj` - 428 bytes cada uno (-74.4%)

## 🔍 Comparación Visual

```
ASM Sucio:     ████████████████████████████████████████ 4,249 bytes
ASM Básico:    ████████                                   582 bytes (-86.3%)
ASM Avanzado:  ████████                                   582 bytes (-86.3%)
ASM Extremo:   ████████                                   531 bytes (-87.5%)

OBJ Sucio:     ████████████████████████████████ 1,669 bytes
OBJ Limpios:   ████████                             428 bytes (-74.4%)
```

## 🔗 Relación entre Archivos

```
CODIGO/test_array_original.c
    │
    ├──→ [gcc -O2] ──→ EXE/test_array_funcional.exe
    │
    └──→ [gcc -S] ──→ ASM/dirty.asm
            │
            ├──→ [CLEAN_CODE básico] ──→ ASM/basic.asm ──→ [GAS] ──→ OBJ/basic.obj
            ├──→ [CLEAN_CODE avanzado] ──→ ASM/advanced.asm ──→ [GAS] ──→ OBJ/advanced.obj
            └──→ [CLEAN_CODE extremo] ──→ ASM/extreme.asm ──→ [GAS] ──→ OBJ/extreme.obj
```

**Para ver la relación completa:** Ejecuta `.\ver_relacion.ps1`

## 📚 Documentación Adicional

Ver carpeta `docs/` para:
- Resultados detallados
- Ideas de mejoras
- Soluciones independientes

## 🎯 Conclusión

**CLEAN_CODE logró:**
- ✅ Reducir ASM en **87.5%** (4,249 → 531 bytes)
- ✅ Reducir líneas en **87.3%** (204 → 26 líneas)
- ✅ Reducir objetos en **74.4%** (1,669 → 428 bytes)
- ✅ Mantener funcionalidad completa

---

**Última actualización:** Diciembre 2025  
**Módulo:** CLEAN_CODE - Modo EXTREMO 🔥

