<div align="center">

# 🇵🇪 .ad — ADead

**ASM is dead (but powerful)**

Simple sintaxis estilo Python • Rendimiento nativo

🎨 **Icono personalizado para archivos `.ad`** - Identidad visual única en Windows

**Desarrollado por:** Eddi Andreé Salazar Matos  
**Fecha:** 11 de Diciembre de 2025

</div>

## 🔄 Flujos de Compilación Establecidos

**ADead soporta múltiples flujos de compilación según la complejidad:**

### 🚀 Flujo Directo: ADead → Zig → NASM (ASM)
**Para casos simples** (floats, expresiones aritméticas básicas):
```
ADead Source (.ad)
  ↓
Zig (parsea y genera ASM directamente)
  ↓
NASM (Assembly x86_64)
  ↓
Ejecutable (.exe)
```
**Ventajas:** Máxima eficiencia, sin overhead de validación
**Uso:** Floats simples, expresiones aritméticas puras

### 🔒 Flujo con Validación: ADead → Zig → Rust → NASM (ASM)
**Para casos complejos** (requiere validación, type checking, seguridad):
```
ADead Source (.ad)
  ↓
Zig (parsea expresiones)
  ↓
Rust (validación de memoria, type checking, seguridad)
  ↓
NASM (Assembly x86_64)
  ↓
Ejecutable (.exe)
```
**Ventajas:** Seguridad garantizada, validación completa
**Uso:** Variables, funciones, structs, OOP, validaciones complejas

### 🛠️ Flujo Rust Directo: ADead → Rust → NASM (ASM)
**Para casos especiales** (cuando Zig no puede parsear):
```
ADead Source (.ad)
  ↓
Rust (parser completo + validación)
  ↓
NASM (Assembly x86_64)
  ↓
Ejecutable (.exe)
```
**Ventajas:** Parser completo en Rust, fallback robusto
**Uso:** Sintaxis compleja, casos edge, fallback

**Ver documentación completa:** [docs/FLUJO-COMPLETO.md](docs/FLUJO-COMPLETO.md)

## ✨ ¿Por qué ADead?

**La promesa:** Sintaxis fácil estilo Python → ASM puro → CPU directo, **sin runtime bloat**

ADead es un lenguaje de programación que combina la simplicidad de Python con el rendimiento nativo de Assembly. No es un "toy language" - es un **lenguaje serio low-level** que democratiza la programación a nivel CPU.

### 🎯 Lo que YA TIENE ADead (MVP Sólido)

**ADead ya tiene una base impresionante para ser considerado más que un MVP básico:**

#### ✅ Características Core Completas
- ✅ **Sintaxis limpia estilo Python** - `print`, `let`, `if/else`, `while`, `fn`
- ✅ **Variables y aritmética** - Enteros con operadores básicos
- ✅ **Estructuras de control** - Condicionales (`if/else`) y loops (`while`)
- ✅ **Funciones** - Parámetros, `return`, llamadas de función
- ✅ **OOP Completo** - Structs, métodos, `init`/`destroy` (RAII), encapsulación (`pub`/`private`)

#### ✅ Arquitectura Técnica Sólida
- ✅ **Arquitectura Híbrida Zig + Rust** - Parsing eficiente + seguridad de memoria
- ✅ **Flujos múltiples inteligentes** - Directo (Zig→NASM) para simple, con Rust para complejo
- ✅ **Generación NASM x86_64** - Funcional en Windows/Linux
- ✅ **CLI modular profesional** - `compile`, `assemble`, `link`, `run`
- ✅ **Floats en proceso** - Ya parsea `3.14` y valores flotantes

#### ✅ Experiencia de Usuario
- ✅ **Ejemplos funcionales reales** - Hello world, factorial, conditional, loops, structs, RAII
- ✅ **Icono personalizado `.ad`** - Identidad visual en Windows Explorer
- ✅ **Compilación completa** - De `.ad` a `.exe` ejecutable

**🎉 Ya es más que muchos lenguajes hobby - tienes MVP funcional con OOP y RAII, que pocos logran tan rápido.**

### 🎯 Lo que FALTA para ser "Lenguaje Completo y Profesional"

Para que ADead sea considerado una alternativa seria low-level (tipo Zig/Rust pero más fácil), necesita:

#### 🔴 Críticos (Prioridad 1 - Sprint 1)
- [ ] **Floats completos** - Aritmética full, print real (conversión float→string en ASM)
- [ ] **Bool nativo** - `true`/`false` con branching optimizado
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

**Sprint 1 (1-2 semanas):** Floats full + Arrays básicos + Bool  
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
3. ✅ Compilación de Rust con linking correcto
4. ✅ Prueba del flujo completo (opcional con `-Test`)

### Requisitos

**Linux (recomendado):**
- Rust (última versión estable)
- NASM (`nasm` en PATH)
- binutils (`ld` en PATH)

**Windows:**
- Rust (última versión estable)
- NASM (`nasm` en PATH)
- MinGW/MSYS2 con `gcc` o binutils con `ld`
- ⚠️ **Nota importante:** El código generado usa syscalls de Linux. Para ejecutar en Windows necesitas:
  - WSL (Windows Subsystem for Linux) - **Recomendado**
  - O usar herramientas de Linux (MSYS2 puede funcionar con algunas limitaciones)

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

### Arquitectura Híbrida Zig + Rust

**Filosofía:** Cada lenguaje hace lo que mejor sabe
- **Zig:** Parsing eficiente y directo (expresiones aritméticas, structs complejos) ⚡
- **Rust:** Seguridad de memoria, borrow checking, validación y generación de código NASM 🔒

### Proceso de Compilación Completo

**ADead utiliza múltiples flujos según la complejidad del código:**

#### 🚀 Flujo 1: Directo (Zig → NASM)
**Para expresiones simples y floats:**
```
ADead Source: print 3.14
  ↓
┌─────────────────────────────────────────┐
│  ZIG (parsea y genera ASM)             │
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

#### 🔒 Flujo 2: Con Validación (Zig → Rust → NASM)
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
│  • Optimizaciones de bajo nivel        │
└─────────────────────────────────────────┘
  ↓
✅ Ejecutable (.exe)
```

**Ventajas de este diseño:**
- ⚡ **Rendimiento:** Flujo directo para casos simples (sin overhead)
- 🔒 **Seguridad:** Validación Rust para código complejo
- 🎯 **Flexibilidad:** El compilador elige automáticamente el mejor flujo

**Ejemplo Práctico:**
```adead
print 2 + 5
```

**Proceso:**
1. **Zig parsea:** `"2 + 5"` → AST Zig → Serializa: `"BINOP:ADD:NUMBER:2:NUMBER:5"`
2. **Rust recibe:** FFI deserializa → `Expr::BinaryOp { op: Add, left: Number(2), right: Number(5) }`
3. **Rust valida:** Borrow checker, type checking, seguridad
4. **Rust genera NASM:** Código assembly para evaluar `2 + 5` y convertir a string
5. **NASM compila:** Genera `.obj` → Linker → `.exe`
6. **Ejecución:** Output: `7`

**Ventajas de esta Arquitectura Multi-Flujo:**
- ✅ **Zig → NASM directo:** Máxima eficiencia para floats y expresiones simples
- ✅ **Zig → Rust → NASM:** Seguridad garantizada para código complejo
- ✅ **Selección automática:** El compilador elige el mejor flujo automáticamente
- ✅ **Rendimiento nativo:** Ejecutable final sin dependencias
- ✅ **Flexibilidad:** Cada caso usa el flujo más apropiado
- ✅ **Separación clara:** Cada lenguaje hace lo que mejor sabe

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
- ✅ **Floats básicos:** Parsing de valores flotantes (`3.14`, etc.) - **en proceso de completarse**

#### Infraestructura Técnica
- ✅ **Generación NASM:** x86_64 para Windows/Linux funcional
- ✅ **CLI profesional:** Comandos modulares (`compile`, `assemble`, `link`, `run`)
- ✅ **Flujo completo:** `ADead → Zig (parsea) → Rust (seguridad) → NASM (ASM) → .exe` funcionando
- ✅ **Arquitectura híbrida:** Flujos múltiples inteligentes (directo para simple, con validación para complejo)

#### Experiencia de Usuario
- ✅ **Ejemplos funcionales:** Hello world, factorial, conditional, loops, structs, encapsulación, RAII
- ✅ **Icono personalizado:** Archivos `.ad` con identidad visual en Windows Explorer
- ✅ **Compilación robusta:** Funcional en Windows con MinGW/MSYS2
- ✅ **Proceso modularizado:** Mejor manejo de errores y diagnósticos

**🎉 Ya es más que un MVP básico - tienes un lenguaje funcional con OOP y RAII, que pocos logran tan rápido.**

### 🚀 En Desarrollo (Sprint 1 - Actual)

**Prioridades críticas para completar el sistema de tipos:**
- 🔄 **Floats completos:** Aritmética full + print real (conversión float→string)
- 🔄 **Bool nativo:** `true`/`false` con branching optimizado
- 🔄 **Arrays básicos:** Declaración, acceso por índice, operaciones básicas

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
3. ✅ **Arquitectura Híbrida**: Zig (parsea) + Rust (seguridad de memoria) integrados
4. ✅ **Flujo completo**: `ADead → Zig (parsea) → Rust (seguridad) → NASM (ASM) → .exe` funcionando
5. ✅ **CLI profesional**: Comandos modulares (`compile`, `assemble`, `link`, `run`)
6. ✅ **Floats básicos**: Parsing de valores flotantes (en proceso de completarse)

### 🔴 Sprint 1: Tipos de Datos Completos (1-2 semanas) - CRÍTICO
1. 🔴 **Floats completos**: Aritmética full, print real (conversión float→string en ASM puro)
2. 🔴 **Bool nativo**: `true`/`false` con branching optimizado en assembly
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

**Hecho con ❤️ en Rust por Eddi Andreé Salazar Matos**

⚡ *ADead - Simple syntax, powerful performance* ⚡

*11 de Diciembre de 2025*

</div>

