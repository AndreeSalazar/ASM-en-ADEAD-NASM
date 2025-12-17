# ADead Compiler Core 🦀

**Compilador ADead - Sintaxis estilo Python que compila a NASM (x86_64)**

> Genera código ASM virgen y simple para ejecución directa en CPU

## 🎯 Pipeline Principal: NASM Prioritario

```
╔════════════════════════════════════════════════════════════════════════════╗
║                        ADead Compiler Pipeline                              ║
║                                                                             ║
║  ┌──────────┐    ┌──────────┐    ┌────────────┐    ┌──────────────────┐   ║
║  │  .ad     │ →  │  Parser  │ →  │   NASM     │ →  │  .asm → .obj →   │   ║
║  │  Source  │    │ (Chumsky)│    │ Generator  │    │  .exe (Linker)   │   ║
║  └──────────┘    └──────────┘    └────────────┘    └──────────────────┘   ║
║                                                                             ║
║  Pipeline Principal: ADead → NASM Directo → Ejecutable                     ║
║  Pipeline Fallback:  ADead → C++ → GCC++ → Cleaner → NASM → Ejecutable     ║
╚════════════════════════════════════════════════════════════════════════════╝
```

## 📁 Estructura del Proyecto

```
rust/
├── crates/
│   ├── adead-cli/              # CLI principal (adeadc.exe)
│   │   ├── main.rs             # Entry point y comandos
│   │   ├── linker.rs           # Integración con Zig/GCC/Clang linkers
│   │   └── c_compiler.rs       # Detección de GCC/Clang
│   │
│   ├── adead-parser/           # Parser y generadores de código
│   │   ├── lib.rs              # Parser principal (Chumsky combinators)
│   │   ├── pipeline_selector.rs # Selección inteligente de pipeline
│   │   ├── clean_asm.rs        # Limpieza ASM (GAS→NASM, optimización)
│   │   ├── cpp_generator.rs    # C++ backend (fallback)
│   │   ├── c_generator.rs      # C backend (fallback)
│   │   ├── c_to_nasm.rs        # Conversión directa C→NASM
│   │   ├── c_manual_parser.rs  # Parser manual para C
│   │   ├── cpp_optimizer.rs    # Optimizador de AST para C++
│   │   ├── module_resolver.rs  # Resolución de imports
│   │   └── parallel_pipeline.rs # Pipeline paralelo con caching
│   │
│   ├── adead-backend/          # NASM Generator principal
│   │   ├── lib.rs              # Generador NASM x86_64 (Windows/Linux)
│   │   ├── optimizer.rs        # Dead code elimination
│   │   ├── stdlib.rs           # Librería estándar embebida
│   │   ├── register_optimizer.rs # Optimización de uso de registros
│   │   ├── dependency_graph.rs # Análisis de dependencias
│   │   ├── usage_analyzer.rs   # Análisis de uso de funciones
│   │   └── memory_pool.rs      # Pool de memoria para arrays
│   │
│   ├── adead-borrow/           # Borrow checker (en desarrollo)
│   │   └── lib.rs              # Sistema de ownership
│   │
│   └── adead-common/           # Utilidades compartidas
│       └── lib.rs              # Tipos de error, traits comunes
│
├── Cargo.toml                  # Workspace configuration
└── Cargo.lock                  # Dependency lock
```

## 🔧 Compilación

```bash
cd CORE/rust
cargo build --release
```

El ejecutable se genera en: `target/release/adeadc.exe`

## 📖 Uso

### Compilar a ASM (NASM directo - Recomendado)

```bash
# Pipeline NASM directo (genera ASM virgen)
adeadc compile programa.ad -o programa.asm

# Especificar backend explícitamente
adeadc compile programa.ad --backend nasm -o programa.asm
```

### Build Completo (ASM → OBJ → EXE)

```bash
# Build completo con detección automática de linker
adeadc build programa.ad -o programa.exe

# Con linker específico
adeadc build programa.ad --linker zig -o programa.exe   # Recomendado
adeadc build programa.ad --linker gcc -o programa.exe
adeadc build programa.ad --linker clang -o programa.exe

# Solo ensamblar (sin linkear)
adeadc build programa.ad --assemble-only
```

### Comandos Individuales

```bash
# Ensamblar .asm a .obj
adeadc assemble programa.asm -o programa.obj

# Linkear .obj a .exe
adeadc link programa.obj -o programa.exe
adeadc link archivo1.obj archivo2.obj -o programa.exe --linker zig
```

## 🎯 Comandos Disponibles

| Comando | Descripción | Uso |
|---------|-------------|-----|
| `compile` | Compila .ad a .asm | `adeadc compile archivo.ad -o salida.asm` |
| `build` | Compila .ad a .exe (completo) | `adeadc build archivo.ad -o programa.exe` |
| `assemble` | Ensambla .asm a .obj | `adeadc assemble archivo.asm -o archivo.obj` |
| `link` | Linkea .obj a .exe | `adeadc link archivo.obj -o programa.exe` |

## 🔄 Pipelines

### 1. NASM Directo (Principal - Prioridad Alta)

```
ADead → Parser (Rust) → NASM Generator (Rust) → ASM Virgen → NASM → .obj → Linker → .exe
```

**Características:**
- ✅ Genera NASM x86_64 directamente desde AST
- ✅ Sin dependencias externas para compilación
- ✅ Windows x64 ABI compliant
- ✅ Dead code elimination integrado
- ✅ Optimizador de registros
- ✅ Librería estándar embebida

**Soporta:** Variables, Arrays, Strings, Control Flow, Funciones, Structs, Classes, Import/Export

### 2. C++ Pipeline (Fallback)

```
ADead → Parser → C++ Generator → GCC++/Clang++ → Rust Cleaner → ASM → NASM → .obj → Linker → .exe
```

**Características:**
- Usado para características avanzadas no implementadas en NASM directo
- Requiere GCC/Clang instalado
- C++20 con fallback automático a C++17
- Rust Cleaner optimiza y convierte GAS→NASM

## 🧪 Tests

```bash
# Todos los tests
cargo test --workspace

# Tests específicos del parser
cargo test -p adead-parser

# Tests del backend NASM
cargo test -p adead-backend

# Test específico
cargo test test_parse_while_loop
```

## 📋 Características del Lenguaje

### Variables
```ad
let x = 42           # Inmutable
let mut y = 0        # Mutable
y = 10               # Asignación
```

### Arrays
```ad
let arr = [1, 2, 3]
arr.append(4)        # Agregar elemento
arr.pop()            # Eliminar último
arr.sort()           # Ordenar
arr.reverse()        # Invertir
let n = len(arr)     # Longitud
let v = arr[0]       # Indexación
```

### Strings
```ad
let s = "hello"
let t = s + " world" # Concatenación
let u = s.upper()    # Mayúsculas
let l = s.lower()    # Minúsculas
let sub = s[0:3]     # Slicing
let n = len(s)       # Longitud
```

### Control de Flujo
```ad
if x > 5 {
    print "grande"
} else {
    print "pequeño"
}

while i < 10 {
    print i
    i = i + 1
}

for i in 0..10 {
    print i
}
```

### Funciones
```ad
fn suma(a, b) {
    return a + b
}

pub fn publica(x) {  # Exportable
    return x * 2
}

let result = suma(5, 3)
```

### Structs y Classes
```ad
struct Punto {
    x
    y
}

class Rectangulo {
    fn new(ancho, alto) {
        self.ancho = ancho
        self.alto = alto
    }
    
    fn area(self) {
        return self.ancho * self.alto
    }
}

let p = Punto { x: 10, y: 20 }
let r = Rectangulo.new(5, 3)
print r.area()
```

### Módulos
```ad
import math          # Importar módulo
let x = math.sqrt(16)
```

### Operadores
```ad
# Aritméticos
+ - * / %

# Comparación
== != < > <= >=

# Lógicos
&& || !
```

## 🏗️ Arquitectura Interna

### Parser (Chumsky)
- Parser combinador recursivo
- AST tipado con soporte de ownership
- Manejo de errores con ubicación precisa

### NASM Generator
- Generación directa de x86_64
- Windows x64 calling convention (RCX, RDX, R8, R9)
- Stack frame management
- Shadow space allocation (32 bytes)

### Optimizaciones
- Dead code elimination (análisis de dependencias)
- Register allocation optimization
- Unused function elimination
- Constant folding (parcial)

### Linker Integration
- Zig (recomendado): ReleaseSmall, strip, single-threaded
- GCC: -nostdlib, -s, -Wl,--gc-sections
- Clang: Similar a GCC

## 📦 Dependencias

```toml
[workspace.dependencies]
anyhow = "1.0"       # Error handling
thiserror = "1.0"    # Error types
clap = "4.5"         # CLI parsing
chumsky = "0.9"      # Parser combinators
logos = "0.14"       # Lexer (opcional)
```

## 👤 Autor

**Eddi Andreé Salazar Matos**

Diciembre 2025
