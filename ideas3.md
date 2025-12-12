# 💡 Ideas3 - Programación Orientada a Objetos (OOP) para ADead

**Documento de ideas para implementar características de Programación Orientada a Objetos en el lenguaje ADead**

> Este documento organiza todas las ideas OOP por categoría, prioridad y complejidad para facilitar la implementación completa de paradigma orientado a objetos.

---

## 📑 Tabla de Contenidos

1. [Tracking de Progreso](#-tracking-de-progreso)
2. [Fundamentos OOP](#-fundamentos-oop)
3. [Clases y Objetos](#-clases-y-objetos)
4. [Encapsulación](#-encapsulación)
5. [Herencia](#-herencia)
6. [Polimorfismo](#-polimorfismo)
7. [Abstracción](#-abstracción)
8. [Características Avanzadas](#-características-avanzadas)
9. [Interfaces y Traits](#-interfaces-y-traits)
10. [Memory Management](#-memory-management)
11. [Generics y Templates](#-generics-y-templates)
12. [Roadmap OOP](#-roadmap-oop)

---

## 📊 Tracking de Progreso

### Resumen General

**Total de Ideas OOP:** 35+  
**Implementadas:** 0 (MVP básico funcionando)  
**En Progreso:** 0  
**Pendientes:** 35+

### 📊 Estado Actual del Proyecto (Análisis Diciembre 2025)

**MVP Funcional:**
- ✅ Parser funcional (chumsky)
- ✅ Generación de código NASM
- ✅ CLI modular (compile, assemble, link, run)
- ✅ Ejemplos básicos funcionando

**Lenguaje Actual (MVP):**
- ✅ Tipos básicos: `int64`, `string`, `void` (muy limitado)
- ✅ Variables: `let` (sin tipos explícitos, sin `mut`)
- ✅ Funciones: `fn nombre(params) { body }`
- ✅ Control: `if/else`, `while`
- ✅ Operadores: aritméticos y de comparación
- ✅ Statements: `print`, `let`, `if`, `while`, `fn`, `return`

**⚠️ Crítico para OOP - NO Implementado:**
- ❌ Sistema de tipos robusto (solo 3 tipos básicos)
- ❌ Ownership y Borrowing
- ❌ Inmutabilidad (`mut` keyword)
- ❌ Option/Result types
- ❌ Type checking/inference

### Por Categoría (Organización Rust-like)

- 🔧 **Pre-requisitos Rust**: 0/4 (0%) - *Ownership, Types, Option/Result*
- 📦 **Fundamentos OOP**: 0/5 (0%) - *Structs/Classes, RAII, Ownership*
- 🏛️ **Clases y Objetos**: 0/6 (0%)
- 🔒 **Encapsulación**: 0/4 (0%) - *Incluye Module System*
- 👨‍👩‍👧 **Herencia**: 0/5 (0%)
- 🎭 **Polimorfismo**: 0/4 (0%)
- 🎨 **Abstracción**: 0/2 (0%)
- ⚡ **Características Avanzadas**: 0/7 (0%) - *Incluye estilo Python*
- 🔌 **Interfaces y Traits**: 0/3 (0%)
- 💾 **Memory Management**: 0/3 (0%) - *RAII, Smart Pointers*
- 🔧 **Generics y Templates**: 0/3 (0%)

### Checklist Completa por Prioridad

#### ⭐⭐⭐ Prioridad Alta (Fundamentos - Estilo Rust Mejorado)

**Fase 1.1: Sistema de Tipos y Ownership (Pre-requisitos Rust-like)**
- [x] **O0.1** - Sistema de Tipos Robusto (prerequisito de ideas2.md L1) ⚠️ **PARCIAL**
  - ✅ Estado: Enum Type extendido (Fase 1 completada)
  - ✅ Actual: Todos los tipos primitivos, Array, Tuple, Option, Result, Ref agregados
  - ✅ Compatibilidad NASM: Métodos `size_bytes()`, `nasm_register_hint()`, `nasm_declaration()`
  - ⏳ Pendiente: Módulo `adead-typecheck` para type checking/inference
  - 📝 Archivos: `crates/adead-common/src/lib.rs` ✅ COMPLETADO
  
- [x] **O0.2** - Ownership y Borrowing Básico ⚠️ **PARCIAL**
  - ✅ Estado: AST extendido, parser funcional, módulo borrow creado
  - ✅ Actual: `Borrow` y `Deref` en AST, parser para `&` y `&mut`, parámetros con borrowing
  - ✅ Módulo `adead-borrow` creado con borrow checker básico
  - ⏳ Pendiente: Verificación completa de reglas (no aliasing mutable, moves), lifetime tracking
  - 📝 Archivos: 
    - `crates/adead-parser/src/lib.rs` ✅ (AST extendido)
    - `crates/adead-borrow/src/lib.rs` ✅ (Borrow checker básico)
    - `crates/adead-backend/src/lib.rs` ✅ (Compatibilidad NASM agregada)
  
- [x] **O0.3** - Inmutabilidad por Defecto ✅ **COMPLETADO**
  - ✅ Estado: Implementado completamente
  - ✅ Actual: `Stmt::Let` tiene campo `mutable: bool`, parser reconoce `let mut`
  - ✅ Verificación: Borrow checker verifica que variables inmutables no pueden ser modificadas
  - ✅ Tests: Tests completos para parser y verificación de mutabilidad
  - 📝 Archivos: 
    - `crates/adead-parser/src/lib.rs` ✅ (parser actualizado)
    - `crates/adead-borrow/src/lib.rs` ✅ (verificación de mutabilidad)
    - `crates/adead-backend/src/lib.rs` ✅ (compatibilidad NASM)
  
- [x] **O0.4** - Option y Result Types (estilo Rust) ✅ **COMPLETADO**
  - ✅ Estado: Implementado completamente
  - ✅ Actual: AST extendido con `Some`, `None`, `Ok`, `Err`, `Match`, `Pattern`, `MatchArm`
  - ✅ Parser: Soporte completo para Option/Result/match expressions
  - ✅ Backend: Compatibilidad básica (implementación completa pendiente para tagged unions)
  - ✅ Borrow Checker: Verificación básica de Option/Result/match
  - ✅ Tests: Tests completos para parsing de Some/None/Ok/Err/match
  - 📝 Archivos: 
    - `crates/adead-parser/src/lib.rs` ✅ (AST y parser extendidos)
    - `crates/adead-common/src/lib.rs` ✅ (Type enum ya tenía Option/Result)
    - `crates/adead-borrow/src/lib.rs` ✅ (verificación básica)
    - `crates/adead-backend/src/lib.rs` ✅ (compatibilidad básica)
  - ✅ Generación de código NASM para tagged unions (Option/Result) **IMPLEMENTADA**
  - ✅ Match exhaustivo con saltos condicionales **IMPLEMENTADO**
  - ✅ Tests completos para Option/Result/match generación de código
  - 📌 Representación: Option/Result como tagged unions (16 bytes = tag 8 bytes + valor 8 bytes)
    - Option: Tag 0 = None, Tag 1 = Some(valor)
    - Result: Tag 0 = Ok(valor), Tag 1 = Err(error)

**Fase 1.2: Estructuras de Datos (Fundación)** ✅ **COMPLETADA**
- [x] O1 - Structs/Clases Básicas (inmutables por defecto) ✅ **COMPLETADO**
  - ✅ AST extendido con `Stmt::Struct`, `Expr::StructLiteral`, `Expr::FieldAccess`, `Expr::MethodCall`
  - ✅ Parser completo para definición de structs, literales, acceso a campos y llamadas a métodos
  - ✅ Campos inmutables por defecto (requieren `mut` para ser mutables)
  - ✅ Sintaxis: `struct Nombre { campo: tipo }`, `Nombre { campo: valor }`, `objeto.campo`, `objeto.metodo(args)`
  - ✅ Tests completos para parsing
- [x] O3 - Propiedades (Fields/Members) con ownership ✅ **COMPLETADO**
  - ✅ `StructField` con campo `mutable: bool` para tracking de ownership
  - ✅ Borrow checker verifica acceso a campos
  - ✅ Backend genera código NASM para acceso a campos (layout simplificado de 8 bytes por campo)
- [x] O4 - Métodos de Instancia (con borrowing) ✅ **COMPLETADO**
  - ✅ Parser para `objeto.metodo(args)`
  - ✅ Backend genera llamadas con `self` como primer argumento
  - ✅ TODO: Implementar dispatch real de métodos y binding de `&self`/`&mut self`

**Fase 1.3: Inicialización y Limpieza (RAII como Rust)**
- [ ] O2 - Constructores y Destructores (RAII automático)
- [ ] O2.1 - Drop Trait (destrucción determinística)

**Fase 1.4: Encapsulación y Seguridad**
- [ ] O5 - Encapsulación (public/private/pub)
- [ ] O5.1 - Visibility Modifiers (como Rust `pub`)

**Resumen Fase 1 (Fundamentos Rust-like):**
- [ ] O0.1 - Sistema de Tipos Robusto
- [ ] O0.2 - Ownership y Borrowing
- [ ] O0.3 - Inmutabilidad por Defecto
- [ ] O0.4 - Option/Result Types
- [ ] O1 - Structs/Clases Básicas
- [ ] O2 - Constructores y RAII
- [ ] O3 - Propiedades con Ownership
- [ ] O4 - Métodos de Instancia
- [ ] O5 - Encapsulación

#### ⭐⭐ Prioridad Media (Herencia y Polimorfismo)
- [ ] O6 - Herencia Simple
- [ ] O7 - Polimorfismo y Virtual Methods
- [ ] O8 - Métodos Estáticos
- [ ] O9 - Propiedades con Getters/Setters
- [ ] O10 - Constructores de Copia

#### ⭐ Prioridad Baja (Avanzado)
- [ ] O11 - Herencia Múltiple
- [ ] O12 - Interfaces/Traits
- [ ] O13 - Generics/Templates
- [ ] O14 - Operator Overloading
- [ ] O15 - Mixins y Composition
- [ ] O29 - Data Classes (Estilo Python)
- [ ] O30 - Slots para Optimización
- [ ] O31 - Duck Typing Mejorado
- [ ] O32 - MRO Mejorado

---

## 📦 Fundamentos OOP (Estilo Rust Mejorado)

### O0.1 - Sistema de Tipos Robusto (Prerequisito) ⭐⭐⭐

- [ ] **Estado:** ❌ NO Implementado  
**Complejidad:** 🔴 Alta | **Impacto:** 🔴 Alto | **Esfuerzo:** 60-80 horas  
**Prioridad:** 🔴 CRÍTICA - Base para todo OOP

**Descripción:** Sistema de tipos robusto y completo (ver ideas2.md L1) - **CRÍTICO antes de OOP**. Sin un sistema de tipos sólido, no se puede implementar ownership, Option/Result, ni OOP de forma segura.

---

## 📊 Análisis Detallado del Estado Actual

### ✅ Lo que Existe (MVP Actual)

**Archivo:** `crates/adead-common/src/lib.rs` (líneas 4-8)

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int64,      // ✅ Solo 1 tipo entero
    String,     // ✅ Strings básicos
    Void,       // ✅ Tipo de retorno vacío
}
```

**Limitaciones críticas:**
- ❌ Solo 3 tipos básicos (necesitamos 15+ tipos)
- ❌ No hay tipos de punto flotante
- ❌ No hay `bool` explícito
- ❌ No hay arrays, tuples, Option, Result
- ❌ No hay type checking/inference
- ❌ No hay métodos útiles (`is_copy()`, `to_string()`, etc.)

### ❌ Lo que Falta (Crítico)

#### 1. Tipos Primitivos Completos
- ❌ Enteros con signo: `int8`, `int16`, `int32`, `int64`
- ❌ Enteros sin signo: `uint8`, `uint16`, `uint32`, `uint64`
- ❌ Punto flotante: `float32`, `float64`
- ❌ Booleanos: `bool`
- ❌ Caracteres: `char` (Unicode)

#### 2. Tipos Compuestos
- ❌ Arrays: `Array<T>` (tamaño fijo y dinámico)
- ❌ Tuples: `Tuple(T1, T2, ...)`

#### 3. Tipos Avanzados (para O0.4)
- ❌ `Option<T>` - valores opcionales
- ❌ `Result<T, E>` - manejo de errores

#### 4. Referencias (para O0.2 - Ownership)
- ❌ `Ref { inner, mutable }` - `&T` y `&mut T`

#### 5. Sistema de Type Checking
- ❌ No existe módulo `adead-typecheck`
- ❌ No hay type inference
- ❌ No hay verificación de tipos en compilación
- ❌ No hay mensajes de error de tipos claros

---

## 🎯 Plan de Implementación Paso a Paso

### Fase 1: Extender Enum Type (Día 1-2)

**Archivo:** `crates/adead-common/src/lib.rs`

#### Paso 1.1: Reemplazar enum Type existente

**Antes (líneas 4-8):**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int64,
    String,
    Void,
}
```

**Después:**
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    // ========== ENTEROS CON SIGNO ==========
    Int8,
    Int16,
    Int32,
    Int64,
    
    // ========== ENTEROS SIN SIGNO ==========
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    
    // ========== PUNTO FLOTANTE ==========
    Float32,
    Float64,
    
    // ========== OTROS PRIMITIVOS ==========
    Bool,
    Char,      // Carácter Unicode
    
    // ========== TIPOS COMPUESTOS ==========
    String,
    Array {
        element_type: Box<Type>,
        size: Option<usize>,  // Some(n) = tamaño fijo, None = dinámico
    },
    Tuple(Vec<Type>),
    
    // ========== TIPOS OPCIONALES Y ERRORES (O0.4) ==========
    Option(Box<Type>),
    Result {
        ok: Box<Type>,
        err: Box<Type>,
    },
    
    // ========== REFERENCIAS (O0.2 - Ownership) ==========
    Ref {
        inner: Box<Type>,
        mutable: bool,  // false = &T, true = &mut T
    },
    
    // ========== OTROS ==========
    Void,
    Never,     // Tipo que nunca retorna (funciones divergentes)
    
    // ========== INFERENCIA ==========
    Unknown,   // Para type inference durante análisis
}
```

**Checklist:**
- [ ] Reemplazar enum Type completo
- [ ] Agregar `Eq, Hash` a derives (necesario para HashMap)
- [ ] Compilar: `cargo build` debe funcionar
- [ ] Verificar que no rompe código existente

#### Paso 1.2: Agregar Métodos Útiles

**Después del enum, agregar `impl Type`:**

```rust
impl Type {
    /// Verificar si un tipo es Copy (se puede copiar, no se mueve)
    pub fn is_copy(&self) -> bool {
        match self {
            // Todos los primitivos son Copy
            Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64 |
            Type::UInt8 | Type::UInt16 | Type::UInt32 | Type::UInt64 |
            Type::Float32 | Type::Float64 |
            Type::Bool | Type::Char => true,
            // Referencias son Copy (la referencia misma, no lo que apunta)
            Type::Ref { .. } => true,
            // Tipos compuestos generalmente no son Copy
            _ => false,
        }
    }
    
    /// Verificar si un tipo es Sized (tamaño conocido en compile-time)
    pub fn is_sized(&self) -> bool {
        match self {
            Type::Array { size: Some(_), .. } => true,
            Type::Array { size: None, .. } => false,  // Array dinámico
            Type::String => false,  // String es dinámico (heap)
            _ => true,
        }
    }
    
    /// Verificar si un tipo es numérico (enteros o flotantes)
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64 |
            Type::UInt8 | Type::UInt16 | Type::UInt32 | Type::UInt64 |
            Type::Float32 | Type::Float64
        )
    }
    
    /// Verificar si un tipo es entero
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64 |
            Type::UInt8 | Type::UInt16 | Type::UInt32 | Type::UInt64
        )
    }
    
    /// Convertir a string legible para el usuario
    pub fn to_string(&self) -> String {
        match self {
            // Primitivos
            Type::Int8 => "int8".to_string(),
            Type::Int16 => "int16".to_string(),
            Type::Int32 => "int32".to_string(),
            Type::Int64 => "int64".to_string(),
            Type::UInt8 => "uint8".to_string(),
            Type::UInt16 => "uint16".to_string(),
            Type::UInt32 => "uint32".to_string(),
            Type::UInt64 => "uint64".to_string(),
            Type::Float32 => "float32".to_string(),
            Type::Float64 => "float64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Char => "char".to_string(),
            Type::String => "string".to_string(),
            Type::Void => "void".to_string(),
            Type::Never => "!".to_string(),
            Type::Unknown => "_".to_string(),
            
            // Tipos compuestos
            Type::Array { element_type, size } => {
                let size_str = match size {
                    Some(n) => format!("{}", n),
                    None => "".to_string(),
                };
                format!("Array<{}{}>", element_type.to_string(), 
                    if size_str.is_empty() { "".to_string() } 
                    else { format!(", {}", size_str) })
            }
            Type::Tuple(types) => {
                let types_str: Vec<String> = types.iter()
                    .map(|t| t.to_string())
                    .collect();
                format!("({})", types_str.join(", "))
            }
            
            // Option/Result
            Type::Option(inner) => format!("Option<{}>", inner.to_string()),
            Type::Result { ok, err } => {
                format!("Result<{}, {}>", ok.to_string(), err.to_string())
            }
            
            // Referencias
            Type::Ref { inner, mutable } => {
                if *mutable {
                    format!("&mut {}", inner.to_string())
                } else {
                    format!("&{}", inner.to_string())
                }
            }
        }
    }
    
    /// Tipo por defecto para literales numéricos (int64)
    pub fn default_int() -> Self {
        Type::Int64
    }
    
    /// Tipo por defecto para literales flotantes (float64)
    pub fn default_float() -> Self {
        Type::Float64
    }
}
```

**Checklist:**
- [ ] Implementar `is_copy()` - crítico para ownership
- [ ] Implementar `is_sized()` - útil para arrays
- [ ] Implementar `is_numeric()` - para operaciones aritméticas
- [ ] Implementar `to_string()` - para mensajes de error
- [ ] Tests básicos para cada método

---

### Fase 2: Crear Módulo Type Checker (Día 3-5)

**Nuevo módulo:** `crates/adead-typecheck/`

#### Paso 2.1: Crear el crate

```bash
cd crates
cargo new --lib adead-typecheck
cd ..
```

#### Paso 2.2: Agregar dependencias en `Cargo.toml`

**Archivo:** `crates/adead-typecheck/Cargo.toml`

```toml
[package]
name = "adead-typecheck"
version = "0.1.0"
edition = "2021"

[dependencies]
adead-common = { path = "../adead-common" }
adead-parser = { path = "../adead-parser" }
```

#### Paso 2.3: Estructura básica del Type Checker

**Archivo:** `crates/adead-typecheck/src/lib.rs`

```rust
use adead_common::{Type, ADeadError, Result};
use adead_parser::{Expr, Stmt, BinOp, Program};
use std::collections::HashMap;

/// Type checker para verificar tipos en tiempo de compilación
pub struct TypeChecker {
    /// Variables y sus tipos
    variables: HashMap<String, Type>,
    /// Funciones: nombre -> (parámetros, tipo_retorno)
    functions: HashMap<String, (Vec<Type>, Type)>,
    /// Scope stack para variables locales
    scope_stack: Vec<HashMap<String, Type>>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            scope_stack: Vec::new(),
        }
    }
    
    /// Verificar tipos en un programa completo
    pub fn check(&mut self, program: &Program) -> Result<()> {
        for stmt in &program.statements {
            self.check_stmt(stmt)?;
        }
        Ok(())
    }
    
    /// Verificar tipo de una expresión
    pub fn infer_expr_type(&self, expr: &Expr) -> Result<Type> {
        match expr {
            Expr::Number(_) => Ok(Type::Int64),  // Por defecto int64
            Expr::String(_) => Ok(Type::String),
            Expr::Ident(name) => {
                // Buscar variable en scope actual o global
                self.find_variable_type(name)
            }
            Expr::BinaryOp { op, left, right } => {
                self.infer_binary_op_type(op, left, right)
            }
            Expr::Assign { name, value } => {
                let value_type = self.infer_expr_type(value)?;
                // Verificar que la variable existe y el tipo coincide
                Ok(value_type)
            }
            Expr::Call { name, args } => {
                self.infer_call_type(name, args)
            }
        }
    }
    
    /// Inferir tipo de operación binaria
    fn infer_binary_op_type(&self, op: &BinOp, left: &Expr, right: &Expr) -> Result<Type> {
        let left_type = self.infer_expr_type(left)?;
        let right_type = self.infer_expr_type(right)?;
        
        match op {
            // Operadores aritméticos: ambos deben ser numéricos del mismo tipo
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                if left_type.is_numeric() && left_type == right_type {
                    Ok(left_type)
                } else {
                    Err(ADeadError::TypeError {
                        message: format!(
                            "Operación aritmética requiere tipos numéricos iguales, \
                             pero se encontró {} y {}",
                            left_type.to_string(),
                            right_type.to_string()
                        ),
                    })
                }
            }
            // Operadores de comparación: retornan bool
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                if left_type == right_type {
                    Ok(Type::Bool)
                } else {
                    Err(ADeadError::TypeError {
                        message: format!(
                            "Comparación requiere tipos iguales, \
                             pero se encontró {} y {}",
                            left_type.to_string(),
                            right_type.to_string()
                        ),
                    })
                }
            }
        }
    }
    
    /// Encontrar tipo de una variable
    fn find_variable_type(&self, name: &str) -> Result<Type> {
        // Buscar en scope actual
        for scope in self.scope_stack.iter().rev() {
            if let Some(typ) = scope.get(name) {
                return Ok(typ.clone());
            }
        }
        // Buscar en variables globales
        self.variables.get(name)
            .cloned()
            .ok_or_else(|| ADeadError::TypeError {
                message: format!("Variable no definida: {}", name),
            })
    }
    
    /// Verificar statement
    fn check_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let { name, value } => {
                let value_type = self.infer_expr_type(value)?;
                self.variables.insert(name.clone(), value_type);
                Ok(())
            }
            Stmt::Print(expr) => {
                let _ = self.infer_expr_type(expr)?;  // Verificar que es válido
                Ok(())
            }
            Stmt::If { condition, then_body, else_body, .. } => {
                let cond_type = self.infer_expr_type(condition)?;
                if cond_type != Type::Bool {
                    return Err(ADeadError::TypeError {
                        message: format!(
                            "Condición if debe ser bool, pero se encontró {}",
                            cond_type.to_string()
                        ),
                    });
                }
                // Verificar then_body y else_body
                self.push_scope();
                for stmt in then_body {
                    self.check_stmt(stmt)?;
                }
                self.pop_scope();
                
                if let Some(else_body) = else_body {
                    self.push_scope();
                    for stmt in else_body {
                        self.check_stmt(stmt)?;
                    }
                    self.pop_scope();
                }
                Ok(())
            }
            Stmt::While { condition, body } => {
                let cond_type = self.infer_expr_type(condition)?;
                if cond_type != Type::Bool {
                    return Err(ADeadError::TypeError {
                        message: format!(
                            "Condición while debe ser bool, pero se encontró {}",
                            cond_type.to_string()
                        ),
                    });
                }
                self.push_scope();
                for stmt in body {
                    self.check_stmt(stmt)?;
                }
                self.pop_scope();
                Ok(())
            }
            Stmt::Fn { name, params, body, .. } => {
                // Registrar función (tipo de retorno Void por ahora)
                let param_types: Vec<Type> = params.iter()
                    .map(|_| Type::Unknown)  // Inferir después
                    .collect();
                self.functions.insert(name.clone(), (param_types, Type::Void));
                Ok(())
            }
            _ => Ok(())  // Otros statements
        }
    }
    
    fn push_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }
    
    fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }
    
    fn infer_call_type(&self, name: &str, args: &[Expr]) -> Result<Type> {
        // Por ahora, retornar Void para funciones
        // TODO: implementar correctamente
        Ok(Type::Void)
    }
}
```

**Checklist:**
- [ ] Crear crate `adead-typecheck`
- [ ] Implementar `TypeChecker` básico
- [ ] Implementar `infer_expr_type()` para expresiones básicas
- [ ] Implementar verificación de tipos para statements
- [ ] Manejo de scopes (variables locales)
- [ ] Tests básicos

---

### Fase 3: Integrar Type Checker (Día 6-7)

#### Paso 3.1: Integrar en CLI

**Archivo:** `crates/adead-cli/src/main.rs`

Agregar type checking antes de generar código:

```rust
use adead_typecheck::TypeChecker;

// En la función compile:
let program = adead_parser::parse(&source)?;

// NUEVO: Type checking
let mut type_checker = TypeChecker::new();
type_checker.check(&program)?;

// Generar código ASM
let asm = adead_backend::generate(&program)?;
```

**Checklist:**
- [ ] Agregar `adead-typecheck` como dependencia en `adead-cli`
- [ ] Integrar type checking en pipeline de compilación
- [ ] Mostrar errores de tipo claros al usuario
- [ ] Probar con ejemplos existentes

---

### Fase 4: Parser para Tipos Explícitos (Opcional - Día 8+)

Para permitir anotaciones de tipo explícitas:

```adead
let x: int32 = 10
let nombre: string = "Juan"
```

Esto requiere extender el parser (ver O0.3 para integración con `mut`).

---

## ✅ Checklist Completo de Implementación

### Fase 1: Tipos Básicos (2-3 días)
- [ ] Extender enum `Type` con todos los primitivos
- [ ] Agregar tipos compuestos (`Array`, `Tuple`)
- [ ] Agregar `Option`, `Result`, `Ref` (preparación)
- [ ] Implementar métodos `is_copy()`, `is_sized()`, `is_numeric()`
- [ ] Implementar `to_string()` completo
- [ ] Tests unitarios para cada tipo

### Fase 2: Type Checker (3-5 días)
- [ ] Crear crate `adead-typecheck`
- [ ] Implementar `TypeChecker` struct
- [ ] Implementar `infer_expr_type()` básico
- [ ] Implementar verificación para operaciones binarias
- [ ] Implementar verificación para statements (`let`, `if`, `while`)
- [ ] Manejo de scopes (variables locales)
- [ ] Mensajes de error claros

### Fase 3: Integración (1-2 días)
- [ ] Integrar type checker en CLI
- [ ] Mostrar errores de tipo al compilar
- [ ] Probar con ejemplos existentes
- [ ] Verificar que no rompe código actual

### Fase 4: Mejoras (Opcional)
- [ ] Type inference más avanzado
- [ ] Anotaciones de tipo explícitas (`let x: int32 = 10`)
- [ ] Type checking para funciones
- [ ] Subtipado básico (coerción de tipos)

---

## 📚 Ejemplos de Uso Después de Implementar

```adead
// Type inference automático
let x = 42              // infiere int64
let y = 3.14            // infiere float64
let texto = "Hola"      // infiere string
let flag = true         // infiere bool

// Arrays (cuando se implemente)
let numeros: Array<int64> = [1, 2, 3]

// Option (cuando se implemente O0.4)
let valor: Option<int64> = Some(42)

// Result (cuando se implemente O0.4)
fn dividir(a: int64, b: int64) -> Result<int64, string> {
    if b == 0 {
        return Err("División por cero")
    }
    return Ok(a / b)
}
```

---

## 🎯 Resultado Esperado

Después de completar O0.1:

1. ✅ Sistema de tipos completo con 15+ tipos
2. ✅ Type checking básico funcionando
3. ✅ Mensajes de error de tipo claros
4. ✅ Base sólida para implementar O0.2, O0.3, O0.4
5. ✅ Preparado para OOP (O1+)

---

## 📖 Documentación de Referencia

- Ver `Ejemplos-Reales/documentacion/IMPLEMENTACION-Fase-1.1.md` para detalles técnicos completos
- Ver `Ejemplos-Reales/documentacion/01-Basico-Tipos.md` para documentación de usuario

**Nota:** Este es prerequisito CRÍTICO de ideas2.md L1 - debe estar implementado antes de OOP

---

### O0.2 - Ownership y Borrowing Básico ⭐⭐⭐
- [ ] **Estado:** ❌ NO Implementado  
**Complejidad:** 🔴 Alta | **Impacto:** 🔴 Alto | **Esfuerzo:** 40-60 horas

**Descripción:** Sistema de ownership estilo Rust pero simplificado (fundación para OOP seguro)

**📊 Análisis del Estado Actual:**

**Lo que existe:**
- ✅ Variables básicas con `let` (sin ownership tracking)
- ✅ Asignación simple (sin move semantics)
- ✅ Parser funcional en `crates/adead-parser/src/lib.rs`

**Lo que falta:**
- ❌ AST sin `Borrow` o `Deref` (líneas 5-22 en parser)
- ❌ No hay parser para `&` o `&mut`
- ❌ No existe módulo `adead-borrow` para borrow checking
- ❌ No hay tracking de ownership
- ❌ No hay reglas de borrowing (no aliasing mutable)
- ❌ No hay lifetime inference

**Sintaxis Propuesta:**
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

**Reglas de Ownership:**
1. **Move semantics**: Asignación mueve ownership (no copia por defecto)
2. **Borrowing**: `&T` = referencia inmutable, `&mut T` = referencia mutable
3. **Lifetime**: Referencias deben vivir mientras el objeto existe
4. **Sin aliasing mutable**: Solo un `&mut` a la vez, o múltiples `&`

**Checklist de Implementación:**
- [ ] Sistema de ownership tracking
- [ ] Borrow checker (verificación en compilación)
- [ ] Move semantics por defecto
- [ ] Borrowing con `&` y `&mut`
- [ ] Lifetime inference básico
- [ ] Reglas de préstamo (no aliasing mutable)
- [ ] Tests para ownership

**Beneficio:** Memory safety sin GC, como Rust

---

### O0.3 - Inmutabilidad por Defecto ⭐⭐⭐
- [ ] **Estado:** ❌ NO Implementado (PRIORIDAD ALTA - Más fácil de implementar)  
**Complejidad:** 🟡 Media | **Impacto:** 🔴 Alto | **Esfuerzo:** 15-25 horas

**Descripción:** Todo es inmutable por defecto (como Rust), `mut` para mutar

**📊 Análisis del Estado Actual:**

**Lo que existe:**
- ✅ `Stmt::Let` en `crates/adead-parser/src/lib.rs` (líneas 41-44):
  ```rust
  Let {
      name: String,
      value: Expr,
  }
  ```
- ✅ Parser para `let` statement (líneas 104-109)

**Lo que falta:**
- ❌ Campo `mutable: bool` en `Stmt::Let`
- ❌ Parser no reconoce `let mut` (solo `let`)
- ❌ No hay verificación de mutabilidad en compilación
- ❌ No se previene mutar variables inmutables

**💡 Recomendación:** Implementar PRIMERO - Es el cambio más simple y no depende de otros sistemas.

**Sintaxis Propuesta:**
```adead
// Inmutable por defecto
let x = 10
// x = 20  // Error: x es inmutable

// Mutable explícito
let mut y = 10
y = 20  // OK

// En structs/clases
struct Punto {
    x: int64
    y: int64
}

let p = Punto(10, 20)
// p.x = 30  // Error: p es inmutable

let mut p2 = Punto(10, 20)
p2.x = 30  // OK: p2 es mutable
```

**Checklist de Implementación:**
- [ ] Inmutabilidad por defecto
- [ ] Keyword `mut` para variables mutables
- [ ] Verificación en compilación
- [ ] Mutabilidad en structs/objetos
- [ ] Tests para inmutabilidad

**Beneficio:** Menos bugs, código más seguro

---

### O0.4 - Option y Result Types (Estilo Rust) ⭐⭐⭐
- [ ] **Estado:** ❌ NO Implementado  
**Complejidad:** 🟡 Media | **Impacto:** 🔴 Alto | **Esfuerzo:** 20-30 horas

**Descripción:** Manejo seguro de errores y valores opcionales (sin null/nil)

**📊 Análisis del Estado Actual:**

**Lo que existe:**
- ✅ `Expr` enum básico en `crates/adead-parser/src/lib.rs` (líneas 5-22)
- ✅ Parser funcional para expresiones básicas

**Lo que falta:**
- ❌ No hay `Some`, `None`, `Ok`, `Err` en `Expr` enum
- ❌ No hay `Match` expression en AST
- ❌ No hay `Option<T>` o `Result<T, E>` en `Type` enum
- ❌ Parser no reconoce `match`, `Some`, `None`, `Ok`, `Err`
- ❌ No hay operador `?` para propagación de errores
- ❌ No hay pattern matching

**Sintaxis Propuesta:**
```adead
// Option<T> - valor opcional (Some o None)
fn buscar(nombre: string) -> Option<Persona> {
    if existe(nombre) {
        return Some(Persona(nombre))
    }
    return None
}

let persona = buscar("Juan")
match persona {
    Some(p) => print p.nombre
    None => print "No encontrado"
}

// Result<T, E> - resultado con error
fn dividir(a: int64, b: int64) -> Result<int64, string> {
    if b == 0 {
        return Err("División por cero")
    }
    return Ok(a / b)
}

let resultado = dividir(10, 2)
match resultado {
    Ok(valor) => print valor
    Err(mensaje) => print "Error: " + mensaje
}

// Pattern matching con `?` operator (propagación)
fn calcular() -> Result<int64, string> {
    let a = dividir(10, 2)?  // Si es Err, retorna Err
    let b = dividir(20, 4)?  // Si es Ok, extrae valor
    return Ok(a + b)
}
```

**Checklist de Implementación:**
- [ ] Type `Option<T>` (Some/None)
- [ ] Type `Result<T, E>` (Ok/Err)
- [ ] Pattern matching con `match`
- [ ] `?` operator para propagación
- [ ] Métodos: `unwrap`, `expect`, `map`, `and_then`
- [ ] Tests para Option/Result

**Beneficio:** Sin null pointer exceptions, manejo de errores explícito

---

### O1 - Structs/Clases Básicas (Inmutables por Defecto) ⭐⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🔴 Alta | **Impacto:** 🔴 Alto | **Esfuerzo:** 40-60 horas

**Descripción:** Sistema básico de clases y objetos - la base de todo OOP

**Sintaxis Propuesta (Rust-like pero sintaxis Python):**
```adead
// Struct simple (inmutable por defecto, como Rust)
struct Persona {
    nombre: string
    edad: int64
}

// Creación simple (sin constructor explícito si es struct simple)
let p = Persona {
    nombre: "Juan",
    edad: 25
}

// Actualizar struct (move + crear nuevo, estilo Rust)
let p2 = Persona {
    nombre: "María",
    ..p  // Copy resto de campos
}

// Clase completa (cuando necesitas métodos)
class Persona {
    nombre: string
    edad: int64
    
    // Constructor
    init(nombre: string, edad: int64) {
        self.nombre = nombre
        self.edad = edad
    }
    
    // Métodos que toman &self (borrowing, no consume)
    fn saludar(&self) {  // &self = inmutable borrow
        print "Hola, soy " + self.nombre
    }
    
    // Métodos que toman &mut self (mutable borrow)
    fn cumplir_anios(&mut self) {  // &mut self = mutable borrow
        self.edad = self.edad + 1
    }
    
    // Métodos que toman self (ownership, consume objeto)
    fn to_string(self) -> string {  // self = move (consume)
        return self.nombre + " (" + self.edad + " años)"
    }
}

// Uso
let mut p = Persona("Juan", 25)
p.saludar()        // Borrow inmutable
p.cumplir_anios()  // Borrow mutable
let texto = p.to_string()  // Move (p ya no es válido)
```

**Conceptos Rust aplicados:**
- ✅ Inmutabilidad por defecto (`let` vs `let mut`)
- ✅ `&self` = borrowing inmutable (métodos que no modifican)
- ✅ `&mut self` = borrowing mutable (métodos que modifican)
- ✅ `self` = ownership (método consume el objeto)
- ✅ Struct simple para datos, Class para comportamiento
- ✅ Update syntax con `..` (estructura actualización)

**Checklist de Implementación:**
- [ ] Distinción `struct` (datos) vs `class` (comportamiento)
- [ ] `&self`, `&mut self`, `self` en métodos
- [ ] Inmutabilidad por defecto
- [ ] Update syntax `..estructura`
- [ ] Ownership tracking en métodos
- [ ] Tests para structs/clases

**Checklist de Implementación:**
- [ ] Extender parser para sintaxis `class`
- [ ] Definir estructura AST para clases
- [ ] Implementar campos (fields) de clase
- [ ] Implementar métodos de instancia
- [ ] Sintaxis `self` o `this` para referenciar instancia
- [ ] Creación de objetos con `new` (opcional) o llamada directa
- [ ] Acceso a miembros con `.` (punto)
- [ ] Generación de código ASM para clases
- [ ] VTable para métodos virtuales (preparación)
- [ ] Tests para clases básicas

**Consideraciones:**
- Representación en memoria (estructura vs clases)
- Layout de objetos en memoria
- Alineación de memoria para eficiencia

---

### O2 - Constructores y RAII (Resource Acquisition Is Initialization) ⭐⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🔴 Alto | **Esfuerzo:** 20-30 horas

**Descripción:** RAII como Rust - recursos adquiridos en construcción, liberados en destrucción

**Sintaxis Propuesta (Mejora sobre Python):**
```adead
class Recurso {
    archivo: FileHandle?
    
    // Constructor con parámetros
    init(ruta: string) {
        self.archivo = abrir_archivo(ruta)
    }
    
    // Constructor sin parámetros (default)
    init() {
        self.archivo = null
    }
    
    // Destructor (se llama automáticamente, mejor que Python)
    destroy() {
        if self.archivo != null {
            cerrar_archivo(self.archivo)
        }
    }
    
    // Context manager (como Python with, pero más simple)
    fn enter() -> Recurso {
        return self
    }
    
    fn exit() {
        self.destroy()
    }
}

// Uso simple
let r = Recurso("archivo.txt")
// Recurso se libera automáticamente al salir de scope

// Context manager style (opcional, como Python with)
using r = Recurso("archivo.txt") {
    // usar recurso
}  // destroy() llamado automáticamente
```

**Mejoras sobre Python:**
- ✅ Destructores automáticos (Python requiere `with` o `__del__` poco confiable)
- ✅ RAII por defecto (más seguro)
- ✅ Sintaxis `using` más simple que `with` de Python

**Checklist de Implementación:**
- [ ] Sintaxis `init` para constructores
- [ ] Sintaxis `destroy` para destructores
- [ ] Múltiples constructores (overloading)
- [ ] Constructor por defecto (si no se define)
- [ ] Llamadas automáticas a destructores (RAII)
- [ ] Orden de inicialización de campos
- [ ] Tests para constructores/destructores

**Nota:** Requiere O1 (Clases Básicas) primero

---

### O3 - Propiedades con Ownership ⭐⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🔴 Alto | **Esfuerzo:** 20-30 horas

**Descripción:** Campos con ownership tracking (estilo Rust)

**Sintaxis Propuesta:**
```adead
// Struct simple (ownership claro)
struct Punto {
    x: int64
    y: int64
}

let p = Punto { x: 10, y: 20 }
let p2 = p  // Move: p ya no es válido

// Clase con ownership tracking
class Persona {
    nombre: string      // Owned
    direccion: &string  // Borrowed (referencia)
    edad: int64        // Copied (primitivo)
}

// Field access con mutabilidad
struct Rectangulo {
    mut ancho: int64   // Campo mutable
    mut alto: int64
    readonly id: int64 // Campo inmutable (readonly)
}

let mut rect = Rectangulo { ancho: 10, alto: 20, id: 1 }
rect.ancho = 15  // OK: ancho es mutable
// rect.id = 2    // Error: id es readonly
```

**Ownership en campos:**
- **Owned fields**: La estructura/clase es dueña del valor
- **Borrowed fields**: `&T` o `&mut T` - referencia prestada
- **Copied fields**: Tipos primitivos (Copy trait)
- **Move semantics**: Campos complejos se mueven

**Checklist de Implementación:**
- [ ] Campos owned (ownership)
- [ ] Campos borrowed (`&T`, `&mut T`)
- [ ] Campos mutables (`mut` keyword)
- [ ] Campos readonly
- [ ] Field-level mutability
- [ ] Copy vs Move para campos
- [ ] Lifetime tracking en borrowed fields
- [ ] Tests para ownership en campos

**Nota:** Requiere O1, O0.2 (Ownership)

---

---

### O4 - Métodos con Borrowing (Rust-style) ⭐⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🔴 Alto | **Esfuerzo:** 25-35 horas

**Descripción:** Métodos con `&self`, `&mut self`, `self` (estilo Rust)

**Sintaxis Propuesta:**
```adead
class Contador {
    valor: int64
    
    init(valor: int64) {
        self.valor = valor
    }
    
    // Método que no modifica (&self - borrowing inmutable)
    fn obtener(&self) -> int64 {
        return self.valor
    }
    
    // Método que modifica (&mut self - borrowing mutable)
    fn incrementar(&mut self) {
        self.valor = self.valor + 1
    }
    
    fn incrementar_por(&mut self, cantidad: int64) {
        self.valor = self.valor + cantidad
    }
    
    // Método que consume (self - ownership)
    fn tomar_valor(self) -> int64 {
        return self.valor  // self se destruye después
    }
    
    // Método estático (sin self)
    static fn nuevo(valor: int64) -> Contador {
        return Contador(valor)
    }
}

// Uso
let mut contador = Contador(10)
print contador.obtener()      // &self: no modifica
contador.incrementar()        // &mut self: modifica
contador.incrementar_por(5)   // &mut self: modifica
let valor = contador.tomar_valor()  // self: consume
// contador ya no es válido aquí
```

**Tipos de métodos:**
1. **`&self`**: Métodos que no modifican (read-only)
2. **`&mut self`**: Métodos que modifican el objeto
3. **`self`**: Métodos que consumen el objeto (move)
4. **`static`**: Métodos que no necesitan instancia

**Checklist de Implementación:**
- [ ] `&self` para métodos inmutables
- [ ] `&mut self` para métodos mutables
- [ ] `self` para métodos que consumen
- [ ] Verificación de borrowing en compilación
- [ ] Métodos estáticos (`static`)
- [ ] Multiple borrows inmutables permitidos
- [ ] Solo un borrow mutable a la vez
- [ ] Tests para diferentes tipos de métodos

**Reglas de Borrowing:**
- Puedes tener múltiples `&self` simultáneos
- Solo puedes tener un `&mut self` a la vez
- No puedes tener `&self` y `&mut self` simultáneos

**Nota:** Requiere O1, O0.2 (Ownership)

---

---

## 🏛️ Clases y Objetos

### O5 - Métodos Estáticos ⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🟡 Medio | **Esfuerzo:** 10-15 horas

**Descripción:** Métodos que pertenecen a la clase, no a instancias

**Sintaxis Propuesta:**
```adead
class Math {
    static fn max(a: int64, b: int64) -> int64 {
        if a > b {
            return a
        }
        return b
    }
}

let resultado = Math.max(10, 20)  // Sin instanciar
```

**Checklist de Implementación:**
- [ ] Keyword `static` para métodos estáticos
- [ ] Llamadas sin instancia (`Clase.metodo()`)
- [ ] No acceso a `self` en métodos estáticos
- [ ] Campos estáticos
- [ ] Tests para métodos estáticos

---

### O6 - Propiedades con Getters/Setters ⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🟡 Medio | **Esfuerzo:** 15-20 horas

**Descripción:** Control de acceso a propiedades (como Python @property pero mejor)

**Sintaxis Propuesta (Estilo Python Mejorado):**
```adead
class Persona {
    private _nombre: string  // Campo privado
    private _edad: int64
    
    // Propiedad simple (getter + setter)
    property nombre: string {
        get {
            return self._nombre
        }
        set(valor: string) {
            if len(valor) > 0 {
                self._nombre = valor
            } else {
                raise "Nombre no puede estar vacío"
            }
        }
    }
    
    // Propiedad de solo lectura (solo getter)
    property edad: int64 {
        get {
            return self._edad
        }
    }
    
    // Propiedad calculada (como Python property)
    property es_mayor_edad: bool {
        get {
            return self._edad >= 18
        }
    }
    
    // Decorador @property style (alternativa más Python-like)
    @property
    fn nombre_completo() -> string {
        return self._nombre + " (" + self._edad + " años)"
    }
}

let p = Persona()
p.nombre = "Juan"        // Llama al setter
print p.nombre           // Llama al getter
print p.es_mayor_edad    // Propiedad calculada
print p.nombre_completo  // Decorador @property
// p.edad = 25           // Error: solo lectura
```

**Mejoras sobre Python:**
- ✅ Sintaxis más explícita y clara
- ✅ Propiedades de solo lectura más fáciles
- ✅ Soporte para decoradores `@property` (familiar)
- ✅ Validación automática en setters

**Checklist de Implementación:**
- [ ] Sintaxis `get` para getters
- [ ] Sintaxis `set` para setters
- [ ] Acceso transparente (parece propiedad normal)
- [ ] Validación en setters
- [ ] Computed properties (propiedades calculadas)
- [ ] Tests para getters/setters

---

### O7 - Constructores de Copia y Asignación ⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🟡 Medio | **Esfuerzo:** 15-20 horas

**Descripción:** Crear copias de objetos y asignación

**Sintaxis Propuesta:**
```adead
class Vector {
    x: int64
    y: int64
    
    init(x: int64, y: int64) {
        self.x = x
        self.y = y
    }
    
    // Constructor de copia
    copy(other: Vector) {
        self.x = other.x
        self.y = other.y
    }
}

let v1 = Vector(10, 20)
let v2 = v1.copy()  // Copia
let v3 = v1         // ¿Referencia o copia? (diseño)
```

**Checklist de Implementación:**
- [ ] Constructor de copia
- [ ] Semántica: copia vs referencia
- [ ] Asignación de objetos
- [ ] Copy vs move semantics
- [ ] Deep copy vs shallow copy
- [ ] Tests para copias

---

## 🔒 Encapsulación

### O5 - Encapsulación con Visibility Modifiers ⭐⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🔴 Alto | **Esfuerzo:** 20-30 horas

**Descripción:** Control de visibilidad estilo Rust (`pub`, `pub(crate)`, etc.)

**Sintaxis Propuesta (Rust-like):**
```adead
// Por defecto: privado (priv)
class Banco {
    // Privado (solo dentro del módulo/archivo)
    saldo: int64
    contraseña: string
    
    // Público (accesible desde cualquier lugar)
    pub nombre: string
    
    // Público dentro del crate/módulo
    pub(crate) tasa_interes: float64
    
    // Público dentro del mismo módulo padre
    pub(super) configuracion: Config
    
    // Constructor público
    pub init(nombre: string) {
        self.nombre = nombre
        self.saldo = 0
        self.contraseña = ""
    }
    
    // Método público
    pub fn depositar(&mut self, monto: int64) {
        self.saldo = self.saldo + monto
    }
    
    // Método privado
    fn verificar_contraseña(&self, pass: string) -> bool {
        return self.contraseña == pass
    }
    
    // Método público que usa privado
    pub fn autenticar(&self, pass: string) -> bool {
        return self.verificar_contraseña(pass)
    }
}

// Uso
let mut banco = Banco("Mi Banco")
banco.depositar(100)          // OK: público
// banco.contraseña = "123"   // Error: privado
// banco.verificar_contraseña("123")  // Error: privado
banco.autenticar("123")       // OK: método público
```

**Visibility Levels (estilo Rust):**
- **`priv`** (default): Solo visible en el módulo actual
- **`pub`**: Público, visible desde cualquier lugar
- **`pub(crate)`**: Visible en todo el crate/módulo
- **`pub(super)`**: Visible en módulo padre
- **`pub(self)`**: Visible solo en módulo actual (igual que priv)

**Checklist de Implementación:**
- [ ] Privado por defecto (más seguro que Python)
- [ ] Keyword `pub` para público
- [ ] `pub(crate)` para crate-level visibility
- [ ] `pub(super)` para módulo padre
- [ ] Verificación de acceso en compilación
- [ ] Errores claros de acceso denegado
- [ ] Tests para encapsulación

**Mejoras sobre Python:**
- ✅ Privado por defecto (Python todo es público)
- ✅ Múltiples niveles de visibilidad
- ✅ Verificación en compilación (Python en runtime)
- ✅ Más seguro y organizado

**Nota:** Requiere O1 (Clases básicas)

---

### O5.1 - Module System para Organización ⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🟡 Medio | **Esfuerzo:** 15-25 horas

**Descripción:** Sistema de módulos estilo Rust para organizar código

**Sintaxis Propuesta:**
```adead
// archivo: banco/ad.ad
pub mod Banco {
    pub struct Cuenta {
        saldo: int64
    }
    
    pub fn crear_cuenta() -> Cuenta {
        return Cuenta { saldo: 0 }
    }
}

// archivo: main.ad
use banco::Banco

let cuenta = Banco::crear_cuenta()
```

**Checklist de Implementación:**
- [ ] Sistema de módulos
- [ ] `mod` para definir módulos
- [ ] `use` para importar
- [ ] Paths de módulos (`::`)
- [ ] Visibility con módulos
- [ ] Tests para módulos

**Nota:** Relacionado con ideas2.md L3 (Módulos)

---

### O9 - Propiedades Readonly e Inmutables ⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟢 Baja | **Impacto:** 🟢 Bajo | **Esfuerzo:** 8-12 horas

**Descripción:** Campos que no pueden modificarse después de inicialización

**Sintaxis Propuesta:**
```adead
class Configuracion {
    readonly version: string = "1.0"
    const PI: float64 = 3.14159
    
    init(version: string) {
        self.version = version  // OK en constructor
    }
    
    fn cambiar_version() {
        self.version = "2.0"  // Error: readonly
    }
}
```

**Checklist de Implementación:**
- [ ] Keyword `readonly` para campos
- [ ] Keyword `const` para constantes de clase
- [ ] Verificación en tiempo de compilación
- [ ] Inicialización en constructor
- [ ] Tests para readonly/const

---

## 👨‍👩‍👧 Herencia

### O10 - Herencia Simple ⭐⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🔴 Alta | **Impacto:** 🔴 Alto | **Esfuerzo:** 40-60 horas

**Descripción:** Una clase puede heredar de otra clase

**Sintaxis Propuesta:**
```adead
class Animal {
    nombre: string
    edad: int64
    
    fn hacer_sonido() {
        print "Algún sonido"
    }
}

class Perro extends Animal {
    raza: string
    
    fn hacer_sonido() {  // Override
        print "Guau guau"
    }
    
    fn correr() {
        print self.nombre + " está corriendo"
    }
}

let perro = Perro()
perro.nombre = "Max"
perro.raza = "Labrador"
perro.hacer_sonido()  // "Guau guau"
```

**Checklist de Implementación:**
- [ ] Keyword `extends` para herencia
- [ ] Herencia de campos
- [ ] Herencia de métodos
- [ ] Acceso a miembros heredados
- [ ] Override de métodos
- [ ] Keyword `super` para acceder a clase padre
- [ ] VTable para dispatch dinámico
- [ ] Tests para herencia

**Nota:** Requiere O1, O3, O4, O8

---

### O11 - Herencia Múltiple ⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🔴 Alta | **Impacto:** 🟡 Medio | **Esfuerzo:** 50-70 horas

**Descripción:** Una clase puede heredar de múltiples clases

**Sintaxis Propuesta:**
```adead
class Nadador {
    fn nadar() {
        print "Nadando..."
    }
}

class Volador {
    fn volar() {
        print "Volando..."
    }
}

class Pato extends Nadador, Volador {
    fn hacer_todo() {
        self.nadar()
        self.volar()
    }
}
```

**Checklist de Implementación:**
- [ ] Sintaxis para múltiples padres
- [ ] Resolución de conflictos de nombres
- [ ] Diamond problem (diamante)
- [ ] Orden de herencia
- [ ] VTable múltiple o estrategia similar
- [ ] Tests complejos para herencia múltiple

**Nota:** Requiere O10 primero. Complejidad alta.

---

### O12 - Constructor de Clase Padre (super) ⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🟡 Medio | **Esfuerzo:** 15-20 horas

**Descripción:** Llamar al constructor de la clase padre

**Sintaxis Propuesta:**
```adead
class Animal {
    nombre: string
    
    init(nombre: string) {
        self.nombre = nombre
    }
}

class Perro extends Animal {
    raza: string
    
    init(nombre: string, raza: string) {
        super(nombre)  // Llama constructor padre
        self.raza = raza
    }
}
```

**Checklist de Implementación:**
- [ ] Keyword `super` en constructores
- [ ] Llamadas a constructores padre
- [ ] Orden de inicialización
- [ ] Constructores por defecto en herencia
- [ ] Tests para super()

**Nota:** Requiere O10

---

### O13 - Clases Abstractas ⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🟡 Medio | **Esfuerzo:** 20-30 horas

**Descripción:** Clases que no pueden instanciarse, solo heredarse

**Sintaxis Propuesta:**
```adead
abstract class Forma {
    abstract fn calcular_area() -> float64
    abstract fn calcular_perimetro() -> float64
    
    fn imprimir_info() {
        print "Área: " + self.calcular_area()
    }
}

class Circulo extends Forma {
    radio: float64
    
    fn calcular_area() -> float64 {
        return 3.14159 * self.radio * self.radio
    }
    
    fn calcular_perimetro() -> float64 {
        return 2 * 3.14159 * self.radio
    }
}

// let forma = Forma()  // Error: clase abstracta
let circulo = Circulo()  // OK
```

**Checklist de Implementación:**
- [ ] Keyword `abstract` para clases
- [ ] Keyword `abstract` para métodos
- [ ] Prevenir instanciación de clases abstractas
- [ ] Forzar implementación de métodos abstractos
- [ ] Tests para clases abstractas

**Nota:** Requiere O10

---

## 🎭 Polimorfismo

### O14 - Métodos Virtuales y Override ⭐⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🔴 Alta | **Impacto:** 🔴 Alto | **Esfuerzo:** 30-40 horas

**Descripción:** Métodos que pueden ser sobrescritos en clases hijas

**Sintaxis Propuesta:**
```adead
class Animal {
    virtual fn hacer_sonido() {
        print "Sonido genérico"
    }
}

class Perro extends Animal {
    override fn hacer_sonido() {
        print "Guau guau"
    }
}

class Gato extends Animal {
    override fn hacer_sonido() {
        print "Miau miau"
    }
}

fn hacer_ruido(animal: Animal) {
    animal.hacer_sonido()  // Dispatch dinámico
}

let perro = Perro()
let gato = Gato()
hacer_ruido(perro)  // "Guau guau"
hacer_ruido(gato)   // "Miau miau"
```

**Checklist de Implementación:**
- [ ] Keyword `virtual` para métodos
- [ ] Keyword `override` para sobrescritura
- [ ] VTable (Virtual Method Table)
- [ ] Dispatch dinámico vs estático
- [ ] Verificación de override correcto
- [ ] Tests para polimorfismo

**Nota:** Requiere O10. Crítico para OOP verdadero.

---

### O15 - Interfaces/Traits ⭐⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🔴 Alta | **Impacto:** 🔴 Alto | **Esfuerzo:** 40-60 horas

**Descripción:** Contratos que las clases deben implementar

**Sintaxis Propuesta:**
```adead
interface Volador {
    fn volar()
    fn aterrizar()
}

interface Nadador {
    fn nadar()
}

class Pato implements Volador, Nadador {
    fn volar() {
        print "Volando como pato"
    }
    
    fn aterrizar() {
        print "Aterrizando"
    }
    
    fn nadar() {
        print "Nadando como pato"
    }
}
```

**Checklist de Implementación:**
- [ ] Keyword `interface` o `trait`
- [ ] Definir métodos en interfaces
- [ ] Keyword `implements`
- [ ] Múltiples interfaces por clase
- [ ] Verificación de implementación completa
- [ ] Interfaces como tipos
- [ ] Tests para interfaces

**Alternativa:** Podría ser similar a Rust traits o Go interfaces

---

### O16 - Type Casting y Type Checking ⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🟡 Medio | **Esfuerzo:** 20-30 horas

**Descripción:** Convertir entre tipos relacionados por herencia

**Sintaxis Propuesta:**
```adead
let animal: Animal = Perro()

// Type checking
if animal is Perro {
    let perro = animal as Perro
    perro.ladrar()
}

// Safe cast (retorna null si falla)
let perro = animal as? Perro
if perro != null {
    perro.ladrar()
}
```

**Checklist de Implementación:**
- [ ] Keyword `is` para type checking
- [ ] Keyword `as` para casting
- [ ] Keyword `as?` para safe cast
- [ ] Verificación en runtime
- [ ] Downcast y upcast
- [ ] Tests para casting

**Nota:** Requiere O10

---

## 🎨 Abstracción

### O17 - Clases y Métodos Finales ⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟢 Baja | **Impacto:** 🟢 Bajo | **Esfuerzo:** 8-12 horas

**Descripción:** Prevenir herencia o override

**Sintaxis Propuesta:**
```adead
final class String {  // No puede heredarse
    // ...
}

class Animal {
    final fn respirar() {  // No puede ser override
        print "Respirando"
    }
}
```

**Checklist de Implementación:**
- [ ] Keyword `final` para clases
- [ ] Keyword `final` para métodos
- [ ] Prevenir herencia de clases finales
- [ ] Prevenir override de métodos finales
- [ ] Tests para final

---

### O18 - Sealed Classes ⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🟢 Bajo | **Esfuerzo:** 10-15 horas

**Descripción:** Clases que solo pueden heredarse en el mismo archivo/módulo

**Sintaxis Propuesta:**
```adead
sealed class Resultado {
    // ...
}

class Exito extends Resultado { }  // OK: mismo archivo
class Error extends Resultado { }  // OK: mismo archivo

// En otro archivo:
// class Otro extends Resultado { }  // Error
```

**Checklist de Implementación:**
- [ ] Keyword `sealed`
- [ ] Verificación de herencia en mismo módulo
- [ ] Útil para pattern matching exhaustivo
- [ ] Tests para sealed

---

## ⚡ Características Avanzadas

### O19 - Operator Overloading ⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🔴 Alta | **Impacto:** 🟡 Medio | **Esfuerzo:** 30-40 horas

**Descripción:** Magic methods estilo Python pero más simple

**Sintaxis Propuesta (Mejora sobre Python):**
```adead
class Vector {
    x: int64
    y: int64
    
    // Magic methods (como Python pero sin doble guión bajo)
    fn __add__(other: Vector) -> Vector {
        return Vector(self.x + other.x, self.y + other.y)
    }
    
    fn __sub__(other: Vector) -> Vector {
        return Vector(self.x - other.x, self.y - other.y)
    }
    
    fn __mul__(escalar: int64) -> Vector {
        return Vector(self.x * escalar, self.y * escalar)
    }
    
    // Comparación
    fn __eq__(other: Vector) -> bool {
        return self.x == other.x && self.y == other.y
    }
    
    // String representation (como __str__ de Python)
    fn __str__() -> string {
        return "Vector(" + self.x + ", " + self.y + ")"
    }
    
    // Representación (como __repr__ de Python)
    fn __repr__() -> string {
        return "Vector(x: " + self.x + ", y: " + self.y + ")"
    }
    
    // Indexing (como __getitem__ de Python)
    fn __getitem__(index: int64) -> int64 {
        if index == 0 { return self.x }
        if index == 1 { return self.y }
        raise "Índice inválido"
    }
    
    fn __setitem__(index: int64, valor: int64) {
        if index == 0 { self.x = valor }
        else if index == 1 { self.y = valor }
        else { raise "Índice inválido" }
    }
}

let v1 = Vector(1, 2)
let v2 = Vector(3, 4)
let v3 = v1 + v2        // Usa __add__
let v4 = v1 * 2         // Usa __mul__
print v1                // Usa __str__: "Vector(1, 2)"
print v1[0]             // Usa __getitem__: 1
v1[0] = 10              // Usa __setitem__
```

**Magic Methods Disponibles:**
- Aritméticos: `__add__`, `__sub__`, `__mul__`, `__div__`, `__mod__`
- Comparación: `__eq__`, `__ne__`, `__lt__`, `__le__`, `__gt__`, `__ge__`
- Conversión: `__str__`, `__repr__`, `__int__`, `__float__`, `__bool__`
- Container: `__len__`, `__getitem__`, `__setitem__`, `__contains__`
- Callable: `__call__` (hacer objeto callable)

**Mejoras sobre Python:**
- ✅ Mismos nombres que Python (familiar)
- ✅ Sin necesidad de definir todos (solo los que necesitas)
- ✅ Type safety mejorado

**Checklist de Implementación:**
- [ ] Sintaxis `operator` para sobrecarga
- [ ] Operadores aritméticos (+, -, *, /)
- [ ] Operadores de comparación (==, !=, <, >)
- [ ] Operadores de asignación (=, +=, etc.)
- [ ] Precedencia de operadores
- [ ] Tests para operator overloading

---

### O20 - Propiedades Indexadas (Indexers) ⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🟢 Bajo | **Esfuerzo:** 15-20 horas

**Descripción:** Acceso tipo array a objetos

**Sintaxis Propuesta:**
```adead
class Lista {
    items: Array<int64>
    
    indexer [i: int64] -> int64 {
        get {
            return self.items[i]
        }
        set (valor: int64) {
            self.items[i] = valor
        }
    }
}

let lista = Lista()
lista[0] = 10
print lista[0]
```

**Checklist de Implementación:**
- [ ] Sintaxis `indexer`
- [ ] Getter y setter para índices
- [ ] Múltiples parámetros (matrices)
- [ ] Tests para indexers

---

### O21 - Métodos de Extensión ⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🟡 Medio | **Esfuerzo:** 15-20 horas

**Descripción:** Agregar métodos a clases existentes sin modificar su código

**Sintaxis Propuesta:**
```adead
class String {
    // definición base
}

extension String {
    fn invertir() -> String {
        // implementación
    }
    
    fn to_upper() -> String {
        // implementación
    }
}

let texto = "Hola"
print texto.invertir()  // "aloH"
```

**Checklist de Implementación:**
- [ ] Keyword `extension`
- [ ] Agregar métodos a clases existentes
- [ ] Sintaxis `self` en extensiones
- [ ] Resolución de métodos
- [ ] Tests para extensiones

---

### O22 - Mixins y Composition ⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🔴 Alta | **Impacto:** 🟡 Medio | **Esfuerzo:** 40-50 horas

**Descripción:** Reutilizar código mediante composición en lugar de herencia

**Sintaxis Propuesta:**
```adead
mixin Logeable {
    fn log(mensaje: string) {
        print "[LOG] " + mensaje
    }
}

class Servidor with Logeable {
    fn iniciar() {
        self.log("Servidor iniciado")
    }
}
```

**Checklist de Implementación:**
- [ ] Keyword `mixin`
- [ ] Keyword `with` para aplicar mixins
- [ ] Múltiples mixins
- [ ] Resolución de conflictos
- [ ] Tests para mixins

**Alternativa:** Podría implementarse con traits/interfaces

---

## 💾 Memory Management

### O23 - Garbage Collection o RAII ⭐⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🔴 Alta | **Impacto:** 🔴 Alto | **Esfuerzo:** 60-80 horas

**Descripción:** Gestión automática de memoria para objetos

**Opciones:**
1. **RAII (Resource Acquisition Is Initialization)**: Destructores automáticos
2. **Reference Counting**: Contar referencias
3. **Garbage Collector**: Recolección automática
4. **Ownership (como Rust)**: Sistema de propiedad

**Sintaxis Propuesta (RAII):**
```adead
class Recurso {
    init() {
        // adquirir recurso
    }
    
    destroy() {
        // liberar recurso automáticamente
    }
}

// Destructor se llama automáticamente al salir de scope
{
    let r = Recurso()
    // usar recurso
}  // destroy() llamado aquí automáticamente
```

**Checklist de Implementación:**
- [ ] Decidir estrategia (RAII recomendado para rendimiento)
- [ ] Implementar destructores automáticos
- [ ] Scope-based cleanup
- [ ] Manejo de ciclos (si reference counting)
- [ ] Tests para memory management

**Recomendación:** Empezar con RAII por simplicidad y rendimiento

---

### O24 - Smart Pointers ⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🔴 Alta | **Impacto:** 🟡 Medio | **Esfuerzo:** 30-40 horas

**Descripción:** Punteros que gestionan automáticamente la memoria

**Sintaxis Propuesta:**
```adead
class Persona {
    nombre: string
}

// Shared pointer (referencia compartida)
let p1 = shared Persona("Juan")
let p2 = p1  // Comparte la misma instancia

// Unique pointer (propiedad única)
let u = unique Persona("María")
// u se destruye automáticamente
```

**Checklist de Implementación:**
- [ ] `shared` pointer (reference counting)
- [ ] `unique` pointer (ownership único)
- [ ] `weak` pointer (referencia débil, opcional)
- [ ] Tests para smart pointers

**Nota:** Requiere O23

---

### O25 - Copy vs Move Semantics ⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🟡 Medio | **Esfuerzo:** 20-30 horas

**Descripción:** Diferencia entre copiar y mover objetos

**Sintaxis Propuesta:**
```adead
class Buffer {
    datos: Array<byte>
    
    // Constructor de move
    move(other: Buffer) {
        self.datos = other.datos
        other.datos = null  // Invalidar origen
    }
}

let b1 = Buffer(...)
let b2 = move b1  // Move, no copy
// b1 ya no es válido
```

**Checklist de Implementación:**
- [ ] Semántica de copia (copy semantics)
- [ ] Semántica de movimiento (move semantics)
- [ ] Keyword `move` opcional
- [ ] Optimización de moves
- [ ] Tests para copy vs move

---

## 🔧 Generics y Templates

### O26 - Generics/Templates Básicos ⭐⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🔴 Alta | **Impacto:** 🔴 Alto | **Esfuerzo:** 50-70 horas

**Descripción:** Clases y métodos genéricos (tipo parámetro)

**Sintaxis Propuesta:**
```adead
class Lista<T> {
    items: Array<T>
    
    fn agregar(item: T) {
        // ...
    }
    
    fn obtener(indice: int64) -> T {
        return self.items[indice]
    }
}

let lista_int = Lista<int64>()
lista_int.agregar(10)

let lista_str = Lista<string>()
lista_str.agregar("Hola")
```

**Checklist de Implementación:**
- [ ] Sintaxis para parámetros de tipo `<T>`
- [ ] Múltiples parámetros de tipo
- [ ] Constraints/bounds (opcional)
- [ ] Monomorphización (especialización)
- [ ] Métodos genéricos
- [ ] Tests para generics

---

### O27 - Constraints y Bounds ⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🔴 Alta | **Impacto:** 🟡 Medio | **Esfuerzo:** 30-40 horas

**Descripción:** Restricciones en tipos genéricos

**Sintaxis Propuesta:**
```adead
// T debe implementar Comparable
class SortedList<T: Comparable> {
    // T tiene método compare()
}

// T debe tener método clone()
fn duplicar<T: Cloneable>(item: T) -> T {
    return item.clone()
}
```

**Checklist de Implementación:**
- [ ] Sintaxis para bounds (`T: Trait`)
- [ ] Multiple bounds (`T: A + B`)
- [ ] Verificación en compilación
- [ ] Tests para constraints

**Nota:** Requiere O26 y O15 (Interfaces)

---

### O28 - Type Inference para Generics ⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🟢 Bajo | **Esfuerzo:** 15-20 horas

**Descripción:** Inferir tipos genéricos automáticamente (como Python pero tipado)

**Sintaxis Propuesta:**
```adead
// Inferencia automática (mejor que Python)
let lista = Lista()  // Infiere Lista<int64> del uso
lista.agregar(10)

// vs explícito
let lista: Lista<int64> = Lista()

// Inferencia desde constructor
let mapa = Dict()  // Infiere Dict<string, string> desde uso
mapa["clave"] = "valor"
```

**Checklist de Implementación:**
- [ ] Inferencia de tipos genéricos
- [ ] Inferencia desde argumentos
- [ ] Inferencia desde contexto
- [ ] Tests para type inference

**Nota:** Requiere O26

---

### O29 - Data Classes (Estilo Python) ⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🟡 Medio | **Esfuerzo:** 15-25 horas

**Descripción:** Clases simples para almacenar datos (como Python dataclass)

**Sintaxis Propuesta:**
```adead
// Data class automática (genera init, __eq__, __str__, etc.)
@dataclass
class Punto {
    x: int64
    y: int64
}

// Equivalente a escribir manualmente:
class Punto {
    x: int64
    y: int64
    
    init(x: int64, y: int64) {
        self.x = x
        self.y = y
    }
    
    fn __eq__(other: Punto) -> bool {
        return self.x == other.x && self.y == other.y
    }
    
    fn __str__() -> string {
        return "Punto(x: " + self.x + ", y: " + self.y + ")"
    }
}

// Uso
let p1 = Punto(10, 20)
let p2 = Punto(10, 20)
print p1 == p2  // true (generado automáticamente)
print p1        // "Punto(x: 10, y: 20)"
```

**Checklist de Implementación:**
- [ ] Decorador `@dataclass`
- [ ] Generar `init` automático
- [ ] Generar `__eq__` automático
- [ ] Generar `__str__` automático
- [ ] Opciones: `frozen`, `order`, etc.
- [ ] Tests para dataclasses

**Mejoras sobre Python:**
- ✅ Mismo concepto que Python `@dataclass`
- ✅ Type safety mejorado
- ✅ Mejor rendimiento (menos overhead)

---

### O30 - Slots para Optimización (Estilo Python) ⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🟢 Bajo | **Esfuerzo:** 10-15 horas

**Descripción:** Optimizar memoria limitando atributos (como Python __slots__)

**Sintaxis Propuesta:**
```adead
// Optimización de memoria (como Python __slots__)
@slots
class Persona {
    nombre: string
    edad: int64
    // Solo estos campos están permitidos
}

let p = Persona()
p.nombre = "Juan"  // OK
// p.otro = "valor"  // Error: atributo no en slots
```

**Beneficio:** Menos memoria, acceso más rápido

---

### O31 - Duck Typing Mejorado ⭐⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🟡 Medio | **Esfuerzo:** 20-30 horas

**Descripción:** Duck typing como Python pero con verificación de tipos opcional

**Sintaxis Propuesta:**
```adead
// Duck typing (como Python)
fn hacer_volar(objeto) {  // Sin tipo específico
    objeto.volar()  // Solo necesita método volar()
}

class Pajaro {
    fn volar() { print "Volando" }
}

class Avion {
    fn volar() { print "Despegando" }
}

hacer_volar(Pajaro())  // OK
hacer_volar(Avion())   // OK

// Structural typing (mejor que duck typing)
interface Volador {
    fn volar()
}

fn hacer_volar_mejor(objeto: Volador) {
    objeto.volar()  // Type-safe duck typing
}
```

**Mejoras sobre Python:**
- ✅ Duck typing opcional (como Python)
- ✅ Structural typing para type safety
- ✅ Mejor que Python: puedes elegir

---

### O32 - Method Resolution Order (MRO) Mejorado ⭐
- [ ] **Estado:** Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🟢 Bajo | **Esfuerzo:** 15-20 horas

**Descripción:** MRO como Python C3 pero más predecible

**Sintaxis Propuesta:**
```adead
class A {
    fn metodo() { print "A" }
}

class B extends A {
    fn metodo() { print "B" }
}

class C extends A {
    fn metodo() { print "C" }
}

class D extends B, C {
    // MRO: D -> B -> C -> A
}

let d = D()
d.metodo()  // "B" (según MRO)
```

**Mejora:** MRO más claro y predecible que Python

---

## 🗺️ Roadmap OOP

### Fase 1: Pre-requisitos Rust (Sprint 0 - 8-12 semanas)
**Objetivo:** Fundamentos estilo Rust necesarios para OOP seguro

```
📅 O0.1 - Sistema de Tipos Robusto (prerequisito) ⭐⭐⭐
📅 O0.2 - Ownership y Borrowing ⭐⭐⭐
📅 O0.3 - Inmutabilidad por Defecto ⭐⭐⭐
📅 O0.4 - Option/Result Types ⭐⭐⭐
```

### Fase 2: Fundamentos OOP con Rust (Sprint 1 - 8-12 semanas)
**Objetivo:** Clases/Structs con ownership y seguridad de Rust

```
📅 O1 - Structs/Clases Básicas (inmutables) ⭐⭐⭐
📅 O3 - Propiedades con Ownership ⭐⭐⭐
📅 O4 - Métodos con Borrowing (&self, &mut self) ⭐⭐⭐
📅 O2 - Constructores y RAII ⭐⭐⭐
📅 O2.1 - Drop Trait ⭐⭐⭐
📅 O5 - Encapsulación (pub/priv) ⭐⭐⭐
```

### Fase 3: Encapsulación Mejorada (Sprint 2 - 4-6 semanas)
**Objetivo:** Sistema de módulos y encapsulación estilo Rust

```
📅 O5.1 - Module System ⭐⭐
📅 O9 - Propiedades Readonly ⭐
📅 O6 - Propiedades con Getters/Setters ⭐⭐
```

### Fase 3: Herencia (Sprint 3 - 8-10 semanas)
**Objetivo:** Sistema de herencia completo

```
📅 O10 - Herencia Simple ⭐⭐⭐
📅 O12 - Constructor de Clase Padre ⭐⭐
📅 O13 - Clases Abstractas ⭐⭐
📅 O11 - Herencia Múltiple ⭐⭐ (opcional, complejo)
```

### Fase 4: Polimorfismo (Sprint 4 - 6-8 semanas)
**Objetivo:** Polimorfismo verdadero

```
📅 O14 - Métodos Virtuales y Override ⭐⭐⭐
📅 O15 - Interfaces/Traits ⭐⭐⭐
📅 O16 - Type Casting ⭐⭐
```

### Fase 5: Memory Management (Sprint 5 - 8-10 semanas)
**Objetivo:** Gestión eficiente de memoria

```
📅 O23 - Garbage Collection o RAII ⭐⭐⭐
📅 O24 - Smart Pointers ⭐⭐
📅 O25 - Copy vs Move Semantics ⭐⭐
```

### Fase 6: Características Avanzadas (Sprint 6 - 10+ semanas)
**Objetivo:** Funcionalidades avanzadas OOP

```
📅 O26 - Generics/Templates ⭐⭐⭐
📅 O19 - Operator Overloading ⭐⭐
📅 O21 - Métodos de Extensión ⭐
📅 O22 - Mixins ⭐⭐
```

---

## 📊 Matriz de Priorización OOP

| Idea | Complejidad | Impacto | Esfuerzo | Prioridad | Dependencias |
|------|-------------|---------|----------|-----------|--------------|
| O1 - Clases Básicas | 🔴 Alta | 🔴 Alto | 40-60h | ⭐⭐⭐ | L1, L2 |
| O2 - Constructores | 🟡 Media | 🔴 Alto | 20-30h | ⭐⭐⭐ | O1 |
| O3 - Propiedades | 🟡 Media | 🔴 Alto | 15-25h | ⭐⭐⭐ | O1, L1 |
| O4 - Métodos | 🟡 Media | 🔴 Alto | 20-30h | ⭐⭐⭐ | O1 |
| O8 - Encapsulación | 🟡 Media | 🔴 Alto | 20-30h | ⭐⭐⭐ | O1 |
| O10 - Herencia | 🔴 Alta | 🔴 Alto | 40-60h | ⭐⭐⭐ | O1, O4, O8 |
| O14 - Virtual Methods | 🔴 Alta | 🔴 Alto | 30-40h | ⭐⭐⭐ | O10 |
| O15 - Interfaces | 🔴 Alta | 🔴 Alto | 40-60h | ⭐⭐⭐ | O10 |
| O23 - Memory Mgmt | 🔴 Alta | 🔴 Alto | 60-80h | ⭐⭐⭐ | O1 |
| O26 - Generics | 🔴 Alta | 🔴 Alto | 50-70h | ⭐⭐⭐ | O1, O10 |

---

## 💭 Notas de Diseño

### Filosofía OOP para ADead: Python Syntax + Rust Safety

**Combinación única:** Sintaxis simple estilo Python + Seguridad y organización de Rust

**Principios Clave:**
1. **Sintaxis Python-like**: Simple, legible, sin boilerplate
2. **Seguridad Rust-like**: Ownership, borrowing, memory safety
3. **Lo mejor de ambos mundos**: Familiaridad de Python + Seguridad de Rust

### Arquitectura: Orden de Implementación (Rust-like)

**Nivel 0: Prerequisitos (Fundación Rust)**
```
O0.1 → O0.2 → O0.3 → O0.4
Tipos → Ownership → Inmutabilidad → Option/Result
```

**Nivel 1: Estructuras Básicas (OOP con Ownership)**
```
O1 → O3 → O4 → O2 → O2.1 → O5
Structs → Campos → Métodos → RAII → Drop → Encapsulación
```

**Nivel 2: OOP Completo**
```
Herencia → Polimorfismo → Interfaces → Generics
```

### Comparación: Python vs Rust vs ADead

| Característica | Python | Rust | ADead |
|----------------|--------|------|-------|
| **Sintaxis** | Simple | Verbosa | Simple (Python-like) |
| **Memory Safety** | GC | Ownership | Ownership (Rust-like) |
| **Tipos** | Dinámicos | Estáticos | Estáticos (opcionales) |
| **Inmutabilidad** | Mutable | Inmutable | Inmutable por defecto |
| **Null Safety** | None | Option<T> | Option<T> |
| **Error Handling** | Exceptions | Result<T,E> | Result<T,E> |
| **Performance** | Lento | Rápido | Rápido (ASM) |

### Filosofía OOP para ADead (Inspirado en Python pero Mejorado)

- **Simplicidad primero**: Empezar con OOP básico, agregar complejidad gradualmente
- **Rendimiento**: Priorizar eficiencia en memoria y velocidad de ejecución
- **Compatibilidad**: Mantener sintaxis simple estilo Python (familiar)
- **Flexibilidad**: Permitir múltiples paradigmas (OOP + funcional)
- **Type Safety**: Python-like pero con tipos opcionales
- **Mejor que Python**: Sin necesidad de `self` explícito en algunos casos, mejor manejo de memoria

### Características Python que Mejoramos

1. **Sintaxis más limpia**: Sin `__init__`, usar `init()` más simple
2. **Tipos opcionales**: Python sin tipos vs ADead con tipos opcionales
3. **RAII por defecto**: Python requiere `with`, ADead automático
4. **Magic methods**: Mismos nombres que Python (familiar) pero mejor tipado
5. **Properties**: Similar a Python pero más explícito
6. **Dataclasses**: Mismo concepto, mejor implementación
7. **Duck typing**: Opcional, con structural typing como alternativa type-safe

### Decisiones de Diseño Clave

1. **`self` vs `this`**: Usar `self` (más Python-like)
2. **Herencia múltiple**: ¿Implementar o usar interfaces/traits?
3. **Memory management**: RAII recomendado para rendimiento
4. **Generics**: Monomorphización (como Rust/C++) vs boxed (como Java)

### Prerequisitos del Compilador (Orden Rust-like)

Antes de implementar OOP completo, **DEBES** tener (en orden):

1. **Sistema de Tipos Robusto** (ideas2.md L1) - CRÍTICO
   - Tipos primitivos
   - Type inference básico
   - Verificación de tipos

2. **Ownership System** (O0.2) - CRÍTICO para seguridad
   - Borrow checker
   - Move semantics
   - Lifetime tracking

3. **Option/Result Types** (O0.4) - CRÍTICO para null safety
   - Sin null/nil
   - Manejo explícito de errores

4. **Arrays y Strings** (ideas2.md L2) - Necesario para estructuras

5. **Módulos** (ideas2.md L3) - Útil para organización
   - Sistema de módulos
   - Visibility con módulos

**Orden de Implementación Recomendado:**
```
1. Sistema de Tipos (ideas2.md L1)
2. Ownership y Borrowing (O0.2)
3. Option/Result (O0.4)
4. Arrays/Strings (ideas2.md L2)
5. Structs/Clases básicas (O1)
6. Módulos (ideas2.md L3)
7. Resto de OOP
```

**⚠️ IMPORTANTE:** No intentar OOP sin ownership system - resultará en código inseguro

---

## 🔍 Resumen Ejecutivo del Estado Actual

### ✅ Lo que Funciona (MVP)

1. **Parser básico funcional**
   - ✅ Parsea `let`, `if`, `while`, `fn`, `print`, `return`
   - ✅ Expresiones: números, strings, operaciones binarias, llamadas a funciones
   - ✅ Archivo: `crates/adead-parser/src/lib.rs`

2. **Generación de código ASM**
   - ✅ Genera NASM funcional
   - ✅ Soporta Windows y Linux
   - ✅ Archivo: `crates/adead-backend/src/lib.rs`

3. **CLI modular**
   - ✅ Comandos: `compile`, `assemble`, `link`, `run`
   - ✅ Archivo: `crates/adead-cli/src/main.rs`

4. **Ejemplos funcionando**
   - ✅ `hello.ad`, `conditional.ad`, `factorial.ad`, `loop.ad`

### ❌ Lo que Falta para Fase 1.1

1. **Sistema de tipos (O0.1)** - Crítico
   - ❌ Solo 3 tipos: `Int64`, `String`, `Void`
   - ❌ Falta: todos los primitivos, arrays, tuples, Option, Result

2. **Ownership (O0.2)** - Crítico
   - ❌ No existe borrowing
   - ❌ No hay `&` o `&mut`
   - ❌ No hay borrow checker

3. **Inmutabilidad (O0.3)** - Fácil de implementar
   - ❌ No hay `mut` keyword
   - ❌ Todo es mutable implícitamente

4. **Option/Result (O0.4)** - Importante
   - ❌ No existe
   - ❌ No hay `match` expressions

### 🎯 Recomendación: Comenzar con O0.3 (Inmutabilidad)

**Por qué empezar aquí:**
1. ✅ Cambio más simple (solo 2 archivos)
2. ✅ No depende de otros sistemas
3. ✅ Impacto alto (seguridad básica)
4. ✅ Puede hacerse en 1-2 días de trabajo

**Siguiente paso:** Ver `Ejemplos-Reales/documentacion/COMENZAR-Fase-1.1.md`

---

## 📚 Ejemplos Completos

### Ejemplo 1: Sistema de Formas Geométricas

```adead
abstract class Forma {
    abstract fn area() -> float64
    abstract fn perimetro() -> float64
    
    fn imprimir() {
        print "Área: " + self.area()
        print "Perímetro: " + self.perimetro()
    }
}

class Rectangulo extends Forma {
    ancho: float64
    alto: float64
    
    init(ancho: float64, alto: float64) {
        self.ancho = ancho
        self.alto = alto
    }
    
    fn area() -> float64 {
        return self.ancho * self.alto
    }
    
    fn perimetro() -> float64 {
        return 2 * (self.ancho + self.alto)
    }
}

class Circulo extends Forma {
    radio: float64
    
    init(radio: float64) {
        self.radio = radio
    }
    
    fn area() -> float64 {
        return 3.14159 * self.radio * self.radio
    }
    
    fn perimetro() -> float64 {
        return 2 * 3.14159 * self.radio
    }
}

fn main() {
    let formas = [Rectangulo(10, 5), Circulo(7)]
    
    for forma in formas {
        forma.imprimir()  // Polimorfismo
    }
}
```

### Ejemplo 2: Sistema de Inventario con Generics

### Ejemplo 3: Estilo Python con Mejoras

```adead
// Data class automática (como Python)
@dataclass
class Coordenada {
    x: float64
    y: float64
    z: float64
}

// Magic methods estilo Python
class Vector {
    x: float64
    y: float64
    
    init(x: float64, y: float64) {
        self.x = x
        self.y = y
    }
    
    // Magic methods familiares
    fn __add__(other: Vector) -> Vector {
        return Vector(self.x + other.x, self.y + other.y)
    }
    
    fn __str__() -> string {
        return "Vector(" + self.x + ", " + self.y + ")"
    }
    
    fn __repr__() -> string {
        return "Vector(x: " + self.x + ", y: " + self.y + ")"
    }
}

// Property estilo Python pero mejor
class Rectangulo {
    private _ancho: float64
    private _alto: float64
    
    property area: float64 {
        get {
            return self._ancho * self._alto
        }
    }
    
    property ancho: float64 {
        get { return self._ancho }
        set(valor: float64) {
            if valor > 0 {
                self._ancho = valor
            }
        }
    }
}

// Uso simple y Python-like
let v1 = Vector(1.0, 2.0)
let v2 = Vector(3.0, 4.0)
let v3 = v1 + v2  // Magic method
print v3          // Usa __str__

let rect = Rectangulo()
rect.ancho = 10.0
print rect.area   // Property calculada
```

### Ejemplo 4: Duck Typing + Type Safety

```adead
class Inventario<T> {
    items: Array<T>
    max_capacidad: int64
    
    init(capacidad: int64) {
        self.items = []
        self.max_capacidad = capacidad
    }
    
    fn agregar(item: T) -> bool {
        if len(self.items) >= self.max_capacidad {
            return false
        }
        self.items.append(item)
        return true
    }
    
    fn obtener(indice: int64) -> T? {
        if indice >= 0 && indice < len(self.items) {
            return self.items[indice]
        }
        return null
    }
}

class Item {
    nombre: string
    precio: float64
}

fn main() {
    let inventario = Inventario<Item>(10)
    
    let item = Item()
    item.nombre = "Espada"
    item.precio = 100.0
    
    inventario.agregar(item)
}
```

---

**¡Sigue construyendo!** 🚀

*Última actualización: Diciembre 2025*

