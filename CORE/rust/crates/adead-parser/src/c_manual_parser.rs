// Parser Manual Especializado para C → Rust → ASM
// Reemplaza Tree-sitter con parsing directo y simple
// Enfocado en while/if y estructuras básicas

use crate::{Stmt, Expr, BinOp, Program};

pub struct CManualParser {
    source: String,
    position: usize,
}

impl CManualParser {
    /// Crear nuevo parser manual
    pub fn new(source: String) -> Self {
        Self {
            source,
            position: 0,
        }
    }

    /// Parsear programa completo desde código ADead
    pub fn parse_program(source: &str) -> Result<Program, Box<dyn std::error::Error>> {
        let mut statements = Vec::new();
        let lines: Vec<&str> = source.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let mut line = lines[i].trim();
            // Eliminar comentarios (líneas que empiezan con ; o comentarios al final de línea)
            if let Some(comment_pos) = line.find(';') {
                line = line[..comment_pos].trim();
            }
            if line.is_empty() {
                i += 1;
                continue;
            }
            
            // Parsear diferentes tipos de statements
            if line.starts_with("print ") {
                let expr_text = line[6..].trim();
                if expr_text.starts_with('"') && expr_text.ends_with('"') {
                    // String literal
                    let string_val = expr_text[1..expr_text.len()-1].to_string();
                    statements.push(Stmt::Print(Expr::String(string_val)));
                } else {
                    // Número o identificador
                    if let Ok(expr) = Self::parse_expr_from_text(expr_text) {
                        statements.push(Stmt::Print(expr));
                    }
                }
                i += 1;
            } else if line.starts_with("let ") {
                // let variable = value
                let rest = line[4..].trim();
                if let Some(eq_pos) = rest.find(" = ") {
                    let var_name = rest[..eq_pos].trim().to_string();
                    let value_text = rest[eq_pos + 3..].trim();
                    if let Ok(value_expr) = Self::parse_expr_from_text(value_text) {
                        statements.push(Stmt::Let {
                            mutable: false,
                            name: var_name,
                            value: value_expr,
                        });
                    }
                }
                i += 1;
            } else if line.starts_with("while ") {
                // Parsear while completo (puede ser multi-línea)
                let mut while_lines = vec![line];
                let mut brace_count = 0;
                let mut while_complete = false;
                i += 1;
                
                // Contar llaves para encontrar el while completo
                for ch in line.chars() {
                    if ch == '{' { brace_count += 1; }
                    if ch == '}' { brace_count -= 1; }
                }
                
                while i < lines.len() && !while_complete {
                    let next_line = lines[i];
                    while_lines.push(next_line);
                    
                    for ch in next_line.chars() {
                        if ch == '{' { brace_count += 1; }
                        if ch == '}' {
                            brace_count -= 1;
                            if brace_count == 0 {
                                while_complete = true;
                                break;
                            }
                        }
                    }
                    i += 1;
                }
                
                let while_text = while_lines.join("\n");
                if let Ok((condition, body)) = Self::parse_while_from_text(&while_text) {
                    statements.push(Stmt::While { condition, body });
                }
            } else if line.contains(".") && line.contains("(") && line.contains(")") && !line.starts_with("print ") {
                // Método: arr.append(x) - parsear como statement
                match Self::parse_expr_from_text(line) {
                    Ok(expr) => {
                        if matches!(expr, Expr::MethodCall { .. }) {
                            statements.push(Stmt::Expr(expr));
                            i += 1;
                            continue;
                        }
                        // Si no es MethodCall, puede ser otra cosa, continuar
                    }
                    Err(_) => {
                        // Si falla el parsing, continuar con otras condiciones
                    }
                }
                i += 1;
            } else if line.contains(" = ") && !line.starts_with("let ") {
                // Asignación: variable = value o arr[0] = value
                if let Some(eq_pos) = line.find(" = ") {
                    let left_side = line[..eq_pos].trim();
                    let right_side = line[eq_pos + 3..].trim();
                    
                    // Verificar si es asignación a índice de array: arr[0] = value
                    if left_side.contains('[') && left_side.ends_with(']') {
                        if let Ok(index_expr) = Self::parse_expr_from_text(left_side) {
                            if let Expr::Index { array, index } = index_expr {
                                if let Ok(value_expr) = Self::parse_expr_from_text(right_side) {
                                    // Crear una asignación especial para array index
                                    // Usamos nombre especial que el generador C detectará
                                    statements.push(Stmt::Expr(Expr::Assign {
                                        name: "_array_set".to_string(), // Marcador especial
                                        value: Box::new(Expr::BinaryOp {
                                            op: BinOp::Eq, // Reutilizamos Eq como marcador
                                            left: Box::new(Expr::Index {
                                                array: array.clone(),
                                                index: index.clone(),
                                            }),
                                            right: Box::new(value_expr),
                                        }),
                                    }));
                                }
                            }
                        }
                    } else {
                        // Asignación normal: variable = value
                        let var_name = left_side.to_string();
                        if let Ok(value_expr) = Self::parse_expr_from_text(right_side) {
                            statements.push(Stmt::Expr(Expr::Assign {
                                name: var_name,
                                value: Box::new(value_expr),
                            }));
                        }
                    }
                }
                i += 1;
            } else {
                // Cualquier otra línea: intentar parsear como expresión
                // Puede ser un método call o cualquier expresión válida
                if let Ok(expr) = Self::parse_expr_from_text(line) {
                    if matches!(expr, Expr::MethodCall { .. }) {
                        // Es un método call, agregar como statement
                        statements.push(Stmt::Expr(expr));
                    }
                    // Si no es MethodCall, lo ignoramos (podría ser solo un identificador)
                }
                i += 1;
            }
        }
        
        Ok(Program { statements })
    }

    /// Parsear while loop completo desde texto
    pub fn parse_while_from_text(while_text: &str) -> Result<(Expr, Vec<Stmt>), Box<dyn std::error::Error>> {
        // Buscar "while" y extraer condición y body
        if !while_text.trim().starts_with("while") {
            return Err("Expected 'while'".into());
        }
        
        // Extraer condición: desde "while" hasta "{"
        let after_while = &while_text[5..].trim();
        let open_brace_pos = after_while.find('{')
            .ok_or("Expected '{' after while condition")?;
        let condition_text = &after_while[..open_brace_pos].trim();
        
        // Parsear condición
        let condition = Self::parse_expr_from_text(condition_text)?;
        
        // Extraer body: desde "{" hasta el "}" correspondiente
        let body_start = open_brace_pos + 1;
        let mut brace_count = 1;
        let mut body_end = None;
        
        for (i, ch) in after_while[body_start..].char_indices() {
            if ch == '{' {
                brace_count += 1;
            } else if ch == '}' {
                brace_count -= 1;
                if brace_count == 0 {
                    body_end = Some(body_start + i);
                    break;
                }
            }
        }
        
        let body_end = body_end.ok_or("Unclosed brace in while body")?;
        let body_text = &after_while[body_start..body_end].trim();
        
        // Parsear statements del body
        let body = Self::parse_statements_from_text(body_text)?;
        
        Ok((condition, body))
    }

    /// Parsear if statement completo desde texto
    pub fn parse_if_from_text(if_text: &str) -> Result<(Expr, Vec<Stmt>), Box<dyn std::error::Error>> {
        if !if_text.trim().starts_with("if") {
            return Err("Expected 'if'".into());
        }
        
        // Extraer condición
        let after_if = &if_text[2..].trim();
        let open_brace_pos = after_if.find('{')
            .ok_or("Expected '{' after if condition")?;
        let condition_text = &after_if[..open_brace_pos].trim();
        
        let condition = Self::parse_expr_from_text(condition_text)?;
        
        // Extraer body
        let body_start = open_brace_pos + 1;
        let mut brace_count = 1;
        let mut body_end = None;
        
        for (i, ch) in after_if[body_start..].char_indices() {
            if ch == '{' {
                brace_count += 1;
            } else if ch == '}' {
                brace_count -= 1;
                if brace_count == 0 {
                    body_end = Some(body_start + i);
                    break;
                }
            }
        }
        
        let body_end = body_end.ok_or("Unclosed brace in if body")?;
        let body_text = &after_if[body_start..body_end].trim();
        let body = Self::parse_statements_from_text(body_text)?;
        
        Ok((condition, body))
    }

    /// Parsear statements desde texto
    fn parse_statements_from_text(text: &str) -> Result<Vec<Stmt>, Box<dyn std::error::Error>> {
        let mut statements = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let mut line = lines[i].trim();
            // Eliminar comentarios (líneas que empiezan con ; o comentarios al final de línea)
            if let Some(comment_pos) = line.find(';') {
                line = line[..comment_pos].trim();
            }
            if line.is_empty() {
                i += 1;
                continue;
            }
            
            // Parsear if
            if line.starts_with("if ") {
                let mut if_lines = vec![line];
                let mut brace_count = 0;
                let mut if_complete = false;
                i += 1;
                
                for ch in line.chars() {
                    if ch == '{' { brace_count += 1; }
                    if ch == '}' { brace_count -= 1; }
                }
                
                while i < lines.len() && !if_complete {
                    let next_line = lines[i];
                    if_lines.push(next_line);
                    
                    for ch in next_line.chars() {
                        if ch == '{' { brace_count += 1; }
                        if ch == '}' {
                            brace_count -= 1;
                            if brace_count == 0 {
                                if_complete = true;
                                break;
                            }
                        }
                    }
                    i += 1;
                }
                
                let if_text = if_lines.join("\n");
                if let Ok((cond, body)) = Self::parse_if_from_text(&if_text) {
                    statements.push(Stmt::If {
                        condition: cond,
                        then_body: body,
                        else_body: None,
                    });
                }
            } else if line.starts_with("print ") {
                let expr_text = line[6..].trim();
                if let Ok(expr) = Self::parse_expr_from_text(expr_text) {
                    statements.push(Stmt::Print(expr));
                }
                i += 1;
            } else if line.contains(".") && line.contains("(") && line.contains(")") && !line.starts_with("print ") {
                // Método: arr.append(x) - parsear como statement
                match Self::parse_expr_from_text(line) {
                    Ok(expr) => {
                        if matches!(expr, Expr::MethodCall { .. }) {
                            statements.push(Stmt::Expr(expr));
                            i += 1;
                            continue;
                        }
                        // Si no es MethodCall, puede ser otra cosa, continuar
                    }
                    Err(_) => {
                        // Si falla el parsing, continuar con otras condiciones
                    }
                }
                i += 1;
            } else if line.contains(" = ") && !line.starts_with("let ") {
                // Asignación: variable = value o arr[0] = value
                if let Some(eq_pos) = line.find(" = ") {
                    let left_side = line[..eq_pos].trim();
                    let right_side = line[eq_pos + 3..].trim();
                    
                    // Verificar si es asignación a índice de array: arr[0] = value
                    if left_side.contains('[') && left_side.ends_with(']') {
                        if let Ok(index_expr) = Self::parse_expr_from_text(left_side) {
                            if let Expr::Index { array, index } = index_expr {
                                if let Ok(value_expr) = Self::parse_expr_from_text(right_side) {
                                    // Crear una asignación especial para array index
                                    // Usamos nombre especial que el generador C detectará
                                    statements.push(Stmt::Expr(Expr::Assign {
                                        name: "_array_set".to_string(), // Marcador especial
                                        value: Box::new(Expr::BinaryOp {
                                            op: BinOp::Eq, // Reutilizamos Eq como marcador
                                            left: Box::new(Expr::Index {
                                                array: array.clone(),
                                                index: index.clone(),
                                            }),
                                            right: Box::new(value_expr),
                                        }),
                                    }));
                                }
                            }
                        }
                    } else {
                        // Asignación normal: variable = value
                        let var_name = left_side.to_string();
                        if let Ok(value_expr) = Self::parse_expr_from_text(right_side) {
                            statements.push(Stmt::Expr(Expr::Assign {
                                name: var_name,
                                value: Box::new(value_expr),
                            }));
                        }
                    }
                }
                i += 1;
            } else {
                // Cualquier otra línea: intentar parsear como expresión
                // Puede ser un método call o cualquier expresión válida
                if let Ok(expr) = Self::parse_expr_from_text(line) {
                    if matches!(expr, Expr::MethodCall { .. }) {
                        // Es un método call, agregar como statement
                        statements.push(Stmt::Expr(expr));
                    }
                    // Si no es MethodCall, lo ignoramos (podría ser solo un identificador)
                }
                i += 1;
            }
        }
        
        Ok(statements)
    }

    /// Parsear expresión desde texto
    pub fn parse_expr_from_text(text: &str) -> Result<Expr, Box<dyn std::error::Error>> {
        let text = text.trim();
        
        if text.is_empty() {
            return Err("Empty expression".into());
        }
        
        // Método: arr.append(x) - buscar .method_name( ANTES de llamadas a función
        // Esto debe ir antes de parsear funciones normales
        if let Some(dot_pos) = text.find('.') {
            // Verificar que después del punto hay un método seguido de paréntesis
            let after_dot = &text[dot_pos+1..];
            if after_dot.contains('(') && text.ends_with(')') {
                let object_str = text[..dot_pos].trim();
                if let Some(args_start) = after_dot.find('(') {
                    let method_name = after_dot[..args_start].trim();
                    // args_text va desde después del '(' hasta el último ')' 
                    // Como text.ends_with(')'), el último carácter de after_dot es ')'
                    let args_text = if args_start + 1 < after_dot.len() {
                        after_dot[args_start+1..after_dot.len()-1].trim()
                    } else {
                        ""
                    };
                    
                    let object_expr = Self::parse_expr_from_text(object_str)?;
                    let args = if args_text.is_empty() {
                        Vec::new()
                    } else {
                        args_text.split(',')
                            .map(|a| Self::parse_expr_from_text(a.trim()))
                            .collect::<Result<Vec<_>, _>>()?
                    };
                    
                    return Ok(Expr::MethodCall {
                        object: Box::new(object_expr),
                        method: method_name.to_string(),
                        args,
                    });
                }
            }
        }
        
        // Llamada a función: len(arr) o función(args)
        if text.contains('(') && text.ends_with(')') {
            if let Some(paren_pos) = text.find('(') {
                let name = text[..paren_pos].trim();
                let args_text = &text[paren_pos+1..text.len()-1].trim();
                
                if !args_text.is_empty() {
                    // Parsear argumentos separados por comas
                    let args: Result<Vec<_>, _> = args_text.split(',')
                        .map(|a| Self::parse_expr_from_text(a.trim()))
                        .collect();
                    return Ok(Expr::Call {
                        module: None,
                        name: name.to_string(),
                        args: args?,
                    });
                } else {
                    // Función sin argumentos
                    return Ok(Expr::Call {
                        module: None,
                        name: name.to_string(),
                        args: Vec::new(),
                    });
                }
            }
        }
        
        // Array literal: [1, 2, 3]
        if text.starts_with('[') && text.ends_with(']') {
            let inner = &text[1..text.len()-1].trim();
            if inner.is_empty() {
                return Ok(Expr::ArrayLiteral(Vec::new()));
            }
            let elements: Result<Vec<_>, _> = inner.split(',')
                .map(|e| Self::parse_expr_from_text(e.trim()))
                .collect();
            return Ok(Expr::ArrayLiteral(elements?));
        }
        
        // Array index: arr[0] o arr[i] (después de verificar métodos)
        if let Some(bracket_pos) = text.rfind('[') {
            if text.ends_with(']') {
                let array_expr_str = &text[..bracket_pos].trim();
                let index_str = &text[bracket_pos+1..text.len()-1].trim();
                
                let array_expr = Self::parse_expr_from_text(array_expr_str)?;
                let index_expr = Self::parse_expr_from_text(index_str)?;
                
                return Ok(Expr::Index {
                    array: Box::new(array_expr),
                    index: Box::new(index_expr),
                });
            }
        }
        
        // Número
        if let Ok(n) = text.parse::<i64>() {
            return Ok(Expr::Number(n));
        }
        
        // Expresiones binarias - operadores de comparación primero
        let comparisons = [
            ("<=", BinOp::Le),
            (">=", BinOp::Ge),
            ("==", BinOp::Eq),
            ("!=", BinOp::Ne),
            ("<", BinOp::Lt),
            (">", BinOp::Gt),
        ];
        
        for (op_str, op) in comparisons.iter() {
            if let Some(pos) = text.find(op_str) {
                let left_text = &text[..pos].trim();
                let right_text = &text[pos + op_str.len()..].trim();
                let left = Self::parse_expr_from_text(left_text)?;
                let right = Self::parse_expr_from_text(right_text)?;
                return Ok(Expr::BinaryOp {
                    op: *op,
                    left: Box::new(left),
                    right: Box::new(right),
                });
            }
        }
        
        // Operadores aritméticos
        let arithmetic_ops = [
            ("%", BinOp::Mod),
            ("*", BinOp::Mul),
            ("/", BinOp::Div),
            ("+", BinOp::Add),
            ("-", BinOp::Sub),
        ];
        
        for (op_str, op) in arithmetic_ops.iter() {
            // Solo si no está al inicio (para números negativos en el caso de -)
            if let Some(pos) = text.find(op_str) {
                if op_str == &"-" && pos == 0 {
                    continue; // Skip si - está al inicio
                }
                let left_text = &text[..pos].trim();
                let right_text = &text[pos + op_str.len()..].trim();
                let left = Self::parse_expr_from_text(left_text)?;
                let right = Self::parse_expr_from_text(right_text)?;
                return Ok(Expr::BinaryOp {
                    op: *op,
                    left: Box::new(left),
                    right: Box::new(right),
                });
            }
        }
        
        // Identificador
        Ok(Expr::Ident(text.to_string()))
    }
}

/// Función helper para extraer while loops del código fuente
pub fn extract_while_loops(source: &str) -> Vec<(usize, usize, String)> {
    let mut results = Vec::new();
    let re = regex::Regex::new(r"while\s+[^{]*\{").unwrap();
    
    let mut search_pos = 0;
    while let Some(mat) = re.find_at(source, search_pos) {
        let start = mat.start();
        // Encontrar el cierre correspondiente
        let mut brace_count = 0;
        let mut found_start = false;
        let mut end_pos = start;
        
        for (i, ch) in source[start..].char_indices() {
            if ch == '{' {
                brace_count += 1;
                found_start = true;
            } else if ch == '}' {
                brace_count -= 1;
                if found_start && brace_count == 0 {
                    end_pos = start + i + 1;
                    break;
                }
            }
        }
        
        if end_pos > start {
            let while_text = source[start..end_pos].to_string();
            results.push((start, end_pos, while_text));
            search_pos = end_pos;
        } else {
            break;
        }
    }
    
    results
}
