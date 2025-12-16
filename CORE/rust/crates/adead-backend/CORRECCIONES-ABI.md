# 🔧 Correcciones ABI Windows x64 - ADead Backend

**Fecha:** Diciembre 2025  
**Autor:** Eddi Andreé Salazar Matos  
**Motivación:** Análisis técnico identificó violaciones del ABI Windows x64

---

## ❌ Problemas Identificados

### 1. Registros No Volátiles No Preservados

**Registros que DEBEN preservarse (callee-saved):**
- `RBX`
- `RBP` (ya se preserva con `push rbp`)
- `RDI`
- `RSI`
- `R12-R15`

**Estado actual:** ❌ Se usan sin preservar

### 2. Stack Alignment Incorrecto

**Requisito Windows x64:**
> RSP debe estar alineado a 16 bytes ANTES de cada `call`

**Estado actual:** ❌ No siempre se cumple (hay `push` antes de `call`)

### 3. Bug Lógico en `array_remove`

**Problema:**
```asm
cmp r10, rax  ; rax ya fue sobrescrito, no es length
```

**Debe ser:**
```asm
cmp r10, [rcx + 8]  ; comparar con length real
```

### 4. Memory Leaks en Strings

**Problema:** Strings creados dinámicamente nunca se liberan

**Solución propuesta:** Ownership explícito estilo Rust

---

## ✅ Plan de Corrección

### Fase 1: Prologue/Epilogue ABI-Safe

Crear función helper que genere:
```asm
function_name:
    ; Prologue ABI-safe
    push rbp
    mov rbp, rsp
    push rbx      ; preservar registros no volátiles
    push rdi
    push rsi
    push r12
    push r13
    push r14
    push r15
    
    ; Asegurar stack alignment
    ; Calcular: (rsp - 8*num_preserved) % 16 debe ser 0
    ; Si no, ajustar con sub rsp, 8
    
    ; Shadow space (si se llama a funciones externas)
    sub rsp, 32
    
    ; ... código de la función ...
    
    ; Epilogue ABI-safe
    add rsp, 32   ; restaurar shadow space
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

### Fase 2: Corregir Funciones Helper

Aplicar prologue/epilogue ABI-safe a:
- [ ] `array_new`
- [ ] `array_from_values`
- [ ] `array_get`
- [ ] `array_set`
- [ ] `array_append`
- [ ] `array_pop`
- [ ] `array_reverse`
- [ ] `array_insert`
- [ ] `array_remove` ⚠️ **CRÍTICO: Bug lógico**
- [ ] `array_index`
- [ ] `array_count`
- [ ] `array_sort`
- [ ] `string_new`
- [ ] `string_from_literal`
- [ ] `string_concat`
- [ ] `string_slice`
- [ ] `string_upper`
- [ ] `string_lower`
- [ ] `string_len`

### Fase 3: Stack Alignment Helper

Crear función que calcule y ajuste stack alignment:
```rust
fn ensure_stack_alignment(&mut self, num_preserved_regs: usize) {
    // Después de push de registros preservados
    // Asegurar que (rsp - 8*num_preserved) % 16 == 0
    // Si no, hacer sub rsp, 8
}
```

### Fase 4: Memory Management

**Decisión de diseño:** Ownership explícito estilo Rust

**Implementación:**
- Variables de tipo String/Array tienen ownership
- Al salir de scope, llamar automáticamente a `drop`
- `drop` libera memoria con `VirtualFree`

---

## 📊 Prioridad

1. 🔥 **CRÍTICO:** Corregir bug en `array_remove`
2. 🔥 **CRÍTICO:** Stack alignment antes de cada `call`
3. ⚠️ **ALTO:** Preservar registros no volátiles
4. ⚠️ **MEDIO:** Memory management (requiere decisión de diseño)

---

## 🎯 Estado

- [ ] Fase 1: Prologue/Epilogue ABI-Safe
- [ ] Fase 2: Corregir funciones helper
- [ ] Fase 3: Stack alignment helper
- [ ] Fase 4: Memory management

**Progreso:** 0% completado

