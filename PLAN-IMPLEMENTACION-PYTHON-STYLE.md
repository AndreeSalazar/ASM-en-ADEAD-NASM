# 🐍 Plan de Implementación: Sintaxis Estilo Python para ADead → NASM Directo

## 🎯 Objetivo Final
Hacer que ADead tenga sintaxis **MUY similar a Python** y genere código **NASM puro optimizado** directamente (sin pasar por C), usando los 5 componentes de la arquitectura pentágono de manera inteligente.

**Meta:** Sintaxis Python Style → NASM ASM virgen y optimizado → CPU directo

---

## 📊 Estado Actual del Proyecto (Diciembre 2025)

### ✅ Lo que YA funciona (vía C Generator):
- ✅ Parser completo: `let`, `if/else`, `while`, `print`, expresiones aritméticas
- ✅ Strings básicos: `let s = "texto"` → genera C → GCC → ASM
- ✅ Funciones básicas: `def nombre(param):` → genera C → GCC → ASM
- ✅ Arrays básicos: `let arr = [1, 2, 3]` → genera C → GCC → ASM
- ✅ Comparaciones: `==`, `!=`, `<`, `<=`, `>`, `>=`
- ✅ Módulos básicos: `import math` (parser + resolver)

### ❌ Lo que FALTA para Python Style TOTAL en NASM directo:
- ❌ **NASM Backend para Arrays**: Generar NASM directamente sin pasar por C
- ❌ **NASM Backend para Strings avanzados**: Concatenación, slicing, métodos
- ❌ **NASM Backend para Funciones completas**: Stack frames, parámetros múltiples, recursión
- ❌ **NASM Backend para Módulos**: Linking de módulos en NASM
- ❌ **Métodos estilo Python**: `arr.append()`, `arr.pop()`, `s.upper()`, etc.
- ❌ **Built-ins estilo Python**: `len()`, `range()`, `print()` mejorado

---

## 🏗️ Arquitectura por Característica (NASM Directo)

### 1️⃣ ARRAYS/LISTAS (Prioridad 1) - ⚠️ PARCIALMENTE IMPLEMENTADO

**Sintaxis Python que queremos:**
```python
arr = [1, 2, 3]
print(arr[0])        # 1
arr.append(4)
print(len(arr))      # 4
arr[0] = 10
print(arr[0])        # 10
```

**Sintaxis ADead objetivo:**
```adead
let arr = [1, 2, 3]
print arr[0]
arr.append(4)
print len(arr)
arr[0] = 10
print arr[0]
```

**Estado Actual:**
- ✅ **Parser Manual (Rust)**: Parsear `[1, 2, 3]` → `ArrayLiteral` ✅
- ✅ **Parser Manual (Rust)**: Parsear `arr[0]` → `Index` ✅
- ✅ **C Generator**: Genera código C con estructura `Array` ✅
- ✅ **C Generator**: Funciones helper (`array_get`, `array_set`, `array_append`) ✅
- ❌ **NASM Backend**: NO genera NASM directamente para arrays ❌
- ❌ **NASM Backend**: NO tiene estructura Array en NASM ❌
- ❌ **NASM Backend**: NO tiene funciones helper en NASM ❌

**Flujo Objetivo (NASM Directo):**
```
1. 📝 Parser Manual (Rust)
   └─> Detecta: let arr = [1, 2, 3]
   └─> Genera: Expr::ArrayLiteral(vec![...])

2. ⚡ Zig (Optimización - Futuro)
   └─> Si array pequeño: Comptime evaluation
   └─> Optimiza acceso por índice

3. 🔒 Rust (Validación)
   └─> Type checking: todos elementos mismo tipo?
   └─> Validación: índices dentro de rango?

4. 🎯 NASM Generator (NUEVO - Generación Directa)
   └─> Genera estructura Array en NASM:
       - .data section: espacio para data, length, capacity
       - Funciones helper en NASM:
         * array_new (crear array vacío)
         * array_from_values (crear desde valores)
         * array_get (acceso por índice)
         * array_set (asignación por índice)
         * array_append (agregar elemento)
         * array_len (obtener longitud)
   └─> Genera código NASM para ArrayLiteral
   └─> Genera código NASM para Index
   └─> Genera código NASM para arr.append()

5. 🔷 D (Metaprogramming - Futuro)
   └─> CTFE para arrays constantes
   └─> Templates para optimización
```

**Implementación Pendiente:**
- [ ] **NASM Backend**: Estructura Array en NASM (data, length, capacity)
- [ ] **NASM Backend**: Función `array_new` en NASM
- [ ] **NASM Backend**: Función `array_from_values` en NASM
- [ ] **NASM Backend**: Función `array_get` en NASM (con bounds checking)
- [ ] **NASM Backend**: Función `array_set` en NASM (con bounds checking)
- [ ] **NASM Backend**: Función `array_append` en NASM (con realloc)
- [ ] **NASM Backend**: Función `array_len` en NASM
- [ ] **NASM Backend**: Generar código NASM para `ArrayLiteral`
- [ ] **NASM Backend**: Generar código NASM para `Index` (lectura)
- [ ] **NASM Backend**: Generar código NASM para `Index` (asignación: `arr[0] = 5`)
- [ ] **Parser**: Detectar `arr.append(x)` → `MethodCall`
- [ ] **NASM Backend**: Generar código NASM para `MethodCall` (append, pop, etc.)
- [ ] **Parser**: Detectar `len(arr)` → función built-in
- [ ] **NASM Backend**: Generar código NASM para `len()` built-in

---

### 2️⃣ STRINGS REALES (Prioridad 1) - ⚠️ PARCIALMENTE IMPLEMENTADO

**Sintaxis Python que queremos:**
```python
s1 = "hola"
s2 = "mundo"
s3 = s1 + " " + s2    # "hola mundo"
print(s3[0:4])        # "hola"
print(len(s3))        # 11
print(s3.upper())     # "HOLA MUNDO"
```

**Sintaxis ADead objetivo:**
```adead
let s1 = "hola"
let s2 = "mundo"
let s3 = s1 + " " + s2
print s3[0:4]
print len(s3)
print s3.upper()
```

**Estado Actual:**
- ✅ **Parser Manual (Rust)**: Parsear `let s = "texto"` → `String` ✅
- ✅ **Parser Manual (Rust)**: Parsear `s1 + s2` → `BinaryOp { op: Add }` ✅
- ✅ **NASM Backend**: Genera strings en `.data` section ✅
- ✅ **NASM Backend**: Genera código para `print` con strings ✅
- ❌ **NASM Backend**: NO genera concatenación dinámica en NASM ❌
- ❌ **NASM Backend**: NO tiene estructura String dinámica en NASM ❌
- ❌ **NASM Backend**: NO tiene slicing `s[0:4]` ❌
- ❌ **NASM Backend**: NO tiene métodos `s.upper()`, `s.lower()`, etc. ❌

**Flujo Objetivo (NASM Directo):**
```
1. 📝 Parser Manual (Rust)
   └─> Detecta: let s = "texto"
   └─> Detecta: s1 + s2 (concatenación)
   └─> Detecta: s[0:4] (slicing)
   └─> Detecta: s.upper() (método)

2. ⚡ Zig (Optimización - Futuro)
   └─> String interning para literales duplicados
   └─> Optimización de concatenación

3. 🔒 Rust (Validación)
   └─> Type checking: strings son inmutables por defecto
   └─> Validación: índices válidos?

4. 🎯 NASM Generator (NUEVO - Generación Directa)
   └─> Genera estructura String en NASM:
       - .data section: espacio para data, length, capacity
       - Funciones helper en NASM:
         * string_new (crear string vacío)
         * string_from_literal (crear desde literal)
         * string_concat (concatenar s1 + s2)
         * string_slice (s[0:4])
         * string_len (obtener longitud)
         * string_upper (s.upper())
         * string_lower (s.lower())
   └─> Genera código NASM para concatenación dinámica
   └─> Genera código NASM para slicing

5. 🔷 D (Metaprogramming - Futuro)
   └─> CTFE para strings constantes
   └─> Templates para optimización de string ops
```

**Implementación Pendiente:**
- [ ] **NASM Backend**: Estructura String dinámica en NASM (data, length, capacity)
- [ ] **NASM Backend**: Función `string_new` en NASM
- [ ] **NASM Backend**: Función `string_from_literal` en NASM
- [ ] **NASM Backend**: Función `string_concat` en NASM (malloc + strcpy)
- [ ] **NASM Backend**: Función `string_slice` en NASM (s[0:4])
- [ ] **NASM Backend**: Función `string_len` en NASM
- [ ] **NASM Backend**: Función `string_upper` en NASM
- [ ] **NASM Backend**: Función `string_lower` en NASM
- [ ] **Parser**: Detectar `s[0:4]` → `Slice` expression
- [ ] **AST**: Agregar `Expr::Slice { start, end }`
- [ ] **NASM Backend**: Generar código NASM para `Slice`
- [ ] **Parser**: Detectar `s.upper()` → `MethodCall`
- [ ] **NASM Backend**: Generar código NASM para métodos de string

---

### 3️⃣ FUNCIONES COMPLETAS (Prioridad 1) - ⚠️ PARCIALMENTE IMPLEMENTADO

**Sintaxis Python que queremos:**
```python
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

result = factorial(5)
```

**Sintaxis ADead objetivo:**
```adead
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

let result = factorial(5)
```

**Estado Actual:**
- ✅ **Parser Manual (Rust)**: Parsear `def nombre(param):` → `Stmt::Fn` ✅
- ✅ **Parser Manual (Rust)**: Parsear `return valor` → `Stmt::Return` ✅
- ✅ **Parser Manual (Rust)**: Parsear `nombre(args)` → `Expr::Call` ✅
- ✅ **NASM Backend**: Genera funciones básicas en NASM ✅
- ✅ **NASM Backend**: Genera llamadas de función en NASM ✅
- ⚠️ **NASM Backend**: Stack frames básicos (necesita mejoras) ⚠️
- ❌ **NASM Backend**: NO maneja múltiples parámetros correctamente ❌
- ❌ **NASM Backend**: NO maneja recursión profunda ❌
- ❌ **NASM Backend**: NO tiene type checking de parámetros ❌

**Flujo Objetivo (NASM Directo):**
```
1. 📝 Parser Manual (Rust)
   └─> Detecta: def nombre(param1, param2):
   └─> Parsea cuerpo con indentación estilo Python
   └─> Genera: Stmt::Fn { name, params, body }

2. ⚡ Zig (Optimización - Futuro)
   └─> Inlining de funciones pequeñas
   └─> Comptime evaluation cuando sea posible

3. 🔒 Rust (Validación)
   └─> Type checking: parámetros y return types
   └─> Validación: todas las rutas retornan?
   └─> Análisis de recursión

4. 🎯 NASM Generator (MEJORAR - Generación Directa)
   └─> Genera función en NASM con:
       - Prologue: push rbp, mov rbp, rsp, sub rsp (local vars)
       - Parámetros en registros (Windows: RCX, RDX, R8, R9)
       - Parámetros en stack (si > 4 parámetros)
       - Shadow space (Windows: 32 bytes)
       - Stack alignment (16 bytes)
       - Local variables en stack
       - Epilogue: mov rsp, rbp, pop rbp, ret
   └─> Genera llamadas con:
       - Setup de parámetros en registros
       - Setup de shadow space
       - call función
       - Cleanup de shadow space
   └─> Maneja recursión correctamente

5. 🔷 D (Metaprogramming - Futuro)
   └─> CTFE para funciones constantes
   └─> Templates para funciones genéricas
```

**Implementación Pendiente:**
- [ ] **NASM Backend**: Mejorar stack frame management (prologue/epilogue)
- [ ] **NASM Backend**: Manejar múltiples parámetros (> 4) en stack
- [ ] **NASM Backend**: Manejar shadow space correctamente (Windows)
- [ ] **NASM Backend**: Manejar stack alignment (16 bytes)
- [ ] **NASM Backend**: Manejar local variables en stack
- [ ] **NASM Backend**: Manejar recursión profunda (stack overflow protection)
- [ ] **NASM Backend**: Type checking de parámetros (validación en runtime)
- [ ] **NASM Backend**: Validación de return types
- [ ] **Parser**: Mejorar manejo de indentación estilo Python
- [ ] **Parser**: Detectar fin de función por dedentación

---

### 4️⃣ MÓDULOS (Prioridad 1) - ⚠️ PARCIALMENTE IMPLEMENTADO

**Sintaxis Python que queremos:**
```python
import math
from utils import helper_function

result = math.sqrt(16)
helper_function()
```

**Sintaxis ADead objetivo:**
```adead
import math
from utils import helper_function

let result = math.sqrt(16)
helper_function()
```

**Estado Actual:**
- ✅ **Parser Manual (Rust)**: Parsear `import math` → `Stmt::Import` ✅
- ✅ **Parser Manual (Rust)**: Parsear `from utils import func` → `Stmt::Import` ✅
- ✅ **Module Resolver**: Resuelve archivos `.ad` ✅
- ✅ **AST**: Soporte para `Expr::Call { module: Some("math"), name: "sqrt" }` ✅
- ❌ **NASM Backend**: NO genera código NASM para módulos ❌
- ❌ **NASM Backend**: NO tiene linking de módulos en NASM ❌
- ❌ **NASM Backend**: NO genera namespaces (`math.sqrt` → `math_sqrt`) ❌

**Flujo Objetivo (NASM Directo):**
```
1. 📝 Parser Manual (Rust)
   └─> Detecta: import "math.ad"
   └─> Detecta: from "utils" import func1, func2
   └─> Resuelve archivos y parsea módulos

2. ⚡ Zig (Optimización - Futuro)
   └─> Lazy loading de módulos
   └─> Comptime linking

3. 🔒 Rust (Validación)
   └─> Type checking: funciones importadas existen?
   └─> Validación: sin conflictos de nombres
   └─> Análisis de dependencias circulares

4. 🎯 NASM Generator (NUEVO - Generación Directa)
   └─> Genera código NASM inline de módulos importados
   └─> O genera archivos NASM separados y linking
   └─> Namespace: `math.sqrt()` → función `math_sqrt` en NASM
   └─> Genera `extern math_sqrt` si es archivo separado
   └─> Genera `global math_sqrt` si es inline

5. 🔷 D (Metaprogramming - Futuro)
   └─> CTFE para módulos completos
   └─> Templates para codegen de módulos
```

**Implementación Pendiente:**
- [ ] **NASM Backend**: Generar código NASM inline de módulos importados
- [ ] **NASM Backend**: Generar archivos NASM separados para módulos
- [ ] **NASM Backend**: Sistema de linking de módulos en NASM
- [ ] **NASM Backend**: Generar `extern` para funciones de otros módulos
- [ ] **NASM Backend**: Generar `global` para funciones exportadas
- [ ] **NASM Backend**: Namespace: `math.sqrt()` → `math_sqrt` en NASM
- [ ] **NASM Backend**: Resolver conflictos de nombres
- [ ] **Module Resolver**: Prevenir dependencias circulares
- [ ] **Module Resolver**: Cache de módulos parseados

---

## 📋 Plan de Implementación por Sprint (NASM Directo)

### Sprint 1: Arrays en NASM Directo (2-3 semanas) 🎯 PRIORIDAD

**Fase 1.1: Estructura Array en NASM**
- [ ] Definir estructura Array en NASM (data, length, capacity)
- [ ] Implementar `array_new` en NASM (crear array vacío)
- [ ] Implementar `array_from_values` en NASM (crear desde valores)
- [ ] Implementar gestión de memoria (malloc/free en NASM)

**Fase 1.2: Operaciones Array en NASM**
- [ ] Implementar `array_get` en NASM (acceso por índice con bounds checking)
- [ ] Implementar `array_set` en NASM (asignación por índice con bounds checking)
- [ ] Implementar `array_append` en NASM (agregar elemento con realloc)
- [ ] Implementar `array_len` en NASM (obtener longitud)

**Fase 1.3: Generación NASM para ArrayLiteral e Index**
- [ ] Generar código NASM para `ArrayLiteral` → llamar `array_from_values`
- [ ] Generar código NASM para `Index` (lectura) → llamar `array_get`
- [ ] Generar código NASM para `Index` (asignación: `arr[0] = 5`) → llamar `array_set`

**Fase 1.4: Métodos Array estilo Python**
- [ ] Parser: Detectar `arr.append(x)` → `MethodCall`
- [ ] Parser: Detectar `arr.pop()` → `MethodCall`
- [ ] NASM Backend: Generar código NASM para `arr.append(x)`
- [ ] NASM Backend: Generar código NASM para `arr.pop()`
- [ ] Parser: Detectar `len(arr)` → función built-in
- [ ] NASM Backend: Generar código NASM para `len()` built-in

**Archivos a modificar:**
- `CORE/rust/crates/adead-backend/src/lib.rs` (agregar generación NASM para arrays)
- `CORE/rust/crates/adead-parser/src/lib.rs` (mejorar parser para métodos)

---

### Sprint 2: Strings Avanzados en NASM Directo (2-3 semanas)

**Fase 2.1: Estructura String Dinámica en NASM**
- [ ] Definir estructura String en NASM (data, length, capacity)
- [ ] Implementar `string_new` en NASM (crear string vacío)
- [ ] Implementar `string_from_literal` en NASM (crear desde literal)
- [ ] Implementar gestión de memoria para strings

**Fase 2.2: Operaciones String en NASM**
- [ ] Implementar `string_concat` en NASM (concatenación dinámica)
- [ ] Implementar `string_slice` en NASM (slicing: `s[0:4]`)
- [ ] Implementar `string_len` en NASM (obtener longitud)
- [ ] Implementar `string_upper` en NASM (conversión a mayúsculas)
- [ ] Implementar `string_lower` en NASM (conversión a minúsculas)

**Fase 2.3: Generación NASM para Concatenación y Slicing**
- [ ] AST: Agregar `Expr::Slice { start, end }`
- [ ] Parser: Detectar `s[0:4]` → `Slice`
- [ ] NASM Backend: Generar código NASM para concatenación (`s1 + s2`)
- [ ] NASM Backend: Generar código NASM para slicing (`s[0:4]`)

**Fase 2.4: Métodos String estilo Python**
- [ ] Parser: Detectar `s.upper()` → `MethodCall`
- [ ] Parser: Detectar `s.lower()` → `MethodCall`
- [ ] NASM Backend: Generar código NASM para métodos de string

**Archivos a modificar:**
- `CORE/rust/crates/adead-backend/src/lib.rs` (agregar generación NASM para strings avanzados)
- `CORE/rust/crates/adead-parser/src/lib.rs` (agregar `Expr::Slice`)

---

### Sprint 3: Funciones Completas en NASM Directo (2-3 semanas)

**Fase 3.1: Mejorar Stack Frame Management**
- [ ] Mejorar prologue/epilogue en NASM Backend
- [ ] Manejar shadow space correctamente (Windows: 32 bytes)
- [ ] Manejar stack alignment (16 bytes)
- [ ] Manejar local variables en stack

**Fase 3.2: Múltiples Parámetros**
- [ ] Manejar parámetros en registros (Windows: RCX, RDX, R8, R9)
- [ ] Manejar parámetros en stack (si > 4 parámetros)
- [ ] Generar código NASM para setup de parámetros

**Fase 3.3: Recursión y Validación**
- [ ] Manejar recursión profunda (stack overflow protection)
- [ ] Type checking de parámetros (validación en runtime)
- [ ] Validación de return types

**Fase 3.4: Indentación Estilo Python**
- [ ] Mejorar parser para manejar indentación estilo Python (4 espacios o tabs)
- [ ] Detectar fin de función por dedentación

**Archivos a modificar:**
- `CORE/rust/crates/adead-backend/src/lib.rs` (mejorar generación de funciones)
- `CORE/rust/crates/adead-parser/src/c_manual_parser.rs` (mejorar parser de funciones)

---

### Sprint 4: Módulos en NASM Directo (2 semanas)

**Fase 4.1: Generación NASM Inline de Módulos**
- [ ] Generar código NASM inline de módulos importados
- [ ] Generar namespaces: `math.sqrt()` → `math_sqrt` en NASM
- [ ] Resolver conflictos de nombres

**Fase 4.2: Linking de Módulos en NASM**
- [ ] Generar archivos NASM separados para módulos
- [ ] Generar `extern` para funciones de otros módulos
- [ ] Generar `global` para funciones exportadas
- [ ] Sistema de linking de módulos en NASM

**Fase 4.3: Validación y Testing**
- [ ] Prevenir dependencias circulares
- [ ] Validar que funciones importadas existen
- [ ] Tests con múltiples módulos

**Archivos a modificar:**
- `CORE/rust/crates/adead-backend/src/lib.rs` (agregar generación NASM para módulos)
- `CORE/rust/crates/adead-parser/src/module_resolver.rs` (mejorar resolución)

---

## 🎯 Resumen: Qué Falta para Python Style TOTAL

### ✅ Ya Implementado (vía C Generator):
- Parser completo (let, if/else, while, print, expresiones)
- Strings básicos
- Funciones básicas
- Arrays básicos (parser + C generator)
- Módulos básicos (parser + resolver)

### ❌ Falta Implementar (NASM Directo):

#### **Sprint 1 - Arrays (CRÍTICO):**
1. Estructura Array en NASM
2. Funciones helper en NASM (`array_get`, `array_set`, `array_append`, etc.)
3. Generación NASM para `ArrayLiteral`
4. Generación NASM para `Index` (lectura y asignación)
5. Métodos estilo Python (`arr.append()`, `arr.pop()`)
6. Built-in `len(arr)`

#### **Sprint 2 - Strings Avanzados:**
1. Estructura String dinámica en NASM
2. Concatenación dinámica en NASM (`s1 + s2`)
3. Slicing en NASM (`s[0:4]`)
4. Métodos de string (`s.upper()`, `s.lower()`)

#### **Sprint 3 - Funciones Completas:**
1. Stack frame management mejorado
2. Múltiples parámetros (> 4)
3. Recursión profunda
4. Indentación estilo Python

#### **Sprint 4 - Módulos:**
1. Generación NASM inline de módulos
2. Linking de módulos en NASM
3. Namespaces (`math.sqrt()` → `math_sqrt`)

---

## 🚀 Comenzando: Sprint 1 - Arrays en NASM Directo

**Orden de implementación:**
1. Crear estructura Array en NASM (data, length, capacity)
2. Implementar funciones helper en NASM (`array_new`, `array_get`, `array_set`, etc.)
3. Generar código NASM para `ArrayLiteral` → llamar `array_from_values`
4. Generar código NASM para `Index` → llamar `array_get` / `array_set`
5. Agregar métodos estilo Python (`arr.append()`, `arr.pop()`)
6. Agregar built-in `len(arr)`

**Archivo principal a modificar:**
- `CORE/rust/crates/adead-backend/src/lib.rs`

**Empecemos! 🎯**

---

**Última actualización:** Diciembre 2025  
**Estado:** Plan actualizado para reflejar estado real del proyecto  
**Objetivo:** Sintaxis Python Style → NASM Directo (sin pasar por C)
