//! Generador de código C++ desde AST de ADead
//! Este módulo convierte el AST de ADead a código C++ equivalente
//! que luego puede ser compilado con GCC++/Clang++ a ASM optimizado
//!
//! Ventajas sobre C Generator:
//! - std::vector para arrays (sin código helper manual)
//! - RAII automático (sin gestión manual de memoria)
//! - constexpr para optimizaciones compile-time
//! - std::string para strings
//! - Código más limpio y expresivo
//!
//! C++20 Features (cuando disponible):
//! - Concepts (para mejor type checking)
//! - Ranges (para operaciones más expresivas)
//! - Modules (para mejor organización)
//! - Coroutines (para async/await futuro)
//! - consteval (para evaluación compile-time más estricta)
//! - constinit (para inicialización compile-time)
//! - std::format (para mejor formateo de strings)

use crate::{Program, Stmt, Expr, BinOp};

/// Generador de código C++
pub struct CppGenerator {
    output: String,
    indent_level: usize,
    variable_counter: u32,
}

impl CppGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            variable_counter: 0,
        }
    }

    /// Generar código C++ completo desde un programa ADead
    pub fn generate(&mut self, program: &Program) -> String {
        // Resetear contador de variables temporales
        self.variable_counter = 0;
        
        // Cabecera estándar de C++
        // El compilador detectará automáticamente si usar C++17 o C++20
        self.output.push_str("#include <iostream>\n");
        self.output.push_str("#include <vector>\n");
        self.output.push_str("#include <string>\n");
        self.output.push_str("#include <algorithm>\n");  // Para std::find, std::count, std::sort, std::reverse, std::remove
        self.output.push_str("#include <cstdint>\n");
        
        // C++20 features (se incluyen condicionalmente si el compilador soporta C++20)
        // El compilador ignorará estas líneas si no soporta C++20
        self.output.push_str("#if __cplusplus >= 202002L\n");
        self.output.push_str("#include <ranges>\n");  // C++20 ranges para operaciones más expresivas
        self.output.push_str("#include <concepts>\n");  // C++20 concepts para mejor type checking
        self.output.push_str("#include <format>\n");  // C++20 std::format para mejor formateo
        self.output.push_str("#endif\n");
        
        self.output.push_str("\n");
        
        // Usar namespace std para simplificar código generado
        self.output.push_str("using namespace std;\n");
        
        // C++20: Usar std::ranges si está disponible
        self.output.push_str("#if __cplusplus >= 202002L\n");
        self.output.push_str("using namespace std::ranges;\n");
        self.output.push_str("#endif\n");
        
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
        self.output.push_str("int main() {\n");
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
                        // C++20: usar std::format si está disponible, sino cout
                        self.output.push_str("#if __cplusplus >= 202002L\n");
                        self.indent();
                        self.output.push_str(&format!("cout << std::format(\"{}\\n\", \"{}\");\n", 
                            "{:s}", s.replace('"', "\\\"")));
                        self.indent();
                        self.output.push_str("#else\n");
                        self.indent();
                        self.output.push_str(&format!("cout << \"{}\" << endl;\n", s.replace('"', "\\\"")));
                        self.indent();
                        self.output.push_str("#endif\n");
                    }
                    Expr::Number(n) => {
                        // C++20: usar std::format si está disponible, sino cout
                        self.output.push_str("#if __cplusplus >= 202002L\n");
                        self.indent();
                        self.output.push_str(&format!("cout << std::format(\"{}\\n\", {}LL);\n", "{:d}", n));
                        self.indent();
                        self.output.push_str("#else\n");
                        self.indent();
                        self.output.push_str(&format!("cout << {}LL << endl;\n", n));
                        self.indent();
                        self.output.push_str("#endif\n");
                    }
                    Expr::Float(f) => {
                        // C++20: usar std::format si está disponible, sino cout
                        self.output.push_str("#if __cplusplus >= 202002L\n");
                        self.indent();
                        self.output.push_str(&format!("cout << std::format(\"{}\\n\", {});\n", "{:f}", f));
                        self.indent();
                        self.output.push_str("#else\n");
                        self.indent();
                        self.output.push_str(&format!("cout << {} << endl;\n", f));
                        self.indent();
                        self.output.push_str("#endif\n");
                    }
                    Expr::Bool(b) => {
                        // C++20: usar std::format si está disponible, sino cout
                        self.output.push_str("#if __cplusplus >= 202002L\n");
                        self.indent();
                        self.output.push_str(&format!("cout << std::format(\"{}\\n\", {});\n", 
                            "{:s}", if *b { "true" } else { "false" }));
                        self.indent();
                        self.output.push_str("#else\n");
                        self.indent();
                        self.output.push_str(&format!("cout << ({}) << endl;\n", if *b { "true" } else { "false" }));
                        self.indent();
                        self.output.push_str("#endif\n");
                    }
                    Expr::Ident(name) => {
                        // C++20: usar std::format si está disponible, sino cout
                        self.output.push_str("#if __cplusplus >= 202002L\n");
                        self.indent();
                        self.output.push_str(&format!("cout << std::format(\"{}\\n\", {});\n", "{:d}", name));
                        self.indent();
                        self.output.push_str("#else\n");
                        self.indent();
                        self.output.push_str(&format!("cout << {} << endl;\n", name));
                        self.indent();
                        self.output.push_str("#endif\n");
                    }
                    _ => {
                        let expr_code = self.generate_expr(expr);
                        // C++20: usar std::format si está disponible, sino cout
                        self.output.push_str("#if __cplusplus >= 202002L\n");
                        self.indent();
                        self.output.push_str(&format!("cout << std::format(\"{}\\n\", {});\n", "{:d}", expr_code));
                        self.indent();
                        self.output.push_str("#else\n");
                        self.indent();
                        self.output.push_str(&format!("cout << {} << endl;\n", expr_code));
                        self.indent();
                        self.output.push_str("#endif\n");
                    }
                }
            }
            Stmt::Let { name, value, .. } => {
                self.indent();
                // Determinar tipo basado en el valor
                match value {
                    Expr::ArrayLiteral(elements) => {
                        if elements.is_empty() {
                            // Array vacío: std::vector<int64_t> arr;
                            self.output.push_str(&format!("vector<int64_t> {};\n", name));
                        } else {
                            // Array con valores iniciales: std::vector<int64_t> arr = {1, 2, 3};
                            let values_code: Vec<String> = elements.iter()
                                .map(|e| self.generate_expr(e))
                                .collect();
                            
                            self.output.push_str(&format!("vector<int64_t> {} = {{ {} }};\n", 
                                name, values_code.join(", ")));
                        }
                    }
                    Expr::Float(_) => {
                        let value_code = self.generate_expr(value);
                        self.output.push_str(&format!("double {} = {};\n", name, value_code));
                    }
                    Expr::Bool(_) => {
                        let value_code = self.generate_expr(value);
                        self.output.push_str(&format!("bool {} = {};\n", name, value_code));
                    }
                    Expr::String(_) => {
                        let value_code = self.generate_expr(value);
                        self.output.push_str(&format!("string {} = {};\n", name, value_code));
                    }
                    _ => {
                        let value_code = self.generate_expr(value);
                        // C++20: usar consteval para constantes simples (más estricto que constexpr)
                        // C++17: usar constexpr para constantes simples
                        if self.is_constant_expr(value) {
                            self.output.push_str("#if __cplusplus >= 202002L\n");
                            self.indent();
                            self.output.push_str(&format!("consteval int64_t {} = {};\n", name, value_code));
                            self.indent();
                            self.output.push_str("#else\n");
                            self.indent();
                            self.output.push_str(&format!("constexpr int64_t {} = {};\n", name, value_code));
                            self.indent();
                            self.output.push_str("#endif\n");
                        } else {
                            self.output.push_str(&format!("int64_t {} = {};\n", name, value_code));
                        }
                    }
                }
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
                // Verificar si es asignación a índice de array: arr[0] = value
                if name == "_array_set" {
                    if let Expr::BinaryOp { left, right, .. } = value.as_ref() {
                        if let Expr::Index { array, index } = left.as_ref() {
                            let array_code = self.generate_expr(array);
                            let index_code = self.generate_expr(index);
                            let value_code = self.generate_expr(right);
                            // C++: arr[index] = value (directo, sin función helper)
                            self.output.push_str(&format!("{}[{}] = {};\n", 
                                array_code, index_code, value_code));
                            return;
                        }
                    }
                }
                // Asignación normal: variable = value
                let value_code = self.generate_expr(value);
                self.output.push_str(&format!("{} = {};\n", name, value_code));
            }
            Stmt::Fn { name, params, body, .. } => {
                // Generar función C++
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
                // Manejar MethodCall con indentación especial para C++20 features
                if let Expr::MethodCall { object, method, args } = expr {
                    let arr_code = self.generate_expr(object);
                    
                    // Para sort y reverse, usar C++20 ranges si está disponible
                    if method == "sort" && args.is_empty() {
                        self.indent();
                        self.output.push_str("#if __cplusplus >= 202002L\n");
                        self.indent();
                        self.output.push_str(&format!("std::ranges::sort({});\n", arr_code));
                        self.indent();
                        self.output.push_str("#else\n");
                        self.indent();
                        self.output.push_str(&format!("std::sort({}.begin(), {}.end());\n", arr_code, arr_code));
                        self.indent();
                        self.output.push_str("#endif\n");
                        return;
                    }
                    if method == "reverse" && args.is_empty() {
                        self.indent();
                        self.output.push_str("#if __cplusplus >= 202002L\n");
                        self.indent();
                        self.output.push_str(&format!("std::ranges::reverse({});\n", arr_code));
                        self.indent();
                        self.output.push_str("#else\n");
                        self.indent();
                        self.output.push_str(&format!("std::reverse({}.begin(), {}.end());\n", arr_code, arr_code));
                        self.indent();
                        self.output.push_str("#endif\n");
                        return;
                    }
                }
                
                self.indent();
                let expr_code = self.generate_expr(expr);
                // Si es MethodCall (como arr.append()), generar como statement con punto y coma
                if matches!(expr, Expr::MethodCall { .. }) {
                    self.output.push_str(&format!("{};\n", expr_code));
                }
                // Otras expresiones se evalúan pero no se usan (pueden tener side effects)
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
                // Manejar funciones especiales como len()
                if name == "len" && args.len() == 1 {
                    // len(arr) -> arr.size() (C++ STL)
                    let arg_code = self.generate_expr(&args[0]);
                    return format!("{}.size()", arg_code);
                }
                
                let args_code = args.iter()
                    .map(|arg| self.generate_expr(arg))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", name, args_code)
            }
            Expr::MethodCall { object, method, args } => {
                let arr_code = self.generate_expr(object);
                
                // Manejar métodos de arrays usando std::vector
                match method.as_str() {
                    "append" if args.len() == 1 => {
                        // arr.append(x) -> arr.push_back(x) (C++ STL)
                        let val_code = self.generate_expr(&args[0]);
                        return format!("{}.push_back({})", arr_code, val_code);
                    }
                    "pop" if args.is_empty() => {
                        // arr.pop() -> arr.pop_back() (C++ STL)
                        return format!("{}.pop_back()", arr_code);
                    }
                    "pop" if args.len() == 1 => {
                        // arr.pop(i) -> arr.erase(arr.begin() + i) (C++ STL)
                        let index_code = self.generate_expr(&args[0]);
                        return format!("{}.erase({}.begin() + {})", arr_code, arr_code, index_code);
                    }
                    "insert" if args.len() == 2 => {
                        // arr.insert(i, x) -> arr.insert(arr.begin() + i, x) (C++ STL)
                        let index_code = self.generate_expr(&args[0]);
                        let val_code = self.generate_expr(&args[1]);
                        return format!("{}.insert({}.begin() + {}, {})", arr_code, arr_code, index_code, val_code);
                    }
                    "remove" if args.len() == 1 => {
                        // arr.remove(x) -> arr.erase(std::remove(arr.begin(), arr.end(), x), arr.end()) (C++ STL)
                        let val_code = self.generate_expr(&args[0]);
                        return format!("{}.erase(std::remove({}.begin(), {}.end(), {}), {}.end())", 
                            arr_code, arr_code, arr_code, val_code, arr_code);
                    }
                    "index" if args.len() == 1 => {
                        // arr.index(x) -> std::find(arr.begin(), arr.end(), x) - arr.begin() (C++ STL)
                        let val_code = self.generate_expr(&args[0]);
                        return format!("std::find({}.begin(), {}.end(), {}) - {}.begin()", 
                            arr_code, arr_code, val_code, arr_code);
                    }
                    "count" if args.len() == 1 => {
                        // arr.count(x) -> std::count(arr.begin(), arr.end(), x) (C++ STL)
                        let val_code = self.generate_expr(&args[0]);
                        return format!("std::count({}.begin(), {}.end(), {})", 
                            arr_code, arr_code, val_code);
                    }
                    "sort" if args.is_empty() => {
                        // Para uso como expresión (no statement), usar std::sort siempre
                        // El statement handling ya maneja C++20 ranges
                        format!("std::sort({}.begin(), {}.end())", arr_code, arr_code)
                    }
                    "reverse" if args.is_empty() => {
                        // Para uso como expresión (no statement), usar std::reverse siempre
                        // El statement handling ya maneja C++20 ranges
                        format!("std::reverse({}.begin(), {}.end())", arr_code, arr_code)
                    }
                    _ => {
                        // Otros métodos (futuro)
                        let args_code = args.iter()
                            .map(|arg| self.generate_expr(arg))
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!("{}.{}({})", arr_code, method, args_code)
                    }
                }
            }
            Expr::ArrayLiteral(elements) => {
                // ArrayLiteral: {1, 2, 3} para inicialización de vector
                let values_code: Vec<String> = elements.iter()
                    .map(|e| self.generate_expr(e))
                    .collect();
                format!("{{ {} }}", values_code.join(", "))
            }
            Expr::Index { array, index } => {
                let array_code = self.generate_expr(array);
                let index_code = self.generate_expr(index);
                // C++: arr[index] (directo, sin función helper)
                format!("{}[{}]", array_code, index_code)
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

    /// Verificar si una expresión es constante (para usar constexpr)
    fn is_constant_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Number(_) | Expr::Float(_) | Expr::Bool(_) => true,
            Expr::BinaryOp { left, right, .. } => {
                self.is_constant_expr(left) && self.is_constant_expr(right)
            }
            _ => false,
        }
    }

    fn indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }
}

impl Default for CppGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Función principal para generar código C++ desde un programa ADead
pub fn generate_cpp_code(program: &Program) -> String {
    let mut generator = CppGenerator::new();
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
        let cpp_code = generate_cpp_code(&program);
        assert!(cpp_code.contains("cout"));
        assert!(cpp_code.contains("Hello, World!"));
    }

    #[test]
    fn test_generate_array() {
        let program = Program {
            statements: vec![
                Stmt::Let {
                    mutable: false,
                    name: "arr".to_string(),
                    value: Expr::ArrayLiteral(vec![
                        Expr::Number(1),
                        Expr::Number(2),
                        Expr::Number(3),
                    ]),
                },
            ],
        };
        let cpp_code = generate_cpp_code(&program);
        assert!(cpp_code.contains("vector<int64_t>"));
        assert!(cpp_code.contains("arr"));
    }
}

