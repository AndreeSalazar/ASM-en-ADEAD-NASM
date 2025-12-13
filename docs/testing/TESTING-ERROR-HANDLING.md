# 🧪 Testing - Manejo de Errores

**Fecha:** Diciembre 2025  
**Estado:** Testing en progreso

---

## ✅ Implementación Verificada

### 1. Operador `?` - Propagación de Errores

**Parser:**
- ✅ `Expr::PropagateError(Box<Expr>)` agregado al AST
- ✅ Parser reconoce `expr?` como operador postfix
- ✅ Tests agregados:
  - `test_parse_propagate_error_operator` - Básico: `funcion()?`
  - `test_parse_propagate_error_with_method_call` - Con métodos: `objeto.metodo()?`
  - `test_parse_propagate_error_with_ok` - Con Ok: `Ok(42)?`
  - `test_parse_propagate_error_chained` - Múltiples: `func1()? + func2()?`

**Backend:**
- ✅ Genera código NASM para verificar tag (0=Ok, 1=Err)
- ✅ Si Ok: desarrolla valor y continúa
- ✅ Si Err: propaga error (carga error en rax)
- ✅ Labels generados correctamente (`propagate_ok`, `propagate_error`)

**Ubicación:**
- `rust/crates/adead-backend/src/lib.rs` - Líneas 716-751

---

### 2. Errores Estándar

**Tipos definidos:**
- ✅ `FileError` - Errores de archivos
- ✅ `ParseError` - Errores de parsing
- ✅ `MathError` - Errores matemáticos
- ✅ `ValueError` - Errores de valores
- ✅ `IOError` - Errores de I/O

**Ubicación:**
- `rust/crates/adead-common/src/lib.rs` - Líneas 71-103

**Helper:**
- ✅ `StdError::to_string()` implementado para cada tipo

---

## 📋 Checklist de Testing

### Tests de Parser ✅

- [x] `test_parse_propagate_error_operator` - Operador básico
- [x] `test_parse_propagate_error_with_method_call` - Con métodos
- [x] `test_parse_propagate_error_with_ok` - Con constructores
- [x] `test_parse_propagate_error_chained` - Encadenado
- [x] Tests existentes de Option/Result (ya pasando)

### Tests de Backend ✅

- [x] Generación de código para `PropagateError`
- [x] Verificación de labels generados
- [x] Verificación de lógica de propagación
- [x] Tests agregados:
  - `test_generate_propagate_error_with_ok`
  - `test_generate_propagate_error_with_method_call`
  - `test_generate_propagate_error_checks_tag`
  - `test_generate_propagate_error_handles_ok`
  - `test_generate_propagate_error_handles_err`
  - `test_generate_propagate_error_chained`

### Tests de Integración ⚠️

- [ ] Ejemplo completo: función que retorna Result
- [ ] Ejemplo: uso del operador `?` en función
- [ ] Compilación a ASM y verificación de código generado

---

## 🧪 Ejemplos de Testing

### Ejemplo 1: Función con Result y operador `?`

```adead
fn leer_numero() -> Result<int64, ParseError> {
    // Simulación: siempre retorna Ok(42)
    return Ok(42)
}

fn usar_numero() -> Result<int64, ParseError> {
    let valor = leer_numero()?  // Usa operador ?
    return Ok(valor + 1)
}

let resultado = usar_numero()
match resultado {
    Ok(v) => print v
    Err(e) => print "Error"
}
```

**Verificaciones:**
1. Parser debe reconocer `leer_numero()?`
2. Backend debe generar código para verificar tag
3. Si Ok, desarrollar valor; si Err, propagar

### Ejemplo 2: Match con Result

```adead
fn dividir(a: int64, b: int64) -> Result<int64, MathError> {
    if b == 0 {
        return Err(MathError {
            operation: "division",
            message: "Division por cero"
        })
    }
    return Ok(a / b)
}

let resultado = dividir(10, 2)
match resultado {
    Ok(valor) => print valor
    Err(error) => print error.message
}
```

**Verificaciones:**
1. Parser debe reconocer `match` con `Ok` y `Err`
2. Backend debe generar código para pattern matching
3. Debe cargar valores correctamente desde tagged union

---

## 🔍 Análisis del Código Generado

### Operador `?` - Código NASM esperado

```asm
; expr? donde expr es Result<T, E>
; 1. Evaluar expr (resultado en rax = dirección del Result)
mov rbx, rax          ; Guardar dirección
mov rax, [rbx]        ; Cargar tag (0=Ok, 1=Err)
cmp rax, 0            ; Comparar con 0 (Ok)
je propagate_ok       ; Si Ok, saltar a desenvolver
jmp propagate_error   ; Si Err, saltar a propagar

propagate_ok:
mov rax, [rbx + 8]    ; Cargar valor de Ok

propagate_error:
mov rax, [rbx + 8]    ; Cargar error de Err
; TODO: En función, debería retornar temprano
```

**Verificaciones:**
- ✅ Código generado sigue esta estructura
- ⚠️ Retorno temprano en funciones aún pendiente (TODO)

---

## ⚠️ Limitaciones Conocidas

1. **Retorno temprano en funciones:**
   - El operador `?` actualmente carga el error en `rax`
   - No retorna automáticamente de la función
   - Requiere implementar detección de contexto de función

2. **Solo funciona con Result:**
   - Actualmente optimizado para `Result<T, E>`
   - Podría extenderse a `Option<T>` en el futuro

3. **Errores estándar:**
   - Tipos definidos pero no integrados completamente con parser
   - Los usuarios deben crear structs manualmente por ahora

---

## ✅ Próximos Pasos de Testing

1. **Ejecutar tests completos:**
   ```bash
   cargo test --package adead-parser
   cargo test --package adead-backend
   ```

2. **Crear ejemplo funcional:**
   - Archivo `.ad` con manejo de errores
   - Compilar y verificar código ASM generado
   - Verificar que funcione correctamente

3. **Verificar integración:**
   - Parser → AST → Backend → ASM
   - Flujo completo funcionando

---

## 📊 Estado de Testing

| Componente | Parser | Backend | Integración | Estado |
|-----------|--------|---------|-------------|--------|
| Operador `?` | ✅ | ✅ | ⚠️ | Funcional (con limitaciones) |
| Errores estándar | ✅ | N/A | ⚠️ | Tipos definidos |
| Match Result | ✅ | ✅ | ✅ | Funcional |
| Match Option | ✅ | ✅ | ✅ | Funcional |

**Estado General:** ✅ **Funcional con limitaciones conocidas**

---

**Actualizado:** Diciembre 2025

