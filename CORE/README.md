# 🏗️ CORE - Constructores de ASM

**Los 4 lenguajes/herramientas que construyen ASM puro para CPU directo**

Esta carpeta contiene los **4 constructores principales** del meta-compilador ADead. Cada uno tiene un rol específico en la construcción de código ASM puro y optimizado.

## 📁 Estructura

```
CORE/
├── d/              # 🔷 D Language - Metaprogramming avanzado
├── zig/            # ⚡ Zig - Parsing eficiente y generación ASM directa
├── rust/           # 🔒 Rust - Seguridad, validación y codegen NASM
└── tree-sitter/    # 🌳 Tree-sitter - Parsing robusto de estructuras complejas
```

## 🎯 Filosofía

**Cada herramienta NO interpreta**  
**Cada herramienta NO compila tradicional**  
**Cada herramienta CONSTRUYE ASM puro**

```
ADead = El Cerebro
  ↓
┌─────────────┬─────────────┬─────────────┬─────────────┐
│ Tree-sitter │     Zig     │      D      │    Rust     │
└──────┬──────┴──────┬──────┴──────┬──────┴──────┬──────┘
       │             │             │             │
       └─────────────┴─────────────┴─────────────┘
                      ↓
          CONSTRUCTORES DE ASM
                      ↓
              NASM (ASM PURO)
                      ↓
              CPU DIRECTO ⚡
```

## 🔷 D Language (`d/`)

**Rol:** Metaprogramming avanzado, CTFE, templates  
**Funciones:**
- Compile-Time Function Execution (CTFE)
- Templates avanzados para generación de código
- Optimización automática de expresiones
- Generación ASM optimizada

**Estado:** 🟡 En desarrollo (stubs activos)

## ⚡ Zig (`zig/`)

**Rol:** Parsing eficiente y generación ASM directa  
**Funciones:**
- Parsing de expresiones aritméticas (comptime)
- Generación directa de NASM sin overhead
- Soporte para floats y números grandes
- Máxima eficiencia para casos simples

**Estado:** 🟢 Funcional (con stubs cuando no disponible)

## 🔒 Rust (`rust/`)

**Rol:** Seguridad, validación y codegen NASM  
**Funciones:**
- Validación de memoria (borrow checker)
- Type checking y validación completa
- Code Generator → NASM
- Pipeline Selector Inteligente
- CLI profesional

**Estado:** 🟢 Completamente funcional

## 🌳 Tree-sitter (`tree-sitter/`)

**Rol:** Parsing robusto de estructuras complejas  
**Funciones:**
- Error recovery automático
- Manejo de bloques anidados (while/if)
- Incremental parsing (preparado para LSP)
- Parsing robusto de estructuras complejas

**Estado:** 🟢 Funcional

## 🔄 Cómo Trabajan Juntos

Los 4 constructores pueden trabajar:
- **Solo** (independiente)
- **Parejas** (2 lenguajes cooperando)
- **Tríos** (3 lenguajes juntos)
- **Cuádruple** (todos juntos - máxima potencia)

El **Pipeline Selector Inteligente** (en Rust) analiza automáticamente el código fuente y selecciona el mejor flujo según las características detectadas.

## 📚 Documentación

- Ver `README.md` en cada subcarpeta para detalles específicos
- Ver `../README.md` para arquitectura completa
- Ver `../docs/` para documentación técnica detallada

## 🚀 Compilación

Cada constructor se compila independientemente:

```powershell
# D Language
cd CORE/d
./build.ps1

# Zig
cd CORE/zig
./build-zig.ps1

# Rust (compila todo automáticamente)
cd CORE/rust
cargo build --release

# Tree-sitter (genera parser)
cd CORE/tree-sitter
tree-sitter generate
```

O usar el script completo desde la raíz:
```powershell
./build-all.ps1
```

---

**Meta-compilador Inteligente:** Construye ASM puro, no interpreta ni compila tradicionalmente.


