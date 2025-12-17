# Convención de Errores Unificada - ADead

**Fecha:** Diciembre 2025  
**Estado:** ✅ **OFICIAL**

---

## 🎯 Regla de Oro

**TODAS las funciones siguen esta convención:**

### Para Funciones que Retornan Valores (int64_t, punteros)

```asm
; Éxito: RAX = valor válido
; Error: RAX = NULL (0) para punteros, o valor especial para números
```

**Específicamente:**
- **Punteros:** `NULL` (0) = error
- **Números:** `0x8000000000000000` (bit 63 activado) = error base
  - `0x8000000000000000` = error genérico
  - `0x8000000000000001` = índice fuera de rango
  - `0x8000000000000002` = array vacío
  - `0x8000000000000003` = valor no encontrado
  - `0x8000000000000004` = error de memoria

### Para Funciones Void (no retornan valor)

```asm
; Éxito: RAX = 0
; Error: RAX = código negativo
```

**Códigos de error:**
- `0` = éxito
- `-1` = índice fuera de rango
- `-2` = array vacío
- `-3` = valor no encontrado
- `-4` = error de memoria
- `-5` = puntero inválido

---

## 📋 Tabla de Códigos de Error

| Código | Significado | Tipo |
|--------|-------------|------|
| `0` | Éxito | Void functions |
| `NULL` (0) | Error | Pointer functions |
| `0x8000000000000000` | Error genérico | Value functions |
| `0x8000000000000001` | Índice fuera de rango | Value functions |
| `0x8000000000000002` | Array vacío | Value functions |
| `0x8000000000000003` | Valor no encontrado | Value functions |
| `0x8000000000000004` | Error de memoria | Value functions |
| `-1` | Índice fuera de rango | Void functions |
| `-2` | Array vacío | Void functions |
| `-3` | Valor no encontrado | Void functions |
| `-4` | Error de memoria | Void functions |
| `-5` | Puntero inválido | Void functions |

---

## ✅ Ejemplos

### Función que Retorna Puntero
```asm
array_new:
    ; Retorna: RAX = puntero al Array, o NULL (0) si error
    ; ...
    test rax, rax
    jz .error
    ret
.error:
    mov rax, 0  ; NULL = error
    ret
```

### Función que Retorna Valor
```asm
array_get:
    ; Retorna: RAX = valor, o 0x8000000000000001 si índice fuera de rango
    ; ...
    cmp rdx, [rcx + 8]  ; comparar índice con length
    jge .error
    mov rax, [r8]  ; valor
    ret
.error:
    mov rax, 0x8000000000000001  ; error: índice fuera de rango
    ret
```

### Función Void
```asm
array_set:
    ; Retorna: RAX = 0 (éxito) o -1 (error: índice fuera de rango)
    ; ...
    cmp rdx, [rcx + 8]
    jge .error
    mov [r8], r9  ; establecer valor
    mov rax, 0  ; éxito
    ret
.error:
    mov rax, -1  ; error: índice fuera de rango
    ret
```

---

## 🔍 Verificación de Errores

### En Código ADead (futuro)
```ad
let arr = array_new()
if arr == NULL:
    panic("Error: no se pudo crear array")

let val = array_get(arr, 5)
if val & 0x8000000000000000 != 0:
    panic("Error: índice fuera de rango")
```

---

**Esta convención es OBLIGATORIA para todas las funciones helper.**

