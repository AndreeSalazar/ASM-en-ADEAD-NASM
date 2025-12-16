# Ownership y Reglas de Liberación de Memoria - ADead

**Fecha:** Diciembre 2025  
**Estado:** ✅ **OFICIAL**

---

## 🎯 Regla de Ownership

**ADead usa ownership explícito estilo Rust, pero manual:**

### Principios

1. **Cada valor tiene UN dueño**
2. **El dueño es responsable de liberar la memoria**
3. **Transferencia de ownership es explícita**

---

## 📋 Reglas por Tipo

### Arrays

**Creación:**
```ad
let arr = [1, 2, 3]  // Ownership: variable 'arr'
```

**Operaciones que CREAN nuevo array (nuevo ownership):**
- `arr1 + arr2` → Nuevo array, caller debe liberar
- `arr.slice(0, 5)` → Nuevo array, caller debe liberar
- `arr.copy()` → Nuevo array, caller debe liberar

**Operaciones que MUTAN array existente (mismo ownership):**
- `arr.append(x)` → Mismo array, no cambia ownership
- `arr.remove(x)` → Mismo array, no cambia ownership
- `arr.sort()` → Mismo array, no cambia ownership

**Liberación:**
```ad
array_free(arr)  // Libera memoria del array
```

### Strings

**Creación:**
```ad
let s = "hola"  // Ownership: variable 's'
let s2 = s1 + s2  // Nuevo string, caller debe liberar
```

**Operaciones que CREAN nuevo string (nuevo ownership):**
- `s1 + s2` → Nuevo string, caller debe liberar
- `s[0:4]` → Nuevo string, caller debe liberar
- `s.upper()` → Nuevo string, caller debe liberar
- `s.lower()` → Nuevo string, caller debe liberar

**Operaciones que NO crean nuevo string:**
- `len(s)` → No crea nuevo string, solo lectura
- `s[0]` → No crea nuevo string, solo lectura

**Liberación:**
```ad
string_free(s)  // Libera memoria del string
```

---

## 🔄 Transferencia de Ownership

### Asignación
```ad
let a = [1, 2, 3]  // Ownership: 'a'
let b = a  // Ownership transferido a 'b', 'a' ya no es válido
// 'a' no debe usarse después de esto
```

### Pasar a Función
```ad
fn procesar(arr):
    // Ownership transferido a función
    // La función es responsable de liberar
    array_free(arr)

let arr = [1, 2, 3]
procesar(arr)  // Ownership transferido
// 'arr' ya no es válido después de esto
```

### Retornar de Función
```ad
fn crear_array():
    return [1, 2, 3]  // Ownership transferido al caller

let arr = crear_array()  // Ownership: 'arr'
// Caller debe liberar 'arr'
```

---

## ⚠️ Memory Leaks Comunes

### ❌ INCORRECTO
```ad
let s1 = "hola"
let s2 = "mundo"
let s3 = s1 + s2  // s3 es nuevo string
// LEAK: s3 nunca se libera
```

### ✅ CORRECTO
```ad
let s1 = "hola"
let s2 = "mundo"
let s3 = s1 + s2
print s3
string_free(s3)  // Liberar explícitamente
```

---

## 📝 Convenciones de Nombres

### Funciones que Crean Nuevos Valores
- Prefijo: `new_`, `create_`, `from_`
- Ejemplo: `array_new()`, `string_from_literal()`

### Funciones que Mutan Valores Existentes
- Sin prefijo especial
- Ejemplo: `array_append()`, `array_sort()`

### Funciones que Liberan Memoria
- Prefijo: `free_`
- Ejemplo: `array_free()`, `string_free()`

---

## 🔍 Verificación de Ownership

### En Código Generado
```asm
; ADead: let s3 = s1 + s2
; Ownership: s3 es nuevo string, caller debe liberar
call string_concat
mov [rbp - 24], rax  ; guardar s3
; ...
; Antes de salir de scope:
mov rcx, [rbp - 24]  ; cargar s3
call string_free  ; liberar s3
```

---

## ✅ Checklist de Ownership

- [x] Arrays: `array_free()` implementado
- [x] Strings: `string_free()` implementado
- [x] Documentación de ownership creada
- [ ] RAII automático (futuro)
- [ ] Verificación en compile-time (futuro)

---

**Esta documentación es OBLIGATORIA para entender el manejo de memoria en ADead.**

