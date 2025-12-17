# 🚀 ADead Pipeline - Compilación a NASM

## Descripción

ADead compila código con sintaxis estilo Python directamente a NASM (x86_64).

```
┌─────────────────────────────────────────────────────────────────┐
│  ADead (.ad) → Parser (Rust) → NASM Generator → NASM → .exe    │
└─────────────────────────────────────────────────────────────────┘
```

## Pipelines Disponibles

### 1. NASM Directo (Prioridad Alta) ✅

**Ruta:** `ADead → Parser (Chumsky) → NASM Generator (Rust) → NASM → .obj → Linker → .exe`

- Genera código NASM puro directamente
- Sin dependencia de GCC/Clang para compilación
- Soporta: Variables, Arrays, Strings, Control Flow, Funciones

```bash
adeadc build programa.ad --backend nasm
```

### 2. C++ Pipeline (Fallback)

**Ruta:** `ADead → Parser → C++ Generator → GCC++/Clang++ → ASM Cleaner → NASM`

- Usado para características no implementadas en NASM directo
- Requiere GCC++ o Clang++ instalado
- ASM Cleaner convierte GAS a NASM automáticamente

```bash
adeadc compile programa.ad --backend cpp -o programa.asm
```

## Componentes

### 1. Parser (Rust - Chumsky)
- **Archivo:** `lib.rs`
- **Función:** Parsea código ADead a AST
- Soporta: let, print, if, while, for, funciones, arrays, strings, structs

### 2. NASM Generator (Rust - adead-backend)
- **Archivo:** `adead-backend/src/lib.rs`
- **Función:** Genera código NASM x86_64 desde AST
- ABI Windows x64 compliant
- Optimizaciones: dead code elimination, register allocation

### 3. ASM Cleaner (Rust)
- **Archivo:** `clean_asm.rs`
- **Función:** Limpia y optimiza código ASM
- Convierte GAS a NASM automáticamente
- Elimina: SEH metadata, frame pointers innecesarios, código muerto

## Uso

### Compilar a NASM

```bash
# Pipeline NASM directo (recomendado)
adeadc compile programa.ad --backend nasm -o programa.asm

# Pipeline C++ (fallback)
adeadc compile programa.ad --backend cpp -o programa.asm
```

### Compilar a Ejecutable

```bash
# Build completo (compile + assemble + link)
adeadc build programa.ad -o programa.exe

# Con linker específico
adeadc build programa.ad --linker zig -o programa.exe
```

### Ensamblar y Linkear

```bash
# Solo ensamblar
adeadc assemble programa.asm -o programa.obj

# Solo linkear
adeadc link programa.obj -o programa.exe
```

## Estado Actual

- ✅ **NASM Generator:** Completo (arrays, strings, control flow, funciones)
- ✅ **Parser:** Completo (todas las características del lenguaje)
- ✅ **ASM Cleaner:** Completo (optimizaciones + conversión GAS→NASM)
- ✅ **C++ Pipeline:** Completo (fallback funcional)
- ✅ **CLI:** Completo (compile, build, assemble, link)

## Beneficios

- ✅ Código NASM limpio y legible
- ✅ Sin runtime (ejecutables pequeños)
- ✅ Sin garbage collector
- ✅ ABI Windows x64 compliant
- ✅ Ejecutables independientes
