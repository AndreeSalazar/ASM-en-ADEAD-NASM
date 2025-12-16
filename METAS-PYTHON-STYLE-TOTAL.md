# 🎯 Metas: Python Style TOTAL → NASM Directo

**Fecha:** Diciembre 2025  
**Autor:** Eddi Andreé Salazar Matos  
**Objetivo:** Sintaxis estilo Python que genere NASM puro directamente

---

## 📋 Objetivo Principal

**Hacer que ADead tenga sintaxis estilo Python y genere código NASM puro directamente**, facilitando el proceso de compilación y obteniendo ASM optimizado sin capas intermedias.

**Pipeline Objetivo:**
```
ADead → NASM Directo → ASM Final
```

**Pipeline Actual:**
```
ADead → C++ Generator → GCC++/Clang++ → Rust Cleaner → ASM
```

---

## 🐍 ¿Qué es "Python Style TOTAL"?

Sintaxis simple y expresiva similar a Python que hace el código fácil de escribir y leer:

```python
# Python
arr = [1, 2, 3]
arr.append(4)
print(arr[0])
print(len(arr))

s1 = "hola"
s2 = "mundo"
s3 = s1 + " " + s2
print(s3[0:4])
print(s3.upper())
```

```ad
# ADead (objetivo Python Style TOTAL)
let arr = [1, 2, 3]
arr.append(4)
print arr[0]
print len(arr)

let s1 = "hola"
let s2 = "mundo"
let s3 = s1 + " " + s2
print s3[0:4]
print s3.upper()
```

**Ventajas:**
- ✅ Sintaxis familiar (cualquiera que conozca Python puede usar ADead)
- ✅ Código más legible y expresivo
- ✅ Menos verbosidad que C/C++
- ✅ Facilita el desarrollo rápido
- ✅ Genera NASM directo (sin capas intermedias)

---

## 🎯 Objetivos Claros para Aplicar

### **Objetivo 1: Arrays en NASM Directo** 🔥 PRIORIDAD ALTA

**Estado Actual:**
- ✅ Parser funciona: `let arr = [1, 2, 3]`
- ✅ Genera NASM directamente (parcialmente)
- ⚠️ Algunos métodos faltan

**Objetivo:**
- [x] Generar estructura Array en NASM (data, length, capacity) ✅
- [x] Funciones helper en NASM: `array_get`, `array_set`, `array_append` ✅
- [x] Generar código NASM para `arr[0]` → llamar `array_get` ✅
- [x] Generar código NASM para `arr[0] = 5` → llamar `array_set` ✅
- [x] Generar código NASM para `arr.append(4)` → llamar `array_append` ✅
- [x] Generar código NASM para `len(arr)` → función built-in ✅
- [x] Generar código NASM para `arr.insert(i, x)` → llamar `array_insert` ✅
- [x] Generar código NASM para `arr.remove(x)` → llamar `array_remove` ✅
- [x] Generar código NASM para `arr.index(x)` → llamar `array_index` ✅
- [x] Generar código NASM para `arr.count(x)` → llamar `array_count` ✅
- [x] Generar código NASM para `arr.sort()` → llamar `array_sort` ✅
- [x] Generar código NASM para `arr.reverse()` → llamar `array_reverse` ✅

**Resultado Esperado:**
```ad
let arr = [1, 2, 3]
arr.append(4)
arr.insert(0, 0)
arr.sort()
arr.reverse()
print arr[0]
print len(arr)
```
↓ Genera NASM directo sin pasar por C++

**Progreso:** ✅ **100% completado** (10/10 métodos) - **COMPLETADO**

---

### **Objetivo 2: Strings Avanzados en NASM Directo** 🔥 PRIORIDAD ALTA

**Estado Actual:**
- ✅ Strings básicos funcionan
- ✅ Genera strings en `.data` section
- ✅ Concatenación dinámica en NASM (`s1 + s2`)
- ✅ Slicing `s[0:4]` en NASM
- ✅ Métodos `s.upper()`, `s.lower()` en NASM
- ✅ `len(s)` en NASM

**Objetivo:**
- [x] Estructura String dinámica en NASM ✅
- [x] Función `string_concat` en NASM (`s1 + s2`) ✅
- [x] Función `string_slice` en NASM (`s[0:4]`) ✅
- [x] Métodos: `s.upper()`, `s.lower()` en NASM ✅
- [x] Métodos: `len(s)` en NASM ✅
- [x] Generar código NASM para concatenación y slicing ✅
- [x] Detección automática de strings en CLI ✅
- [x] Mejora en `is_string_expr` ✅

**Resultado Esperado:**
```ad
let s1 = "hola"
let s2 = "mundo"
let s3 = s1 + " " + s2
print s3[0:4]
print s3.upper()
print len(s3)
```
↓ Genera NASM directo sin pasar por C++ ✅

**Progreso:** ✅ **100% completado** (8/8 funcionalidades) - **COMPLETADO**

---

### **Objetivo 3: Funciones Completas en NASM Directo** ⚡ PRIORIDAD MEDIA

**Estado Actual:**
- ✅ Funciones básicas funcionan
- ✅ Stack frames básicos
- ⚠️ NO maneja múltiples parámetros correctamente
- ⚠️ NO maneja recursión profunda

**Objetivo:**
- [ ] Mejorar stack frame management (prologue/epilogue)
- [ ] Manejar múltiples parámetros (> 4) en stack
- [ ] Manejar shadow space (Windows: 32 bytes)
- [ ] Manejar stack alignment (16 bytes)
- [ ] Manejar recursión profunda
- [ ] Optimizar llamadas de función

**Resultado Esperado:**
```ad
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

let result = factorial(5)
```
↓ Genera NASM directo con stack frames correctos

**Progreso:** 40% completado (2/5 funcionalidades)

---

### **Objetivo 4: Módulos en NASM Directo** ⚡ PRIORIDAD MEDIA

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
- [ ] Resolver dependencias de módulos

**Resultado Esperado:**
```ad
import math
let result = math.sqrt(16)
```
↓ Genera NASM directo con módulos linkeados

**Progreso:** 0% completado (0/5 funcionalidades)

---

## 📊 Plan de Implementación (Orden de Prioridad)

### **Fase 1: Arrays Completos** ✅ COMPLETADO

1. ✅ Implementar `array_insert` en NASM
2. ✅ Implementar `array_remove` en NASM
3. ✅ Implementar `array_index` en NASM
4. ✅ Implementar `array_count` en NASM
5. ✅ Implementar `array_sort` en NASM
6. ✅ Implementar `array_reverse` en NASM (ya existía)
7. ✅ Generación NASM para métodos faltantes
8. ✅ Testing completo (6/6 tests pasan)

**Progreso:** ✅ **100% completado** - **COMPLETADO**

---

### **Fase 2: Strings Avanzados** ✅ COMPLETADO

1. ✅ Estructura String dinámica en NASM (ya existía)
2. ✅ Concatenación dinámica (`s1 + s2`)
3. ✅ Slicing (`s[0:4]`)
4. ✅ Métodos (`s.upper()`, `s.lower()`, `len(s)`)
5. ✅ Detección automática de strings en CLI
6. ✅ Mejora en `is_string_expr`
7. ✅ Testing completo (6/6 tests pasan, todos generan NASM directo)

**Progreso:** ✅ **100% completado** - **COMPLETADO**

---

### **Fase 3: Funciones Completas (2-3 semanas)** ⚡

1. Stack frame management mejorado
2. Múltiples parámetros
3. Recursión profunda
4. Optimizaciones

**Progreso:** 40% completado

---

### **Fase 4: Módulos (2 semanas)** ⚡

1. Generación NASM inline de módulos
2. Linking de módulos
3. Namespaces
4. Resolución de dependencias

**Progreso:** 0% completado

---

## ✅ Criterios de Éxito

### **Para Arrays:**
- ✅ `let arr = [1, 2, 3]` genera NASM directo
- ✅ `arr[0]` genera llamada a `array_get`
- ✅ `arr[0] = 5` genera llamada a `array_set`
- ✅ `arr.append(4)` genera llamada a `array_append`
- ✅ `len(arr)` genera llamada a función built-in
- ✅ `arr.insert(i, x)` genera llamada a `array_insert`
- ✅ `arr.remove(x)` genera llamada a `array_remove`
- ✅ `arr.index(x)` genera llamada a `array_index`
- ✅ `arr.count(x)` genera llamada a `array_count`
- ✅ `arr.sort()` genera llamada a `array_sort`
- ✅ `arr.reverse()` genera llamada a `array_reverse`

### **Para Strings:**
- [ ] `s1 + s2` genera llamada a `string_concat`
- [ ] `s[0:4]` genera llamada a `string_slice`
- [ ] `s.upper()` genera llamada a `string_upper`
- [ ] `s.lower()` genera llamada a `string_lower`
- [ ] `s.len()` genera llamada a función built-in

### **Para Funciones:**
- [ ] Funciones con múltiples parámetros funcionan
- [ ] Recursión funciona correctamente
- [ ] Stack frames son correctos
- [ ] Shadow space manejado correctamente

### **Para Módulos:**
- [ ] `import math` genera código NASM inline
- [ ] `math.sqrt()` genera llamada a `math_sqrt`
- [ ] Linking funciona correctamente
- [ ] Dependencias resueltas correctamente

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
- `CORE/rust/crates/adead-parser/src/lib.rs` - Parser

### **Para Módulos:**
- `CORE/rust/crates/adead-backend/src/lib.rs` - Generación NASM
- `CORE/rust/crates/adead-parser/src/module_resolver.rs` - Resolución

---

## 🎯 Resumen Ejecutivo

**Objetivo:** Sintaxis Python Style TOTAL → NASM Directo

**Beneficios:**
- ✅ Código más fácil de escribir (sintaxis Python)
- ✅ Código más fácil de entender (NASM directo)
- ✅ Mejor control sobre el ASM generado
- ✅ Proceso más rápido (sin capas intermedias)
- ✅ Sin dependencias externas (solo NASM)

**Prioridad:**
1. 🔥 **Arrays completos** (60% completado - faltan 4 métodos)
2. 🔥 **Strings avanzados** (0% completado - crítico)
3. ⚡ **Funciones completas** (40% completado)
4. ⚡ **Módulos** (0% completado)

**Tiempo Estimado:** 6-10 semanas para implementación completa

**Progreso Total:** ~50% completado (Fase 1 y Fase 2 completadas: Arrays 100%, Strings 100%)

---

## 🚀 Comenzar Ahora

**Paso 1:** Completar métodos de arrays faltantes  
**Paso 2:** Implementar strings avanzados en NASM  
**Paso 3:** Mejorar funciones  
**Paso 4:** Implementar módulos

**Archivo principal:** `CORE/rust/crates/adead-backend/src/lib.rs`

---

**Última actualización:** Diciembre 2025  
**Estado:** Plan claro para Python Style TOTAL  
**Objetivo:** Python Style → NASM Directo

