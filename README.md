<div align="center">

# 🇵🇪 .ad — ADead

**ASM is dead (but powerful)**

Simple sintaxis estilo Python • Rendimiento nativo

🎨 **Icono personalizado para archivos `.ad`** - Identidad visual única en Windows

**Desarrollado por:** Eddi Andreé Salazar Matos  
**Fecha:** 11 de Diciembre de 2025

</div>

## 🔄 Arquitectura Inteligente Cuádruple: Tree-sitter + Zig + D Language + Rust

**ADead utiliza una arquitectura inteligente única de 4 lenguajes/herramientas que analiza automáticamente el código fuente y selecciona el mejor pipeline según las características detectadas, trabajando solos, combinados o independientemente hasta generar código ASM puro y optimizado para la CPU:**

### 🌳 Tree-sitter + Rust → NASM (ASM)
**Para estructuras complejas** (while/if anidados, parsing robusto):
```
ADead Source (.ad)
  ↓
┌─────────────────────────────────────┐
│  Tree-sitter (parsing robusto)      │
│  • Error recovery automático        │
│  • Bloques anidados                 │
│  • Genera AST Tree-sitter           │
└─────────────────────────────────────┘
  ↓ (AST Tree-sitter)
┌─────────────────────────────────────┐
│  Rust (conversión + validación)     │
│  • AST Tree-sitter → AST Rust      │
│  • Validación memoria               │
│  • Type checking                    │
│  • Codegen → NASM                   │
└─────────────────────────────────────┘
  ↓
┌─────────────────────────────────────┐
│  NASM (Assembly x86_64)            │
│  • Código ASM puro                 │
└─────────────────────────────────────┘
  ↓
✅ Ejecutable (.exe) → CPU Directo
```
**Ventajas:** Parsing robusto de estructuras anidadas, error recovery automático  
**Uso:** Loops complejos, estructuras anidadas, programas grandes  
**Trabaja:** Independiente o combinado con otros

### ⚡ Zig → NASM Directo (ASM)
**Para casos simples** (floats, expresiones aritméticas básicas):
```
ADead Source (.ad)
  ↓
┌─────────────────────────────────────┐
│  Zig (parsea y genera ASM)         │
│  • Comptime (compile-time)          │
│  • Parsing eficiente                │
│  • Generación ASM directa           │
│  • Sin overhead                     │
└─────────────────────────────────────┘
  ↓
┌─────────────────────────────────────┐
│  NASM (Assembly x86_64)            │
│  • Código ASM puro                 │
└─────────────────────────────────────┘
  ↓
✅ Ejecutable (.exe) → CPU Directo
```
**Ventajas:** Máxima eficiencia, sin overhead de validación  
**Uso:** Floats simples, expresiones aritméticas puras, máxima performance  
**Trabaja:** Independiente (solo) o combinado con Rust

### 🔒 Zig → Rust → NASM (ASM)
**Para código que requiere validación** (variables, funciones, structs):
```
ADead Source (.ad)
  ↓
┌─────────────────────────────────────┐
│  Zig (parsing eficiente)            │
│  • Comptime evaluation              │
│  • Expresiones aritméticas          │
│  • Genera datos estructurados       │
└─────────────────────────────────────┘
  ↓ (Datos estructurados)
┌─────────────────────────────────────┐
│  Rust (validación + seguridad)      │
│  • Validación memoria               │
│  • Type checking                    │
│  • Borrow checker                   │
│  • Codegen → NASM                   │
└─────────────────────────────────────┘
  ↓
┌─────────────────────────────────────┐
│  NASM (Assembly x86_64)            │
│  • Código ASM puro                 │
└─────────────────────────────────────┘
  ↓
✅ Ejecutable (.exe) → CPU Directo
```
**Ventajas:** Seguridad garantizada, validación completa, parsing eficiente  
**Uso:** Variables, funciones, structs, OOP, expresiones complejas  
**Trabaja:** Cooperación Zig + Rust (2 lenguajes)

### 🛠️ Rust Directo → NASM (ASM)
**Para casos especiales** (fallback cuando otros fallan):
```
ADead Source (.ad)
  ↓
┌─────────────────────────────────────┐
│  Rust (parser Chumsky completo)     │
│  • Parser completo en Rust          │
│  • Validación completa              │
│  • Type checking                    │
│  • Codegen → NASM                   │
└─────────────────────────────────────┘
  ↓
┌─────────────────────────────────────┐
│  NASM (Assembly x86_64)            │
│  • Código ASM puro                 │
└─────────────────────────────────────┘
  ↓
✅ Ejecutable (.exe) → CPU Directo
```
**Ventajas:** Parser completo en Rust, fallback robusto  
**Uso:** Sintaxis compleja, casos edge, último recurso  
**Trabaja:** Independiente (solo Rust)

### 🔷 D Language → NASM (ASM) ⭐ NUEVO
**Para metaprogramming avanzado** (CTFE, templates, generación ASM optimizada):
```
ADead Source (.ad)
  ↓
┌─────────────────────────────────────┐
│  D Language (metaprogramming)       │
│  • CTFE (Compile-Time Execution)    │
│  • Templates avanzados              │
│  • Validación compile-time          │
│  • Optimización automática          │
│  • Genera código ASM directamente   │
└─────────────────────────────────────┘
  ↓
┌─────────────────────────────────────┐
│  NASM (Assembly x86_64)            │
│  • Código ASM puro optimizado      │
└─────────────────────────────────────┘
  ↓
✅ Ejecutable (.exe) → CPU Directo
```
**Ventajas:** Metaprogramming más poderoso, CTFE, generación ASM optimizada  
**Uso:** Generación de código avanzada, optimizaciones, metaprogramming complejo  
**Trabaja:** Independiente (solo D) o combinado con Rust

### 🌟 Tree-sitter + D Language + Rust → NASM (ASM) ⭐ NUEVO
**Para máxima potencia** (parsing robusto + metaprogramming + seguridad):
```
ADead Source (.ad)
  ↓
┌─────────────────────────────────────┐
│  Tree-sitter (parsing robusto)      │
│  • Estructuras complejas            │
│  • Error recovery                   │
└─────────────────────────────────────┘
  ↓ (AST Tree-sitter)
┌─────────────────────────────────────┐
│  D Language (metaprogramming)       │
│  • CTFE: Validación compile-time    │
│  • Templates: Generación ASM        │
│  • Optimización automática          │
└─────────────────────────────────────┘
  ↓ (ASM mejorado)
┌─────────────────────────────────────┐
│  Rust (seguridad final)             │
│  • Validación memoria               │
│  • Codegen NASM final               │
└─────────────────────────────────────┘
  ↓
┌─────────────────────────────────────┐
│  NASM (Assembly x86_64)            │
│  • Código ASM puro optimizado      │
└─────────────────────────────────────┘
  ↓
✅ Ejecutable (.exe) → CPU Directo
```
**Ventajas:** Combinación de parsing robusto + metaprogramming + seguridad  
**Uso:** Proyectos grandes, máxima optimización, código crítico  
**Trabaja:** Cooperación de 3 lenguajes (Tree-sitter + D + Rust)

### 🤖 Sistema de Selección Inteligente

**ADead utiliza un Pipeline Selector Inteligente que analiza automáticamente el código fuente y selecciona el mejor flujo de compilación:**

1. **Análisis Automático**: Detecta características del código (while loops, if statements, anidamiento, variables, expresiones, complejidad)
2. **Selección Óptima**: Elige el mejor pipeline según las características detectadas
3. **Generación Optimizada**: Genera código ASM puro y limpio
4. **Fallback Inteligente**: Si un pipeline falla, intenta automáticamente otros pipelines

El compilador elige automáticamente el mejor flujo según el código:

```
┌─────────────────────────────────────────────────────────────┐
│  ADead Source (.ad)                                         │
└─────────────────────────────────────────────────────────────┘
                          ↓
        ┌─────────────────┴─────────────────┐
        │  Análisis del código fuente       │
        └─────────────────┬─────────────────┘
                          ↓
    ┌─────────────────────┼─────────────────────┐
    │                     │                     │
    ↓                     ↓                     ↓
┌─────────┐        ┌──────────┐        ┌──────────┐
│ Complejo│        │ Simple   │        │ Crítico  │
│ (loops, │        │ (floats, │        │ (necesita│
│  ifs)   │        │ expres.) │        │  optim.) │
└────┬────┘        └────┬─────┘        └────┬─────┘
     │                  │                    │
     ↓                  ↓                    ↓
┌─────────┐        ┌──────────┐        ┌──────────┐
│Tree-    │        │ Zig →    │        │ D → NASM │
│sitter   │        │ NASM     │        │ (directo)│
└────┬────┘        └────┬─────┘        └────┬─────┘
     │                  │                    │
     ↓                  ↓                    ↓
┌─────────┐        ┌──────────┐        ┌──────────┐
│ D + Rust│        │ Rust     │        │ Rust     │
│(meta+   │        │(valid.)  │        │(final)   │
│ valid.) │        └────┬─────┘        └────┬─────┘
└────┬────┘             │                    │
     │                  │                    │
     └──────────────────┴────────────────────┘
                        ↓
              ┌─────────────────────┐
              │  NASM (ASM puro)    │
              │  • Código assembly  │
              │  • Optimizado       │
              └─────────────────────┘
                        ↓
              ┌─────────────────────┐
              │  Ejecutable (.exe)  │
              └─────────────────────┘
                        ↓
              ⚡ CPU Directo ⚡
```

**Lógica de Selección Inteligente:**
1. **Análisis del código:** Detecta características (while, if, anidamiento, variables, expresiones, complejidad)
2. **Selección automática según características:**
   - Código simple → Zig → NASM directo (máxima eficiencia)
   - While loops → Zig → NASM directo (optimizado para loops)
   - Estructuras anidadas → Tree-sitter → Rust → NASM (parsing robusto)
   - Expresiones con variables → Zig → Rust → NASM (validación necesaria)
   - Proyectos grandes → D → Tree-sitter → Rust → NASM (máxima potencia)
3. **Fallback automático:** Si un pipeline falla, intenta automáticamente otros en orden de robustez
4. **Último recurso:** Rust directo (compatibilidad total)

**Ver documentación completa:** [docs/FLUJO-COMPLETO.md](docs/FLUJO-COMPLETO.md)

## ✨ ¿Por qué ADead?

**La promesa:** Sintaxis fácil estilo Python → ASM puro → CPU directo, **sin runtime bloat**

ADead es un lenguaje de programación que combina la simplicidad de Python con el rendimiento nativo de Assembly. No es un "toy language" - es un **lenguaje serio low-level** que democratiza la programación a nivel CPU.

### 🎯 Lo que YA TIENE ADead (MVP Sólido)

**ADead ya tiene una base impresionante para ser considerado más que un MVP básico:**

#### ✅ Características Core Completas
- ✅ **Sintaxis limpia estilo Python** - `print`, `let`, `if/else`, `while`, `fn`
- ✅ **Variables y aritmética** - Enteros con operadores básicos
- ✅ **Tipos de datos nativos** - Enteros, Floats, Bool (`true`/`false`)
- ✅ **Estructuras de control** - Condicionales (`if/else`) y loops (`while`)
- ✅ **Funciones** - Parámetros, `return`, llamadas de función
- ✅ **OOP Completo** - Structs, métodos, `init`/`destroy` (RAII), encapsulación (`pub`/`private`)
- ✅ **Floats completos** - ✅ **IMPLEMENTADO Y VERIFICADO** (Diciembre 2025)
  - ✅ Literales float (`3.14`, `.5`, `5.`)
  - ✅ Operaciones aritméticas: `+`, `-`, `*`, `/`
  - ✅ Evaluación compile-time de expresiones float
  - ✅ Formateo inteligente (versión optimizada y precisa)
  - ✅ Precisión Float64 verificada (~15-17 dígitos decimales)

#### ✅ Arquitectura Técnica Sólida
- ✅ **Arquitectura Cuádruple: Tree-sitter + Zig + D Language + Rust** - Parsing robusto + eficiente + metaprogramming + seguro
  - **🌳 Tree-sitter:** Parsing robusto de estructuras complejas (while/if anidados)
  - **⚡ Zig:** Parsing eficiente y generación directa a ASM para casos simples
  - **🔷 D Language:** Metaprogramming avanzado, CTFE, templates, generación ASM optimizada ⭐ **NUEVO**
  - **🔒 Rust:** Validación de memoria, type checking, seguridad y codegen NASM
- ✅ **Flujos múltiples inteligentes** - Selección automática del mejor parser según el código
- ✅ **Generación NASM x86_64** - Código ASM puro → CPU directo
- ✅ **CLI modular profesional** - `compile`, `assemble`, `link`, `run`
- ✅ **Floats completamente funcionales** - Literales, expresiones, operaciones verificadas

#### ✅ Experiencia de Usuario
- ✅ **Ejemplos funcionales reales** - Hello world, factorial, conditional, loops, structs, RAII
- ✅ **Icono personalizado `.ad`** - Identidad visual en Windows Explorer
- ✅ **Compilación completa** - De `.ad` a `.exe` ejecutable

**🎉 Ya es más que muchos lenguajes hobby - tienes MVP funcional con OOP y RAII, que pocos logran tan rápido.**

### 🎯 Lo que FALTA para ser "Lenguaje Completo y Profesional"

Para que ADead sea considerado una alternativa seria low-level (tipo Zig/Rust pero más fácil), necesita:

#### 🔴 Críticos (Prioridad 1 - Sprint 1)
- [x] **Floats completos** - ✅ **COMPLETADO** (Diciembre 2025)
  - ✅ Aritmética completa (`+`, `-`, `*`, `/`)
  - ✅ Print de literales y expresiones
  - ✅ Evaluación compile-time
  - ✅ Precisión Float64 verificada
  - ⏳ Variables con floats (debería funcionar, necesita testing)
- [x] **Bool nativo** - ✅ **COMPLETADO** (Diciembre 2025)
  - ✅ Literales `true`/`false`
  - ✅ Print de booleanos
  - ✅ Branching optimizado (`cmp rax, 0`)
  - ✅ Funciona en `if`/`while` statements
- [ ] **Arrays/listas básicas** - `let arr = [1, 2, 3]`, acceso `arr[0]`, `length`, `push`/`pop`

#### 🟠 Esenciales (Prioridad 2 - Sprint 2-3)
- [ ] **Sistema de módulos e imports** - `import "std/math.ad"`, `import "mylib.ad"` (proyectos multi-archivo)
- [ ] **Strings reales** - No solo hardcoded, sino concatenación, `length`, `substr` (en `.data` o stack)
- [ ] **Librería estándar mínima** - `std.io`, `std.math`, `std.string`, `std.array`
- [ ] **Estructuras de control avanzadas** - `for` loops (`for i in 0..10`), `break`/`continue`

#### 🟡 Profesionales (Prioridad 3 - Sprint 4-6)
- [ ] **Manejo de errores** - Option/Result o panic simple con mensajes claros
- [ ] **Match/switch** - Para enums futuros y pattern matching
- [ ] **Pointers y memoria manual** - Opcional, con `unsafe` block (estilo Rust)
- [ ] **Enums y unions** - Tipos de datos avanzados
- [ ] **Generics básicos** - Comptime (estilo Zig) para reusabilidad
- [ ] **Inline ASM** - Para casos extremos de optimización
- [ ] **Optimizaciones avanzadas** - Más registros, peephole opts, flag `--release`
- [ ] **Soporte ARM64** - Para mobile/Apple Silicon

#### 🔵 Ecosistema (Futuro)
- [ ] **Package Manager** - Ecosistema distribuido de librerías
- [ ] **Interoperabilidad C/Rust** - FFI completo
- [ ] **Documentación completa** - Tutorial oficial, website, playground online
- [ ] **Pipeline optimizado** - Caching, compilación incremental

### 🗺️ Roadmap Priorizado: De MVP a Lenguaje Completo

**Sprint 1 (1-2 semanas):** ✅ Floats full ✅ + ⏳ Arrays básicos + ✅ Bool ✅  
**Sprint 2 (2-3 semanas):** Módulos/imports + Strings reales + std mínima  
**Sprint 3 (2-3 semanas):** Manejo errores + for/match + break/continue  
**Sprint 4 (3-4 semanas):** Pointers/unsafe + Enums + Generics básicos  
**Sprint 5 (3-4 semanas):** std.math completo + Optimizaciones + ARM64  

**Con estos sprints, ADead pasa de "MVP impresionante" a lenguaje serio que respeta ASM puro y envía directo al CPU, democratizando low-level como nadie.**

---

## 🚀 Quickstart

### 🔧 Compilación Rápida

**Para compilar todo (Zig + Rust) en un solo comando:**

```powershell
# Compilar todo y probar
.\build-all.ps1 -Test

# Solo compilar sin pruebas
.\build-all.ps1

# Limpiar y recompilar desde cero
.\build-all.ps1 -Clean -Test
```

El script `build-all.ps1` automatiza:
1. ✅ Compilación de Zig (`zig build-lib`)
2. ✅ Generación de `adead_zig.lib`
3. ✅ Generación de parser Tree-sitter (`tree-sitter generate`)
4. ✅ Compilación de Rust con linking correcto
5. ✅ Prueba del flujo completo (opcional con `-Test`)

### Requisitos

**Linux (recomendado):**
- Rust (última versión estable)
- NASM (`nasm` en PATH)
- binutils (`ld` en PATH)

**Windows:**
- Rust (última versión estable)
- Zig (última versión estable) - Para parsing eficiente
- Node.js (última versión LTS) - Para Tree-sitter
- NASM (`nasm` en PATH)
- MinGW/MSYS2 con `gcc` o binutils con `ld`
- ⚠️ **Nota importante:** El código generado usa syscalls de Linux. Para ejecutar en Windows necesitas:
  - WSL (Windows Subsystem for Linux) - **Recomendado**
  - O usar herramientas de Linux (MSYS2 puede funcionar con algunas limitaciones)

**Instalación rápida de Tree-sitter (una vez):**
```powershell
# Instalar tree-sitter CLI globalmente
npm install -g tree-sitter-cli

# Verificar instalación
tree-sitter --version
```

### Instalación

```bash
# Clonar el repo
git clone https://github.com/tuusuario/adead.git
cd adead

# Compilar
cargo build --release

# El binario estará en: target/release/adeadc
```

### Uso

#### 🎨 Icono Personalizado (Windows)

Los archivos `.ad` incluyen un icono personalizado en Windows Explorer. El icono se aplica automáticamente al instalar o mediante scripts de configuración.

#### 🚀 Método Simple: Un Solo Comando

**Desde cualquier lugar:**
```powershell
# Windows - Genera .exe y ejecuta automáticamente
.\target\release\adeadc.exe run Ejemplos-Reales\hello.ad

# Linux/Mac - Genera ejecutable y ejecuta
./target/release/adeadc run examples/hello.ad
```

El comando `run` automáticamente:
1. ✅ Compila el `.ad` a `.asm`
2. ✅ Ensambla a `.obj` (Windows) o `.o` (Linux)
3. ✅ Enlaza a `.exe` (Windows) o ejecutable (Linux)
4. ✅ Ejecuta el programa
5. ✅ Limpia archivos temporales (a menos que uses `--keep-temp`)

**El `.exe` se genera en la misma carpeta que el archivo `.ad`**

#### Opciones Avanzadas

```powershell
# Mantener archivos temporales para debugging
.\target\release\adeadc.exe run Ejemplos-Reales\hello.ad --keep-temp

# Solo compilar a ASM (sin ejecutar)
.\target\release\adeadc.exe compile Ejemplos-Reales\hello.ad -o hello.asm
```

#### Ejemplos Reales

Los ejemplos funcionales están en la carpeta `Ejemplos-Reales/`:

```powershell
# Ejecutar desde la raíz
.\run.ps1 Ejemplos-Reales\hello.ad

# O desde la carpeta Ejemplos-Reales
cd Ejemplos-Reales
.\ejecutar.ps1 hello.ad
```

Ver [Ejemplos-Reales/README.md](Ejemplos-Reales/README.md) para más detalles.

## 📝 Ejemplos

### Hello World

```adead
print "Hola Mundo"
```

### Variables y Aritmética

```adead
let x = 10
let y = 20
let sum = x + y
```

### Condicionales

```adead
if x > 5 {
    print "x is greater than 5"
} else {
    print "x is less than or equal to 5"
}
```

### Booleanos

```adead
print true
print false

if true {
    print "yes"
} else {
    print "no"
}
```

### Loops

```adead
let i = 0
while i < 10 {
    print "Iteration: "
    print i
    i = i + 1
}
```

### Funciones

```adead
fn add(a, b) {
    return a + b
}

let result = add(5, 3)
```

## 🏗️ Arquitectura

### Arquitectura Triple: Zig + Tree-sitter + Rust

**Filosofía:** Cada herramienta hace lo que mejor sabe - trabajan juntos o independientemente según lo necesario

- **🌳 Tree-sitter:** Parser generator especializado en parsing robusto de estructuras complejas
  - Maneja bloques anidados perfectamente (while con if dentro)
  - Error recovery automático
  - Incremental parsing (preparado para LSP futuro)
  - Usado por VS Code, GitHub, Atom
  
- **⚡ Zig:** Parsing eficiente y generación directa a ASM
  - Máximo rendimiento para casos simples
  - Generación directa de NASM sin overhead
  - Parsing de expresiones aritméticas rápido
  
- **🔒 Rust:** Seguridad, validación y codegen robusto
  - Validación de memoria (borrow checking)
  - Type checking y validación completa
  - Generación de código NASM optimizado
  - Parser de fallback (Chumsky) para compatibilidad total

### Proceso de Compilación Completo: De Sintaxis a CPU

**ADead utiliza múltiples flujos que trabajan solos, en parejas, tríos o todos juntos según las necesidades, hasta generar código ASM puro para la CPU:**

```
╔═══════════════════════════════════════════════════════════════════════╗
║                    ARQUITECTURA CUÁDRUPLE                             ║
║            Tree-sitter + Zig + D Language + Rust                      ║
╚═══════════════════════════════════════════════════════════════════════╝

┌─────────────────────────────────────────────────────────────────────┐
│  ENTRADA: ADead Source (.ad)                                        │
│  • Sintaxis estilo Python                                           │
│  • Simple y legible                                                 │
└─────────────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────────────┐
│  FASE 1: SELECCIÓN DEL FLUJO OPTIMAL                                │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐          │
│  │ Complejo │  │ Simple   │  │ Crítico  │  │ Fallback │          │
│  │ (loops)  │  │ (floats) │  │ (optim.) │  │          │          │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘          │
└─────────────────────────────────────────────────────────────────────┘
                          ↓
        ┌─────────────────┴─────────────────┐
        │  Múltiples flujos disponibles     │
        └─────────────────┬─────────────────┘
                          ↓
    ═══════════════════════════════════════════════════════════════
    FLUJO 1: SOLO (Trabaja Independiente)
    ═══════════════════════════════════════════════════════════════
    
    ┌─────────────┐      ┌─────────────┐      ┌─────────────┐
    │ Tree-sitter │  OR  │    Zig      │  OR  │ D Language  │  OR  │  Rust  │
    │   (solo)    │      │   (solo)    │      │   (solo)    │      │ (solo) │
    └──────┬──────┘      └──────┬──────┘      └──────┬──────┘      └───┬───┘
           │                    │                     │                 │
           └────────────────────┴─────────────────────┴─────────────────┘
                                 │
                                 ↓
                    ┌─────────────────────┐
                    │  NASM (ASM puro)    │
                    └─────────────────────┘
                                 │
                                 ↓
                    ⚡ CPU Directo ⚡
    
    ═══════════════════════════════════════════════════════════════
    FLUJO 2: PAREJAS (Cooperación de 2 lenguajes)
    ═══════════════════════════════════════════════════════════════
    
    ┌──────────────┐      ┌──────────────┐      ┌──────────────┐
    │Tree-sitter → │      │   Zig →      │      │   D →        │
    │    Rust      │  OR  │    Rust      │  OR  │    Rust      │
    └──────┬───────┘      └──────┬───────┘      └──────┬───────┘
           │                     │                      │
           ↓                     ↓                      ↓
    ┌──────────────┐      ┌──────────────┐      ┌──────────────┐
    │   Rust       │      │   Rust       │      │   Rust       │
    │ (codegen)    │      │ (validación) │      │ (seguridad)  │
    └──────┬───────┘      └──────┬───────┘      └──────┬───────┘
           │                     │                      │
           └─────────────────────┴──────────────────────┘
                                 │
                                 ↓
                    ┌─────────────────────┐
                    │  NASM (ASM puro)    │
                    └─────────────────────┘
                                 │
                                 ↓
                    ⚡ CPU Directo ⚡
    
    ═══════════════════════════════════════════════════════════════
    FLUJO 3: TRÍOS (Cooperación de 3 lenguajes) ⭐ MÁXIMA POTENCIA
    ═══════════════════════════════════════════════════════════════
    
    ┌──────────────┐
    │Tree-sitter   │ → Parsing robusto
    └──────┬───────┘
           │ (AST)
           ↓
    ┌──────────────┐
    │ D Language   │ → Metaprogramming + CTFE + Optimización
    └──────┬───────┘
           │ (ASM mejorado)
           ↓
    ┌──────────────┐
    │   Rust       │ → Validación final + Codegen NASM
    └──────┬───────┘
           │
           ↓
    ┌──────────────┐
    │  NASM (ASM)  │ → Código Assembly puro optimizado
    └──────┬───────┘
           │
           ↓
    ┌──────────────┐
    │ Ejecutable   │ → .exe listo para ejecutar
    └──────┬───────┘
           │
           ↓
    ⚡ CPU Directo ⚡ → Máxima Performance

    ═══════════════════════════════════════════════════════════════
    FLUJO 4: CUÁDRUPLE (Todos juntos) 🚀 MÁXIMA ROBUSTEZ
    ═══════════════════════════════════════════════════════════════
    
    ┌──────────────┐
    │Tree-sitter   │ → Parsing robusto (estructuras complejas)
    └──────┬───────┘
           │
           ↓
    ┌──────────────┐      ┌──────────────┐
    │    Zig       │ ───→ │ D Language   │ → Parsing eficiente +
    │ (complementa)│      │ (metaprog)   │   metaprogramming
    └──────────────┘      └──────┬───────┘
                                 │
                                 ↓
    ┌───────────────────────────────────┐
    │         Rust                      │
    │  • Validación memoria             │
    │  • Type checking                  │
    │  • Codegen NASM final             │
    └───────────────┬───────────────────┘
                    │
                    ↓
    ┌───────────────────────────────────┐
    │    NASM (Assembly x86_64)        │
    │    • Código ASM puro             │
    │    • Optimizado por D            │
    │    • Validado por Rust           │
    └───────────────┬───────────────────┘
                    │
                    ↓
    ┌───────────────────────────────────┐
    │    Ejecutable (.exe)              │
    │    • Sin dependencias             │
    │    • Performance nativa           │
    └───────────────┬───────────────────┘
                    │
                    ↓
            ⚡ CPU Directo ⚡
            (Rendimiento máximo)
```

**Compatibilidad y Modos de Trabajo:**

| Combinación | Lenguajes | Cuándo se usa | Ventajas |
|-------------|-----------|---------------|----------|
| **Solo** | Tree-sitter | Estructuras complejas | Parsing robusto |
| **Solo** | Zig | Expresiones simples | Máxima eficiencia |
| **Solo** | D Language | Metaprogramming | CTFE + Templates |
| **Solo** | Rust | Fallback | Compatibilidad total |
| **Pareja** | Tree-sitter + Rust | Estructuras + seguridad | Robusto + seguro |
| **Pareja** | Zig + Rust | Eficiencia + seguridad | Rápido + seguro |
| **Pareja** | D + Rust | Metaprogramming + seguridad | Potente + seguro |
| **Trío** | Tree + D + Rust | Máxima potencia | Robusto + potente + seguro |
| **Cuádruple** | Todos juntos | Proyectos críticos | Todo lo anterior |

**Ventajas de la Arquitectura Cuádruple:**
- ✅ **Flexibilidad:** Cada lenguaje puede trabajar solo o combinado
- ✅ **Robustez:** Múltiples fallbacks si algo falla
- ✅ **Performance:** Selección automática del flujo más eficiente
- ✅ **Seguridad:** Validación en múltiples capas
- ✅ **ASM Puro:** Código assembly directo para la CPU, sin overhead

#### 🌳 Flujo 1: Tree-sitter → Rust → NASM (Parsing Robusto)
**Para estructuras complejas y programas grandes:**
```
ADead Source: while x <= limite { if x % 10 == 0 { print x } }
  ↓
┌─────────────────────────────────────────┐
│  TREE-SITTER (parsing robusto)         │
│  • Maneja bloques anidados perfectamente│
│  • Error recovery automático            │
│  • Incremental parsing                  │
│  • Genera AST Tree-sitter               │
└─────────────────────────────────────────┘
  ↓ (AST Tree-sitter)
┌─────────────────────────────────────────┐
│  RUST (conversión + validación)        │
│  • Convertir AST Tree-sitter → AST Rust│
│  • Validación de memoria (borrow checker)│
│  • Type checking y validación           │
│  • Code Generator → NASM                │
└─────────────────────────────────────────┘
  ↓
┌─────────────────────────────────────────┐
│  NASM (Assembly x86_64)                │
│  • Generación de código assembly       │
└─────────────────────────────────────────┘
  ↓
✅ Ejecutable (.exe)
```

#### ⚡ Flujo 2: Zig → NASM Directo (Máxima Eficiencia)
**Para expresiones simples y floats:**
```
ADead Source: print 3.14
  ↓
┌─────────────────────────────────────────┐
│  ZIG (parsea y genera ASM directamente)│
│  • Parsea: readFloat() → 3.14          │
│  • Genera NASM directamente            │
│  • Crea .data section: float_0: dq 3.14│
│  • Genera .text: movsd xmm0, [rel ...] │
│  • FFI: generate_nasm_ffi()            │
└─────────────────────────────────────────┘
  ↓ (Código NASM completo)
┌─────────────────────────────────────────┐
│  NASM (Assembly x86_64)                │
│  • Ensamblado directo                  │
└─────────────────────────────────────────┘
  ↓
✅ Ejecutable (.exe)
```

#### 🔒 Flujo 3: Zig → Rust → NASM (Eficiente + Seguro)
**Para código que requiere validación:**
```
ADead Source: let x = 2 + 5
  ↓
┌─────────────────────────────────────────┐
│  ZIG PARSER (parsea expresiones)       │
│  • Expresiones aritméticas (2 + 5)      │
│  • Operadores con precedencia correcta  │
│  • FFI: parse_expr_ffi()                │
└─────────────────────────────────────────┘
  ↓ (Serialización: "BINOP:ADD:NUMBER:2:NUMBER:5")
┌─────────────────────────────────────────┐
│  RUST (seguridad de memoria)            │
│  • Wrapper FFI: parse_expr_with_zig()  │
│  • Conversión a AST Rust (Expr)         │
│  • Validación de memoria (borrow checker)│
│  • Type checking y validación           │
│  • Code Generator → NASM                │
└─────────────────────────────────────────┘
  ↓
┌─────────────────────────────────────────┐
│  NASM (Assembly x86_64)                │
│  • Generación de código assembly       │
└─────────────────────────────────────────┘
  ↓
✅ Ejecutable (.exe)
```

#### 🛠️ Flujo 4: Rust Directo → NASM (Fallback)
**Para casos especiales cuando otros fallan:**
```
ADead Source: (cualquier código complejo)
  ↓
┌─────────────────────────────────────────┐
│  RUST (parser Chumsky completo)        │
│  • Parser completo en Rust             │
│  • Validación completa                 │
│  • Code Generator → NASM                │
└─────────────────────────────────────────┘
  ↓
┌─────────────────────────────────────────┐
│  NASM (Assembly x86_64)                │
└─────────────────────────────────────────┘
  ↓
✅ Ejecutable (.exe)
```

**Selección Automática Inteligente:**
El compilador prueba los flujos en orden de robustez y potencia:
1. **🌳 Tree-sitter + 🔷 D + 🔒 Rust** (máxima potencia) - Para proyectos grandes y críticos
2. **🌳 Tree-sitter + 🔒 Rust** (robusto + seguro) - Para estructuras complejas
3. **🔷 D + 🔒 Rust** (metaprogramming + seguridad) - Para código que necesita optimización
4. **⚡ Zig + 🔒 Rust** (eficiente + seguro) - Para código con validación
5. **⚡ Zig directo** (máximo rendimiento) - Para casos simples
6. **🔷 D directo** (metaprogramming puro) - Para generación ASM avanzada
7. **🛠️ Rust directo** (fallback) - Último recurso

**Ejemplo Práctico - Estructura Compleja:**
```adead
while suma <= limite {
    if suma % intervalo == 0 {
        print suma
    }
    suma = suma + 1
}
```

**Proceso:**
1. **Tree-sitter parsea:** Maneja bloques anidados perfectamente → AST Tree-sitter
2. **Rust convierte:** Tree-sitter AST → AST Rust
3. **Rust valida:** Borrow checker, type checking
4. **Rust genera NASM:** Código assembly con loops y condiciones
5. **NASM compila:** Genera `.obj` → Linker → `.exe`

**Ventajas de la Arquitectura Inteligente Cuádruple:**
- ✅ **🤖 Pipeline Selector Inteligente:** Analiza código automáticamente y selecciona el mejor pipeline
- ✅ **🌳 Tree-sitter:** Parsing robusto de estructuras complejas (while/if anidados) con error recovery
- ✅ **⚡ Zig:** Máxima eficiencia para casos simples y while loops optimizados (sin overhead)
- ✅ **🔷 D Language:** Metaprogramming avanzado, CTFE, templates, generación ASM optimizada ⭐ **NUEVO**
- ✅ **🔒 Rust:** Seguridad garantizada y validación completa
- ✅ **🛠️ Fallback Inteligente:** Sistema automático de fallbacks si un pipeline falla
- ✅ **Selección automática optimizada:** El compilador analiza y elige el mejor flujo automáticamente
- ✅ **Optimizaciones:** Limpieza de código ASM redundante, formato consistente
- ✅ **Rendimiento nativo:** Ejecutable final sin dependencias
- ✅ **Flexibilidad máxima:** Cada herramienta trabaja sola, en parejas, tríos o todos juntos
- ✅ **ASM Puro y Limpio:** Código assembly optimizado directo para la CPU, máximo rendimiento

### Comandos Modulares

Puedes ejecutar cada paso por separado para mayor control:

```powershell
# 1. Compilar a Assembly
.\target\release\adeadc.exe compile Ejemplos-Reales\hello.ad

# 2. Ensamblar a objeto
.\target\release\adeadc.exe assemble Ejemplos-Reales\hello.asm

# 3. Enlazar a ejecutable
.\target\release\adeadc.exe link Ejemplos-Reales\hello.obj

# 4. Ejecutar
.\target\release\adeadc.exe run Ejemplos-Reales\hello.exe

# O todo en uno:
.\target\release\adeadc.exe run Ejemplos-Reales\hello.ad
```

## 📚 Documentación

### Guías Principales
- [Comandos Fáciles](Fácil_Comando.md) - ⚡ Guía rápida para empezar
- [Roadmap Profesional](docs/ROADMAP-PROFESIONAL.md) - 🎯 Plan completo para hacer ADead profesional

### Documentación Técnica
- [Arquitectura](docs/ARQUITECTURA.md) - Zig + Rust integrados
- [Flujo de Compilación](docs/FLUJO-COMPILACION.md) - Proceso completo
- [Ideas OOP](ideas3.md) - Programación Orientada a Objetos
- [Ideas Futuras](ideas2.md) - Roadmap e ideas de desarrollo

### Referencias
- [Gramática](docs/grammar.md)
- [Tutorial](docs/tutorial.md)
- [Diseño Técnico](docs/design.md)
- [Uso Rápido](USO-RAPIDO.md)
- [Contribuir](CONTRIBUTING.md)
- [Autores](AUTHORS.md)
- [Changelog](CHANGELOG.md)

## 🛠️ Estado del Proyecto

### ✅ MVP Funcional (Completado)

**ADead ya tiene una base sólida que supera a muchos lenguajes hobby:**

#### Características Core
- ✅ **Parser completo:** Zig + Rust integrados
  - **Zig:** Parsea expresiones aritméticas y structs complejos de forma eficiente
  - **Rust:** Seguridad de memoria (borrow checker), validación y generación de código NASM
- ✅ **Sintaxis completa:** `print`, `let`, `if/else`, `while`, `fn` con parámetros y `return`
- ✅ **OOP completo:** Structs, métodos, `init`/`destroy` (RAII), encapsulación (`pub`/`private`)
- ✅ **Floats completos:** ✅ **IMPLEMENTADO Y VERIFICADO** (Diciembre 2025)
  - ✅ Literales float (`3.14`, `.5`, `5.`)
  - ✅ Operaciones: suma, resta, multiplicación, división
  - ✅ Expresiones complejas (`print 3.14 + 2.5`)
  - ✅ Evaluación compile-time con precisión Float64
  - ✅ Formateo inteligente (optimizado y preciso)

#### Infraestructura Técnica
- ✅ **Generación NASM:** x86_64 para Windows/Linux funcional
- ✅ **CLI profesional:** Comandos modulares (`compile`, `assemble`, `link`, `run`)
- ✅ **Arquitectura Triple:** Tree-sitter + Zig + Rust con flujos múltiples inteligentes
- ✅ **Flujos flexibles:** Selección automática del mejor parser según complejidad del código
- ✅ **Parsing robusto:** Tree-sitter para estructuras complejas, Zig para eficiencia, Rust para seguridad

#### Experiencia de Usuario
- ✅ **Ejemplos funcionales:** Hello world, factorial, conditional, loops, structs, encapsulación, RAII
- ✅ **Icono personalizado:** Archivos `.ad` con identidad visual en Windows Explorer
- ✅ **Compilación robusta:** Funcional en Windows con MinGW/MSYS2
- ✅ **Proceso modularizado:** Mejor manejo de errores y diagnósticos

**🎉 Ya es más que un MVP básico - tienes un lenguaje funcional con OOP y RAII, que pocos logran tan rápido.**

### 🚀 En Desarrollo (Sprint 1 - Actual)

**Prioridades críticas para completar el sistema de tipos:**
- ✅ **Floats completos:** ✅ **COMPLETADO** - Aritmética full + print + evaluación compile-time verificada
- 🔄 **Bool nativo:** `true`/`false` con branching optimizado
- 🔄 **Arrays básicos:** Declaración, acceso por índice, operaciones básicas
- 🔄 **Variables con floats:** Testing y validación completa

📖 **Ver sección [Roadmap](#-roadmap-de-mvp-a-lenguaje-completo) para el plan completo de desarrollo.**

## 🧪 Testing

```bash
cargo test --workspace
```

## 🤝 Contribuir

¡Las contribuciones son bienvenidas! Por favor lee [CONTRIBUTING.md](CONTRIBUTING.md) para más detalles.

## 👨‍💻 Autor

**Eddi Andreé Salazar Matos**

- Proyecto iniciado: 11 de Diciembre de 2025
- ⚡ Lenguaje .ad - Simple y poderoso

Para más información, ver [AUTHORS.md](AUTHORS.md)

## 📄 Licencia

MIT License - ver [LICENSE](LICENSE) para más detalles.

Copyright (c) 2025 Eddi Andreé Salazar Matos

## 🎯 Roadmap: De MVP a Lenguaje Completo

### ✅ Completado (MVP Funcional)
1. ✅ **Sintaxis Core**: `print`, `let`, `if/else`, `while`, `fn` + tests
2. ✅ **OOP Completo**: Structs, métodos, `init`/`destroy` (RAII), encapsulación (`pub`/`private`)
3. ✅ **Arquitectura Triple**: Zig + Tree-sitter + Rust trabajando juntos, solos o independientemente
4. ✅ **Flujos múltiples**: Tree-sitter → Rust, Zig → Rust, Zig directo, Rust directo funcionando
5. ✅ **CLI profesional**: Comandos modulares (`compile`, `assemble`, `link`, `run`)
6. ✅ **Floats completos**: ✅ **IMPLEMENTADO Y VERIFICADO** (Diciembre 2025)
   - ✅ Literales float, operaciones aritméticas completas
   - ✅ Expresiones complejas con evaluación compile-time
   - ✅ Precisión Float64 verificada (~15-17 dígitos decimales)
   - ✅ Formateo inteligente (optimizado y preciso)
7. ✅ **Bool nativo**: ✅ **IMPLEMENTADO Y VERIFICADO** (Diciembre 2025)
   - ✅ Literales `true`/`false`
   - ✅ Print de booleanos
   - ✅ Branching optimizado en assembly (`cmp rax, 0`)
   - ✅ Funciona en estructuras de control (`if`/`while`)

### 🔴 Sprint 1: Tipos de Datos Completos (1-2 semanas) - CRÍTICO
1. ✅ **Floats completos**: ✅ **COMPLETADO** (Diciembre 2025)
   - ✅ Aritmética full (`+`, `-`, `*`, `/`)
   - ✅ Print de literales y expresiones con evaluación compile-time
   - ✅ Precisión Float64 verificada y funcionando
   - ⏳ Variables con floats (testing pendiente)
2. ✅ **Bool nativo**: ✅ **COMPLETADO** (Diciembre 2025)
   - ✅ Literales `true`/`false`
   - ✅ Print de booleanos
   - ✅ Branching optimizado (`cmp rax, 0`)
   - ✅ Funciona en `if`/`while` statements
3. 🔴 **Arrays básicos**: `let arr = [1, 2, 3]`, acceso `arr[0]`, `length`, `push`/`pop`

### 🟠 Sprint 2-3: Módulos y Librería Estándar (2-3 semanas) - ESENCIAL
1. 🟠 **Sistema de módulos**: `import "std/math.ad"`, `import "mylib.ad"` (proyectos multi-archivo)
2. 🟠 **Strings reales**: Concatenación, `length`, `substr` (en `.data` o stack)
3. 🟠 **Librería estándar mínima**: `std.io` (print, read_line), `std.math` (sin, cos, pow, sqrt), `std.string`, `std.array`
4. 🟠 **Estructuras avanzadas**: `for` loops (`for i in 0..10`), `break`/`continue`

### 🟡 Sprint 4-6: Características Profesionales (3-4 semanas cada uno) - AVANZADO
1. 🟡 **Manejo de errores**: Option/Result funcionales o panic simple con mensajes claros
2. 🟡 **Match/switch**: Pattern matching para enums y control flow avanzado
3. 🟡 **Pointers y memoria manual**: Opcional, con `unsafe` block (estilo Rust)
4. 🟡 **Enums y unions**: Tipos de datos avanzados
5. 🟡 **Generics básicos**: Comptime (estilo Zig) para reusabilidad
6. 🟡 **Inline ASM**: Para casos extremos de optimización
7. 🟡 **Optimizaciones avanzadas**: Más registros, peephole opts, flag `--release` con optimizaciones agresivas
8. 🟡 **Soporte ARM64**: Para mobile/Apple Silicon

### 🔵 Futuro: Ecosistema Completo
1. 🔵 **Package Manager**: Ecosistema distribuido de librerías
2. 🔵 **Interoperabilidad C/Rust**: FFI completo y robusto
3. 🔵 **Documentación completa**: Tutorial oficial, website, playground online
4. 🔵 **Pipeline optimizado**: Caching inteligente, compilación incremental

📖 **Ver [docs/ROADMAP-PROFESIONAL.md](docs/ROADMAP-PROFESIONAL.md) para el plan detallado de 6 meses.**

**🎯 Objetivo Final:** ADead pasa de "MVP impresionante" a **lenguaje serio que respeta ASM puro y envía directo al CPU**, democratizando low-level como nadie. ⚡

---

<div align="center">

**Hecho con ❤️ en 4 lenguajes: Tree-sitter + Zig + D Language + Rust = "Adead" por Eddi Andreé Salazar Matos**

⚡ *ADead - Simple syntax, powerful performance* ⚡

**Arquitectura Inteligente Cuádruple:** Selección automática del mejor pipeline

*11 de Diciembre de 2025*

</div>

