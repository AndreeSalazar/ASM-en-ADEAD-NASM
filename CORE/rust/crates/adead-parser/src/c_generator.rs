//! Generador de código C desde AST de ADead
//! Este módulo convierte el AST de ADead a código C equivalente
//! que luego puede ser compilado con GCC/Clang a ASM optimizado

use crate::{Program, Stmt, Expr, BinOp};

/// Generador de código C
pub struct CGenerator {
    output: String,
    indent_level: usize,
    variable_counter: u32,
}

impl CGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            variable_counter: 0,
        }
    }

    /// Generar código C completo desde un programa ADead
    pub fn generate(&mut self, program: &Program) -> String {
        // Cabecera estándar de C
        self.output.push_str("#include <stdio.h>\n");
        self.output.push_str("#include <stdlib.h>\n");
        self.output.push_str("#include <stdint.h>\n");
        self.output.push_str("#include <stdbool.h>\n");
        self.output.push_str("\n");

        // Separar funciones y código principal
        let mut functions = Vec::new();
        let mut main_statements = Vec::new();

        for stmt in &program.statements {
            match stmt {
                Stmt::Fn { .. } => {
                    functions.push(stmt);
                }
                Stmt::While { .. } | Stmt::If { .. } | Stmt::Print(_) | Stmt::Let { .. } | Stmt::Expr(_) | Stmt::Return(_) => {
                    main_statements.push(stmt);
                }
                _ => {
                    // Otros statements también van a main
                    main_statements.push(stmt);
                }
            }
        }

        // Generar funciones primero (fuera de main)
        for stmt in &functions {
            self.generate_stmt(stmt);
        }

        // Generar función main con el código principal
        self.output.push_str("int main(void) {\n");
        self.indent_level = 1;
        
        // Generar código principal dentro de main
        for stmt in &main_statements {
            self.generate_stmt(stmt);
        }

        self.output.push_str("    return 0;\n");
        self.output.push_str("}\n");

        self.output.clone()
    }

    fn generate_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Print(expr) => {
                self.indent();
                match expr {
                    Expr::String(s) => {
                        self.output.push_str(&format!("printf(\"{}\\n\"); fflush(stdout);\n", s.replace('"', "\\\"")));
                    }
                    Expr::Number(n) => {
                        self.output.push_str(&format!("printf(\"%ld\\n\", (int64_t){}); fflush(stdout);\n", n));
                    }
                    Expr::Float(f) => {
                        self.output.push_str(&format!("printf(\"%.15g\\n\", {}); fflush(stdout);\n", f));
                    }
                    Expr::Bool(b) => {
                        self.output.push_str(&format!("printf(\"{}\\n\"); fflush(stdout);\n", if *b { "true" } else { "false" }));
                    }
                    Expr::Ident(name) => {
                        self.output.push_str(&format!("printf(\"%ld\\n\", {}); fflush(stdout);\n", name));
                    }
                    _ => {
                        let expr_code = self.generate_expr(expr);
                        self.output.push_str(&format!("printf(\"%ld\\n\", {}); fflush(stdout);\n", expr_code));
                    }
                }
            }
            Stmt::Let { name, value, .. } => {
                self.indent();
                let value_code = self.generate_expr(value);
                // Determinar tipo basado en el valor
                let type_str = match value {
                    Expr::Float(_) => "double",
                    Expr::Bool(_) => "bool",
                    _ => "int64_t",
                };
                self.output.push_str(&format!("{} {} = {};\n", type_str, name, value_code));
            }
            Stmt::If { condition, then_body, else_body } => {
                self.indent();
                let cond_code = self.generate_expr(condition);
                self.output.push_str(&format!("if ({}) {{\n", cond_code));
                self.indent_level += 1;
                for stmt in then_body {
                    self.generate_stmt(stmt);
                }
                self.indent_level -= 1;
                self.indent();
                self.output.push_str("}");
                if let Some(else_body) = else_body {
                    self.output.push_str(" else {\n");
                    self.indent_level += 1;
                    for stmt in else_body {
                        self.generate_stmt(stmt);
                    }
                    self.indent_level -= 1;
                    self.indent();
                    self.output.push_str("}\n");
                } else {
                    self.output.push_str("\n");
                }
            }
            Stmt::While { condition, body } => {
                self.indent();
                let cond_code = self.generate_expr(condition);
                self.output.push_str(&format!("while ({}) {{\n", cond_code));
                self.indent_level += 1;
                for stmt in body {
                    self.generate_stmt(stmt);
                }
                self.indent_level -= 1;
                self.indent();
                self.output.push_str("}\n");
            }
            Stmt::Expr(Expr::Assign { name, value }) => {
                self.indent();
                let value_code = self.generate_expr(value);
                self.output.push_str(&format!("{} = {};\n", name, value_code));
            }
            Stmt::Fn { name, params, body, .. } => {
                // Generar función C
                self.output.push_str(&format!("int64_t {}({}) {{\n", 
                    name,
                    params.iter()
                        .map(|p| format!("int64_t {}", p.name))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
                self.indent_level += 1;
                for stmt in body {
                    self.generate_stmt(stmt);
                }
                self.indent_level -= 1;
                self.output.push_str("}\n\n");
            }
            Stmt::Expr(expr) => {
                self.indent();
                let _expr_code = self.generate_expr(expr);
                // Las expresiones puras se evalúan pero no se usan (pueden tener side effects)
                // self.output.push_str(&format!("{};\n", expr_code));
            }
            Stmt::Return(Some(expr)) => {
                self.indent();
                let expr_code = self.generate_expr(expr);
                self.output.push_str(&format!("return {};\n", expr_code));
            }
            Stmt::Return(None) => {
                self.indent();
                self.output.push_str("return;\n");
            }
            _ => {
                // Otros statements aún no implementados
                self.indent();
                self.output.push_str("// TODO: Statement no implementado\n");
            }
        }
    }

    fn generate_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Number(n) => format!("{}LL", n),
            Expr::Float(f) => format!("{}", f),
            Expr::Bool(b) => format!("{}", if *b { "true" } else { "false" }),
            Expr::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            Expr::Ident(name) => name.clone(),
            Expr::BinaryOp { op, left, right } => {
                let left_code = self.generate_expr(left);
                let right_code = self.generate_expr(right);
                let op_str = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Mod => "%",
                    BinOp::Eq => "==",
                    BinOp::Ne => "!=",
                    BinOp::Lt => "<",
                    BinOp::Le => "<=",
                    BinOp::Gt => ">",
                    BinOp::Ge => ">=",
                };
                format!("({} {} {})", left_code, op_str, right_code)
            }
            Expr::Call { name, args, .. } => {
                let args_code = args.iter()
                    .map(|arg| self.generate_expr(arg))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", name, args_code)
            }
            Expr::Assign { name, value } => {
                let value_code = self.generate_expr(value);
                format!("{} = {}", name, value_code)
            }
            _ => {
                format!("/* TODO: Expresión no implementada */")
            }
        }
    }

    fn indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }
}

impl Default for CGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Función principal para generar código C desde un programa ADead
pub fn generate_c_code(program: &Program) -> String {
    let mut generator = CGenerator::new();
    generator.generate(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Program;

    #[test]
    fn test_generate_simple_print() {
        let program = Program {
            statements: vec![
                Stmt::Print(Expr::String("Hello, World!".to_string())),
            ],
        };
        let c_code = generate_c_code(&program);
        assert!(c_code.contains("printf"));
        assert!(c_code.contains("Hello, World!"));
    }

    #[test]
    fn test_generate_arithmetic() {
        let program = Program {
            statements: vec![
                Stmt::Let {
                    mutable: false,
                    name: "x".to_string(),
                    value: Expr::BinaryOp {
                        op: BinOp::Add,
                        left: Box::new(Expr::Number(10)),
                        right: Box::new(Expr::Number(20)),
                    },
                },
            ],
        };
        let c_code = generate_c_code(&program);
        assert!(c_code.contains("int64_t x"));
        assert!(c_code.contains("10LL"));
        assert!(c_code.contains("20LL"));
    }
}

