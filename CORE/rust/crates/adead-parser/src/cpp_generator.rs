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

use crate::{Program, Stmt, Expr, BinOp, StructField, StructMethod};
use std::collections::HashMap;

/// Información de una clase/struct para generación de código
#[derive(Debug, Clone)]
struct ClassInfo {
    name: String,
    fields: Vec<String>,
    has_constructor: bool,
    constructor_params: Vec<String>,
    methods: Vec<(String, Vec<String>, Vec<Stmt>)>,  // (nombre, params, body)
}

/// Generador de código C++
pub struct CppGenerator {
    output: String,
    indent_level: usize,
    variable_counter: u32,
    classes: HashMap<String, ClassInfo>,  // Clases definidas
}

impl CppGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            variable_counter: 0,
            classes: HashMap::new(),
        }
    }

    /// Generar código C++ completo desde un programa ADead
    pub fn generate(&mut self, program: &Program) -> String {
        // Resetear contador de variables temporales
        self.variable_counter = 0;
        
        // Cabecera estándar de C (usando printf para ASM más limpio)
        // Esto genera código ASM más simple y compatible con NASM
        self.output.push_str("#include <stdio.h>\n");
        self.output.push_str("#include <stdlib.h>\n");
        self.output.push_str("#include <stdint.h>\n");
        self.output.push_str("#include <string.h>\n");
        self.output.push_str("\n");

        // Separar structs/classes, funciones y código principal
        let mut structs = Vec::new();
        let mut functions = Vec::new();
        let mut main_statements = Vec::new();

        for stmt in &program.statements {
            match stmt {
                Stmt::Struct { .. } => {
                    structs.push(stmt);
                }
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

        // Generar structs/clases primero (definiciones de tipo)
        for stmt in &structs {
            self.generate_stmt(stmt);
        }

        // Generar funciones (fuera de main)
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
                        // Usar printf para ASM limpio
                        self.output.push_str(&format!("printf(\"{}\\n\");\n", s.replace('"', "\\\"")));
                    }
                    Expr::Number(n) => {
                        self.output.push_str(&format!("printf(\"%lld\\n\", (long long){});\n", n));
                    }
                    Expr::Float(f) => {
                        self.output.push_str(&format!("printf(\"%f\\n\", {});\n", f));
                    }
                    Expr::Bool(b) => {
                        self.output.push_str(&format!("printf(\"%s\\n\", \"{}\");\n", if *b { "true" } else { "false" }));
                    }
                    Expr::Ident(name) => {
                        // Detectar si es string o número
                        let is_string = self.is_string_expr(expr);
                        if is_string {
                            self.output.push_str(&format!("printf(\"%s\\n\", {});\n", name));
                        } else {
                            self.output.push_str(&format!("printf(\"%lld\\n\", (long long){});\n", name));
                        }
                    }
                    _ => {
                        let expr_code = self.generate_expr(expr);
                        // Detectar si es string o número
                        let is_string = self.is_string_expr(expr);
                        if is_string {
                            self.output.push_str(&format!("printf(\"%s\\n\", {});\n", expr_code));
                        } else {
                            self.output.push_str(&format!("printf(\"%lld\\n\", (long long){});\n", expr_code));
                        }
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
                    Expr::StructLiteral { name: struct_name, fields } => {
                        // Struct literal: let p = Punto { x: 10, y: 20 }
                        // En C++: Punto p{10, 20};
                        let args: Vec<String> = fields.iter()
                            .map(|(_, v)| self.generate_expr(v))
                            .collect();
                        self.output.push_str(&format!("{} {}{{{}}};\n", struct_name, name, args.join(", ")));
                    }
                    Expr::Call { module: Some(class_name), name: method_name, args } if method_name == "new" => {
                        // Class.new() call: let p = Punto.new(10, 20)
                        // En C++: Punto p{10, 20};
                        let args_code: Vec<String> = args.iter()
                            .map(|a| self.generate_expr(a))
                            .collect();
                        self.output.push_str(&format!("{} {}{{{}}};\n", class_name, name, args_code.join(", ")));
                    }
                    _ => {
                        // Verificar si es una expresión de string (concatenación, etc.)
                        if self.is_string_expr(value) {
                            let value_code = self.generate_expr(value);
                            self.output.push_str(&format!("string {} = {};\n", name, value_code));
                        } else {
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
                // Manejar MethodCall - por ahora simplificado para OOP básico
                if let Expr::MethodCall { object: _, method, args: _ } = expr {
                    // TODO: Implementar métodos de arrays cuando sea necesario
                    if method == "sort" || method == "reverse" {
                        self.indent();
                        self.output.push_str("// TODO: Método de array no implementado\n");
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
            Stmt::Struct { name, fields, init, destroy } => {
                // Generar clase C++ desde struct ADead
                self.generate_class(name, fields, init, destroy);
            }
            Stmt::For { var, start, end, body } => {
                // For loop: for (int64_t var = start; var < end; var++)
                self.indent();
                let start_code = self.generate_expr(start);
                let end_code = self.generate_expr(end);
                self.output.push_str(&format!("for (int64_t {} = {}; {} < {}; {}++) {{\n", 
                    var, start_code, var, end_code, var));
                self.indent_level += 1;
                for stmt in body {
                    self.generate_stmt(stmt);
                }
                self.indent_level -= 1;
                self.indent();
                self.output.push_str("}\n");
            }
            Stmt::Break => {
                self.indent();
                self.output.push_str("break;\n");
            }
            Stmt::Continue => {
                self.indent();
                self.output.push_str("continue;\n");
            }
            _ => {
                // Otros statements aún no implementados
                self.indent();
                self.output.push_str("// TODO: Statement no implementado\n");
            }
        }
    }

    /// Generar clase/struct C++ desde struct/class ADead
    fn generate_class(&mut self, name: &str, fields: &[StructField], init: &Option<StructMethod>, _destroy: &Option<StructMethod>) {
        if init.is_some() {
            // Si tiene constructor, generar como clase
            self.output.push_str(&format!("class {} {{\n", name));
            self.output.push_str("public:\n");
            self.indent_level = 1;
            
            // Generar campos
            for field in fields {
                self.indent();
                self.output.push_str(&format!("int64_t {};\n", field.name));
            }
            self.output.push_str("\n");
            
            // Generar constructor
            if let Some(init_method) = init {
                self.indent();
                let params: Vec<String> = init_method.params.iter()
                    .map(|p| format!("int64_t {}", p.name))
                    .collect();
                self.output.push_str(&format!("{}({}) {{\n", name, params.join(", ")));
                self.indent_level += 1;
                
                // Generar cuerpo del constructor
                for stmt in &init_method.body {
                    self.generate_constructor_stmt(stmt);
                }
                
                self.indent_level -= 1;
                self.indent();
                self.output.push_str("}\n");
            }
            
            self.indent_level = 0;
            self.output.push_str("};\n\n");
        } else {
            // Si no tiene constructor, generar como struct simple (aggregate)
            // Esto permite inicialización con llaves: Punto p{10, 20};
            self.output.push_str(&format!("struct {} {{\n", name));
            
            // Generar campos
            for field in fields {
                self.output.push_str(&format!("    int64_t {};\n", field.name));
            }
            
            self.output.push_str("};\n\n");
        }
        
        // Registrar la clase para uso posterior
        let class_info = ClassInfo {
            name: name.to_string(),
            fields: fields.iter().map(|f| f.name.clone()).collect(),
            has_constructor: init.is_some(),
            constructor_params: init.as_ref()
                .map(|m| m.params.iter().map(|p| p.name.clone()).collect())
                .unwrap_or_default(),
            methods: Vec::new(),
        };
        self.classes.insert(name.to_string(), class_info);
    }

    /// Generar statement dentro de un constructor (maneja self.campo = valor)
    fn generate_constructor_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(Expr::Assign { name, value }) => {
                self.indent();
                // Convertir self.campo a this->campo (o simplemente campo en C++)
                let cpp_name = if name.starts_with("self.") {
                    name.strip_prefix("self.").unwrap().to_string()
                } else {
                    name.clone()
                };
                let value_code = self.generate_expr(value);
                self.output.push_str(&format!("{} = {};\n", cpp_name, value_code));
            }
            _ => {
                self.generate_stmt(stmt);
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
                    BinOp::And => "&&",
                    BinOp::Or => "||",
                };
                format!("({} {} {})", left_code, op_str, right_code)
            }
            Expr::Call { module, name, args } => {
                // Manejar funciones especiales como len()
                if name == "len" && args.len() == 1 {
                    // len(arr) -> arr.size() (C++ STL)
                    let arg_code = self.generate_expr(&args[0]);
                    return format!("{}.size()", arg_code);
                }
                
                // Manejar Class.new() -> Class::new_()
                if name == "new" {
                    if let Some(class_name) = module {
                        let args_code = args.iter()
                            .map(|arg| self.generate_expr(arg))
                            .collect::<Vec<_>>()
                            .join(", ");
                        return format!("{}::new_({})", class_name, args_code);
                    }
                }
                
                // Llamada con módulo: modulo.funcion()
                if let Some(mod_name) = module {
                    let args_code = args.iter()
                        .map(|arg| self.generate_expr(arg))
                        .collect::<Vec<_>>()
                        .join(", ");
                    return format!("{}::{}({})", mod_name, name, args_code);
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
            Expr::Slice { object, start, end } => {
                // Slicing de strings: s[0:4] -> s.substr(0, 4)
                // En C++, substr(start, length) donde length = end - start
                let obj_code = self.generate_expr(object);
                let start_code = self.generate_expr(start);
                let end_code = self.generate_expr(end);
                // Generar: obj.substr(start, end - start)
                format!("{}.substr({}, {} - {})", obj_code, start_code, end_code, start_code)
            }
            Expr::Assign { name, value } => {
                let value_code = self.generate_expr(value);
                format!("{} = {}", name, value_code)
            }
            Expr::Not(inner) => {
                // Negación lógica: !expr
                let inner_code = self.generate_expr(inner);
                format!("(!({}))", inner_code)
            }
            Expr::StructLiteral { name, fields } => {
                // StructLiteral: Nombre { campo1: valor1, campo2: valor2 }
                // En C++: Nombre{valor1, valor2} (inicialización agregada)
                let args: Vec<String> = fields.iter()
                    .map(|(_, value)| self.generate_expr(value))
                    .collect();
                // Usar inicialización agregada C++: Nombre{val1, val2}
                format!("{}{{{}}}", name, args.join(", "))
            }
            Expr::FieldAccess { object, field } => {
                // FieldAccess: objeto.campo
                let obj_code = self.generate_expr(object);
                format!("{}.{}", obj_code, field)
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

    /// Verificar si una expresión es de tipo string
    /// Detecta strings literales, variables string, y concatenaciones de strings
    fn is_string_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::String(_) => true,
            Expr::Ident(name) => {
                // Verificar si es una variable string conocida
                // Heurística mejorada:
                // 1. Si el nombre es solo 's' (variable común para strings)
                // 2. Si el nombre empieza con 's' seguido de un número o letra (s1, s2, str1, etc.)
                // 3. Si contiene "str", "text", "msg" en el nombre
                // 4. Nombres comunes como "texto", "mensaje"
                let lower_name = name.to_lowercase();
                name == "s"  // Variable común 's' para strings (una sola letra)
                || (name.starts_with('s') && name.len() > 1 && name.chars().nth(1).map_or(false, |c| c.is_alphanumeric()))
                || lower_name.contains("str")
                || lower_name.contains("text")
                || lower_name.contains("msg")
                || lower_name == "texto"
                || lower_name == "mensaje"
            }
            Expr::BinaryOp { op: BinOp::Add, left, right } => {
                // Concatenación de strings: si al menos uno de los operandos es string,
                // asumimos que es concatenación de strings (C++ permite string + string)
                // Verificamos ambos lados para mayor precisión
                match (left.as_ref(), right.as_ref()) {
                    (Expr::String(_), _) | (_, Expr::String(_)) => true,
                    (Expr::Ident(_), Expr::Ident(_)) => {
                        // Si ambos son identificadores, verificamos si alguno parece string
                        self.is_string_expr(left) || self.is_string_expr(right)
                    }
                    _ => {
                        // Si alguno de los operandos es string, el resultado es string
                        self.is_string_expr(left) || self.is_string_expr(right)
                    }
                }
            }
            Expr::Call { name, .. } => {
                // Algunas funciones retornan strings (futuro)
                name == "to_string" || name.contains("string")
            }
            Expr::MethodCall { method, .. } => {
                // Métodos de strings retornan strings (futuro)
                matches!(method.as_str(), "upper" | "lower" | "slice" | "substring")
            }
            Expr::Slice { object, .. } => {
                // Slicing siempre retorna string: s[0:4] -> string
                self.is_string_expr(object)
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

