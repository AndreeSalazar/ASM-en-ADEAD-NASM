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
            let line = lines[i].trim();
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
            } else {
                // Otros statements
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
            let line = lines[i].trim();
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
            } else if line.contains(" = ") && !line.starts_with("let ") {
                // Asignación
                if let Some(eq_pos) = line.find(" = ") {
                    let var_name = line[..eq_pos].trim().to_string();
                    let value_text = line[eq_pos + 3..].trim();
                    if let Ok(value_expr) = Self::parse_expr_from_text(value_text) {
                        statements.push(Stmt::Expr(Expr::Assign {
                            name: var_name,
                            value: Box::new(value_expr),
                        }));
                    }
                }
                i += 1;
            } else {
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
