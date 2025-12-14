# 🚀 Opciones para Avanzar - ADead

Guía completa de qué falta por hacer y cómo seguir mejorando ADead.

## 📊 Estado Actual

**Sprint 1:** 🟢 **100% COMPLETADO** ✅
- ✅ Manejo de errores (Option/Result/Match): 100%
- ✅ Arrays básicos: 100%
- ✅ Import básico: 100% + Testing profundo

**Quick Wins:**
- ✅ Print de números: 100% COMPLETADO

**Features implementadas:** ~77% del MVP completo

**Próximo hito:** Sprint 2 - Librería Estándar

---

## 🎯 OPCIÓN 1: Quick Wins (MEJORAS RÁPIDAS - Prioridad Alta)

### 1.1 Print de Números y Expresiones Aritméticas ⚡ COMPLETADO

**Estado:** 🟢 **IMPLEMENTADO** ✅  
**Por qué:** Muy solicitado, muy simple, mejora UX inmediatamente

**Implementación:**
```rust
// En generate_stmt_windows() y generate_stmt() para print
match expr {
    Expr::Number(n) => {
        // Simplificado: convertir número a string en tiempo de compilación
        let num_str = format!("{}{}", n, "\n");
        let label = self.add_string_data(&num_str);
        // Usar WriteFile/sys_write como string normal
    }
    _ => {
        // Expresiones aritméticas: evaluar y convertir a string en runtime
        // Función helper int_to_str_runtime convierte int64 a string
        // Preserva registros según convención Windows x64 ABI
    }
}
```

**Archivos modificados:**
- ✅ `rust/crates/adead-backend/src/lib.rs` - `generate_stmt_windows()` y `generate_stmt()`
- ✅ Soporta números literales positivos y cero
- ✅ Soporta expresiones aritméticas complejas (`print 2 + 5`, `print x * y + z`)
- ✅ Windows y Linux funcionando
- ✅ Integración con Zig parser para expresiones

**Funcionalidad:**
- ✅ `print 42` - Funciona (compilación)
- ✅ `print 0` - Funciona
- ✅ `print 1234567890` - Funciona
- ✅ `print 2 + 5` - Funciona (runtime conversion) ✅ **NUEVO**
- ✅ `print x + y` - Funciona (expresiones con variables) ✅ **NUEVO**
- ✅ `print (a + b) * c` - Funciona (expresiones complejas) ✅ **NUEVO**

**Mejoras Implementadas:**
- ✅ Función helper `int_to_str_runtime` para conversión runtime
- ✅ Preservación correcta de registros (RBX, RDX, R8) según Windows x64 ABI
- ✅ Manejo correcto de stack alignment (`and rsp, -16`)
- ✅ Loop de reversión optimizado para strings numéricos
- ✅ Soporte para números negativos

**Optimizaciones Futuras Sugeridas (Ver sección 6.3):**
- 🔄 Inline de función helper para números pequeños (evitar call overhead)
- 🔄 Cachear handles de stdout/stderr (evitar llamadas repetidas a GetStdHandle)
- 🔄 Optimización para números de un solo dígito (pre-calcular strings)
- 🔄 Pool de buffers para conversiones (reutilizar memoria)

**Impacto:** 🟢 ALTO - Mejora UX inmediatamente ✅  
**Desbloquea:** Debugging más fácil, programas más informativos, expresiones en print

---

### 1.2 Operadores Lógicos (4 horas)

**Estado:** 🔴 NO IMPLEMENTADO  
**Por qué:** Necesarios para lógica compleja, muy común

**Implementación:**
```adead
// AST: Agregar a BinOp
And,    // &&
Or,     // ||
Not,    // ! (unario)

// Parser: Agregar precedencia
// Backend: Generar código NASM para operaciones booleanas
```

**Archivos a modificar:**
- `rust/crates/adead-parser/src/lib.rs` - `BinOp` enum y parser
- `rust/crates/adead-backend/src/lib.rs` - Codegen para &&, ||, !

**Impacto:** 🟡 MEDIO - Expresiones booleanas más claras  
**Desbloquea:** Lógica condicional compleja

---

### 1.3 Break y Continue (5 horas)

**Estado:** 🔴 NO IMPLEMENTADO  
**Por qué:** Control de loops esencial, sintaxis estándar

**Implementación:**
```rust
// AST: Agregar a Stmt
Break,
Continue,

// Parser: Palabras clave simples
// Backend: Jump a label de fin/inicio de loop
```

**Archivos a modificar:**
- `rust/crates/adead-parser/src/lib.rs` - `Stmt` enum y parser
- `rust/crates/adead-backend/src/lib.rs` - Manejo de labels en loops

**Impacto:** 🟡 MEDIO - Control de flujo mejorado  
**Desbloquea:** Loops más expresivos

---

### 1.4 Asignación a Array Index (6 horas)

**Estado:** 🔴 NO IMPLEMENTADO  
**Por qué:** Arrays son read-only actualmente, muy limitante

**Implementación:**
```rust
// Modificar Stmt::Assign para soportar Expr::Index
Stmt::Assign {
    target: Expr,  // Puede ser Ident o Index
    value: Expr,
}

// Backend: Calcular dirección y almacenar
```

**Archivos a modificar:**
- `rust/crates/adead-parser/src/lib.rs` - Parser de asignación
- `rust/crates/adead-backend/src/lib.rs` - Codegen para `arr[i] = valor`

**Impacto:** 🟡 MEDIO - Arrays mutables, mucho más útiles  
**Desbloquea:** Algoritmos de ordenamiento, estructuras de datos

---

### 1.5 Tipos Nativos Bool (5 horas)

**Estado:** 🔴 NO IMPLEMENTADO (actualmente bool = int64)  
**Por qué:** Claridad semántica, mejor type safety

**Implementación:**
```rust
// AST: Agregar Type::Bool
// Parser: Reconocer `bool` como tipo
// Backend: Optimizar código (usar registros de flags)
```

**Archivos a modificar:**
- `rust/crates/adead-common/src/lib.rs` - `Type` enum
- `rust/crates/adead-parser/src/lib.rs` - Parser de tipos
- `rust/crates/adead-backend/src/lib.rs` - Optimizaciones

**Impacto:** 🟡 MEDIO - Mejor type safety, código más claro  
**Desbloquea:** Validaciones de tipo más estrictas

---

## 🎯 OPCIÓN 2: Sprint 2 - Librería Estándar (ALTA PRIORIDAD)

### 2.1 Tipos Float64 y Float32 (15 horas)

**Estado:** 🔴 NO IMPLEMENTADO  
**Por qué:** CRÍTICO - Necesario para matemáticas, ciencias, gráficos

**Implementación detallada:**
```rust
// AST: Agregar
Expr::Float(f64),
Type::Float64,
Type::Float32,

// Parser: Reconocer literales 3.14, 2.5e10
// Backend: Usar registros xmm0-xmm15 (SSE/AVX)
// Operaciones: +, -, *, /, sqrt, pow
```

**Fases:**
1. **Parser (5h):** Literales float, tipos float
2. **Backend Windows (5h):** Codegen con SSE (xmm0-xmm7)
3. **Backend Linux (3h):** Codegen con System V ABI
4. **Tests (2h):** Operaciones básicas, conversiones

**Archivos:**
- `rust/crates/adead-common/src/lib.rs` - Tipos
- `rust/crates/adead-parser/src/lib.rs` - Parser
- `rust/crates/adead-backend/src/lib.rs` - Codegen SSE

**Impacto:** 🔴 CRÍTICO - Base para todo avanzado  
**Desbloquea:** Matemáticas, ciencias, gráficos, ML básico

---

### 2.2 Strings Completos (25 horas)

**Estado:** 🟡 PARCIAL (solo literales básicos)  
**Por qué:** ALTO - Necesario para casi todo

**Implementación:**

#### Fase 1: Operaciones Básicas (10h)
```adead
// Concatenación
let resultado = "hola" + " mundo"

// Longitud
let len = str.len()

// Acceso a caracteres
let char = str[0]  // Similar a arrays
```

#### Fase 2: Funciones Utiles (10h)
```adead
// Búsqueda
let pos = str.find("substring")
let contains = str.contains("text")

// Manipulación
let upper = str.to_upper()
let lower = str.to_lower()
let trimmed = str.trim()

// Conversión
let num_str = num.to_string()
let num = str.parse_int()
```

#### Fase 3: Interpolación (5h)
```adead
let nombre = "Juan"
let mensaje = "Hola {nombre}, tienes {edad} años"
```

**Archivos:**
- `rust/crates/adead-parser/src/lib.rs` - Operadores, métodos
- `rust/crates/adead-backend/src/lib.rs` - Funciones helper
- `rust/crates/adead-stdlib/` (NUEVO) - Implementación runtime

**Impacto:** 🔴 ALTO - Desbloquea procesamiento de texto, parsers, I/O  
**Dependencias:** Arrays (✅ completo)

---

### 2.3 std.math - Funciones Matemáticas (20 horas)

**Estado:** 🔴 NO IMPLEMENTADO  
**Por qué:** ALTO - Necesario para ciencias de datos, gráficos

**Funciones a implementar:**

#### Básicas (5h)
```adead
import std.math

let raiz = math.sqrt(25.0)
let potencia = math.pow(2.0, 10.0)
let absoluto = math.abs(-5.0)
let maximo = math.max(a, b)
let minimo = math.min(a, b)
```

#### Trigonométricas (8h)
```adead
let seno = math.sin(angle)
let coseno = math.cos(angle)
let tangente = math.tan(angle)
let arcoseno = math.asin(x)
let arcocoseno = math.acos(x)
let arcotangente = math.atan2(y, x)
```

#### Logarítmicas y Exponenciales (5h)
```adead
let exponencial = math.exp(x)
let logaritmo = math.log(x)  // ln
let log10 = math.log10(x)
let log2 = math.log2(x)
```

#### Constantes (2h)
```adead
let pi = math.PI
let e = math.E
```

**Implementación:**
- Usar libm (biblioteca matemática estándar C)
- FFI con funciones C: `sqrt`, `sin`, `cos`, etc.
- Wrapper en módulo `std/math.ad`

**Archivos:**
- `std/math.ad` - Wrappers ADead
- `rust/crates/adead-backend/src/lib.rs` - FFI helpers
- `docs/stdlib/MATH.md` - Documentación

**Impacto:** 🔴 ALTO - Desbloquea ciencias, gráficos, ML básico  
**Dependencias:** Float64 (2.1)

---

### 2.4 std.array - Funciones de Array (18 horas)

**Estado:** 🟡 PARCIAL (arrays básicos ✅, funciones ❌)  
**Por qué:** MEDIO-ALTO - Hace arrays realmente útiles

**Funciones a implementar:**

#### Básicas (5h)
```adead
import std.array

let longitud = array.len(arr)
let vacio = array.is_empty(arr)

// Mutación
array.push(arr, item)
let ultimo = array.pop(arr)
array.insert(arr, index, item)
array.remove(arr, index)
```

#### Funcionales (8h)
```adead
// Map: Transformar cada elemento
let cuadrados = array.map([1, 2, 3], fn(x) { return x * x })

// Filter: Filtrar elementos
let pares = array.filter([1, 2, 3, 4], fn(x) { return x % 2 == 0 })

// Reduce: Reducir a un valor
let suma = array.reduce([1, 2, 3], fn(acc, x) { return acc + x }, 0)

// ForEach: Ejecutar acción
array.forEach(arr, fn(item) { print item })
```

#### Búsqueda y Orden (5h)
```adead
let indice = array.find(arr, valor)
let contiene = array.contains(arr, valor)
let index_of = array.indexOf(arr, valor)

array.sort(arr)  // Ordenar in-place
let sorted = array.sorted(arr)  // Nueva copia ordenada
array.reverse(arr)
```

**Implementación:**
- Funciones en módulo `std/array.ad`
- Usar funciones de bajo nivel (comparaciones, swaps)
- Closures como parámetros (requiere mejoras en funciones)

**Archivos:**
- `std/array.ad` - Implementación
- `rust/crates/adead-backend/src/lib.rs` - Optimizaciones
- `docs/stdlib/ARRAY.md` - Documentación

**Impacto:** 🟡 MEDIO-ALTO - Arrays realmente prácticos  
**Dependencias:** Arrays básicos (✅), Funciones como valores (futuro)

---

## 🎯 OPCIÓN 3: Mejoras de Lenguaje

### 3.1 For Loops (10 horas)

**Estado:** 🔴 NO IMPLEMENTADO  
**Por qué:** MEDIO - Más intuitivo que while, sintaxis estándar

**Implementación:**
```adead
// For con rango
for i in 0..10 {
    print i
}

// For con array
for item in [1, 2, 3] {
    print item
}

// For con índice y valor
for (i, item) in array.enumerate() {
    print "{i}: {item}"
}
```

**Fases:**
1. **Parser (4h):** Sintaxis for, rangos `0..10`, iteradores
2. **Backend (5h):** Generar loops optimizados
3. **Tests (1h):** Varios casos de uso

**Archivos:**
- `rust/crates/adead-parser/src/lib.rs` - Parser for
- `rust/crates/adead-backend/src/lib.rs` - Codegen loops

**Impacto:** 🟡 MEDIO - Sintaxis más limpia y expresiva  
**Desbloquea:** Código más legible, patrones comunes

---

### 3.2 Closures / Funciones Anónimas (20 horas)

**Estado:** 🔴 NO IMPLEMENTADO  
**Por qué:** ALTO - Necesario para funciones de orden superior

**Implementación:**
```adead
// Funciones anónimas
let add = fn(a: int64, b: int64) -> int64 { return a + b }

// Closures (capturan variables)
let x = 10
let add_x = fn(n: int64) -> int64 { return n + x }

// Uso con arrays
let cuadrados = array.map([1, 2, 3], fn(x) { return x * x })
```

**Fases:**
1. **AST (3h):** `Expr::Closure` o funciones como valores
2. **Parser (5h):** Sintaxis `fn(...) { ... }`
3. **Backend (10h):** Captura de variables, trampolines si necesario
4. **Tests (2h):** Closures simples y complejos

**Archivos:**
- `rust/crates/adead-parser/src/lib.rs` - Parser closures
- `rust/crates/adead-backend/src/lib.rs` - Codegen
- `rust/crates/adead-borrow/src/lib.rs` - Análisis de capturas

**Impacto:** 🔴 ALTO - Desbloquea programación funcional  
**Desbloquea:** `map`, `filter`, `reduce` realmente útiles

---

### 3.3 Pattern Matching Avanzado (15 horas)

**Estado:** 🟡 PARCIAL (match básico ✅)  
**Por qué:** MEDIO - Más expresivo que if/else

**Mejoras:**
```adead
// Destructuring
match resultado {
    Ok(valor) => print valor
    Err(FileError { path, message }) => print "Error en {path}: {message}"
    _ => print "Otro error"
}

// Guards
match numero {
    x if x < 0 => print "Negativo"
    x if x > 0 => print "Positivo"
    _ => print "Cero"
}

// Matching en asignaciones
let Ok(valor) = resultado  // Desempaquetar directamente
```

**Fases:**
1. **Parser (6h):** Destructuring, guards, pattern matching en let
2. **Backend (8h):** Generar código eficiente
3. **Tests (1h):** Casos complejos

**Archivos:**
- `rust/crates/adead-parser/src/lib.rs` - Pattern matching avanzado
- `rust/crates/adead-backend/src/lib.rs` - Codegen

**Impacto:** 🟡 MEDIO - Código más expresivo y seguro  
**Desbloquea:** Manejo de errores más elegante

---

### 3.4 Generics / Templates (30 horas)

**Estado:** 🔴 NO IMPLEMENTADO  
**Por qué:** MEDIO - Reutilización de código, type safety

**Implementación:**
```adead
// Funciones genéricas
fn max<T>(a: T, b: T) -> T {
    if a > b { return a }
    return b
}

// Structs genéricos
struct Option<T> {
    Some(T),
    None,
}

// Uso
let max_num = max(5, 10)
let max_str = max("a", "b")
```

**Fases:**
1. **AST (5h):** Parámetros de tipo, tipos genéricos
2. **Parser (8h):** Sintaxis `<T>`, inferencia de tipos
3. **Type checker (10h):** Validación de tipos genéricos
4. **Backend (5h):** Monomorfización (generar código específico)
5. **Tests (2h):** Generics simples y complejos

**Archivos:**
- `rust/crates/adead-common/src/lib.rs` - Tipos genéricos
- `rust/crates/adead-parser/src/lib.rs` - Parser
- `rust/crates/adead-borrow/src/lib.rs` - Type checking
- `rust/crates/adead-backend/src/lib.rs` - Monomorfización

**Impacto:** 🟡 MEDIO - Código más reutilizable  
**Desbloquea:** Librerías genéricas, containers

---

## 🎯 OPCIÓN 4: Sistema de Módulos Avanzado

### 4.1 Sistema de Módulos Completo (35 horas)

**Estado:** 🟡 PARCIAL (import básico ✅)  
**Por qué:** MEDIO - Importante para proyectos grandes

**Mejoras:**

#### Re-exports (5h)
```adead
// math/addition.ad
pub fn add(a: int64, b: int64) -> int64 { return a + b }

// math.ad
pub use addition.add  // Re-exportar

// main.ad
import math
math.add(5, 3)  // Funciona sin conocer estructura interna
```

#### Namespaces Jerárquicos (10h)
```adead
import std.collections.hashmap
import std.io.file

// O
import std

let map = std.collections.hashmap.new()
let file = std.io.file.open("test.txt")
```

#### Módulos Anidados (8h)
```adead
// math/number.ad
module math.number {
    pub fn factorial(n: int64) -> int64 { ... }
}

// main.ad
import math.number
let result = math.number.factorial(5)
```

#### Compilación Incremental (12h)
- Cache de módulos parseados
- Re-compilar solo módulos modificados
- Dependency tracking

**Archivos:**
- `rust/crates/adead-parser/src/module_resolver.rs` - Mejoras
- `rust/crates/adead-cli/src/main.rs` - Compilación incremental
- `docs/modules/MODULES.md` - Documentación

**Impacto:** 🟡 MEDIO - Proyectos profesionales  
**Desbloquea:** Organización de código a gran escala

---

### 4.2 Package Manager (60 horas)

**Estado:** 🔴 NO IMPLEMENTADO  
**Por qué:** ALTO - Facilita distribución y reutilización

**Funcionalidades:**
```bash
# Instalar paquete
adeadpm install math

# Usar en código
import math

# Crear paquete
adeadpm init mi-paquete
adeadpm publish

# Gestión de dependencias
# adead.toml
[dependencies]
math = "1.0.0"
utils = "2.1.0"
```

**Implementación:**
- Repositorio de paquetes (simple, Git-based inicialmente)
- Resolución de dependencias
- Versionado semántico
- Lock file

**Fases:**
1. **CLI (15h):** Comandos básicos
2. **Resolución (20h):** Dependency resolution
3. **Instalación (15h):** Descargar y organizar
4. **Publicación (10h):** Subir paquetes

**Impacto:** 🔴 ALTO - Ecosistema completo  
**Desbloquea:** Comunidad, librerías compartidas

---

## 🎯 OPCIÓN 5: Interoperabilidad (FFI)

### 5.1 FFI con C (35 horas)

**Estado:** 🔴 NO IMPLEMENTADO  
**Por qué:** CRÍTICO - Acceso a ecosistema C completo

**Implementación:**
```adead
// Declarar función externa
extern fn printf(format: *const char, ...) -> int32

// Usar
printf("Hello %s\n", name)

// Structs C
extern struct FILE {
    // Campos...
}

// Llamar funciones C
extern fn fopen(path: *const char, mode: *const char) -> *FILE
```

**Fases:**
1. **Parser (8h):** `extern fn`, `extern struct`, tipos C
2. **Backend (20h):** Generar código compatible con ABI C
3. **Linking (5h):** Enlazar con librerías C
4. **Tests (2h):** Funciones simples y complejas

**Archivos:**
- `rust/crates/adead-parser/src/lib.rs` - Parser extern
- `rust/crates/adead-backend/src/lib.rs` - ABI C
- `docs/ffi/C-FFI.md` - Guía

**Impacto:** 🔴 CRÍTICO - Todo el ecosistema C  
**Desbloquea:** OpenGL, SDL, SQLite, etc.

---

### 5.2 FFI con Rust (25 horas)

**Estado:** 🔴 NO IMPLEMENTADO  
**Por qué:** ALTO - Usar crates de Rust

**Implementación:**
```rust
// En Rust (crate)
#[no_mangle]
pub extern "C" fn rust_function(x: i64) -> i64 {
    x * 2
}
```

```adead
// En ADead
extern fn rust_function(x: int64) -> int64

let result = rust_function(5)
```

**Fases:**
1. **Interfaz Rust (10h):** Macros helper, wrappers
2. **Linking (10h):** Compilar y enlazar crates Rust
3. **Tests (5h):** Ejemplos completos

**Impacto:** 🔴 ALTO - Ecosistema Rust  
**Desbloquea:** Todas las crates de Rust

---

## 🎯 OPCIÓN 6: Optimizaciones del Compilador

### 6.1 Optimizaciones Básicas (25 horas)

**Estado:** 🔴 NO IMPLEMENTADO  
**Por qué:** MEDIO - Mejora performance generada

**Optimizaciones:**

#### Dead Code Elimination (5h)
- Eliminar código no alcanzable
- Eliminar variables no usadas

#### Constant Folding (5h)
```adead
let x = 2 + 3  // → let x = 5
if false { ... }  // → Eliminar
```

#### Simple Inlining (8h)
```adead
fn add(a: int64, b: int64) -> int64 { return a + b }
let x = add(5, 3)  // → let x = 5 + 3
```

#### Register Allocation (7h)
- Mejor uso de registros
- Menos movimientos innecesarios

**Archivos:**
- `rust/crates/adead-optimizer/` (NUEVO)
- `docs/optimizations/OPTIMIZATIONS.md`

**Impacto:** 🟡 MEDIO - Código más eficiente  
**Desbloquea:** Performance competitiva

---

### 6.2 Mejoras de Compilación (20 horas)

**Estado:** 🟡 PARCIAL  
**Por qué:** MEDIO - Mejor experiencia de desarrollo

**Mejoras:**
- Caching de compilación
- Compilación paralela de módulos
- Flags: `-O0`, `-O1`, `-O2`, `-O3`
- Debug info: `-g`
- Warnings: `-W`
- Verbose: `-v`

**Implementación:**
- Sistema de cache basado en hashes
- Paralelización con Rayon
- Flags en CLI

**Impacto:** 🟡 MEDIO - Desarrollo más rápido  
**Desbloquea:** Iteración rápida

---

### 6.3 Optimizaciones Runtime para Print (12 horas)

**Estado:** 🔴 NO IMPLEMENTADO  
**Por qué:** MEDIO - Mejora performance de programas generados

**Optimizaciones Específicas:**

#### 6.3.1 Inline de Función Helper para Números Pequeños (4h)
```rust
// Para números 0-9, evitar call overhead
// Inline directamente la conversión
let digit = rax + '0'  // Conversión directa
mov [buffer], digit
// Evitar loop completo para un solo dígito
```

**Beneficio:** Reduce overhead de llamada a función para números comunes

#### 6.3.2 Cachear Handles de I/O (3h)
```rust
// Actualmente: GetStdHandle se llama en cada print
// Optimización: Cachear handle globalmente
static mut STDOUT_HANDLE: Option<HANDLE> = None;
if STDOUT_HANDLE.is_none() {
    STDOUT_HANDLE = Some(GetStdHandle(-11));
}
// Usar handle cacheado
```

**Beneficio:** Evita llamadas redundantes a GetStdHandle

#### 6.3.3 Optimización para Números Pre-calculados (2h)
```rust
// Para literales numéricos en print, pre-calcular string en compilación
// print 42 → ya convertir a "42\n" en tiempo de compilación
// Solo usar runtime conversion para expresiones
```

**Beneficio:** Elimina conversión runtime para casos simples

#### 6.3.4 Pool de Buffers (3h)
```rust
// Reutilizar buffers para conversiones numéricas
// Evitar alloc/dealloc en cada print
static mut CONVERSION_BUFFER: [u8; 32] = [0; 32];
// Usar buffer estático en lugar de stack local
```

**Beneficio:** Reduce presión en stack y mejora cache locality

**Implementación:**
- Detectar casos especiales (números pequeños, literales)
- Generar código optimizado según caso
- Variables estáticas para handles y buffers

**Archivos:**
- `rust/crates/adead-backend/src/lib.rs` - Optimizaciones en `generate_stmt_windows`
- `rust/crates/adead-optimizer/` (NUEVO) - Análisis de optimizaciones

**Impacto:** 🟡 MEDIO - Performance mejorada  
**Desbloquea:** Programas más rápidos, menor overhead de runtime

**Prioridad:** ⭐⭐⭐ (Después de optimizaciones básicas)

---

## 🎯 OPCIÓN 7: Herramientas de Desarrollo

### 7.1 Language Server Protocol (LSP) (40 horas)

**Estado:** 🔴 NO IMPLEMENTADO  
**Por qué:** ALTO - Integración con IDEs

**Funcionalidades:**
- Autocompletado
- Go to definition
- Hover information
- Error highlighting
- Format on save

**Implementación:**
- Server LSP en Rust
- Protocolo estándar
- Integración con VS Code, Vim, etc.

**Impacto:** 🔴 ALTO - Desarrollo profesional  
**Desbloquea:** IDE support completo

---

### 7.2 Debugger (50 horas)

**Estado:** 🔴 NO IMPLEMENTADO  
**Por qué:** MEDIO - Debugging es esencial

**Funcionalidades:**
- Breakpoints
- Step over/into/out
- Inspect variables
- Call stack
- Watch expressions

**Implementación:**
- Integración con GDB/LLDB
- Debug info generation
- Protocolo DAP (Debug Adapter Protocol)

**Impacto:** 🟡 MEDIO - Debugging más fácil  
**Desbloquea:** Desarrollo eficiente

---

### 7.3 Formatter (15 horas)

**Estado:** 🔴 NO IMPLEMENTADO  
**Por qué:** MEDIO - Consistencia de código

**Implementación:**
```bash
adeadfmt archivo.ad  # Formatear archivo
adeadfmt --check .   # Verificar formato
```

- Reglas de formato consistentes
- Preservar comentarios
- Configurable

**Impacto:** 🟡 MEDIO - Código consistente  
**Desbloquea:** Mejor legibilidad

---

## 🚀 NUEVAS IDEAS: Parser Híbrido y Soluciones Avanzadas (Diciembre 2025)

### 8.1 Parser Híbrido Multi-Pass con Fallback Inteligente (30 horas)

**Estado:** 🔴 PROPUESTA NUEVA  
**Por qué:** CRÍTICO - Soluciona problemas de parsing complejos (while anidados, estructuras complejas)

**Problema Identificado:**
- Zig y Rust tienen limitaciones en parsing recursivo complejo
- Estructuras anidadas (while con if dentro) fallan silenciosamente
- Falta manejo de errores robusto en parsing recursivo

**Solución Propuesta: Parser Multi-Pass con Validación Incremental**

#### Fase 1: Parser de Bloques Robusto (10h)
```rust
// Nuevo módulo: rust/crates/adead-parser/src/block_parser.rs
pub struct BlockParser {
    // Parser que maneja bloques anidados correctamente
    // Usa stack de contextos para rastrear niveles de anidación
}

impl BlockParser {
    // Parsear bloque completo con validación de llaves
    pub fn parse_block(&self, input: &str) -> Result<Vec<Statement>> {
        // 1. Escanear y validar estructura de llaves
        // 2. Identificar statements dentro del bloque
        // 3. Parsear cada statement recursivamente
        // 4. Validar que todas las llaves estén balanceadas
    }
}
```

**Ventajas:**
- Validación previa de estructura antes de parsing
- Manejo robusto de bloques anidados
- Mejores mensajes de error (línea exacta del problema)

#### Fase 2: Parser Híbrido Zig→Rust→Fallback (12h)
```rust
// Estrategia de parsing inteligente:
// 1. Intentar Zig parser (rápido, eficiente)
// 2. Si falla, intentar Rust parser (robusto)
// 3. Si ambos fallan, usar parser de fallback (simple pero funcional)

pub enum ParserStrategy {
    ZigDirect,      // Zig → NASM directo
    ZigRust,        // Zig → Rust → NASM
    RustDirect,     // Rust → NASM
    Fallback,       // Parser simple pero robusto
}

pub fn parse_with_fallback(input: &str) -> Result<Program> {
    // Intentar estrategias en orden de complejidad
    match parse_with_zig(input) {
        Ok(program) => Ok(program),
        Err(_) => match parse_with_rust(input) {
            Ok(program) => Ok(program),
            Err(_) => parse_with_fallback_simple(input),  // Último recurso
        }
    }
}
```

**Ventajas:**
- Máxima robustez: siempre hay un parser que funciona
- Performance optimizada: usa el parser más rápido posible
- Degradación elegante: si Zig falla, usa Rust; si Rust falla, usa fallback

#### Fase 3: Validación Incremental (8h)
```rust
// Validar estructura antes de parsing profundo
pub fn validate_structure(input: &str) -> Result<StructureInfo> {
    // 1. Contar llaves abiertas/cerradas
    // 2. Validar keywords (while, if, etc.)
    // 3. Identificar bloques anidados
    // 4. Detectar problemas estructurales antes de parsing
}

// Si validación falla, dar mensaje de error claro
// Si pasa, proceder con parsing completo
```

**Impacto:** 🔴 CRÍTICO - Soluciona parsing de estructuras complejas  
**Desbloquea:** while/if anidados, programas complejos, mejor experiencia de usuario

---

### 8.2 Parser de Expresiones Unificado con Backtracking (20 horas)

**Estado:** 🔴 PROPUESTA NUEVA  
**Por qué:** MEDIO - Mejora parsing de expresiones complejas

**Problema:**
- Parsers actuales fallan con expresiones ambiguas
- No hay backtracking para resolver ambigüedades
- Operadores complejos (%, ==, <=) causan problemas

**Solución: Parser con Backtracking Inteligente**

```rust
// Nuevo módulo: rust/crates/adead-parser/src/expr_unified.rs
pub struct UnifiedExprParser {
    // Parser que usa backtracking para resolver ambigüedades
    // Mantiene múltiples hipótesis y prueba la mejor
}

impl UnifiedExprParser {
    pub fn parse_with_backtracking(&self, input: &str) -> Result<Expr> {
        // 1. Generar múltiples hipótesis de parsing
        // 2. Probar cada una hasta encontrar la correcta
        // 3. Validar resultado
        // 4. Retornar mejor match
    }
}
```

**Ventajas:**
- Maneja expresiones ambiguas correctamente
- Mejores mensajes de error
- Más robusto para casos edge

---

### 8.3 Sistema de Parsing por Fases (Parser Pipeline) (25 horas)

**Estado:** 🔴 PROPUESTA NUEVA  
**Por qué:** MEDIO-ALTO - Arquitectura más robusta y mantenible

**Idea: Dividir parsing en fases claras**

#### Fase 1: Tokenización (Lexer) (8h)
```rust
// Separar tokenización de parsing
// Ventaja: detectar errores de sintaxis temprano
pub struct Lexer {
    // Convierte string → tokens
    // Detecta keywords, identificadores, operadores, etc.
}
```

#### Fase 2: Parsing Estructural (10h)
```rust
// Parsear estructura (statements, bloques)
// Sin evaluar expresiones todavía
pub struct StructuralParser {
    // Identifica: while, if, let, etc.
    // Construye árbol estructural
}
```

#### Fase 3: Parsing de Expresiones (7h)
```rust
// Parsear expresiones dentro de estructura
// Usar árbol estructural para contexto
pub struct ExprParser {
    // Parsea expresiones con contexto completo
    // Sabe en qué tipo de statement está
}
```

**Ventajas:**
- Separación clara de responsabilidades
- Más fácil de debuggear
- Mejor manejo de errores
- Más mantenible

---

### 8.4 Parser Incremental con Error Recovery (30 horas)

**Estado:** 🔴 PROPUESTA NUEVA  
**Por qué:** ALTO - Mejora experiencia de desarrollo

**Idea: Continuar parsing incluso con errores**

```rust
// Parser que no se detiene en primer error
// Continúa y reporta todos los errores encontrados
pub struct IncrementalParser {
    // Parsear todo el archivo
    // Reportar múltiples errores
    // Permitir corrección de múltiples problemas a la vez
}

// Ejemplo:
// Error en línea 10: missing '}'
// Error en línea 15: undefined variable 'x'
// Error en línea 20: type mismatch
// → Usuario corrige todos a la vez
```

**Ventajas:**
- Desarrollo más rápido
- Menos iteraciones de compilación
- Mejor experiencia de usuario

---

## 📋 Tabla Comparativa Completa (Actualizada)

| Opción | Esfuerzo | Impacto | Prioridad | Dependencias | Desbloquea |
|--------|----------|---------|-----------|--------------|------------|
| **1.1 Print números** | 3h | 🟡 MEDIO | ⭐⭐⭐⭐⭐ | - | Debugging |
| **1.2 Operadores lógicos** | 4h | 🟡 MEDIO | ⭐⭐⭐⭐ | - | Lógica compleja |
| **1.3 Break/Continue** | 5h | 🟡 MEDIO | ⭐⭐⭐⭐ | - | Control loops |
| **1.4 Asignación arrays** | 6h | 🟡 MEDIO | ⭐⭐⭐⭐ | Arrays ✅ | Arrays mutables |
| **1.5 Bool nativo** | 5h | 🟡 MEDIO | ⭐⭐⭐ | - | Type safety |
| **2.1 Float64/Float32** | 15h | 🔴 CRÍTICO | ⭐⭐⭐⭐⭐ | - | Matemáticas |
| **2.2 Strings completos** | 25h | 🔴 ALTO | ⭐⭐⭐⭐⭐ | Arrays ✅ | Texto, I/O |
| **2.3 std.math** | 20h | 🔴 ALTO | ⭐⭐⭐⭐ | Float64 (2.1) | Ciencias |
| **2.4 std.array** | 18h | 🟡 MEDIO-ALTO | ⭐⭐⭐⭐ | Arrays ✅ | Arrays útiles |
| **3.1 For loops** | 10h | 🟡 MEDIO | ⭐⭐⭐ | - | Sintaxis |
| **3.2 Closures** | 20h | 🔴 ALTO | ⭐⭐⭐⭐ | - | Funcional |
| **3.3 Pattern matching** | 15h | 🟡 MEDIO | ⭐⭐⭐ | Match ✅ | Expresivo |
| **3.4 Generics** | 30h | 🟡 MEDIO | ⭐⭐⭐ | - | Reutilización |
| **4.1 Módulos avanzado** | 35h | 🟡 MEDIO | ⭐⭐⭐ | Import ✅ | Proyectos grandes |
| **4.2 Package manager** | 60h | 🔴 ALTO | ⭐⭐⭐⭐ | Módulos (4.1) | Ecosistema |
| **5.1 FFI C** | 35h | 🔴 CRÍTICO | ⭐⭐⭐⭐⭐ | - | Ecosistema C |
| **5.2 FFI Rust** | 25h | 🔴 ALTO | ⭐⭐⭐⭐ | FFI C (5.1) | Ecosistema Rust |
| **6.1 Optimizaciones** | 25h | 🟡 MEDIO | ⭐⭐⭐ | - | Performance |
| **6.2 Compilación** | 20h | 🟡 MEDIO | ⭐⭐⭐ | - | Desarrollo rápido |
| **6.3 Optimizaciones Runtime Print** | 12h | 🟡 MEDIO | ⭐⭐⭐ | Print ✅ | Performance print |
| **7.1 LSP** | 40h | 🔴 ALTO | ⭐⭐⭐⭐ | - | IDE support |
| **7.2 Debugger** | 50h | 🟡 MEDIO | ⭐⭐⭐ | - | Debugging |
| **7.3 Formatter** | 15h | 🟡 MEDIO | ⭐⭐⭐ | - | Consistencia |
| **8.1 Parser Híbrido Multi-Pass** | 30h | 🔴 CRÍTICO | ⭐⭐⭐⭐⭐ | - | Parsing robusto |
| **8.2 Parser Unificado Backtracking** | 20h | 🟡 MEDIO | ⭐⭐⭐ | - | Expresiones complejas |
| **8.3 Parser por Fases** | 25h | 🟡 MEDIO-ALTO | ⭐⭐⭐⭐ | - | Arquitectura robusta |
| **8.4 Parser Incremental** | 30h | 🔴 ALTO | ⭐⭐⭐⭐ | - | Mejor UX desarrollo |

---

## 🎯 Plan de Acción Recomendado (Actualizado Diciembre 2025)

### Fase 0: CRÍTICO - Fix Parsing Robusto (2 semanas) - 30 horas

**Prioridad MÁXIMA:** Solucionar parsing de estructuras complejas

1. **Parser Híbrido Multi-Pass (30h)** - 🔴 CRÍTICO
   - Implementar parser de bloques robusto
   - Sistema de fallback Zig→Rust→Simple
   - Validación incremental
   - **Resultado:** Parsing confiable de while/if anidados

**Resultado:** Base sólida para todo lo demás

---

### Fase 1: Quick Wins (1 semana) - 23 horas

1. **Print de números** (3h) - ⚡ Más rápido, impacto inmediato ✅ COMPLETADO
2. **Operadores lógicos** (4h) - Fácil, útil
3. **Break/Continue** (5h) - Simple, mejora loops
4. **Asignación arrays** (6h) - Arrays mutables
5. **Bool nativo** (5h) - Base importante

**Resultado:** Mejoras inmediatas, UX mejorada

---

### Fase 2: Fundación Crítica (2-3 semanas) - 55 horas

6. **Float64/Float32** (15h) - 🔴 CRÍTICO - Base para todo
7. **Strings completos** (25h) - 🔴 ALTO - Necesario para casi todo
8. **std.math básico** (15h) - Funciones esenciales

**Resultado:** Base sólida para librería estándar

---

### Fase 3: Librería Estándar (2-3 semanas) - 38 horas

9. **std.math completo** (5h restantes) - Funciones avanzadas
10. **std.array** (18h) - Funciones útiles para arrays
11. **Closures** (15h) - Necesario para funciones de orden superior

**Resultado:** Stdlib funcional y completa

---

### Fase 4: Mejoras de Lenguaje (1-2 semanas) - 25 horas

12. **For loops** (10h)
13. **Pattern matching avanzado** (15h)

**Resultado:** Sintaxis más completa y expresiva

---

### Fase 5: Interoperabilidad (3-4 semanas) - 60 horas

14. **FFI con C** (35h) - 🔴 CRÍTICO
15. **FFI con Rust** (25h)

**Resultado:** Acceso a ecosistemas completos

---

### Fase 6: Herramientas (2-3 semanas) - 90 horas

16. **LSP** (40h)
17. **Debugger** (50h)

**Resultado:** Desarrollo profesional

---

### Fase 7: Optimizaciones y Polish (2 semanas) - 45 horas

18. **Optimizaciones básicas** (25h)
19. **Mejoras de compilación** (20h)

**Resultado:** Performance y desarrollo rápido

---

## 💡 Quick Wins Prioritarios

Si quieres resultados rápidos con máximo impacto:

1. **Parser Híbrido Multi-Pass** (30h) - 🔴 CRÍTICO - Soluciona parsing complejo
2. **Print de números** (3h) - ⚡ Más rápido ✅ COMPLETADO
3. **Operadores lógicos** (4h) - Muy útil
4. **Asignación arrays** (6h) - Desbloquea algoritmos
5. **Break/Continue** (5h) - Control esencial

**Total:** ~48 horas, transforman la experiencia

---

## 🔮 Vision: Roadmap a 6 Meses

### Mes 0: Fundación Sólida de Parsing (2 semanas)
- ✅ Parser Híbrido Multi-Pass implementado
- ✅ Parsing robusto de estructuras complejas
- ✅ Sistema de fallback funcional

**Resultado:** Parsing confiable y robusto

---

### Mes 1: Fundación Sólida
- ✅ Sprint 1 completo
- ✅ Quick wins (print, operadores, arrays)
- ✅ Float64 implementado
- ✅ Strings básicos

**Resultado:** Lenguaje útil para proyectos pequeños

---

### Mes 2: Librería Estándar
- ✅ Strings completos
- ✅ std.math completo
- ✅ std.array
- ✅ Closures

**Resultado:** Stdlib funcional

---

### Mes 3: Mejoras de Lenguaje
- ✅ For loops
- ✅ Pattern matching avanzado
- ✅ Generics básicos
- ✅ Sistema de módulos avanzado

**Resultado:** Sintaxis completa y moderna

---

### Mes 4: Interoperabilidad
- ✅ FFI con C
- ✅ FFI con Rust
- ✅ Ejemplos de uso

**Resultado:** Acceso a ecosistemas

---

### Mes 5: Herramientas
- ✅ LSP
- ✅ Debugger básico
- ✅ Formatter

**Resultado:** Desarrollo profesional

---

### Mes 6: Polish y Optimizaciones
- ✅ Optimizaciones
- ✅ Package manager
- ✅ Documentación completa
- ✅ Ejemplos y tutoriales

**Resultado:** Lenguaje listo para público

---

## 📋 Checklist de Prioridades Actualizado (Diciembre 2025)

### 🔴 CRÍTICO (Hacer ahora)
- [ ] **Parser Híbrido Multi-Pass (30h)** - 🔴 NUEVO - Soluciona parsing complejo
- [x] Import básico (15h) ✅ **COMPLETADO**
- [x] Print de números (3h) ✅ **COMPLETADO** (2h reales)
- [ ] Float64/Float32 (15h)
- [ ] Strings completos (25h)
- [ ] FFI con C (35h)

### 🟡 ALTA PRIORIDAD (Próximas 2-3 semanas)
- [ ] Operadores lógicos (4h)
- [ ] Asignación arrays (6h)
- [ ] std.math (20h)
- [ ] std.array (18h)
- [ ] Closures (20h)

### 🟢 MEDIA PRIORIDAD (Próximas 4-6 semanas)
- [ ] For loops (10h)
- [ ] Break/Continue (5h)
- [ ] Pattern matching avanzado (15h)
- [ ] Bool nativo (5h)
- [ ] Sistema módulos avanzado (35h)
- [ ] Optimizaciones Runtime Print (12h) - Mejorar performance de print
- [ ] Parser Incremental con Error Recovery (30h) - 🔴 NUEVO

### 🔵 BAJA PRIORIDAD (Más adelante)
- [ ] Generics (30h)
- [ ] Package manager (60h)
- [ ] LSP (40h)
- [ ] Debugger (50h)
- [ ] Optimizaciones (45h)
- [ ] Parser Unificado Backtracking (20h) - 🔴 NUEVO
- [ ] Parser por Fases (25h) - 🔴 NUEVO

---

## 🎯 Siguiente Paso Recomendado (ACTUALIZADO)

**OPCIÓN A: Fix Parsing Crítico (MÁXIMA PRIORIDAD)**
```
→ Parser Híbrido Multi-Pass (30h)
→ Resultado: Parsing robusto de estructuras complejas (while/if anidados)
→ Impacto: Crítico, tiempo: Medio
```

**OPCIÓN B: Quick Wins (Máximo impacto rápido)**
```
→ Print números (3h) ✅ + Operadores lógicos (4h) + Break/Continue (5h)
→ Resultado: Mejoras inmediatas en UX (12h total)
→ Impacto: Alto, tiempo: Bajo
```

**OPCIÓN C: Fundación Crítica (Largo plazo)**
```
→ Float64 (15h) + Strings completos (25h)
→ Resultado: Base sólida para todo (40h total)
→ Impacto: Crítico, tiempo: Medio
```

**OPCIÓN D: Interoperabilidad (Desbloquea ecosistemas)**
```
→ FFI con C (35h)
→ Resultado: Acceso a todo el ecosistema C
→ Impacto: Crítico, tiempo: Alto
```

---

## 🔧 Mejoras Técnicas Recientes (Diciembre 2025)

### Print de Expresiones Aritméticas - Implementación Completa ✅

**Problemas Resueltos:**
- ✅ Preservación correcta de registros según Windows x64 ABI
- ✅ Manejo de conflictos entre R8 (buffer vs longitud)
- ✅ Preservación de RBX durante loops de reversión
- ✅ Alineación de stack (`and rsp, -16`) implementada
- ✅ Función helper `int_to_str_runtime` optimizada

**Arquitectura Final:**
```
Print Statement → Zig Parser (expresiones) → Rust Backend
  → Generar código NASM:
    1. Evaluar expresión (RAX = resultado)
    2. Llamar int_to_str_runtime(RAX, RDX=buffer)
       - Convierte int64 a string decimal
       - Retorna: RAX=longitud, RDX=buffer
    3. WriteFile(handle, RDX=buffer, R8=longitud, ...)
```

**Convención Windows x64 Respeta:**
- ✅ RCX: Handle (stdout)
- ✅ RDX: Buffer pointer (preservado por helper)
- ✅ R8: Length (directo desde RAX)
- ✅ R9: lpNumberOfBytesWritten
- ✅ [rsp+32]: lpOverlapped = NULL
- ✅ Stack alignment: 16 bytes
- ✅ Shadow space: 32 bytes reservados

**Lecciones Aprendidas:**
1. **No sobrescribir registros antes de usarlos:** R8 usado para buffer y longitud causaba conflictos
2. **Preservar registros en funciones helper:** Usar registros no volátiles (R8-R15) y restaurar correctamente
3. **Stack alignment es crítico:** Windows x64 requiere alineación de 16 bytes
4. **Convención de llamadas debe respetarse:** WriteFile espera parámetros específicos en registros específicos

---

## 🌐 OPCIÓN 9: Arquitectura Multi-Lenguaje - Tercer Lenguaje Especializado en Parsing (NUEVO)

**Estado:** 🔴 PROPUESTA NUEVA  
**Por qué:** CRÍTICO - Compensar debilidades de Zig y Rust en parsing recursivo complejo

### 9.1 Análisis del Problema Actual

**Arquitectura Actual:**
```
ADead Source
  ↓
Zig (parsing rápido, eficiente) → ✅ Bueno para expresiones simples
  ↓
Rust (seguridad, codegen) → ✅ Bueno para validación
  ↓
NASM → Ejecutable
```

**Problemas Identificados:**
- ❌ Zig: Parsing recursivo complejo falla con estructuras anidadas
- ❌ Rust: Chumsky tiene limitaciones con backtracking y bloques anidados profundos
- ❌ Ambos: Faltan herramientas especializadas para parsing estructurado

### 9.2 Opciones de Tercer Lenguaje/Tool Especializado

#### 🥇 OPCIÓN A: Tree-sitter (Recomendado) (40 horas)

**Por qué Tree-sitter:**
- ✅ **Parser generator especializado** - Diseñado específicamente para parsing robusto
- ✅ **Incremental parsing** - Parse solo cambia lo necesario
- ✅ **Error recovery avanzado** - Continúa parsing incluso con errores
- ✅ **Múltiples lenguajes** - Bindings en C, Rust, Python, JavaScript
- ✅ **Usado por VS Code, GitHub, etc.** - Probado en producción

**Implementación:**
```javascript
// grammar.js (Tree-sitter grammar para ADead)
module.exports = grammar({
  name: 'adead',
  
  rules: {
    source_file: $ => repeat($._statement),
    
    _statement: $ => choice(
      $.print_statement,
      $.let_statement,
      $.while_statement,
      $.if_statement,
      $.function_definition
    ),
    
    while_statement: $ => seq(
      'while',
      $.expression,
      $.block  // Maneja bloques anidados automáticamente
    ),
    
    block: $ => seq(
      '{',
      repeat($._statement),
      '}'
    ),
    // ... más reglas
  }
});
```

**Arquitectura Propuesta:**
```
ADead Source
  ↓
Tree-sitter (parsing robusto) → AST Tree-sitter
  ↓
Rust (conversión AST + validación) → AST Rust
  ↓
Rust (codegen) → NASM
  ↓
Ejecutable
```

**Ventajas:**
- ✅ Parsing robusto de estructuras anidadas
- ✅ Error recovery automático
- ✅ Incremental parsing (útil para LSP futuro)
- ✅ Syntax highlighting automático (bonus)

**Desventajas:**
- ⚠️ Requiere Node.js para generar grammar
- ⚠️ C binding necesario para Rust FFI
- ⚠️ Curva de aprendizaje media

**Tiempo:** 40 horas  
**Impacto:** 🔴 CRÍTICO - Soluciona parsing complejo definitivamente

---

#### 🥈 OPCIÓN B: Pest (Rust PEG Parser) (25 horas)

**Por qué Pest:**
- ✅ **Parsing Expression Grammar (PEG)** - Muy potente para expresiones complejas
- ✅ **100% Rust** - Sin FFI, integración nativa
- ✅ **Backtracking automático** - Maneja ambigüedades
- ✅ **Librería madura** - Usada por muchos proyectos Rust

**Implementación:**
```rust
// grammar.pest
WHITESPACE = _{ " " | "\t" | "\n" | "\r" }

source_file = { statement* }

statement = {
    print_stmt |
    let_stmt |
    while_stmt |
    if_stmt |
    function_def
}

while_stmt = { "while" ~ expression ~ "{" ~ statement* ~ "}" }

block = { "{" ~ statement* ~ "}" }

expression = { 
    comparison |
    additive |
    multiplicative |
    primary
}

comparison = { additive ~ (("==" | "!=" | "<" | "<=" | ">" | ">=") ~ additive)* }
additive = { multiplicative ~ (("+" | "-") ~ multiplicative)* }
multiplicative = { primary ~ (("*" | "/" | "%") ~ primary)* }
primary = { number | identifier | "(" ~ expression ~ ")" }
```

**Arquitectura Propuesta:**
```
ADead Source
  ↓
Pest Parser (grammar.pest) → Pest AST
  ↓
Rust (conversión a AST interno) → AST Rust
  ↓
Rust (codegen) → NASM
  ↓
Ejecutable
```

**Ventajas:**
- ✅ 100% Rust (sin FFI)
- ✅ PEG muy potente
- ✅ Backtracking automático
- ✅ Fácil de mantener

**Desventajas:**
- ⚠️ Aún es un parser, puede tener problemas con casos muy complejos
- ⚠️ Menos maduro que Tree-sitter para parsing incremental

**Tiempo:** 25 horas  
**Impacto:** 🔴 ALTO - Solución Rust nativa

---

#### 🥉 OPCIÓN C: LALRPOP (Rust LR Parser) (30 horas)

**Por qué LALRPOP:**
- ✅ **LR(1) Parser Generator** - Parsing determinístico y eficiente
- ✅ **100% Rust** - Integración nativa
- ✅ **Error messages excelentes** - Muy útil para debugging
- ✅ **Usado por Rustc internamente** - Probado en proyectos grandes

**Implementación:**
```rust
// grammar.lalrpop
grammar;

pub SourceFile: Vec<Statement> = {
    <statements:Statement*> => statements
}

pub Statement: Statement = {
    WhileStmt,
    IfStmt,
    LetStmt,
    PrintStmt,
    FunctionDef,
}

WhileStmt: Statement = {
    "while" <cond:Expression> "{" <body:Statement*> "}" =>
        Statement::While { condition: cond, body: body }
}

Expression: Expr = {
    Comparison,
}

Comparison: Expr = {
    Additive ("<=" | ">=" | "<" | ">" | "==" | "!=") Additive =>
        Expr::BinaryOp { op: <>, left: <>, right: <> },
    Additive,
}
// ... más reglas
```

**Ventajas:**
- ✅ LR parser muy robusto
- ✅ Error messages excelentes
- ✅ 100% Rust
- ✅ Determinístico

**Desventajas:**
- ⚠️ Más complejo de configurar inicialmente
- ⚠️ Puede ser sobrekill para sintaxis simple

**Tiempo:** 30 horas  
**Impacto:** 🟡 ALTO - Solución robusta Rust

---

#### OPCIÓN D: OCaml con Menhir (50 horas)

**Por qué OCaml:**
- ✅ **Excelente para parsing** - Usado en compiladores (Rust, Coq, etc.)
- ✅ **Menhir parser generator** - Muy potente
- ✅ **Pattern matching nativo** - Perfecto para AST
- ✅ **Type safety fuerte** - Menos errores

**Implementación:**
```ocaml
%token WHILE IF LET PRINT
%token <int> NUMBER
%token <string> IDENTIFIER
%token EOF

%start <ast.program> program

%%

program:
  | statements = list(statement) EOF { { statements } }

statement:
  | WHILE cond = expression LBRACE body = list(statement) RBRACE
    { While (cond, body) }
  | IF cond = expression LBRACE then_body = list(statement) RBRACE
      else_body = option(ELSE LBRACE list(statement) RBRACE)
    { If (cond, then_body, Option.value else_body ~default:[]) }
  | LET name = IDENTIFIER EQ value = expression
    { Let (name, value) }
  | PRINT expr = expression
    { Print expr }
```

**Arquitectura Propuesta:**
```
ADead Source
  ↓
OCaml (parser con Menhir) → AST OCaml
  ↓
FFI (OCaml → Rust) → AST Rust
  ↓
Rust (codegen) → NASM
  ↓
Ejecutable
```

**Ventajas:**
- ✅ Excelente para parsing complejo
- ✅ Type safety fuerte
- ✅ Pattern matching nativo

**Desventajas:**
- ⚠️ Requiere OCaml toolchain
- ⚠️ FFI OCaml→Rust más complejo
- ⚠️ Menos común en ecosistema Rust

**Tiempo:** 50 horas  
**Impacto:** 🟡 MEDIO-ALTO - Excelente pero más complejo

---

#### OPCIÓN E: Nim (35 horas)

**Por qué Nim:**
- ✅ **Compila a C** - Fácil FFI con Rust
- ✅ **Macros poderosos** - Puede generar parsers
- ✅ **Sintaxis limpia** - Fácil de escribir
- ✅ **Performance nativa** - Sin overhead

**Implementación:**
```nim
# parser.nim
import macros, strutils

proc parseStatement(s: string): Statement =
  # Parser recursivo con backtracking
  if s.startsWith("while"):
    # Parse while loop
    let cond = parseExpression(...)
    let body = parseBlock(...)
    return WhileStmt(cond, body)
  # ... más casos
```

**Ventajas:**
- ✅ Fácil integración con Rust (via C)
- ✅ Sintaxis limpia
- ✅ Performance nativa

**Desventajas:**
- ⚠️ Requiere toolchain Nim
- ⚠️ Menos especializado en parsing

**Tiempo:** 35 horas  
**Impacto:** 🟡 MEDIO - Alternativa interesante

---

#### OPCIÓN F: Python + Lark (20 horas) - Prototipo Rápido

**Por qué Python + Lark:**
- ✅ **Lark parser** - Muy fácil de usar
- ✅ **Rápido de prototipar** - Validar idea rápido
- ✅ **Python FFI con Rust** - PyO3
- ✅ **Excelente para MVP** - Probar conceptos

**Implementación:**
```python
# grammar.py
from lark import Lark

grammar = """
    source_file: statement*

    statement: while_stmt | if_stmt | let_stmt | print_stmt

    while_stmt: "while" expression "{" statement* "}"
    if_stmt: "if" expression "{" statement* "}" ["else" "{" statement* "}"]
    
    expression: comparison
    comparison: additive (("<=" | ">=" | "<" | ">" | "==" | "!=") additive)*
    additive: multiplicative (("+" | "-") multiplicative)*
    multiplicative: primary (("*" | "/" | "%") primary)*
    primary: NUMBER | IDENTIFIER | "(" expression ")"
"""

parser = Lark(grammar, start='source_file')

def parse_adead(source: str) -> dict:
    tree = parser.parse(source)
    return convert_to_ast(tree)
```

**Ventajas:**
- ✅ Muy rápido de implementar
- ✅ Excelente para prototipar
- ✅ Validar arquitectura antes de invertir mucho tiempo

**Desventajas:**
- ⚠️ Dependencia de Python runtime
- ⚠️ Más lento que soluciones nativas
- ⚠️ Mejor para MVP que producción

**Tiempo:** 20 horas  
**Impacto:** 🟡 MEDIO - Bueno para validar concepto

---

### 9.3 Comparativa de Opciones

| Opción | Tiempo | Complejidad | FFI | Robustez | Incremental | Recomendación |
|--------|--------|-------------|-----|----------|-------------|---------------|
| **Tree-sitter** | 40h | Media | C FFI | ⭐⭐⭐⭐⭐ | ✅ Sí | 🥇 **RECOMENDADO** |
| **Pest** | 25h | Baja | N/A (Rust) | ⭐⭐⭐⭐ | ❌ No | 🥈 Buena opción Rust |
| **LALRPOP** | 30h | Media | N/A (Rust) | ⭐⭐⭐⭐⭐ | ❌ No | 🥉 Excelente LR parser |
| **OCaml+Menhir** | 50h | Alta | Complejo | ⭐⭐⭐⭐⭐ | ❌ No | Si ya conoces OCaml |
| **Nim** | 35h | Media | C FFI | ⭐⭐⭐ | ❌ No | Alternativa interesante |
| **Python+Lark** | 20h | Baja | PyO3 | ⭐⭐⭐ | ❌ No | Solo para MVP/prototipo |

---

### 9.4 Recomendación Final: Tree-sitter

**¿Por qué Tree-sitter?**
1. ✅ **Especializado en parsing robusto** - Exactamente lo que necesitamos
2. ✅ **Incremental parsing** - Útil para LSP futuro
3. ✅ **Error recovery avanzado** - Continúa parsing con errores
4. ✅ **Mantenido activamente** - VS Code, GitHub, etc.
5. ✅ **Documentación excelente** - Fácil de aprender

**Plan de Implementación:**

#### Fase 1: Setup Tree-sitter (5h)
```bash
# Instalar tree-sitter CLI
npm install -g tree-sitter-cli

# Crear grammar básico
tree-sitter generate
```

#### Fase 2: Grammar Completo (15h)
- Definir grammar completo para ADead
- Tests con casos complejos (while anidados, etc.)
- Validar parsing robusto

#### Fase 3: FFI Rust (10h)
- Binding Rust para Tree-sitter C library
- Conversión de AST Tree-sitter → AST Rust
- Integración con codegen existente

#### Fase 4: Migración Gradual (10h)
- Reemplazar parser actual por Tree-sitter
- Validar todos los casos de uso
- Optimizar performance

**Total:** 40 horas

---

### 9.5 Arquitectura Final Propuesta

**Arquitectura Híbrida Triple:**
```
ADead Source (.ad)
  ↓
┌─────────────────────────────────────────┐
│  TREE-SITTER (Parsing Robusto)         │
│  • Maneja estructuras anidadas         │
│  • Error recovery automático           │
│  • Incremental parsing                 │
└─────────────────────────────────────────┘
  ↓ (AST Tree-sitter)
┌─────────────────────────────────────────┐
│  RUST (Conversión + Validación)        │
│  • Convertir AST Tree-sitter → AST Rust│
│  • Validación de tipos                 │
│  • Borrow checking                     │
└─────────────────────────────────────────┘
  ↓ (AST Rust validado)
┌─────────────────────────────────────────┐
│  RUST (Code Generation)                │
│  • Generar NASM x86_64                 │
│  • Optimizaciones                      │
└─────────────────────────────────────────┘
  ↓ (NASM Assembly)
┌─────────────────────────────────────────┐
│  NASM + Linker                         │
│  • Ensamblar a .obj/.o                 │
│  • Enlazar a ejecutable                │
└─────────────────────────────────────────┘
  ↓
✅ Ejecutable (.exe)
```

**Flujo de Fallback:**
1. **Primero:** Tree-sitter (parsing robusto)
2. **Si falla:** Pest parser (fallback Rust)
3. **Si falla:** Parser actual Chumsky (último recurso)

**Ventajas de esta Arquitectura:**
- ✅ **Robustez máxima:** Triple fallback
- ✅ **Performance:** Tree-sitter muy rápido
- ✅ **Mantenibilidad:** Cada herramienta hace lo mejor
- ✅ **Escalabilidad:** Fácil agregar más parsers si es necesario

---

## 📚 Documentación Relacionada

- `docs/roadmap/PROGRESO-SPRINT1.md` - Estado actual Sprint 1
- `docs/roadmap/ROADMAP-PROFESIONAL.md` - Plan completo 6 meses
- `docs/testing/TESTING-IMPORTS.md` - Guía de testing
- `docs/stdlib/` - Documentación de librería estándar (futuro)
- `docs/avances/ZIG-COMPARACIONES-IMPLEMENTADO.md` - Integración Zig parser

---

**Última actualización:** Diciembre 2025  
**Sprint 1:** ✅ 100% Completado  
**Print de Expresiones:** ✅ 100% Completado con optimizaciones  
**Parsing Robusto:** 🔴 PRIORIDAD CRÍTICA - Nuevas ideas agregadas (Sección 8)  
**Recomendación:** Empezar con **OPCIÓN A (Parser Híbrido Multi-Pass)** para solucionar parsing complejo, luego continuar con Quick Wins
