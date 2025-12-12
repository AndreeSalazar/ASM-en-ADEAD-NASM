# 🛠️ Implementación: Fase 1.1 - Sistema de Tipos y Ownership

**Guía técnica para implementar Fase 1.1 según ideas3.md**

> Esta fase implementa los fundamentos estilo Rust necesarios para OOP seguro

---

## 📋 Objetivos de Fase 1.1

### Componentes a Implementar

1. **O0.1 - Sistema de Tipos Robusto** ⭐⭐⭐
2. **O0.2 - Ownership y Borrowing** ⭐⭐⭐
3. **O0.3 - Inmutabilidad por Defecto** ⭐⭐⭐
4. **O0.4 - Option/Result Types** ⭐⭐⭐

---

## 🎯 O0.1 - Sistema de Tipos Robusto

### Estado Actual

**Archivo:** `crates/adead-common/src/lib.rs`

```rust
pub enum Type {
    Int64,
    String,
    Void,
}
```

**Limitaciones:**
- Solo 3 tipos básicos
- No hay tipos de enteros diferentes (int32, uint32, etc.)
- No hay tipos de punto flotante
- No hay bool explícito
- No hay arrays, tuples, Option, Result

### Plan de Implementación

#### Paso 1: Extender Tipos Primitivos

**Archivo a modificar:** `crates/adead-common/src/lib.rs`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    // Enteros con signo
    Int8,
    Int16,
    Int32,
    Int64,
    
    // Enteros sin signo
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    
    // Punto flotante
    Float32,
    Float64,
    
    // Otros primitivos
    Bool,
    Char,      // Carácter Unicode
    
    // Tipos compuestos
    String,
    Array {
        element_type: Box<Type>,
        size: Option<usize>,  // None = dinámico
    },
    Tuple(Vec<Type>),
    
    // Tipos opcionales y resultados (O0.4)
    Option(Box<Type>),
    Result {
        ok: Box<Type>,
        err: Box<Type>,
    },
    
    // Referencias (para O0.2 - Ownership)
    Ref {
        inner: Box<Type>,
        mutable: bool,  // &T vs &mut T
    },
    
    // Otros
    Void,
    Never,     // Tipo que nunca se retorna (divergente)
    
    // Inferencia
    Unknown,   // Para type inference
}
```

#### Paso 2: Métodos Útiles para Type

```rust
impl Type {
    /// Verificar si un tipo es Copy (se puede copiar)
    pub fn is_copy(&self) -> bool {
        match self {
            Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64 |
            Type::UInt8 | Type::UInt16 | Type::UInt32 | Type::UInt64 |
            Type::Float32 | Type::Float64 |
            Type::Bool | Type::Char => true,
            Type::Ref { .. } => true,  // Referencias son Copy
            _ => false,
        }
    }
    
    /// Verificar si un tipo es Sized (tamaño conocido en compile-time)
    pub fn is_sized(&self) -> bool {
        match self {
            Type::Array { size: Some(_), .. } => true,
            Type::Array { size: None, .. } => false,
            Type::String => false,  // String es dinámico
            _ => true,
        }
    }
    
    /// Convertir a string para mostrar al usuario
    pub fn to_string(&self) -> String {
        match self {
            Type::Int64 => "int64".to_string(),
            Type::String => "string".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Option(inner) => format!("Option<{}>", inner.to_string()),
            Type::Result { ok, err } => {
                format!("Result<{}, {}>", ok.to_string(), err.to_string())
            }
            Type::Ref { inner, mutable } => {
                if *mutable {
                    format!("&mut {}", inner.to_string())
                } else {
                    format!("&{}", inner.to_string())
                }
            }
            _ => format!("{:?}", self),
        }
    }
}
```

#### Paso 3: Type Inference Básico

**Archivo nuevo:** `crates/adead-typecheck/src/lib.rs`

```rust
use adead_common::Type;
use adead_parser::{Expr, Stmt};

pub struct TypeChecker {
    variables: HashMap<String, Type>,
    functions: HashMap<String, (Vec<Type>, Type)>,  // (params, return)
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }
    
    pub fn infer_expr_type(&self, expr: &Expr) -> Result<Type, String> {
        match expr {
            Expr::Number(_) => Ok(Type::Int64),
            Expr::String(_) => Ok(Type::String),
            Expr::Ident(name) => {
                self.variables.get(name)
                    .cloned()
                    .ok_or_else(|| format!("Variable no definida: {}", name))
            }
            Expr::BinaryOp { op, left, right } => {
                let left_type = self.infer_expr_type(left)?;
                let right_type = self.infer_expr_type(right)?;
                
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                        // Aritméticos: ambos deben ser numéricos
                        if left_type == right_type && left_type.is_numeric() {
                            Ok(left_type)
                        } else {
                            Err(format!("Tipos incompatibles: {} y {}", 
                                left_type.to_string(), right_type.to_string()))
                        }
                    }
                    BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | 
                    BinOp::Gt | BinOp::Ge => {
                        // Comparaciones: retornan bool
                        Ok(Type::Bool)
                    }
                }
            }
            _ => Err("Tipo no inferido".to_string()),
        }
    }
}
```

### Checklist de Implementación

- [ ] Extender enum `Type` con todos los tipos primitivos
- [ ] Agregar tipos compuestos (Array, Tuple)
- [ ] Implementar métodos `is_copy()`, `is_sized()`, `to_string()`
- [ ] Crear módulo `adead-typecheck` para type checking
- [ ] Implementar type inference básico para expresiones
- [ ] Implementar type checking para statements
- [ ] Agregar verificación de tipos en funciones
- [ ] Tests para type checking

---

## 🎯 O0.2 - Ownership y Borrowing

### Concepto

Sistema de ownership estilo Rust pero adaptado a sintaxis Python-like.

**Reglas:**
1. Cada valor tiene un único owner
2. Solo un owner a la vez
3. Cuando el owner sale de scope, el valor se libera
4. Puedes prestar (`&`) o prestar mutable (`&mut`) sin transferir ownership

### Sintaxis Propuesta

```adead
// Ownership: cada valor tiene un único dueño
let s = "Hola"        // s es owner
let s2 = s            // Move: s ya no es válido
// print s            // Error: s fue movido

// Borrowing (prestar referencia)
fn imprimir(tex: &string) {  // & = borrow (read-only)
    print tex
}

let texto = "Mundo"
imprimir(&texto)      // Prestar referencia
print texto           // OK: texto sigue siendo owner

// Mutable borrow
fn modificar(tex: &mut string) {  // &mut = mutable borrow
    tex.append("!")
}

let mut msg = "Hola"
modificar(&mut msg)   // Mutable borrow
print msg             // "Hola!"
```

### Plan de Implementación

#### Paso 1: Extender AST con Borrowing

**Archivo:** `crates/adead-parser/src/lib.rs`

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // ... existentes ...
    
    // Nuevas expresiones para borrowing
    Borrow {
        expr: Box<Expr>,
        mutable: bool,  // & vs &mut
    },
    Deref(Box<Expr>),  // * para dereferenciar
}
```

#### Paso 2: Parser para Borrowing

```rust
// En expr_parser()
let borrow = just("&")
    .then(just("mut").or_not())
    .then(expr.clone())
    .map(|((_, mutable), expr)| Expr::Borrow {
        expr: Box::new(expr),
        mutable: mutable.is_some(),
    });
```

#### Paso 3: Ownership Tracker

**Archivo nuevo:** `crates/adead-borrow/src/lib.rs`

```rust
use std::collections::HashMap;
use adead_parser::Stmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Ownership {
    Owned,           // Valor es dueño
    Borrowed(bool),  // Prestado (mutable: bool)
    Moved,           // Fue movido
}

pub struct BorrowChecker {
    variables: HashMap<String, Ownership>,
    scope_stack: Vec<HashMap<String, Ownership>>,
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            scope_stack: Vec::new(),
        }
    }
    
    pub fn check(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let { name, value } => {
                // Verificar que value puede ser movido/owned
                self.check_expr(value)?;
                self.variables.insert(name.clone(), Ownership::Owned);
                Ok(())
            }
            // ... más casos
        }
    }
    
    fn check_expr(&self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Ident(name) => {
                match self.variables.get(name) {
                    Some(Ownership::Moved) => {
                        Err(format!("Variable {} fue movida", name))
                    }
                    _ => Ok(())
                }
            }
            Expr::Borrow { expr, mutable } => {
                // Verificar que podemos tomar referencia
                self.check_expr(expr)?;
                Ok(())
            }
            // ... más casos
        }
    }
}
```

### Checklist de Implementación

- [ ] Extender AST con `Borrow` y `Deref`
- [ ] Implementar parser para `&` y `&mut`
- [ ] Crear módulo `adead-borrow` para borrow checking
- [ ] Implementar tracking de ownership
- [ ] Implementar verificación de borrows
- [ ] Verificar reglas: no aliasing mutable, etc.
- [ ] Lifetime inference básico
- [ ] Tests para ownership/borrowing

---

## 🎯 O0.3 - Inmutabilidad por Defecto

### Sintaxis

```adead
// Inmutable por defecto
let x = 10
// x = 20  // Error: x es inmutable

// Mutable explícito
let mut y = 10
y = 20  // OK
```

### Implementación

**Archivo:** `crates/adead-parser/src/lib.rs`

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    // ... existentes ...
    Let {
        mutable: bool,  // nuevo campo
        name: String,
        value: Expr,
        type_annotation: Option<Type>,
    },
}
```

**Parser:**

```rust
let let_stmt = just("let")
    .padded()
    .then(just("mut").or_not())  // Opcional "mut"
    .then(ident.clone())
    .then(type_annotation().or_not())  // : Type opcional
    .then_ignore(just("=").padded())
    .then(expr.clone())
    .map(|((((_, mutable), name), type_ann), value)| Stmt::Let {
        mutable: mutable.is_some(),
        name,
        value,
        type_annotation: type_ann,
    });
```

### Checklist

- [ ] Agregar campo `mutable: bool` a `Stmt::Let`
- [ ] Parser para `let mut`
- [ ] Verificación: no permitir mutación de variables inmutables
- [ ] Tests para inmutabilidad

---

## 🎯 O0.4 - Option/Result Types

### Sintaxis Propuesta

```adead
// Option<T> - valor opcional
fn buscar(nombre: string) -> Option<Persona> {
    if existe(nombre) {
        return Some(Persona(nombre))
    }
    return None
}

// Result<T, E> - resultado con error
fn dividir(a: int64, b: int64) -> Result<int64, string> {
    if b == 0 {
        return Err("División por cero")
    }
    return Ok(a / b)
}

// Pattern matching
match resultado {
    Ok(valor) => print valor
    Err(mensaje) => print "Error: " + mensaje
}
```

### Implementación

#### Paso 1: AST para Option/Result

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // ... existentes ...
    
    // Option/Result constructors
    Some(Box<Expr>),
    None,
    Ok(Box<Expr>),
    Err(Box<Expr>),
    
    // Pattern matching
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Some(Box<Pattern>),
    None,
    Ok(Box<Pattern>),
    Err(Box<Pattern>),
    Ident(String),  // Variable binding
    Literal(Expr),
}
```

#### Paso 2: Parser

```rust
let some_expr = just("Some")
    .ignore_then(expr.delimited_by(just("("), just(")")))
    .map(|e| Expr::Some(Box::new(e)));

let match_expr = just("match")
    .ignore_then(expr.clone())
    .then_ignore(just("{"))
    .then(match_arm().repeated())
    .then_ignore(just("}"))
    .map(|(expr, arms)| Expr::Match {
        expr: Box::new(expr),
        arms,
    });
```

### Checklist

- [ ] Extender AST con Option/Result
- [ ] Parser para `Some`, `None`, `Ok`, `Err`
- [ ] Parser para `match` expressions
- [ ] Type checking para Option/Result
- [ ] Pattern matching exhaustivo
- [ ] Operador `?` para propagación (opcional)
- [ ] Tests para Option/Result

---

## 📊 Orden de Implementación Recomendado

1. **O0.1 - Sistema de Tipos** (base para todo)
   - Extender tipos primitivos
   - Type inference básico
   - Type checking

2. **O0.3 - Inmutabilidad** (simple, no depende de otros)
   - Parser `let mut`
   - Verificación de mutabilidad

3. **O0.4 - Option/Result** (depende de tipos)
   - Option/Result types
   - Match expressions

4. **O0.2 - Ownership** (más complejo, depende de tipos)
   - Borrow tracking
   - Lifetime inference
   - Borrow checker

---

## 🧪 Estrategia de Testing

### Unit Tests

Cada componente debe tener tests unitarios:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_type_inference() {
        // ...
    }
    
    #[test]
    fn test_ownership_move() {
        // ...
    }
}
```

### Integration Tests

Tests de programas completos:

```rust
#[test]
fn test_complete_program() {
    let source = r#"
        let x = 10
        let mut y = 20
        y = x + y
    "#;
    // ...
}
```

---

## 📝 Notas de Implementación

- **Sintaxis Python-like**: Mantener simplicidad pero con seguridad de Rust
- **Mensajes de error claros**: Ayudar al usuario a entender problemas
- **Incremental**: Implementar una característica a la vez, testear bien
- **Documentación**: Documentar cada decisión de diseño

---

*Documento de implementación - Fase 1.1*
*Última actualización: Diciembre 2025*

