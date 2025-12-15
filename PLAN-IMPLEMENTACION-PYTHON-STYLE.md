# 🐍 Plan de Implementación: Sintaxis Estilo Python para ADead

## 🎯 Objetivo
Hacer que ADead tenga sintaxis **MUY similar a Python** usando los 5 componentes de la arquitectura pentágono de manera inteligente.

---

## 🏗️ Arquitectura por Característica

### 1️⃣ ARRAYS/LISTAS (Prioridad 1)

**Sintaxis Python que queremos:**
```python
arr = [1, 2, 3]
print(arr[0])        # 1
arr.append(4)
print(len(arr))      # 4
```

**Sintaxis ADead objetivo:**
```adead
let arr = [1, 2, 3]
print arr[0]
arr.append(4)
print len(arr)
```

**Flujo de los 5 componentes:**
```
1. 📝 Parser Manual (Rust)
   └─> Detecta: let arr = [1, 2, 3]
   └─> Genera: Expr::ArrayLiteral(vec![...])

2. ⚡ Zig (Opcional - Optimización)
   └─> Si array pequeño: Comptime evaluation
   └─> Optimiza acceso por índice

3. 🔒 Rust (Validación)
   └─> Type checking: todos elementos mismo tipo?
   └─> Validación: índices dentro de rango?

4. 🔧 C Generator (Generación)
   └─> Genera: int64_t arr[] = {1, 2, 3};
   └─> O: struct Array { int64_t* data; size_t len; }

5. 🔷 D (Metaprogramming - Futuro)
   └─> CTFE para arrays constantes
   └─> Templates para optimización
```

**Implementación:**
- ✅ AST ya tiene `ArrayLiteral` y `Index`
- ❌ Parser Manual necesita parsear `[1, 2, 3]`
- ❌ C Generator necesita generar código C para arrays
- ❌ Necesitamos estructura de datos Array en C

---

### 2️⃣ STRINGS REALES (Prioridad 1)

**Sintaxis Python que queremos:**
```python
s1 = "hola"
s2 = "mundo"
s3 = s1 + " " + s2    # "hola mundo"
print(s3[0:4])        # "hola"
print(len(s3))        # 11
```

**Sintaxis ADead objetivo:**
```adead
let s1 = "hola"
let s2 = "mundo"
let s3 = s1 + " " + s2
print s3[0:4]
print len(s3)
```

**Flujo de los 5 componentes:**
```
1. 📝 Parser Manual (Rust)
   └─> Detecta: let s = "texto"
   └─> Detecta: s1 + s2 (concatenación)
   └─> Genera: Expr::String(...) y Expr::BinaryOp { op: Add, ... }

2. ⚡ Zig (Opcional - Optimización)
   └─> String interning para literales duplicados
   └─> Optimización de concatenación

3. 🔒 Rust (Validación)
   └─> Type checking: strings son inmutables por defecto
   └─> Validación: índices válidos?

4. 🔧 C Generator (Generación)
   └─> Genera: char* s = "texto";
   └─> O: struct String { char* data; size_t len; }
   └─> Concatenación: strcat() o malloc + strcpy

5. 🔷 D (Metaprogramming - Futuro)
   └─> CTFE para strings constantes
   └─> Templates para optimización de string ops
```

**Implementación:**
- ✅ AST ya tiene `Expr::String(String)`
- ❌ Parser Manual necesita parsear strings en `let`
- ❌ C Generator necesita soporte para concatenación
- ❌ Necesitamos estructura de datos String en C

---

### 3️⃣ FUNCIONES (Prioridad 1)

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

**Flujo de los 5 componentes:**
```
1. 📝 Parser Manual (Rust)
   └─> Detecta: def nombre(param1, param2):
   └─> Parsea cuerpo con indentación estilo Python
   └─> Genera: Stmt::Fn { name, params, body }

2. ⚡ Zig (Opcional - Optimización)
   └─> Inlining de funciones pequeñas
   └─> Comptime evaluation cuando sea posible

3. 🔒 Rust (Validación)
   └─> Type checking: parámetros y return types
   └─> Validación: todas las rutas retornan?
   └─> Análisis de recursión

4. 🔧 C Generator (Generación)
   └─> Genera: int64_t factorial(int64_t n) { ... }
   └─> Genera llamadas: factorial(5)

5. 🔷 D (Metaprogramming - Futuro)
   └─> CTFE para funciones constantes
   └─> Templates para funciones genéricas
```

**Implementación:**
- ✅ AST ya tiene `Stmt::Fn` y `Expr::Call`
- ⚠️ Parser Manual necesita parsear `def` con indentación Python
- ⚠️ C Generator tiene código pero no está verificado
- ❌ Necesitamos soporte para indentación estilo Python

---

### 4️⃣ MÓDULOS (Prioridad 1)

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

**Flujo de los 5 componentes:**
```
1. 📝 Parser Manual (Rust)
   └─> Detecta: import "math.ad"
   └─> Detecta: from "utils" import func1, func2
   └─> Resuelve archivos y parsea módulos

2. ⚡ Zig (Opcional - Optimización)
   └─> Lazy loading de módulos
   └─> Comptime linking

3. 🔒 Rust (Validación)
   └─> Type checking: funciones importadas existen?
   └─> Validación: sin conflictos de nombres
   └─> Análisis de dependencias circulares

4. 🔧 C Generator (Generación)
   └─> Genera: #include "math.h" o código inline
   └─> Genera: math_sqrt(16) o math.sqrt(16)

5. 🔷 D (Metaprogramming - Futuro)
   └─> CTFE para módulos completos
   └─> Templates para codegen de módulos
```

**Implementación:**
- ⚠️ AST tiene soporte básico para módulos
- ❌ Parser Manual necesita parsear `import` y `from import`
- ❌ Sistema de resolución de módulos
- ❌ C Generator necesita generar includes o código inline

---

## 📋 Plan de Implementación por Sprint

### Sprint 1: Arrays (1-2 semanas)

**Fase 1.1: Parser Manual para Arrays**
- [ ] Parsear `[1, 2, 3]` → `ArrayLiteral`
- [ ] Parsear `arr[0]` → `Index`
- [ ] Parsear `arr[i]` con variable como índice

**Fase 1.2: C Generator para Arrays**
- [ ] Generar `struct Array { int64_t* data; size_t len; }`
- [ ] Generar inicialización: `[1, 2, 3]`
- [ ] Generar acceso: `arr[0]`
- [ ] Generar asignación: `arr[0] = 5`

**Fase 1.3: Funciones Array Básicas**
- [ ] `len(arr)` → función helper en C
- [ ] `arr.append(x)` → función helper en C
- [ ] `arr.pop()` → función helper en C

---

### Sprint 2: Strings Reales (1-2 semanas)

**Fase 2.1: Parser Manual para Strings**
- [ ] Parsear `let s = "texto"` → asignar String a variable
- [ ] Parsear `s1 + s2` → concatenación
- [ ] Parsear `s[0:4]` → substring (futuro)

**Fase 2.2: C Generator para Strings**
- [ ] Generar `struct String { char* data; size_t len; }`
- [ ] Generar asignación: `let s = "texto"`
- [ ] Generar concatenación: `s1 + s2`
- [ ] Generar acceso: `s[0]` (carácter)

**Fase 2.3: Funciones String Básicas**
- [ ] `len(s)` → función helper en C
- [ ] `s.substring(start, end)` → función helper en C

---

### Sprint 3: Funciones Completas (2-3 semanas)

**Fase 3.1: Parser Manual con Indentación Python**
- [ ] Parsear `def nombre(param1, param2):`
- [ ] Manejar indentación estilo Python (4 espacios o tabs)
- [ ] Parsear `return valor`
- [ ] Detectar fin de función por dedentación

**Fase 3.2: C Generator para Funciones**
- [ ] Generar `int64_t nombre(int64_t param1, int64_t param2) { ... }`
- [ ] Generar llamadas: `nombre(arg1, arg2)`
- [ ] Generar return statements

**Fase 3.3: Validación y Testing**
- [ ] Type checking básico
- [ ] Validación de parámetros
- [ ] Tests con funciones recursivas

---

### Sprint 4: Módulos (2 semanas)

**Fase 4.1: Sistema de Resolución**
- [ ] Resolver `import "file.ad"` → leer archivo
- [ ] Resolver `from "utils" import func` → importar funciones específicas
- [ ] Cache de módulos parseados

**Fase 4.2: Generación C con Módulos**
- [ ] Generar código inline de módulos importados
- [ ] O generar includes si es archivo separado
- [ ] Namespace: `math.sqrt()` → `math_sqrt()`

**Fase 4.3: Validación**
- [ ] Detectar conflictos de nombres
- [ ] Validar que funciones importadas existen
- [ ] Prevenir dependencias circulares

---

## 🚀 Comenzando: Sprint 1 - Arrays

**Orden de implementación:**
1. Extender Parser Manual para parsear arrays
2. Extender C Generator para generar código C de arrays
3. Testing con ejemplos simples
4. Agregar funciones helper (`len`, `append`, `pop`)

**Empecemos! 🎯**

