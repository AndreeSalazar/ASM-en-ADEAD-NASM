# 🎯 Meta: Python Style → NASM Directo

## 📋 Objetivo Principal

**Hacer que ADead tenga sintaxis estilo Python y genere código NASM puro directamente**, facilitando el proceso de compilación y obteniendo ASM optimizado sin capas intermedias.

---

## 🐍 ¿Qué es "Python Style"?

Sintaxis simple y expresiva similar a Python que hace el código fácil de escribir y leer:

```python
# Python
arr = [1, 2, 3]
arr.append(4)
print(arr[0])
print(len(arr))
```

```ad
# ADead (objetivo)
let arr = [1, 2, 3]
arr.append(4)
print arr[0]
print len(arr)
```

**Ventajas:**
- ✅ Sintaxis familiar (cualquiera que conozca Python puede usar ADead)
- ✅ Código más legible y expresivo
- ✅ Menos verbosidad que C/C++
- ✅ Facilita el desarrollo rápido

---

## ⚡ ¿Por Qué NASM Directo?

### **Problema Actual:**
```
ADead → C++ → GCC → ASM → Rust Cleaner → ASM Final
```

**Desventajas:**
- ❌ Múltiples capas intermedias
- ❌ Código generado más complejo
- ❌ Menos control sobre el ASM final
- ❌ Dependencia de compilador C++

### **Solución Objetivo:**
```
ADead → NASM Directo → ASM Final
```

**Ventajas:**
- ✅ Control total sobre el ASM generado
- ✅ Código más limpio y optimizado
- ✅ Sin dependencias externas
- ✅ Proceso más rápido y directo
- ✅ Mejor para entender cómo funciona el código

---

## 🎯 Objetivos Claros para Aplicar

### **Objetivo 1: Arrays en NASM Directo** 🎯 PRIORIDAD ALTA

**Estado Actual:**
- ✅ Parser funciona: `let arr = [1, 2, 3]`
- ✅ Genera C++ con `std::vector`
- ❌ NO genera NASM directamente

**Objetivo:**
- [ ] Generar estructura Array en NASM (data, length, capacity)
- [ ] Funciones helper en NASM: `array_get`, `array_set`, `array_append`
- [ ] Generar código NASM para `arr[0]` → llamar `array_get`
- [ ] Generar código NASM para `arr[0] = 5` → llamar `array_set`
- [ ] Generar código NASM para `arr.append(4)` → llamar `array_append`
- [ ] Generar código NASM para `len(arr)` → función built-in

**Resultado Esperado:**
```ad
let arr = [1, 2, 3]
arr.append(4)
print arr[0]
print len(arr)
```
↓ Genera NASM directo sin pasar por C++

---

### **Objetivo 2: Strings Avanzados en NASM Directo** 🎯 PRIORIDAD ALTA

**Estado Actual:**
- ✅ Strings básicos funcionan
- ✅ Genera strings en `.data` section
- ❌ NO tiene concatenación dinámica
- ❌ NO tiene slicing `s[0:4]`
- ❌ NO tiene métodos `s.upper()`

**Objetivo:**
- [ ] Estructura String dinámica en NASM
- [ ] Función `string_concat` en NASM (`s1 + s2`)
- [ ] Función `string_slice` en NASM (`s[0:4]`)
- [ ] Métodos: `s.upper()`, `s.lower()` en NASM
- [ ] Generar código NASM para concatenación y slicing

**Resultado Esperado:**
```ad
let s1 = "hola"
let s2 = "mundo"
let s3 = s1 + " " + s2
print s3[0:4]
print s3.upper()
```
↓ Genera NASM directo sin pasar por C++

---

### **Objetivo 3: Funciones Completas en NASM Directo** 🎯 PRIORIDAD MEDIA

**Estado Actual:**
- ✅ Funciones básicas funcionan
- ✅ Stack frames básicos
- ❌ NO maneja múltiples parámetros correctamente
- ❌ NO maneja recursión profunda

**Objetivo:**
- [ ] Mejorar stack frame management (prologue/epilogue)
- [ ] Manejar múltiples parámetros (> 4) en stack
- [ ] Manejar shadow space (Windows: 32 bytes)
- [ ] Manejar stack alignment (16 bytes)
- [ ] Manejar recursión profunda

**Resultado Esperado:**
```ad
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

let result = factorial(5)
```
↓ Genera NASM directo con stack frames correctos

---

### **Objetivo 4: Módulos en NASM Directo** 🎯 PRIORIDAD MEDIA

**Estado Actual:**
- ✅ Parser funciona: `import math`
- ✅ Resuelve archivos `.ad`
- ❌ NO genera código NASM para módulos
- ❌ NO tiene linking de módulos

**Objetivo:**
- [ ] Generar código NASM inline de módulos importados
- [ ] Generar namespaces: `math.sqrt()` → `math_sqrt` en NASM
- [ ] Sistema de linking de módulos en NASM
- [ ] Generar `extern` y `global` correctamente

**Resultado Esperado:**
```ad
import math
let result = math.sqrt(16)
```
↓ Genera NASM directo con módulos linkeados

---

## 📊 Plan de Implementación (Orden de Prioridad)

### **Fase 1: Arrays (2-3 semanas)** 🔥 CRÍTICO
1. Estructura Array en NASM
2. Funciones helper (`array_get`, `array_set`, `array_append`)
3. Generación NASM para `ArrayLiteral` e `Index`
4. Métodos estilo Python (`arr.append()`, `arr.pop()`)
5. Built-in `len(arr)`

### **Fase 2: Strings Avanzados (2-3 semanas)** 🔥 CRÍTICO
1. Estructura String dinámica en NASM
2. Concatenación dinámica (`s1 + s2`)
3. Slicing (`s[0:4]`)
4. Métodos (`s.upper()`, `s.lower()`)

### **Fase 3: Funciones Completas (2-3 semanas)**
1. Stack frame management mejorado
2. Múltiples parámetros
3. Recursión profunda

### **Fase 4: Módulos (2 semanas)**
1. Generación NASM inline de módulos
2. Linking de módulos
3. Namespaces

---

## 🚀 Cómo Facilitar el Proceso en NASM

### **1. Estructuras de Datos en NASM**

En lugar de generar código C++ complejo, generar estructuras simples en NASM:

```asm
; Estructura Array en NASM
array_data:     dq 0        ; Puntero a datos
array_length:   dq 0        ; Longitud actual
array_capacity: dq 0        ; Capacidad total
```

### **2. Funciones Helper en NASM**

Funciones reutilizables que simplifican el código generado:

```asm
; array_get: Obtener elemento por índice
array_get:
    ; RAX = array, RBX = index
    ; Retorna valor en RAX
    ; ...
    ret

; array_set: Asignar elemento por índice
array_set:
    ; RAX = array, RBX = index, RCX = value
    ; ...
    ret
```

### **3. Generación Directa**

El generador NASM crea código directamente desde el AST:

```rust
// AST: Expr::Index { array: "arr", index: 0 }
// Genera NASM:
// mov rax, arr
// mov rbx, 0
// call array_get
```

**Ventaja:** Control total sobre el código generado.

---

## ✅ Criterios de Éxito

### **Para Arrays:**
- ✅ `let arr = [1, 2, 3]` genera NASM directo
- ✅ `arr[0]` genera llamada a `array_get`
- ✅ `arr[0] = 5` genera llamada a `array_set`
- ✅ `arr.append(4)` genera llamada a `array_append`
- ✅ `len(arr)` genera llamada a función built-in

### **Para Strings:**
- ✅ `s1 + s2` genera llamada a `string_concat`
- ✅ `s[0:4]` genera llamada a `string_slice`
- ✅ `s.upper()` genera llamada a `string_upper`

### **Para Funciones:**
- ✅ Funciones con múltiples parámetros funcionan
- ✅ Recursión funciona correctamente
- ✅ Stack frames son correctos

### **Para Módulos:**
- ✅ `import math` genera código NASM inline
- ✅ `math.sqrt()` genera llamada a `math_sqrt`
- ✅ Linking funciona correctamente

---

## 📝 Archivos Clave a Modificar

### **Para Arrays:**
- `CORE/rust/crates/adead-backend/src/lib.rs` - Generación NASM
- `CORE/rust/crates/adead-parser/src/lib.rs` - Parser de métodos

### **Para Strings:**
- `CORE/rust/crates/adead-backend/src/lib.rs` - Generación NASM
- `CORE/rust/crates/adead-parser/src/lib.rs` - Parser de slicing

### **Para Funciones:**
- `CORE/rust/crates/adead-backend/src/lib.rs` - Stack frames
- `CORE/rust/crates/adead-parser/src/c_manual_parser.rs` - Parser

### **Para Módulos:**
- `CORE/rust/crates/adead-backend/src/lib.rs` - Generación NASM
- `CORE/rust/crates/adead-parser/src/module_resolver.rs` - Resolución

---

## 🎯 Resumen Ejecutivo

**Objetivo:** Sintaxis Python Style → NASM Directo

**Beneficios:**
- ✅ Código más fácil de escribir (sintaxis Python)
- ✅ Código más fácil de entender (NASM directo)
- ✅ Mejor control sobre el ASM generado
- ✅ Proceso más rápido (sin capas intermedias)
- ✅ Sin dependencias externas (solo NASM)

**Prioridad:**
1. 🔥 **Arrays** (crítico - base para todo)
2. 🔥 **Strings avanzados** (crítico - uso común)
3. ⚡ **Funciones completas** (importante)
4. ⚡ **Módulos** (importante)

**Tiempo Estimado:** 8-12 semanas para implementación completa

---

## 🚀 Comenzar Ahora

**Paso 1:** Implementar estructura Array en NASM  
**Paso 2:** Implementar funciones helper (`array_get`, `array_set`)  
**Paso 3:** Generar código NASM para `ArrayLiteral` e `Index`  
**Paso 4:** Agregar métodos estilo Python (`arr.append()`)  

**Archivo principal:** `CORE/rust/crates/adead-backend/src/lib.rs`

---

**Última actualización:** Diciembre 2025  
**Estado:** Plan claro para implementación  
**Objetivo:** Python Style → NASM Directo

