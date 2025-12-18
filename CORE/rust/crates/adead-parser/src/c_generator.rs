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
        // Resetear contador de variables temporales
        self.variable_counter = 0;
        // Cabecera estándar de C
        self.output.push_str("#include <stdio.h>\n");
        self.output.push_str("#include <stdlib.h>\n");
        self.output.push_str("#include <stdint.h>\n");
        self.output.push_str("#include <stdbool.h>\n");
        self.output.push_str("#include <string.h>\n");
        self.output.push_str("\n");
        
        // Estructura Array dinámica (estilo Python list)
        self.output.push_str("// Estructura Array dinámica\n");
        self.output.push_str("typedef struct {\n");
        self.output.push_str("    int64_t* data;\n");
        self.output.push_str("    size_t length;\n");
        self.output.push_str("    size_t capacity;\n");
        self.output.push_str("} Array;\n\n");
        
        // Funciones helper para Array
        self.output.push_str("// Crear array vacío\n");
        self.output.push_str("Array array_new(void) {\n");
        self.output.push_str("    Array arr;\n");
        self.output.push_str("    arr.length = 0;\n");
        self.output.push_str("    arr.capacity = 4;\n");
        self.output.push_str("    arr.data = (int64_t*)malloc(arr.capacity * sizeof(int64_t));\n");
        self.output.push_str("    return arr;\n");
        self.output.push_str("}\n\n");
        
        self.output.push_str("// Crear array desde valores iniciales\n");
        self.output.push_str("Array array_from_values(size_t count, int64_t* values) {\n");
        self.output.push_str("    Array arr;\n");
        self.output.push_str("    arr.length = count;\n");
        self.output.push_str("    arr.capacity = count > 4 ? count * 2 : 4;\n");
        self.output.push_str("    arr.data = (int64_t*)malloc(arr.capacity * sizeof(int64_t));\n");
        self.output.push_str("    memcpy(arr.data, values, count * sizeof(int64_t));\n");
        self.output.push_str("    return arr;\n");
        self.output.push_str("}\n\n");
        
        self.output.push_str("// Agregar elemento al array\n");
        self.output.push_str("void array_append(Array* arr, int64_t value) {\n");
        self.output.push_str("    if (arr->length >= arr->capacity) {\n");
        self.output.push_str("        arr->capacity *= 2;\n");
        self.output.push_str("        arr->data = (int64_t*)realloc(arr->data, arr->capacity * sizeof(int64_t));\n");
        self.output.push_str("    }\n");
        self.output.push_str("    arr->data[arr->length++] = value;\n");
        self.output.push_str("}\n\n");
        
        self.output.push_str("// Obtener elemento por índice\n");
        self.output.push_str("int64_t array_get(Array* arr, size_t index) {\n");
        self.output.push_str("    if (index >= arr->length) {\n");
        self.output.push_str("        fprintf(stderr, \"Error: índice fuera de rango\\n\");\n");
        self.output.push_str("        exit(1);\n");
        self.output.push_str("    }\n");
        self.output.push_str("    return arr->data[index];\n");
        self.output.push_str("}\n\n");
        
        self.output.push_str("// Establecer elemento por índice\n");
        self.output.push_str("void array_set(Array* arr, size_t index, int64_t value) {\n");
        self.output.push_str("    if (index >= arr->length) {\n");
        self.output.push_str("        fprintf(stderr, \"Error: índice fuera de rango\\n\");\n");
        self.output.push_str("        exit(1);\n");
        self.output.push_str("    }\n");
        self.output.push_str("    arr->data[index] = value;\n");
        self.output.push_str("}\n\n");
        
        self.output.push_str("// Obtener longitud del array\n");
        self.output.push_str("size_t array_len(Array* arr) {\n");
        self.output.push_str("    return arr->length;\n");
        self.output.push_str("}\n\n");
        
        self.output.push_str("// Eliminar y retornar último elemento (pop)\n");
        self.output.push_str("int64_t array_pop(Array* arr) {\n");
        self.output.push_str("    if (arr->length == 0) {\n");
        self.output.push_str("        fprintf(stderr, \"Error: pop de array vacío\\n\");\n");
        self.output.push_str("        exit(1);\n");
        self.output.push_str("    }\n");
        self.output.push_str("    return arr->data[--arr->length];\n");
        self.output.push_str("}\n\n");
        
        self.output.push_str("// Eliminar y retornar elemento en índice específico\n");
        self.output.push_str("int64_t array_pop_at(Array* arr, size_t index) {\n");
        self.output.push_str("    if (index >= arr->length) {\n");
        self.output.push_str("        fprintf(stderr, \"Error: índice fuera de rango\\n\");\n");
        self.output.push_str("        exit(1);\n");
        self.output.push_str("    }\n");
        self.output.push_str("    int64_t value = arr->data[index];\n");
        self.output.push_str("    // Mover elementos hacia la izquierda\n");
        self.output.push_str("    for (size_t i = index; i < arr->length - 1; i++) {\n");
        self.output.push_str("        arr->data[i] = arr->data[i + 1];\n");
        self.output.push_str("    }\n");
        self.output.push_str("    arr->length--;\n");
        self.output.push_str("    return value;\n");
        self.output.push_str("}\n\n");
        
        self.output.push_str("// Insertar elemento en posición específica\n");
        self.output.push_str("void array_insert(Array* arr, size_t index, int64_t value) {\n");
        self.output.push_str("    if (index > arr->length) {\n");
        self.output.push_str("        fprintf(stderr, \"Error: índice fuera de rango\\n\");\n");
        self.output.push_str("        exit(1);\n");
        self.output.push_str("    }\n");
        self.output.push_str("    // Redimensionar si es necesario\n");
        self.output.push_str("    if (arr->length >= arr->capacity) {\n");
        self.output.push_str("        arr->capacity *= 2;\n");
        self.output.push_str("        arr->data = (int64_t*)realloc(arr->data, arr->capacity * sizeof(int64_t));\n");
        self.output.push_str("    }\n");
        self.output.push_str("    // Mover elementos hacia la derecha\n");
        self.output.push_str("    for (size_t i = arr->length; i > index; i--) {\n");
        self.output.push_str("        arr->data[i] = arr->data[i - 1];\n");
        self.output.push_str("    }\n");
        self.output.push_str("    arr->data[index] = value;\n");
        self.output.push_str("    arr->length++;\n");
        self.output.push_str("}\n\n");
        
        self.output.push_str("// Eliminar primera ocurrencia de valor\n");
        self.output.push_str("void array_remove(Array* arr, int64_t value) {\n");
        self.output.push_str("    for (size_t i = 0; i < arr->length; i++) {\n");
        self.output.push_str("        if (arr->data[i] == value) {\n");
        self.output.push_str("            // Mover elementos hacia la izquierda\n");
        self.output.push_str("            for (size_t j = i; j < arr->length - 1; j++) {\n");
        self.output.push_str("                arr->data[j] = arr->data[j + 1];\n");
        self.output.push_str("            }\n");
        self.output.push_str("            arr->length--;\n");
        self.output.push_str("            return;\n");
        self.output.push_str("        }\n");
        self.output.push_str("    }\n");
        self.output.push_str("    fprintf(stderr, \"Error: valor no encontrado en array\\n\");\n");
        self.output.push_str("    exit(1);\n");
        self.output.push_str("}\n\n");
        
        self.output.push_str("// Encontrar índice de valor\n");
        self.output.push_str("size_t array_index(Array* arr, int64_t value) {\n");
        self.output.push_str("    for (size_t i = 0; i < arr->length; i++) {\n");
        self.output.push_str("        if (arr->data[i] == value) {\n");
        self.output.push_str("            return i;\n");
        self.output.push_str("        }\n");
        self.output.push_str("    }\n");
        self.output.push_str("    fprintf(stderr, \"Error: valor no encontrado en array\\n\");\n");
        self.output.push_str("    exit(1);\n");
        self.output.push_str("}\n\n");
        
        self.output.push_str("// Contar ocurrencias de valor\n");
        self.output.push_str("size_t array_count(Array* arr, int64_t value) {\n");
        self.output.push_str("    size_t count = 0;\n");
        self.output.push_str("    for (size_t i = 0; i < arr->length; i++) {\n");
        self.output.push_str("        if (arr->data[i] == value) {\n");
        self.output.push_str("            count++;\n");
        self.output.push_str("        }\n");
        self.output.push_str("    }\n");
        self.output.push_str("    return count;\n");
        self.output.push_str("}\n\n");
        
        self.output.push_str("// Ordenar array (bubble sort simple)\n");
        self.output.push_str("void array_sort(Array* arr) {\n");
        self.output.push_str("    for (size_t i = 0; i < arr->length; i++) {\n");
        self.output.push_str("        for (size_t j = 0; j < arr->length - i - 1; j++) {\n");
        self.output.push_str("            if (arr->data[j] > arr->data[j + 1]) {\n");
        self.output.push_str("                int64_t temp = arr->data[j];\n");
        self.output.push_str("                arr->data[j] = arr->data[j + 1];\n");
        self.output.push_str("                arr->data[j + 1] = temp;\n");
        self.output.push_str("            }\n");
        self.output.push_str("        }\n");
        self.output.push_str("    }\n");
        self.output.push_str("}\n\n");
        
        self.output.push_str("// Invertir orden del array\n");
        self.output.push_str("void array_reverse(Array* arr) {\n");
        self.output.push_str("    for (size_t i = 0; i < arr->length / 2; i++) {\n");
        self.output.push_str("        int64_t temp = arr->data[i];\n");
        self.output.push_str("        arr->data[i] = arr->data[arr->length - 1 - i];\n");
        self.output.push_str("        arr->data[arr->length - 1 - i] = temp;\n");
        self.output.push_str("    }\n");
        self.output.push_str("}\n\n");

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
                // Determinar tipo basado en el valor
                match value {
                    Expr::ArrayLiteral(elements) => {
                        if elements.is_empty() {
                            self.output.push_str(&format!("Array {} = array_new();\n", name));
                        } else {
                            // Generar valores
                            let values_code: Vec<String> = elements.iter()
                                .map(|e| self.generate_expr(e))
                                .collect();
                            
                            // Crear variable temporal para los valores
                            let temp_var = format!("_init_arr_{}", self.variable_counter);
                            self.variable_counter += 1;
                            
                            // Declarar array temporal estático
                            self.output.push_str(&format!("int64_t {}[] = {{ {} }};\n", temp_var, values_code.join(", ")));
                            self.indent();
                            // Inicializar Array usando la variable temporal
                            self.output.push_str(&format!("Array {} = array_from_values({}, {});\n", 
                                name, elements.len(), temp_var));
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
                    _ => {
                        let value_code = self.generate_expr(value);
                        self.output.push_str(&format!("int64_t {} = {};\n", name, value_code));
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
                // Detectamos esto cuando name == "_array_set" (marcador especial del parser)
                if name == "_array_set" {
                    if let Expr::BinaryOp { left, right, .. } = value.as_ref() {
                        if let Expr::Index { array, index } = left.as_ref() {
                            let array_code = self.generate_expr(array);
                            let index_code = self.generate_expr(index);
                            let value_code = self.generate_expr(right);
                            self.output.push_str(&format!("array_set(&{}, (size_t)({}), {});\n", 
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
                
                // Casos especiales para operadores que necesitan funciones
                if matches!(op, BinOp::Pow) {
                    return format!("pow({}, {})", left_code, right_code);
                }
                if matches!(op, BinOp::FloorDiv) {
                    return format!("({} / {})", left_code, right_code);
                }
                
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
                    BinOp::Pow | BinOp::FloorDiv => unreachable!(),
                };
                format!("({} {} {})", left_code, op_str, right_code)
            }
            Expr::Call { name, args, .. } => {
                // Manejar funciones especiales como len()
                if name == "len" && args.len() == 1 {
                    // len(arr) -> array_len(&arr)
                    let arg_code = self.generate_expr(&args[0]);
                    return format!("array_len(&{})", arg_code);
                }
                
                let args_code = args.iter()
                    .map(|arg| self.generate_expr(arg))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", name, args_code)
            }
            Expr::MethodCall { object, method, args } => {
                let arr_code = self.generate_expr(object);
                
                // Manejar métodos de arrays
                match method.as_str() {
                    "append" if args.len() == 1 => {
                        // arr.append(x) -> array_append(&arr, x)
                        let val_code = self.generate_expr(&args[0]);
                        return format!("array_append(&{}, {})", arr_code, val_code);
                    }
                    "pop" if args.is_empty() => {
                        // arr.pop() -> array_pop(&arr)
                        return format!("array_pop(&{})", arr_code);
                    }
                    "pop" if args.len() == 1 => {
                        // arr.pop(i) -> array_pop_at(&arr, i)
                        let index_code = self.generate_expr(&args[0]);
                        return format!("array_pop_at(&{}, (size_t)({}))", arr_code, index_code);
                    }
                    "insert" if args.len() == 2 => {
                        // arr.insert(i, x) -> array_insert(&arr, i, x)
                        let index_code = self.generate_expr(&args[0]);
                        let val_code = self.generate_expr(&args[1]);
                        return format!("array_insert(&{}, (size_t)({}), {})", arr_code, index_code, val_code);
                    }
                    "remove" if args.len() == 1 => {
                        // arr.remove(x) -> array_remove(&arr, x)
                        let val_code = self.generate_expr(&args[0]);
                        return format!("array_remove(&{}, {})", arr_code, val_code);
                    }
                    "index" if args.len() == 1 => {
                        // arr.index(x) -> array_index(&arr, x)
                        let val_code = self.generate_expr(&args[0]);
                        return format!("array_index(&{}, {})", arr_code, val_code);
                    }
                    "count" if args.len() == 1 => {
                        // arr.count(x) -> array_count(&arr, x)
                        let val_code = self.generate_expr(&args[0]);
                        return format!("array_count(&{}, {})", arr_code, val_code);
                    }
                    "sort" if args.is_empty() => {
                        // arr.sort() -> array_sort(&arr)
                        return format!("array_sort(&{})", arr_code);
                    }
                    "reverse" if args.is_empty() => {
                        // arr.reverse() -> array_reverse(&arr)
                        return format!("array_reverse(&{})", arr_code);
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
            Expr::ArrayLiteral(_) => {
                // ArrayLiteral no debería usarse directamente en expresiones
                // Solo se usa en Stmt::Let, que lo maneja de forma especial
                // Si llegamos aquí, es un error, pero generamos código de respaldo
                "array_new()".to_string()
            }
            Expr::Index { array, index } => {
                let array_code = self.generate_expr(array);
                let index_code = self.generate_expr(index);
                // Convertir índice a size_t (cast explícito)
                format!("array_get(&{}, (size_t)({}))", array_code, index_code)
            }
            Expr::Assign { name, value } => {
                // Si es asignación a índice de array: arr[0] = value
                // Necesitamos detectar esto de manera especial
                let value_code = self.generate_expr(value);
                format!("{} = {}", name, value_code)
            }
            Expr::Not(inner) => {
                // Negación lógica: !expr
                let inner_code = self.generate_expr(inner);
                format!("(!({}))", inner_code)
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

