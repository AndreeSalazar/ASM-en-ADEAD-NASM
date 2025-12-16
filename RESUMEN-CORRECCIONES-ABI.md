# 🔧 Resumen: Correcciones ABI Windows x64

**Fecha:** Diciembre 2025  
**Análisis:** Otra IA Pragmática identificó violaciones críticas del ABI

---

## ✅ Corrección Aplicada

### Bug Crítico en `array_remove` ✅ CORREGIDO

**Problema identificado:**
```asm
mov rax, [r13]  ; cargar valor fuente
; ... más código ...
cmp r10, rax  ; ❌ rax ya no tiene length, tiene valor del array
```

**Riesgo:** Corrupción de memoria, lecturas fuera de rango

**Solución aplicada:**
- Preservar `length` en stack antes del loop
- Usar `r14` para valores temporales (no `rax`)
- Restaurar `length` antes de cada comparación

**Estado:** ✅ Compilado y verificado

---

## ⏳ Correcciones Pendientes (Prioridad)

### 1. Preservar Registros No Volátiles 🔥 ALTA PRIORIDAD

**Problema:** Se usan `RBX`, `RDI`, `RSI`, `R12-R15` sin preservarlos

**Riesgo:** Crashes aleatorios cuando funciones externas esperan estos registros intactos

**Solución requerida:**
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
    ; ... código ...
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

**Afecta:** Todas las funciones helper (arrays y strings)

---

### 2. Stack Alignment 🔥 ALTA PRIORIDAD

**Problema:** RSP no siempre está alineado a 16 bytes antes de `call`

**Riesgo:** Crashes en Windows x64 (requisito estricto del ABI)

**Ejemplo problemático:**
```asm
push rcx      ; desalinea stack
push rdx      ; desalinea más
sub rsp, 32   ; intenta alinear
call VirtualAlloc  ; ❌ puede fallar si stack no está alineado
```

**Solución requerida:**
- Calcular alineación después de cada `push`
- Ajustar con `sub rsp, 8` si es necesario
- O usar diseño fijo que siempre alinee correctamente

---

### 3. Memory Management ⚠️ MEDIA PRIORIDAD

**Problema:** Strings creados dinámicamente nunca se liberan

**Riesgo:** Memory leaks permanentes

**Solución propuesta:** Ownership explícito estilo Rust
- Variables de tipo String/Array tienen ownership
- Al salir de scope, llamar automáticamente a `drop`
- `drop` libera memoria con `VirtualFree`

**Nota:** Requiere decisión de diseño del lenguaje

---

## 📊 Estado Actual

| Corrección | Prioridad | Estado | Riesgo |
|------------|-----------|--------|--------|
| Bug `array_remove` | 🔥 CRÍTICO | ✅ COMPLETADO | Corrupción de memoria |
| Preservar registros | 🔥 ALTA | ⏳ PENDIENTE | Crashes aleatorios |
| Stack alignment | 🔥 ALTA | ⏳ PENDIENTE | Crashes en Windows |
| Memory management | ⚠️ MEDIA | ⏳ PENDIENTE | Memory leaks |

**Progreso:** 25% completado (1/4 correcciones críticas)

---

## 🎯 Próximos Pasos

1. **Implementar prologue/epilogue ABI-safe** para todas las funciones helper
2. **Crear función helper** para asegurar stack alignment
3. **Aplicar correcciones** a todas las funciones (arrays y strings)
4. **Decidir modelo de ownership** para memory management

---

## 💡 Nota sobre `print`

**Estado:** ✅ `print` SÍ está implementado correctamente

El análisis mencionó que `print` no estaba implementado, pero en realidad:
- ✅ Usa `WriteFile` correctamente
- ✅ Maneja strings dinámicos
- ✅ Maneja números, floats, bools

**Conclusión:** `print` funciona, pero las funciones helper que llama tienen problemas de ABI.

---

**Siguiente paso recomendado:** Implementar preservación de registros no volátiles en todas las funciones helper.

