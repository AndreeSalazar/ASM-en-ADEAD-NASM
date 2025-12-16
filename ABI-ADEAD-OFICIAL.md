# ABI Oficial ADead - Windows x64

**Versión:** 1.0  
**Fecha:** Diciembre 2025  
**Plataforma:** Windows x64 (x86-64)

---

## 📋 Contrato de Llamadas (Calling Convention)

### Parámetros
- **Primeros 4 parámetros enteros:** RCX, RDX, R8, R9
- **Parámetros adicionales:** Stack (desde [RSP+32] hacia arriba, shadow space incluido)
- **Parámetros flotantes:** XMM0-XMM3 (primeros 4), luego stack

### Valor de Retorno
- **Enteros:** RAX
- **Punteros:** RAX
- **Flotantes:** XMM0
- **Estructuras grandes (> 8 bytes):** Puntero en RCX, datos en memoria

### Shadow Space
- **32 bytes obligatorios** antes de cada `call` a función externa
- Debe reservarse incluso si no se usan todos los parámetros

### Stack Alignment
- **RSP debe estar alineado a 16 bytes** antes de cada `call`
- Prologue de funciones asegura alineación inicial

---

## 🔒 Registros Preservados (Callee-Saved)

Las siguientes funciones **DEBEN** preservar estos registros:

| Registro | Uso |
|----------|-----|
| RBX | Preservar |
| RBP | Frame pointer (preservado automáticamente) |
| RDI | Preservar |
| RSI | Preservar |
| R12-R15 | Preservar |
| XMM6-XMM15 | Preservar (si se usan) |

**Registros Caller-Saved (no preservar):**
- RAX, RCX, RDX, R8-R11, XMM0-XMM5

---

## 📦 Estructuras de Datos

### Array (24 bytes)
```
[offset + 0]  : data (qword) - puntero a memoria dinámica (int64_t*)
[offset + 8]  : length (qword) - número de elementos
[offset + 16] : capacity (qword) - capacidad total
```

### String (32 bytes)
```
[offset + 0]  : data (qword) - puntero a memoria dinámica (char*)
[offset + 8]  : length (qword) - número de caracteres
[offset + 16] : capacity (qword) - capacidad total
[offset + 24] : hash (qword) - hash cacheado (0 = no calculado)
```

---

## ⚠️ Contrato de Errores

### Funciones que Retornan Valores

**Convención:**
- **Valores válidos:** Cualquier valor normal
- **Error:** Valores especiales con bit 63 activado (`0x8000000000000000` o superior)

**Ejemplos:**
- `array_get(arr, idx)` → Retorna valor o `0x8000000000000000` (índice fuera de rango)
- `array_pop(arr)` → Retorna valor o `0x8000000000000001` (array vacío)

### Funciones Void (Retornan Código de Estado)

**Convención:**
- **Éxito:** `RAX = 0`
- **Error:** `RAX = código negativo` (-1, -2, -3, etc.)

**Códigos de Error:**
- `-1`: Índice fuera de rango
- `-2`: Array/String vacío
- `-3`: Valor no encontrado
- `-4`: Fallo de memoria (VirtualAlloc falló)

**Ejemplos:**
- `array_set(arr, idx, val)` → Retorna `0` (éxito) o `-1` (error)
- `array_append(arr, val)` → Retorna `0` (éxito) o `-4` (error)
- `array_remove(arr, val)` → Retorna `0` (éxito) o `-3` (error)

### Funciones que Retornan Punteros

**Convención:**
- **Éxito:** Puntero válido (no NULL)
- **Error:** `NULL` (0)

**Ejemplos:**
- `string_slice(s, start, end)` → Retorna puntero a String o `NULL` (error)
- `array_new()` → Retorna puntero a Array o `NULL` (error de memoria)

---

## 🧹 Ownership y Memory Management

### Ownership Explícito

ADead usa **ownership explícito estilo Rust**:

- Cada Array/String tiene **un único dueño**
- El dueño es responsable de liberar la memoria
- Al salir de scope, se debe llamar a `free` explícitamente

### Funciones de Liberación

#### `array_free(Array* arr)`
- **Parámetros:** RCX = puntero al Array
- **Retorna:** RAX = 0 (éxito) o -4 (error)
- **Comportamiento:** Libera Array struct (24 bytes) + data buffer (capacity * 8 bytes)
- **Seguridad:** Liberar NULL es seguro (no-op, retorna 0)

#### `string_free(String* str)`
- **Parámetros:** RCX = puntero al String
- **Retorna:** RAX = 0 (éxito) o -4 (error)
- **Comportamiento:** Libera String struct (32 bytes) + data buffer (capacity bytes)
- **Seguridad:** Liberar NULL es seguro (no-op, retorna 0)

**Uso:**
```asm
mov rcx, arr_ptr
call array_free
test rax, rax
jnz error_handler
```

---

## 📐 Stack Frame Estándar

### Prologue ABI-Safe
```asm
function_name:
    push rbp
    mov rbp, rsp
    push rbx      ; preservar registros no volátiles
    push rdi
    push rsi
    push r12
    push r13
    push r14
    push r15
    sub rsp, 8    ; alinear stack (56 bytes % 16 = 8)
    sub rsp, 32   ; shadow space (si se llama a funciones externas)
```

### Epilogue ABI-Safe
```asm
    add rsp, 32   ; restaurar shadow space
    add rsp, 8    ; restaurar alineación
    pop r15
    pop r14
    pop r13
    pop r12
    pop rsi
    pop rdi
    pop rbx
    leave
    ret
```

---

## 🔍 Verificación de Stack Alignment

Antes de cada `call` a función externa:
1. Verificar que RSP esté alineado a 16 bytes
2. Si no, ajustar con `sub rsp, 8` o `add rsp, 8`

**Regla:** Después del prologue estándar, el stack está siempre alineado.

---

## 📝 Mutabilidad

### Funciones Read-Only
- `array_get`, `array_len`, `array_index`, `array_count`
- `string_len`
- No modifican el Array/String

### Funciones Mutadoras
- `array_set`, `array_append`, `array_pop`, `array_insert`, `array_remove`, `array_reverse`, `array_sort`
- Modifican el Array in-place

### Funciones Constructoras
- `array_new`, `array_from_values`
- `string_new`, `string_from_literal`
- Crean nuevos objetos en heap

### Funciones Transformadoras
- `string_concat`, `string_slice`, `string_upper`, `string_lower`
- Crean nuevos objetos (no modifican el original)

---

## ✅ Garantías ABI

1. **Stack siempre alineado** antes de `call`
2. **Registros preservados** según especificación
3. **Shadow space** siempre presente para funciones externas
4. **Errores nunca matan el proceso** (solo retornan códigos)
5. **Memory leaks prevenibles** con `free` explícito
6. **Liberar NULL es seguro** (no-op en `array_free`/`string_free`)
7. **Código "virgen y limpio"** sin metadata innecesaria

---

**Este documento es la especificación oficial del ABI ADead para Windows x64.**

