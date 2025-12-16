# ✅ Correcciones ABI Aplicadas

**Fecha:** Diciembre 2025  
**Motivación:** Análisis técnico identificó violaciones del ABI Windows x64

---

## ✅ Correcciones Completadas

### 1. Bug Crítico en `array_remove` ✅ CORREGIDO

**Problema:**
```asm
mov rax, [r13]  ; cargar valor fuente
; ... más código ...
cmp r10, rax  ; ❌ rax ya no tiene length, tiene valor del array
```

**Solución:**
- Preservar `length` en stack antes del loop
- Usar `r14` para valores temporales en lugar de `rax`
- Restaurar `length` antes de cada comparación

**Estado:** ✅ Corregido

---

## ⏳ Correcciones Pendientes

### 2. Preservar Registros No Volátiles

**Registros que deben preservarse:**
- `RBX`, `RDI`, `RSI`, `R12-R15`

**Estado:** ⏳ Pendiente

### 3. Stack Alignment

**Requisito:** RSP alineado a 16 bytes antes de cada `call`

**Estado:** ⏳ Pendiente

### 4. Memory Management

**Problema:** Strings nunca se liberan (memory leaks)

**Estado:** ⏳ Pendiente (requiere decisión de diseño)

---

## 📊 Progreso

- ✅ Bug crítico en `array_remove` - CORREGIDO
- ⏳ Preservación de registros - PENDIENTE
- ⏳ Stack alignment - PENDIENTE
- ⏳ Memory management - PENDIENTE

**Progreso Total:** 25% completado (1/4 correcciones críticas)

