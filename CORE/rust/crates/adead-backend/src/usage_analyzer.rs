// Usage Analyzer para Dead Code Elimination
// Analiza el AST y detecta qué funciones del runtime se usan

use adead_parser::{Expr, Program, Stmt};
use crate::dependency_graph::DependencyGraph;

pub struct UsageAnalyzer;

impl UsageAnalyzer {
    /// Analizar el programa completo y marcar funciones usadas en el dependency graph
    pub fn analyze_program(program: &Program, deps: &mut DependencyGraph) {
        for stmt in &program.statements {
            Self::analyze_stmt(stmt, deps);
        }
    }
    
    /// Analizar una declaración
    fn analyze_stmt(stmt: &Stmt, deps: &mut DependencyGraph) {
        match stmt {
            Stmt::Let { value, .. } => {
                Self::analyze_expr(value, deps);
            }
            Stmt::Print(expr) => {
                Self::analyze_expr(expr, deps);
                // print siempre necesita int_to_str_runtime para números
                // (strings se detectan en analyze_expr)
                deps.mark_used("int_to_str_runtime");
            }
            Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    Self::analyze_expr(expr, deps);
                }
            }
            Stmt::If { condition, then_body, else_body } => {
                Self::analyze_expr(condition, deps);
                for stmt in then_body {
                    Self::analyze_stmt(stmt, deps);
                }
                if let Some(else_body) = else_body {
                    for stmt in else_body {
                        Self::analyze_stmt(stmt, deps);
                    }
                }
            }
            Stmt::While { condition, body } => {
                Self::analyze_expr(condition, deps);
                for stmt in body {
                    Self::analyze_stmt(stmt, deps);
                }
            }
            Stmt::For { start, end, body, .. } => {
                Self::analyze_expr(start, deps);
                Self::analyze_expr(end, deps);
                for stmt in body {
                    Self::analyze_stmt(stmt, deps);
                }
            }
            Stmt::Break | Stmt::Continue => {
                // No necesitan funciones del runtime
            }
            Stmt::Expr(expr) => {
                Self::analyze_expr(expr, deps);
            }
            Stmt::Fn { body, .. } => {
                for stmt in body {
                    Self::analyze_stmt(stmt, deps);
                }
            }
            _ => {
                // Otros tipos de statements (Struct, Import, etc.) no necesitan análisis especial
            }
        }
    }
    
    /// Analizar una expresión
    fn analyze_expr(expr: &Expr, deps: &mut DependencyGraph) {
        match expr {
            Expr::Number(_) | Expr::Float(_) | Expr::Bool(_) => {
                // Literales no necesitan funciones del runtime
            }
            Expr::String(_) => {
                // String literal: necesita string_from_literal
                deps.mark_used("string_from_literal");
            }
            Expr::Ident(_) => {
                // Variables simples no necesitan funciones del runtime
            }
            Expr::BinaryOp { left, right, .. } => {
                Self::analyze_expr(left, deps);
                Self::analyze_expr(right, deps);
            }
            Expr::Assign { value, .. } => {
                Self::analyze_expr(value, deps);
            }
            Expr::Call { name, args, .. } => {
                // Detectar llamadas a funciones del runtime
                match name.as_str() {
                    // Arrays
                    "array_new" => deps.mark_used("array_new"),
                    "array_from_values" => deps.mark_used("array_from_values"),
                    "array_get" => deps.mark_used("array_get"),
                    "array_set" => deps.mark_used("array_set"),
                    "array_len" | "len" => deps.mark_used("array_len"),
                    "array_append" | "append" => deps.mark_used("array_append"),
                    "array_pop" | "pop" => deps.mark_used("array_pop"),
                    "array_insert" | "insert" => deps.mark_used("array_insert"),
                    "array_remove" | "remove" => deps.mark_used("array_remove"),
                    "array_index" | "index" => deps.mark_used("array_index"),
                    "array_count" | "count" => deps.mark_used("array_count"),
                    "array_sort" | "sort" => deps.mark_used("array_sort"),
                    "array_reverse" | "reverse" => deps.mark_used("array_reverse"),
                    "array_free" => deps.mark_used("array_free"),
                    
                    // Strings
                    "string_new" => deps.mark_used("string_new"),
                    "string_from_literal" => deps.mark_used("string_from_literal"),
                    "string_len" => deps.mark_used("string_len"),
                    "string_concat" | "concat" => deps.mark_used("string_concat"),
                    "string_slice" | "slice" => deps.mark_used("string_slice"),
                    "string_upper" | "upper" => deps.mark_used("string_upper"),
                    "string_lower" | "lower" => deps.mark_used("string_lower"),
                    "string_free" => deps.mark_used("string_free"),
                    
                    // Otras funciones (stdlib, etc.)
                    _ => {
                        // Funciones definidas por el usuario o stdlib
                        // Se analizan recursivamente
                    }
                }
                
                // Analizar argumentos
                for arg in args {
                    Self::analyze_expr(arg, deps);
                }
            }
            Expr::Index { array, index } => {
                Self::analyze_expr(array, deps);
                Self::analyze_expr(index, deps);
                // Acceso a array: necesita array_get
                deps.mark_used("array_get");
            }
            Expr::ArrayLiteral(elements) => {
                // Array literal: necesita array_from_values
                deps.mark_used("array_from_values");
                for elem in elements {
                    Self::analyze_expr(elem, deps);
                }
            }
            Expr::Slice { object, .. } => {
                // Slice: necesita string_slice
                Self::analyze_expr(object, deps);
                deps.mark_used("string_slice");
            }
            Expr::Match { expr, arms } => {
                Self::analyze_expr(expr, deps);
                for arm in arms {
                    // MatchArm.body es Box<Expr>, necesitamos dereferenciar
                    Self::analyze_expr(&*arm.body, deps);
                }
            }
            _ => {
                // Otras expresiones (Borrow, Deref, Some, None, Ok, Err, StructLiteral, FieldAccess, MethodCall, etc.)
                // No necesitan funciones del runtime directamente
            }
        }
    }
}
