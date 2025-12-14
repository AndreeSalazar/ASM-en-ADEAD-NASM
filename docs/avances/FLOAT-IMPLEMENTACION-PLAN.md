# 🎯 Plan Detallado: Implementación Float64/Float32

## 📊 Estado Actual - Lo que TENEMOS

### ✅ Types (adead-common/src/lib.rs) - **COMPLETO**

**Enum Type ya incluye:**
```rust
Float32,   // 32 bits: XMM0-XMM15 (NASM: movss xmm0, value)
Float64,   // 64 bits: XMM0-XMM15 (NASM: movsd xmm0, value)
```

**Métodos helper ya implementados:**
- ✅ `is_float()` - Verifica si es Float32 o Float64
- ✅ `default_float()` - Retorna `Type::Float64`
- ✅ `nasm_register_hint()` - Retorna `("xmm0", "dword")` para Float32, `("xmm0", "qword")` para Float64
- ✅ `nasm_declaration()` - Genera `dd 0` para Float32, `dq 0` para Float64
- ✅ `size_bytes()` - Retorna 4 para Float32, 8 para Float64
- ✅ `is_numeric()` - Incluye Float32 y Float64
- ✅ `is_copy()` - Float32 y Float64 son Copy

**Conclusión:** El sistema de tipos está **100% listo** para floats.

---

### ✅ Flujo Directo Zig → NASM - **FUNCIONANDO** 🎉

**🚀 NUEVO: Flujo independiente para floats simples**

**Ubicación:** `zig/src/nasm_generator.zig` y `zig/src/expr_parser.zig`

**Estado actual:**
- ✅ **Parsing de floats en Zig:** `readFloat()` parsea `3.14`, `.5`, `5.`, etc.
- ✅ **Generación NASM directa:** `generatePrintFloat()` convierte float a string en compile-time
- ✅ **FFI funcionando:** `generate_nasm_ffi()` permite a Rust llamar a Zig
- ✅ **Flujo completo:** `print 3.14` → Zig parsea → Zig genera NASM → Ejecutable funciona
- ✅ **Verificado:** `debug-float.exe` ejecuta correctamente y muestra "3.14"

**Archivos clave:**
- `zig/src/expr_parser.zig`: `readFloat()`, `parse_expr_ffi()`
- `zig/src/nasm_generator.zig`: `generatePrintFloat()`, `generate_nasm_ffi()`
- `rust/crates/adead-parser/src/zig_nasm_generator.rs`: Wrapper FFI
- `rust/crates/adead-cli/src/main.rs`: Lógica de detección de flujo directo

**Lo que FUNCIONA:**
- ✅ `print 3.14` - Print de literales float simples
- ✅ Conversión float → string en compile-time
- ✅ Generación NASM completa con `.data` y `.text` sections
- ✅ WriteFile calls para Windows x64

**Limitaciones actuales:**
- ⏳ Solo funciona para literales simples (`print 3.14`)
- ⏳ Expresiones float (`print 3.14 + 2.5`) aún necesitan flujo Rust
- ⏳ Variables con floats aún necesitan flujo Rust

---

### ✅ AST (Expr enum) - **COMPLETADO**

**Ubicación:** `rust/crates/adead-parser/src/lib.rs`

**Estado actual:**
```rust
pub enum Expr {
    Number(i64),      // ✅ Existe
    Float(f64),       // ✅ AGREGADO - Literal flotante
    String(String),   // ✅ Existe
    Ident(String),    // ✅ Existe
    BinaryOp { ... }, // ✅ Existe
    // ... otros
}
```

**✅ Float(f64) agregado al enum Expr**

---

### ✅ Parser Rust - **COMPLETADO (pero hay flujo alternativo)**

**Ubicación:** `rust/crates/adead-parser/src/lib.rs`

**Estado actual:**
- ✅ Parser Rust implementado para floats
- ✅ Maneja: `3.14`, `.5`, `5.`, `2.5e10`, etc.
- ⚠️ **Pero:** Para casos simples, el flujo directo Zig se usa automáticamente

**Nota:** El parser Rust sigue disponible como fallback para casos complejos.

---

### ⏳ Backend Rust - **PARCIAL (hay alternativa Zig)**

**Ubicación:** `rust/crates/adead-backend/src/lib.rs`

#### ✅ Lo que SÍ funciona (en Rust):

1. **Print de Number:**
   ```rust
   Expr::Number(n) => {
       let num_str = format!("{}", n);
       let label = self.add_string_data(&num_str);
       // ... WriteFile call
   }
   ```

2. **Print de Float (compilación):**
   ```rust
   Expr::Float(f) => {
       let float_str = format!("{}", f);
       let label = self.add_string_data(&float_str);
       // ... WriteFile call
   }
   ```

3. **Expresiones de Number:**
   ```rust
   Expr::Number(n) => {
       self.text_section.push(format!("    mov rax, {}", n));
   }
   ```

4. **Operaciones SSE:** `addsd`, `subsd`, `mulsd`, `divsd` implementadas

#### ⚠️ Lo que tiene alternativa en Zig:

1. ✅ **Print de floats simples** - Ya funciona via flujo directo Zig → NASM
2. ✅ **Conversión float a string** - Funciona en Zig (compile-time)

#### ❌ Lo que FALTA en Rust (y no tiene alternativa Zig):

1. ⏳ **Print de expresiones float complejas** - Necesita `float_to_str_runtime` helper
2. ⏳ **Variables con floats** - Necesita testing y validación
3. ⏳ **Conversión int ↔ float** - Parcialmente implementado (`cvtsi2sd`)

---

## 🎯 Plan de Implementación Completo

### FASE 1: AST y Parser (5 horas)

#### 1.1 Agregar Float al AST (30 min)

**Archivo:** `rust/crates/adead-parser/src/lib.rs`

```rust
pub enum Expr {
    Number(i64),
    Float(f64),  // ← AGREGAR ESTO
    String(String),
    // ... resto
}
```

**Tareas:**
- [ ] Agregar `Float(f64)` al enum Expr
- [ ] Actualizar todos los `match` que usan `Expr::Number` para incluir `Expr::Float`
- [ ] Actualizar tests si es necesario

---

#### 1.2 Implementar Parser de Floats (4 horas)

**Archivo:** `rust/crates/adead-parser/src/lib.rs` - función `expr_parser()`

**Parser necesario:**
```rust
// Parser para literales flotantes
let float_parser = 
    // Caso 1: 3.14 (entero.punto.decimal)
    text::int(10)
        .then(just('.'))
        .then(text::digits(10))
        .then(
            // Opcional: notación científica
            just('e').or(just('E'))
                .then(text::int(10).or(just('+').ignore_then(text::int(10))).or(just('-').ignore_then(text::int(10))))
                .or_not()
        )
        .map(|(((int_part, _), dec_part), exp)| {
            // Construir string y parsear a f64
            let mut float_str = format!("{}.{}", int_part, dec_part);
            if let Some((_, exp_part)) = exp {
                float_str.push_str(&format!("e{}", exp_part));
            }
            float_str.parse::<f64>().unwrap()
        })
        .map(Expr::Float)
    
    // Caso 2: .5 (punto.decimal)
    .or(
        just('.')
            .then(text::digits(10))
            .then(/* notación científica opcional */)
            .map(|((_, dec_part), exp)| {
                // Similar construcción
            })
            .map(Expr::Float)
    )
    
    // Caso 3: 5. (entero.punto)
    .or(
        text::int(10)
            .then(just('.'))
            .then(/* notación científica opcional */)
            .map(|((int_part, _), exp)| {
                // Similar construcción
            })
            .map(Expr::Float)
    )
    
    .labelled("float");
```

**Simplificación usando chumsky:**
```rust
use chumsky::prelude::*;

let float_literal = text::int(10)
    .then(just('.').then(text::digits(10)).or_not())
    .then(
        just('e').or(just('E'))
            .ignore_then(
                just('+').or(just('-')).or_not()
                    .then(text::int(10))
                    .map(|(sign, num)| {
                        if sign == Some('-') {
                            format!("-{}", num)
                        } else {
                            num
                        }
                    })
            )
            .or_not()
    )
    .try_map(|((int, dec), exp), span| {
        let float_str = match dec {
            Some((_, d)) => format!("{}.{}", int, d),
            None => format!("{}.0", int),
        };
        let float_str = match exp {
            Some(e) => format!("{}e{}", float_str, e),
            None => float_str,
        };
        float_str.parse::<f64>()
            .map_err(|_| Simple::custom(span, "Invalid float literal"))
    })
    .map(Expr::Float)
    .labelled("float");
```

**Tareas:**
- [ ] Implementar parser de floats con chumsky
- [ ] Manejar casos: `3.14`, `.5`, `5.`, `2.5e10`, `1e-5`
- [ ] Integrar con parser de expresiones (debe venir después de `number`)
- [ ] Tests: `parse("3.14")`, `parse("2.5e10")`, etc.

---

### FASE 2: Backend Windows - Generación NASM (5 horas)

#### 2.1 Agregar manejo de Expr::Float en generate_expr_windows (1 hora)

**Archivo:** `rust/crates/adead-backend/src/lib.rs`

**Ubicación:** Función `generate_expr_windows()` después de `Expr::Number`

```rust
Expr::Float(f) => {
    // Cargar constante flotante en XMM0
    // Estrategia: almacenar float en .data y cargar desde ahí
    let label = self.add_float_data(*f);
    self.text_section.push(format!("    movsd xmm0, [rel {}]  ; cargar float {}", label, f));
}
```

**Helper necesario:**
```rust
fn add_float_data(&mut self, value: f64) -> String {
    let label = format!("float_{}", self.data_counter);
    self.data_counter += 1;
    self.data_section.push(format!("{}: dq {}  ; float64 literal", label, value));
    label
}
```

**Tareas:**
- [ ] Agregar `add_float_data()` method
- [ ] Agregar case `Expr::Float(f)` en `generate_expr_windows()`
- [ ] Usar `movsd` (64-bit) o `movss` (32-bit) según precisión

---

#### 2.2 Operaciones Aritméticas con Floats (2 horas)

**Ubicación:** Función `generate_expr_windows()` - case `Expr::BinaryOp`

**Estado actual (enteros):**
```rust
BinOp::Add => {
    self.text_section.push("    add rax, rbx".to_string());
}
```

**Necesario (floats):**
```rust
// Detectar si operando es float
// Si left o right es Float, usar operaciones SSE

// Para Float64 (double precision):
BinOp::Add => {
    if is_float_expr(left) || is_float_expr(right) {
        // XMM0 ya tiene left, XMM1 tiene right
        self.text_section.push("    addsd xmm0, xmm1  ; float64 addition".to_string());
    } else {
        self.text_section.push("    add rax, rbx".to_string());
    }
}
```

**Operaciones SSE necesarias:**
- `addsd xmm0, xmm1` - Float64 addition
- `subsd xmm0, xmm1` - Float64 subtraction
- `mulsd xmm0, xmm1` - Float64 multiplication
- `divsd xmm0, xmm1` - Float64 division

**Para Float32 (single precision):**
- `addss xmm0, xmm1` - Float32 addition
- `subss xmm0, xmm1` - Float32 subtraction
- `mulss xmm0, xmm1` - Float32 multiplication
- `divss xmm0, xmm1` - Float32 division

**Tareas:**
- [ ] Crear helper `is_float_expr()` para detectar tipos float
- [ ] Modificar `BinaryOp` handling para detectar floats
- [ ] Generar código SSE apropiado según tipo (Float32 vs Float64)
- [ ] Manejar conversiones int → float si es necesario

---

#### 2.3 Print de Floats (1.5 horas)

**Ubicación:** `generate_stmt_windows()` - case `Stmt::Print`

**Estrategia similar a Number pero con conversión float → string**

```rust
Expr::Float(f) => {
    // Opción 1: Compilación (simple, como Number)
    let float_str = format!("{}", f);
    let label = self.add_string_data(&float_str);
    // ... WriteFile call
    
    // Opción 2: Runtime (para expresiones float)
    // Necesita función helper float_to_str_runtime (similar a int_to_str_runtime)
}
```

**Para expresiones float (ej: `print 3.14 + 2.5`):**
1. Evaluar expresión → XMM0 contiene resultado float
2. Llamar `float_to_str_runtime` (similar a `int_to_str_runtime`)
3. Función convierte float en XMM0 a string en buffer
4. Usar WriteFile con buffer

**Función helper necesaria:**
```asm
float_to_str_runtime:
    ; Entrada: XMM0 = float64
    ; Salida: RAX = longitud, RDX = buffer address
    ; Similar a int_to_str_runtime pero con conversión float
    
    ; Usar algoritmo:
    ; 1. Separar parte entera y decimal
    ; 2. Convertir parte entera (similar a int)
    ; 3. Agregar punto decimal
    ; 4. Convertir parte decimal
    ; 5. Retornar string
```

**Tareas:**
- [ ] Agregar case `Expr::Float(f)` en print (compilación)
- [ ] Crear función helper `float_to_str_runtime` en NASM
- [ ] Integrar con generación de código para expresiones float

---

#### 2.4 Variables y Asignación con Floats (30 min)

**Ubicación:** `generate_stmt_windows()` - case `Stmt::Let`

**Ya debería funcionar si:**
- AST incluye `Expr::Float`
- `generate_expr_windows` maneja `Expr::Float`
- Stack allocation funciona (ya maneja 8 bytes para int64, igual para float64)

**Verificar:**
- [ ] Stack offset se incrementa correctamente (8 bytes para float64)
- [ ] Almacenamiento desde XMM0 a stack: `movsd [rbp - offset], xmm0`
- [ ] Carga desde stack: `movsd xmm0, [rbp - offset]`

---

### FASE 3: Backend Linux - Generación NASM (3 horas)

**Similar a Windows pero con convenciones System V ABI:**
- Float32: XMM0-XMM7 para parámetros
- Float64: XMM0-XMM7 para parámetros
- Return: XMM0 para floats

**Tareas:**
- [ ] Adaptar `generate_expr()` para Linux
- [ ] Usar sys_write en lugar de WriteFile
- [ ] Verificar convenciones System V ABI para floats

---

### FASE 4: Tests y Validación (2 horas)

#### 4.1 Tests del Parser
```rust
#[test]
fn test_parse_float() {
    assert_eq!(parse("3.14"), Expr::Float(3.14));
    assert_eq!(parse(".5"), Expr::Float(0.5));
    assert_eq!(parse("5."), Expr::Float(5.0));
    assert_eq!(parse("2.5e10"), Expr::Float(2.5e10));
}
```

#### 4.2 Tests de Backend
- [x] Compilar y ejecutar: `print 3.14` ✅ **VERIFICADO - Funciona via flujo directo Zig → NASM**
- [ ] Compilar y ejecutar: `let x = 2.5; print x` ⏳ **PENDIENTE** (necesita flujo Rust)
- [ ] Compilar y ejecutar: `print 3.14 + 2.5` ⏳ **PENDIENTE** (necesita flujo Rust con runtime helper)
- [ ] Compilar y ejecutar: `print 10.0 / 3.0` ⏳ **PENDIENTE** (necesita flujo Rust con runtime helper)

#### 4.3 Tests de Precisión
- [ ] Verificar que Float64 mantiene precisión
- [ ] Verificar operaciones aritméticas correctas

---

## 📋 Checklist Completo

### AST y Parser
- [x] Agregar `Float(f64)` al enum Expr ✅ **COMPLETADO**
- [x] Implementar parser de literales float ✅ **COMPLETADO** (estructura completa)
- [x] Manejar casos: `3.14`, `.5`, `5.`, `2.5e10` ✅ **COMPLETADO** (parser corregido con then_ignore)
- [x] Integrar parser float con expresiones ✅ **COMPLETADO**
- [x] Tests del parser ✅ **COMPLETADO** (test-float-parser.ad, test-float-expr.ad, test-float-mixed.ad)

### 🚀 Flujo Directo Zig → NASM (NUEVO)
- [x] Parsing de floats en Zig (`readFloat`) ✅ **COMPLETADO Y FUNCIONANDO**
- [x] Generación NASM directa (`generatePrintFloat`) ✅ **COMPLETADO Y FUNCIONANDO**
- [x] FFI para Rust (`generate_nasm_ffi`) ✅ **COMPLETADO Y FUNCIONANDO**
- [x] Detección automática de flujo directo ✅ **COMPLETADO**
- [x] Print de literales float simples (`print 3.14`) ✅ **VERIFICADO Y FUNCIONANDO** 🎉
- [x] Conversión float → string en compile-time ✅ **FUNCIONANDO**
- [x] Generación completa de NASM (`.data` + `.text`) ✅ **FUNCIONANDO**
- [ ] Print de expresiones float complejas (`print 3.14 + 2.5`) ⏳ **PENDIENTE** (necesita flujo Rust)

### Backend Windows (Rust)
- [x] Agregar `add_float_data()` helper ✅ **COMPLETADO**
- [x] Manejar `Expr::Float` en `generate_expr_windows()` ✅ **COMPLETADO**
- [x] Operaciones SSE: `addsd`, `subsd`, `mulsd`, `divsd` ✅ **COMPLETADO**
- [x] Detección de tipos float en operaciones binarias ✅ **COMPLETADO** (is_float_expr helper)
- [x] Conversión int → float (`cvtsi2sd`) ✅ **COMPLETADO**
- [x] Print de floats (compilación) ✅ **COMPLETADO** (también funciona via Zig)
- [ ] Print de expresiones float (runtime con helper) ⏳ **PENDIENTE** (expresiones como `3.14 + 2.5`)
- [ ] Función helper `float_to_str_runtime` ⏳ **PENDIENTE**
- [ ] Variables y asignación con floats ⏳ **PENDIENTE** (debería funcionar, necesita testing)

### Backend Linux
- [x] Adaptar `generate_expr()` para floats ✅ **COMPLETADO**
- [x] Verificar convenciones System V ABI ✅ **COMPLETADO** (XMM0 para return)
- [ ] Tests en Linux ⏳ **PENDIENTE**

### Tests y Documentación
- [ ] Tests unitarios del parser
- [ ] Tests de integración (compilar y ejecutar)
- [ ] Tests de precisión
- [ ] Actualizar documentación

---

## 🔧 Detalles Técnicos

### Registros SSE (x86-64)

**XMM0-XMM15:** Registros de 128 bits para SIMD
- Para Float32: usar parte baja (32 bits) con `movss`, `addss`, etc.
- Para Float64: usar parte baja (64 bits) con `movsd`, `addsd`, etc.

### Instrucciones NASM

**Float64 (double precision):**
```asm
movsd xmm0, [rel float_const]  ; Cargar desde memoria
movsd xmm0, xmm1               ; Copiar entre registros
addsd xmm0, xmm1               ; Sumar
subsd xmm0, xmm1               ; Restar
mulsd xmm0, xmm1               ; Multiplicar
divsd xmm0, xmm1               ; Dividir
movsd [rbp - 8], xmm0          ; Guardar en stack
movsd xmm0, [rbp - 8]          ; Cargar desde stack
```

**Float32 (single precision):**
```asm
movss xmm0, [rel float_const]  ; Cargar desde memoria
movss xmm0, xmm1               ; Copiar entre registros
addss xmm0, xmm1               ; Sumar
subss xmm0, xmm1               ; Restar
mulss xmm0, xmm1               ; Multiplicar
divss xmm0, xmm1               ; Dividir
```

### Convenciones de Llamadas

**Windows x64:**
- Float32/Float64: XMM0-XMM3 para parámetros (si hay <4)
- Return float: XMM0

**Linux System V ABI:**
- Float32/Float64: XMM0-XMM7 para parámetros
- Return float: XMM0

---

## 🎯 Prioridades

1. **Crítico:** AST + Parser básico (Fase 1)
2. **Crítico:** Backend Windows básico (Fase 2.1 + 2.2)
3. **Alto:** Print de floats (Fase 2.3)
4. **Medio:** Variables y asignación (Fase 2.4)
5. **Medio:** Backend Linux (Fase 3)
6. **Bajo:** Tests completos (Fase 4)

---

## 📚 Referencias

- [x86-64 SSE Instructions](https://www.felixcloutier.com/x86/)
- [Windows x64 Calling Convention](https://docs.microsoft.com/en-us/cpp/build/x64-calling-convention)
- [System V ABI](https://www.uclibc.org/docs/psABI-x86_64.pdf)
- [NASM Floating Point](https://www.nasm.us/docs.php)

---

---

## 🎉 Logros Recientes (Diciembre 2025)

### ✅ Flujo Directo Zig → NASM Operativo

**Estado:** **FUNCIONANDO Y VERIFICADO** 🎉

**Qué funciona:**
- ✅ `print 3.14` ejecuta correctamente y muestra "3.14"
- ✅ Flujo completo: ADead → Zig (parsing) → Zig (NASM) → Ejecutable
- ✅ Conversión float → string en compile-time
- ✅ Generación NASM completa y correcta para Windows x64

**Archivo de prueba:** `Ejemplos-Reales/compilados/debug-float.ad`
```adead
print 3.14
```

**Comando para probar:**
```powershell
.\rust\target\release\adeadc.exe run Ejemplos-Reales\compilados\debug-float.ad
```

**Resultado:** ✅ Ejecutable genera correctamente y muestra "3.14"

---

**Última actualización:** Diciembre 2025  
**Estado:** Flujo directo Zig → NASM funcionando para floats simples  
**Siguiente paso:** Implementar expresiones float complejas (`print 3.14 + 2.5`) y variables

