# 🔧 Implementación de Ownership y Borrowing (O0.2)

**Documentación técnica sobre la implementación de Ownership y Borrowing en ADead**

---

## ✅ Estado de Implementación

### Completado

1. **AST Extendido** (`crates/adead-parser/src/lib.rs`)
   - ✅ `Expr::Borrow { expr, mutable }` - Borrowing
   - ✅ `Expr::Deref(expr)` - Dereferenciación
   - ✅ `FnParam { name, borrow_type }` - Parámetros con borrowing
   - ✅ `BorrowType` enum (Owned, Borrowed, MutBorrowed)

2. **Parser Actualizado**
   - ✅ Parser para `&expr` (borrow inmutable)
   - ✅ Parser para `&mut expr` (borrow mutable)
   - ✅ Parser para `*expr` (dereferenciar)
   - ✅ Parser para parámetros de función con `&T` y `&mut T`

3. **Borrow Checker** (`crates/adead-borrow/src/lib.rs`)
   - ✅ Estructura básica de `BorrowChecker`
   - ✅ Tracking de ownership de variables
   - ✅ Verificación básica de movimientos
   - ✅ Soporte para scopes (variables locales)

4. **Backend NASM** (`crates/adead-backend/src/lib.rs`)
   - ✅ Soporte básico para `Borrow` y `Deref` en generación de código
   - ✅ Compatibilidad con parámetros con borrowing

---

## 📝 Sintaxis Implementada

### Borrowing Inmutable

```adead
let x = 10
let r = &x        // Prestar referencia inmutable
print r           // OK: usar referencia
```

### Borrowing Mutable

```adead
let mut x = 10
let r = &mut x    // Prestar referencia mutable
*r = 20           // Modificar a través de referencia
```

### Dereferenciación

```adead
let ptr = &x
let valor = *ptr  // Obtener valor desde referencia
```

### Funciones con Borrowing

```adead
// Función que toma borrowing inmutable
fn imprimir(&texto) {
    print texto
}

// Función que toma borrowing mutable
fn incrementar(&mut valor) {
    valor = valor + 1
}

// Uso
let x = "Hola"
imprimir(&x)      // OK: prestar referencia

let mut y = 10
incrementar(&mut y)  // OK: prestar mutable
```

---

## 🔍 Estructura del Código

### AST - Expresiones de Borrowing

```rust
pub enum Expr {
    // ... otras expresiones ...
    
    Borrow {
        expr: Box<Expr>,
        mutable: bool,  // false = &T, true = &mut T
    },
    Deref(Box<Expr>),  // *expr
}
```

### Parámetros de Función

```rust
pub struct FnParam {
    pub name: String,
    pub borrow_type: BorrowType,
}

pub enum BorrowType {
    Owned,        // Valor owned (por defecto)
    Borrowed,     // &T - referencia inmutable
    MutBorrowed,  // &mut T - referencia mutable
}
```

### Borrow Checker

```rust
pub struct BorrowChecker {
    variables: HashMap<String, VariableInfo>,
    scope_stack: Vec<HashMap<String, VariableInfo>>,
}

enum OwnershipState {
    Owned,
    Borrowed,
    MutBorrowed,
    Moved,
}
```

---

## 🎯 Próximos Pasos (Pendientes)

### Verificación Completa de Reglas

- [ ] Verificar "no aliasing mutable" (solo un `&mut` a la vez)
- [ ] Verificar que no puedes tener `&` y `&mut` simultáneamente
- [ ] Tracking completo de movimientos
- [ ] Lifetime inference básico
- [ ] Verificación de borrowing en llamadas a funciones

### Generación de Código NASM

- [ ] Generar código para `&variable` (usar `lea` en NASM)
- [ ] Generar código para `*reference` (cargar desde dirección)
- [ ] Manejar parámetros con borrowing correctamente en NASM

### Integración

- [ ] Integrar borrow checker en pipeline de compilación
- [ ] Mostrar errores de borrowing claros al usuario
- [ ] Tests de integración completos

---

## 🧪 Tests

Los siguientes tests pasan correctamente:

```rust
test_parse_borrow              // ✅ Parser para &
test_parse_mut_borrow          // ✅ Parser para &mut
test_parse_deref               // ✅ Parser para *
test_parse_fn_with_borrow_param    // ✅ Parámetros &T
test_parse_fn_with_mut_borrow_param // ✅ Parámetros &mut T
```

---

## 📚 Ejemplos de Uso

### Ejemplo 1: Borrowing Básico

```adead
let mensaje = "Hola"
let referencia = &mensaje
print referencia      // OK: usando referencia
```

### Ejemplo 2: Borrowing Mutable

```adead
let mut contador = 0
let ref_mut = &mut contador
*ref_mut = 10         // Modificar a través de referencia
print contador        // 10
```

### Ejemplo 3: Funciones con Borrowing

```adead
fn duplicar(&mut x) {
    x = x * 2
}

let mut numero = 5
duplicar(&mut numero)
print numero          // 10
```

---

## ⚠️ Limitaciones Actuales

1. **Verificación de reglas básica**: No verifica completamente "no aliasing mutable"
2. **Lifetime tracking**: No hay verificación de lifetimes todavía
3. **Código NASM**: Generación de código para borrowing es básica (TODO)
4. **Move semantics**: Tracking de movimientos es básico

---

*Documentación técnica - Ownership Implementation*
*Última actualización: Diciembre 2025*

