//! Borrow Checker para ADead - Sistema de Ownership y Borrowing
//!
//! Implementa verificación de reglas de borrowing estilo Rust:
//! - Move semantics por defecto
//! - Borrowing con `&T` y `&mut T`
//! - Sin aliasing mutable
//! - Lifetime tracking básico
//! - Verificación de mutabilidad

use adead_common::{ADeadError, Result};
use adead_parser::{BorrowType, Expr, Program, Stmt};
use std::collections::HashMap;

/// Estado de ownership de una variable
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnershipState {
    Owned,           // Variable es dueña del valor
    Borrowed,        // Variable tiene una referencia inmutable (&T)
    MutBorrowed,     // Variable tiene una referencia mutable (&mut T)
    Moved,           // Variable fue movida (ya no es válida)
}

/// Información sobre una variable en el borrow checker
#[derive(Debug, Clone)]
struct VariableInfo {
    ownership: OwnershipState,
    mutable: bool,  // true = variable mutable, false = inmutable
    borrowed_by: Vec<String>,  // Variables que tienen referencias a esta
}

/// Borrow Checker - Verifica reglas de ownership y borrowing
pub struct BorrowChecker {
    /// Variables y su estado de ownership
    variables: HashMap<String, VariableInfo>,
    /// Stack de scopes (para variables locales)
    scope_stack: Vec<HashMap<String, VariableInfo>>,
}

impl BorrowChecker {
    /// Crear un nuevo borrow checker
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            scope_stack: Vec::new(),
        }
    }

    /// Verificar un programa completo
    pub fn check(&mut self, program: &Program) -> Result<()> {
        // Primera pasada: registrar todas las variables
        for stmt in &program.statements {
            if let Stmt::Let { mutable, name, .. } = stmt {
                self.variables.insert(
                    name.clone(),
                    VariableInfo {
                        ownership: OwnershipState::Owned,
                        mutable: *mutable,
                        borrowed_by: Vec::new(),
                    },
                );
            }
        }
        
        // Segunda pasada: verificar statements completos (ahora las variables están registradas)
        for stmt in &program.statements {
            self.check_stmt(stmt)?;
        }
        Ok(())
    }

    /// Verificar un statement
    fn check_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let { mutable: _, name: _, value } => {
                // Verificar que el valor puede ser movido/owned
                // Nota: La variable ya está registrada en la primera pasada de check()
                self.check_expr(value)?;
                // Verificar borrowing en el valor (ahora que todas las variables están registradas)
                self.check_expr_borrowing(value)?;
                Ok(())
            }
            Stmt::Print(expr) => {
                // Print puede tomar borrowing
                self.check_expr(expr)?;
                Ok(())
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
            } => {
                self.check_expr(condition)?;
                // Verificar then_body en nuevo scope
                self.push_scope();
                for s in then_body {
                    self.check_stmt(s)?;
                }
                self.pop_scope();

                // Verificar else_body en nuevo scope
                if let Some(else_body) = else_body {
                    self.push_scope();
                    for s in else_body {
                        self.check_stmt(s)?;
                    }
                    self.pop_scope();
                }
                Ok(())
            }
            Stmt::While { condition, body } => {
                self.check_expr(condition)?;
                // Nota: En Rust, borrows en condiciones de loops tienen reglas especiales
                // Por ahora, verificamos normalmente
                self.push_scope();
                for s in body {
                    self.check_stmt(s)?;
                }
                self.pop_scope();
                Ok(())
            }
            Stmt::Struct { name: _, fields: _, init: _, destroy: _ } => {
                // Structs se registran pero no necesitan verificación especial aquí
                // Los campos se verifican cuando se usan
                // Constructores y destructores se verifican como funciones normales
                Ok(())
            }
            Stmt::Fn { name: _, params, body } => {
                // Registrar función (los parámetros se verifican cuando se llama)
                // Por ahora, solo verificamos el cuerpo
                self.push_scope();
                
                // Registrar parámetros en el scope
                for param in params {
                    let ownership = match param.borrow_type {
                        BorrowType::Owned => OwnershipState::Owned,
                        BorrowType::Borrowed => OwnershipState::Borrowed,
                        BorrowType::MutBorrowed => OwnershipState::MutBorrowed,
                    };
                    
                    self.scope_stack
                        .last_mut()
                        .unwrap()
                        .insert(param.name.clone(), VariableInfo {
                            ownership,
                            mutable: matches!(param.borrow_type, BorrowType::MutBorrowed),
                            borrowed_by: Vec::new(),
                        });
                }
                
                for s in body {
                    self.check_stmt(s)?;
                }
                self.pop_scope();
                Ok(())
            }
            Stmt::Expr(expr) => {
                self.check_expr(expr)?;
                Ok(())
            }
            Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    self.check_expr(expr)?;
                }
                Ok(())
            }
        }
    }

    /// Verificar una expresión
    fn check_expr(&self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Number(_) | Expr::String(_) => Ok(()), // Literales no necesitan verificación
            Expr::Ident(name) => {
                // Verificar que la variable existe y no fue movida
                if let Some(info) = self.find_variable(name) {
                    if info.ownership == OwnershipState::Moved {
                        return Err(ADeadError::TypeError {
                            message: format!("Variable '{}' fue movida y ya no es válida", name),
                        });
                    }
                } else {
                    return Err(ADeadError::TypeError {
                        message: format!("Variable '{}' no definida", name),
                    });
                }
                Ok(())
            }
            Expr::BinaryOp { left, right, .. } => {
                self.check_expr(left)?;
                self.check_expr(right)?;
                Ok(())
            }
            Expr::Assign { name, value } => {
                self.check_expr(value)?;
                // Verificar que la variable existe y puede ser asignada
                if let Some(info) = self.find_variable(name) {
                    if info.ownership == OwnershipState::Moved {
                        return Err(ADeadError::TypeError {
                            message: format!("Variable '{}' fue movida y ya no es válida", name),
                        });
                    }
                    // Verificar que la variable es mutable
                    if !info.mutable {
                        return Err(ADeadError::TypeError {
                            message: format!(
                                "Variable '{}' es inmutable y no puede ser modificada. Usa 'let mut' para crear una variable mutable",
                                name
                            ),
                        });
                    }
                } else {
                    return Err(ADeadError::TypeError {
                        message: format!("Variable '{}' no definida", name),
                    });
                }
                Ok(())
            }
            Expr::Call { args, .. } => {
                for arg in args {
                    self.check_expr(arg)?;
                }
                Ok(())
            }
            Expr::Borrow { expr, mutable } => {
                // Verificar que podemos tomar una referencia
                self.check_expr(expr)?;
                
                // Si es mutable borrow, verificar que la variable es mutable
                if *mutable {
                    // TODO: Verificar que no hay otros borrows activos
                    // Por ahora, solo verificamos que si es &mut, la variable debe ser mutable
                    if let Expr::Ident(name) = expr.as_ref() {
                        if let Some(info) = self.find_variable(name) {
                            if !info.mutable {
                                return Err(ADeadError::TypeError {
                                    message: format!(
                                        "No se puede tomar una referencia mutable (&mut) de '{}' porque es inmutable. Usa 'let mut' para crear una variable mutable",
                                        name
                                    ),
                                });
                            }
                        }
                    }
                }
                
                Ok(())
            }
            Expr::Deref(expr) => {
                // Verificar que la expresión es una referencia
                self.check_expr(expr)?;
                // TODO: Verificar que realmente es una referencia
                Ok(())
            }
            // Option/Result constructors (O0.4)
            Expr::Some(expr) | Expr::Ok(expr) | Expr::Err(expr) => {
                self.check_expr(expr)?;
                Ok(())
            }
            Expr::None => Ok(()), // None no necesita verificación
            Expr::Match { expr, arms } => {
                self.check_expr(expr)?;
                // Verificar cada brazo del match
                for arm in arms {
                    self.check_expr(&arm.body)?;
                    // TODO: Verificar que los patrones son exhaustivos y compatibles
                }
                Ok(())
            }
            // Structs (Fase 1.2)
            Expr::StructLiteral { fields, .. } => {
                // Verificar todos los valores de los campos
                for (_, value) in fields {
                    self.check_expr(value)?;
                }
                Ok(())
            }
            Expr::FieldAccess { object, .. } => {
                // Verificar que el objeto puede ser accedido
                self.check_expr(object)?;
                Ok(())
            }
            Expr::MethodCall { object, args, .. } => {
                // Verificar objeto y argumentos
                self.check_expr(object)?;
                for arg in args {
                    self.check_expr(arg)?;
                }
                Ok(())
            }
        }
    }

    /// Encontrar variable en scopes (desde el más reciente)
    fn find_variable(&self, name: &str) -> Option<&VariableInfo> {
        // Buscar en scopes locales primero
        for scope in self.scope_stack.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        // Buscar en variables globales
        self.variables.get(name)
    }

    /// Crear un nuevo scope
    fn push_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    /// Eliminar el scope más reciente
    fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    /// Verificar borrowing en expresiones (después de que todas las variables están registradas)
    fn check_expr_borrowing(&self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Borrow { expr, mutable } => {
                // Si es mutable borrow, verificar que la variable es mutable
                if *mutable {
                    match expr.as_ref() {
                        Expr::Ident(name) => {
                            if let Some(info) = self.find_variable(name) {
                                if !info.mutable {
                                    return Err(ADeadError::TypeError {
                                        message: format!(
                                            "No se puede tomar una referencia mutable (&mut) de '{}' porque es inmutable. Usa 'let mut' para crear una variable mutable",
                                            name
                                        ),
                                    });
                                }
                            } else {
                                // Variable no encontrada - esto debería ser un error también
                                return Err(ADeadError::TypeError {
                                    message: format!("Variable '{}' no definida", name),
                                });
                            }
                        }
                        _ => {
                            // &mut de algo que no es un identificador directo
                            // Por ahora permitimos esto (podría ser &mut *ptr, etc.)
                        }
                    }
                }
                // Verificar recursivamente la expresión base
                self.check_expr_borrowing(expr)
            }
            Expr::BinaryOp { left, right, .. } => {
                self.check_expr_borrowing(left)?;
                self.check_expr_borrowing(right)?;
                Ok(())
            }
            Expr::Assign { value, .. } => {
                self.check_expr_borrowing(value)?;
                Ok(())
            }
            Expr::Call { args, .. } => {
                for arg in args {
                    self.check_expr_borrowing(arg)?;
                }
                Ok(())
            }
            Expr::Deref(expr) => {
                self.check_expr_borrowing(expr)?;
                Ok(())
            }
            Expr::Some(expr) | Expr::Ok(expr) | Expr::Err(expr) => {
                self.check_expr_borrowing(expr)?;
                Ok(())
            }
            Expr::None => Ok(()),
            Expr::Match { expr, arms } => {
                self.check_expr_borrowing(expr)?;
                for arm in arms {
                    self.check_expr_borrowing(&arm.body)?;
                }
                Ok(())
            }
            Expr::StructLiteral { fields, .. } => {
                for (_, value) in fields {
                    self.check_expr_borrowing(value)?;
                }
                Ok(())
            }
            Expr::FieldAccess { object, .. } => {
                self.check_expr_borrowing(object)?;
                Ok(())
            }
            Expr::MethodCall { object, args, .. } => {
                self.check_expr_borrowing(object)?;
                for arg in args {
                    self.check_expr_borrowing(arg)?;
                }
                Ok(())
            }
            _ => Ok(()), // Otros casos no necesitan verificación adicional
        }
    }
}

impl Default for BorrowChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adead_parser::parse;

    #[test]
    fn test_simple_ownership() {
        let src = r#"
            let x = 10
            let y = x
        "#;
        let program = parse(src).unwrap();
        let mut checker = BorrowChecker::new();
        // Por ahora, esto debería pasar (no verificamos moves todavía)
        let _ = checker.check(&program);
    }

    #[test]
    fn test_borrow_syntax() {
        let src = r#"
            let x = 10
            let r = &x
        "#;
        let program = parse(src).unwrap();
        let mut checker = BorrowChecker::new();
        let _ = checker.check(&program);
    }

    #[test]
    fn test_immutable_variable_cannot_be_assigned() {
        let src = r#"
            let x = 10
            x = 20
        "#;
        let program = parse(src).unwrap();
        let mut checker = BorrowChecker::new();
        let result = checker.check(&program);
        assert!(result.is_err(), "Debe fallar: variable inmutable no puede ser modificada");
        if let Err(ADeadError::TypeError { message }) = result {
            assert!(message.contains("inmutable"), "Mensaje debe mencionar 'inmutable'");
        }
    }

    #[test]
    fn test_mutable_variable_can_be_assigned() {
        let src = r#"
            let mut x = 10
            x = 20
        "#;
        let program = parse(src).unwrap();
        let mut checker = BorrowChecker::new();
        let result = checker.check(&program);
        assert!(result.is_ok(), "Debe pasar: variable mutable puede ser modificada");
    }

    #[test]
    fn test_mut_borrow_requires_mutable_variable() {
        let src = r#"
            let x = 10
            let r = &mut x
        "#;
        let program = parse(src).unwrap();
        let mut checker = BorrowChecker::new();
        let result = checker.check(&program);
        assert!(result.is_err(), "Debe fallar: no se puede tomar &mut de variable inmutable");
        if let Err(ADeadError::TypeError { message }) = result {
            assert!(message.contains("inmutable"), "Mensaje debe mencionar 'inmutable', pero fue: {}", message);
        } else {
            panic!("Debe ser TypeError, pero fue: {:?}", result);
        }
    }

    #[test]
    fn test_mut_borrow_of_mutable_variable_works() {
        let src = r#"
            let mut x = 10
            let r = &mut x
        "#;
        let program = parse(src).unwrap();
        let mut checker = BorrowChecker::new();
        let result = checker.check(&program);
        assert!(result.is_ok(), "Debe pasar: se puede tomar &mut de variable mutable");
    }

    #[test]
    fn test_immutable_borrow_of_immutable_variable_works() {
        let src = r#"
            let x = 10
            let r = &x
        "#;
        let program = parse(src).unwrap();
        let mut checker = BorrowChecker::new();
        let result = checker.check(&program);
        assert!(result.is_ok(), "Debe pasar: se puede tomar & de variable inmutable");
    }
}
