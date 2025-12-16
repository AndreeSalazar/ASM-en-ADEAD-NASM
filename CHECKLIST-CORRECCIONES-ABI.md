# ✅ Checklist Completo: Correcciones ABI Windows x64

**Fecha:** Diciembre 2025  
**Objetivo:** Aplicar correcciones ABI-safe a todas las funciones helper

---

## 📋 Funciones Helper a Corregir

### Arrays (13 funciones)

- [x] `array_new` - ✅ CORREGIDO
- [x] `array_from_values` - ✅ CORREGIDO
- [x] `array_get` - ✅ CORREGIDO
- [x] `array_set` - ✅ CORREGIDO
- [x] `array_len` - ✅ CORREGIDO (simple, no necesita prologue completo)
- [x] `array_pop` - ✅ CORREGIDO
- [x] `array_append` - ✅ CORREGIDO
- [x] `array_reverse` - ✅ CORREGIDO
- [x] `array_insert` - ✅ CORREGIDO
- [x] `array_remove` - ✅ CORREGIDO (bug lógico también corregido)
- [x] `array_index` - ✅ CORREGIDO
- [x] `array_count` - ✅ CORREGIDO
- [x] `array_sort` - ✅ CORREGIDO

### Strings (7 funciones)

- [x] `string_new` - ✅ CORREGIDO
- [x] `string_from_literal` - ✅ CORREGIDO
- [x] `string_len` - ✅ CORREGIDO (simple, no necesita prologue completo)
- [x] `string_concat` - ✅ CORREGIDO
- [x] `string_slice` - ✅ CORREGIDO
- [x] `string_upper` - ✅ CORREGIDO
- [x] `string_lower` - ✅ CORREGIDO

---

## 🔧 Correcciones a Aplicar

### 1. Prologue ABI-Safe

**Reemplazar:**
```asm
push rbp
mov rbp, rsp
sub rsp, 32  ; shadow space
```

**Con:**
```rust
self.generate_abi_prologue(true/false);  // true si necesita shadow space
```

**Reglas:**
- `true` si la función llama a `VirtualAlloc`, `VirtualFree`, `ExitProcess`
- `false` si la función solo hace operaciones internas

---

### 2. Epilogue ABI-Safe

**Reemplazar:**
```asm
leave
ret
```

**Con:**
```rust
self.generate_abi_epilogue(true/false);  // mismo valor que prologue
```

---

### 3. Stack Alignment antes de Call

**Agregar antes de cada `call VirtualAlloc/VirtualFree`:**
```rust
self.ensure_stack_alignment_before_call("VirtualAlloc");
```

---

### 4. Usar Registros Preservados

**En lugar de `push/pop` para valores temporales, usar:**
- `r12`, `r13`, `r14`, `r15` - Registros preservados
- `rbx`, `rdi`, `rsi` - Registros preservados

**Ejemplo:**
```asm
; ❌ Antes:
push rax
; ... código ...
pop rax

; ✅ Después:
mov r12, rax  ; r12 está preservado
; ... código ...
mov rax, r12
```

---

## 📊 Progreso

**Arrays:** 13/13 completado (100%) ✅  
**Strings:** 7/7 completado (100%) ✅  
**Total:** 20/20 completado (100%) ✅

---

## ⚠️ Notas Importantes

1. **Funciones sin shadow space:**
   - `array_get`, `array_set`, `array_len`, `array_reverse`, `array_index`, `array_count`, `array_sort`
   - `string_len`
   - Usar `generate_abi_prologue(false)` y `generate_abi_epilogue(false)`

2. **Funciones con shadow space:**
   - Todas las que llaman a `VirtualAlloc` o `VirtualFree`
   - Usar `generate_abi_prologue(true)` y `generate_abi_epilogue(true)`

3. **Stack alignment:**
   - El prologue ya asegura alineación inicial
   - Si hay `push` adicionales antes de `call`, verificar alineación
   - Usar `ensure_stack_alignment_before_call()` antes de cada `call`

4. **Registros preservados:**
   - Preferir `r12-r15` para valores temporales
   - Evitar `push/pop` innecesarios que desalinean el stack

---

## 🎯 Próximos Pasos

1. ✅ Crear funciones helper ABI-safe
2. ✅ Corregir `array_new`
3. ✅ Corregir `array_from_values`
4. ✅ Continuar con funciones restantes de arrays
5. ✅ Corregir todas las funciones de strings
6. ✅ Verificar compilación
7. ⏳ Crear tests para verificar ABI compliance
8. ⏳ Decidir modelo de ownership para memory management

---

**Estado:** ✅ **COMPLETADO** (100% - 20/20 funciones corregidas)

