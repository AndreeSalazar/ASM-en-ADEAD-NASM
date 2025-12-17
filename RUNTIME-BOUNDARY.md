# Runtime Boundary - ADead

**Fecha:** Diciembre 2025  
**Estado:** ✅ **OFICIAL**

---

## 🎯 Objetivo

**Marcar claramente qué es parte del runtime y qué es código generado del usuario.**

---

## 📋 Componentes del Runtime

### 1. Funciones Helper del Runtime

**Ubicación:** Generadas antes del main, marcadas con `RUNTIME:`

**Incluye:**
- `array_*` - Todas las funciones de arrays
- `string_*` - Todas las funciones de strings
- `stdlib_*` - Funciones de la librería estándar

**Marcado en código:**
```asm
; ============================================
; RUNTIME: Funciones Helper de Array
; ============================================
array_new:
    ; ...
```

### 2. Librería Estándar (Stdlib)

**Ubicación:** Generada después de helpers, antes del main

**Incluye:**
- `stdlib_min(a, b)`
- `stdlib_max(a, b)`
- `stdlib_abs(n)`
- `stdlib_pow(base, exp)`

**Marcado en código:**
```asm
; ============================================
; RUNTIME: Librería Estándar (Stdlib)
; ============================================
stdlib_min:
    ; ...
```

### 3. Código Generado del Usuario

**Ubicación:** Después de runtime, incluye main

**Incluye:**
- Funciones definidas por el usuario (`fn nombre`)
- Código del main
- Variables y expresiones del usuario

**Marcado en código:**
```asm
; ============================================
; RUNTIME BOUNDARY END: Código Generado del Usuario
; ============================================

; ADead: line 1 - function definition: fn suma(a, b)
fn_suma:
    ; ...
```

---

## 🔍 Identificación en Código NASM

### Runtime Functions
```asm
; RUNTIME: Funciones Helper de Array
array_new:
    ; ...
```

### User Code
```asm
; ADead: line 5 - let resultado = suma(5, 3)
; ADead: line 6 - print resultado
```

---

## ✅ Beneficios

1. **Claridad:** Fácil identificar qué es runtime vs código usuario
2. **Debugging:** Saber dónde buscar problemas
3. **Optimización:** Separar optimizaciones de runtime vs usuario
4. **Documentación:** Mejor comprensión del código generado

---

**Esta separación es OBLIGATORIA en todo el código generado.**

