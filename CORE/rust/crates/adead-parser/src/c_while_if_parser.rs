// Parser especializado para While e If en el flujo C → Rust
// Prioriza el parsing robusto de estas estructuras usando extracción de texto + parser Rust

use crate::{Stmt, Expr};

/// Parser especializado para While: extrae el body completo desde el texto
pub fn parse_while_from_text(while_text: &str, source: &str) -> Result<(Expr, Vec<Stmt>), Box<dyn std::error::Error>> {
    // Buscar la condición (después de "while" y antes de "{")
    let while_text = while_text.trim();
    
    if !while_text.starts_with("while") {
        return Err("Not a while statement".into());
    }
    
    // Extraer condición: desde "while" hasta "{"
    let after_while = &while_text[5..].trim();
    
    // Buscar el bloque {
    if let Some(open_brace_pos) = after_while.find('{') {
        let condition_text = &after_while[..open_brace_pos].trim();
        
        // Parsear condición
        let condition = parse_expression_from_text(condition_text)?;
        
        // Extraer body: desde "{" hasta el "}" correspondiente
        let mut brace_count = 0;
        let mut body_start = open_brace_pos;
        let mut body_end = None;
        
        for (i, ch) in after_while[open_brace_pos..].char_indices() {
            if ch == '{' {
                brace_count += 1;
            } else if ch == '}' {
                brace_count -= 1;
                if brace_count == 0 {
                    body_end = Some(open_brace_pos + i);
                    break;
                }
            }
        }
        
        if let Some(end_pos) = body_end {
            let body_text = &after_while[open_brace_pos + 1..end_pos];
            
            // Parsear body usando parser Rust
            let body_statements = parse_statements_from_text(body_text)?;
            
            Ok((condition, body_statements))
        } else {
            Err("Unclosed brace in while statement".into())
        }
    } else {
        Err("No opening brace found in while statement".into())
    }
}

/// Parser especializado para If: extrae el body completo desde el texto
pub fn parse_if_from_text(if_text: &str) -> Result<(Expr, Vec<Stmt>, Option<Vec<Stmt>>), Box<dyn std::error::Error>> {
    let if_text = if_text.trim();
    
    if !if_text.starts_with("if") {
        return Err("Not an if statement".into());
    }
    
    // Extraer condición: desde "if" hasta "{"
    let after_if = &if_text[2..].trim();
    
    // Buscar el bloque {
    if let Some(open_brace_pos) = after_if.find('{') {
        let condition_text = &after_if[..open_brace_pos].trim();
        
        // Parsear condición
        let condition = parse_expression_from_text(condition_text)?;
        
        // Extraer body: desde "{" hasta el "}" correspondiente
        let mut brace_count = 0;
        let mut body_start = open_brace_pos;
        let mut body_end = None;
        
        for (i, ch) in after_if[open_brace_pos..].char_indices() {
            if ch == '{' {
                brace_count += 1;
            } else if ch == '}' {
                brace_count -= 1;
                if brace_count == 0 {
                    body_end = Some(open_brace_pos + i);
                    break;
                }
            }
        }
        
        if let Some(end_pos) = body_end {
            let body_text = &after_if[open_brace_pos + 1..end_pos];
            
            // Parsear body
            let then_body = parse_statements_from_text(body_text)?;
            
            // TODO: else_body parsing si existe
            
            Ok((condition, then_body, None))
        } else {
            Err("Unclosed brace in if statement".into())
        }
    } else {
        Err("No opening brace found in if statement".into())
    }
}

/// Parsear expresión desde texto (simplificado para condiciones)
fn parse_expression_from_text(text: &str) -> Result<Expr, Box<dyn std::error::Error>> {
    let text = text.trim();
    
    // Operadores de comparación
    if text.contains("<=") {
        let parts: Vec<&str> = text.splitn(2, "<=").collect();
        if parts.len() == 2 {
            let left = parse_simple_expr(parts[0].trim())?;
            let right = parse_simple_expr(parts[1].trim())?;
            return Ok(Expr::BinaryOp {
                op: crate::BinOp::Le,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
    } else if text.contains(">=") {
        let parts: Vec<&str> = text.splitn(2, ">=").collect();
        if parts.len() == 2 {
            let left = parse_simple_expr(parts[0].trim())?;
            let right = parse_simple_expr(parts[1].trim())?;
            return Ok(Expr::BinaryOp {
                op: crate::BinOp::Ge,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
    } else if text.contains("==") {
        let parts: Vec<&str> = text.splitn(2, "==").collect();
        if parts.len() == 2 {
            let left = parse_simple_expr(parts[0].trim())?;
            let right = parse_simple_expr(parts[1].trim())?;
            return Ok(Expr::BinaryOp {
                op: crate::BinOp::Eq,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
    } else if text.contains("!=") {
        let parts: Vec<&str> = text.splitn(2, "!=").collect();
        if parts.len() == 2 {
            let left = parse_simple_expr(parts[0].trim())?;
            let right = parse_simple_expr(parts[1].trim())?;
            return Ok(Expr::BinaryOp {
                op: crate::BinOp::Ne,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
    } else if text.contains("<") && !text.contains("<=") {
        let parts: Vec<&str> = text.splitn(2, "<").collect();
        if parts.len() == 2 {
            let left = parse_simple_expr(parts[0].trim())?;
            let right = parse_simple_expr(parts[1].trim())?;
            return Ok(Expr::BinaryOp {
                op: crate::BinOp::Lt,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
    } else if text.contains(">") && !text.contains(">=") {
        let parts: Vec<&str> = text.splitn(2, ">").collect();
        if parts.len() == 2 {
            let left = parse_simple_expr(parts[0].trim())?;
            let right = parse_simple_expr(parts[1].trim())?;
            return Ok(Expr::BinaryOp {
                op: crate::BinOp::Gt,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
    }
    
    // Expresiones aritméticas simples - IMPORTANTE: parsear % antes de otros operadores
    // porque puede aparecer en expresiones como "suma % intervalo"
    if text.contains('%') {
        let parts: Vec<&str> = text.splitn(2, '%').collect();
        if parts.len() == 2 {
            let left = parse_simple_expr(parts[0].trim())?;
            let right = parse_simple_expr(parts[1].trim())?;
            return Ok(Expr::BinaryOp {
                op: crate::BinOp::Mod,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
    }
    
    parse_simple_expr(text)
}

/// Parsear expresión simple (identificador, número, o expresión binaria)
pub fn parse_simple_expr(text: &str) -> Result<Expr, Box<dyn std::error::Error>> {
    let text = text.trim();
    
    // Número
    if let Ok(n) = text.parse::<i64>() {
        return Ok(Expr::Number(n));
    }
    
    // Expresión binaria con +, -, *, /, %
    if text.contains('+') {
        let parts: Vec<&str> = text.splitn(2, '+').collect();
        if parts.len() == 2 {
            let left = parse_simple_expr(parts[0].trim())?;
            let right = parse_simple_expr(parts[1].trim())?;
            return Ok(Expr::BinaryOp {
                op: crate::BinOp::Add,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
    } else if text.contains('-') && !text.starts_with('-') {
        let parts: Vec<&str> = text.splitn(2, '-').collect();
        if parts.len() == 2 {
            let left = parse_simple_expr(parts[0].trim())?;
            let right = parse_simple_expr(parts[1].trim())?;
            return Ok(Expr::BinaryOp {
                op: crate::BinOp::Sub,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
    } else if text.contains('*') {
        let parts: Vec<&str> = text.splitn(2, '*').collect();
        if parts.len() == 2 {
            let left = parse_simple_expr(parts[0].trim())?;
            let right = parse_simple_expr(parts[1].trim())?;
            return Ok(Expr::BinaryOp {
                op: crate::BinOp::Mul,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
    } else if text.contains('/') {
        let parts: Vec<&str> = text.splitn(2, '/').collect();
        if parts.len() == 2 {
            let left = parse_simple_expr(parts[0].trim())?;
            let right = parse_simple_expr(parts[1].trim())?;
            return Ok(Expr::BinaryOp {
                op: crate::BinOp::Div,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
    } else if text.contains('%') {
        let parts: Vec<&str> = text.splitn(2, '%').collect();
        if parts.len() == 2 {
            let left = parse_simple_expr(parts[0].trim())?;
            let right = parse_simple_expr(parts[1].trim())?;
            return Ok(Expr::BinaryOp {
                op: crate::BinOp::Mod,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
    }
    
    // Identificador
    Ok(Expr::Ident(text.to_string()))
}

/// Parsear statements desde texto usando parser Rust
fn parse_statements_from_text(body_text: &str) -> Result<Vec<Stmt>, Box<dyn std::error::Error>> {
    use crate::parse_with_dir;
    
    // Wrappear en un programa mínimo para usar el parser Rust
    let wrapped = format!("fn _temp() {{ {} }}", body_text);
    
    // Usar el parser Rust para parsear el body
    match parse_with_dir(&wrapped, None) {
        Ok(program) => {
            // Extraer statements del body de la función temporal
            for stmt in program.statements {
                if let Stmt::Fn { body, .. } = stmt {
                    return Ok(body);
                }
            }
            Ok(Vec::new())
        }
        Err(_) => {
            // Si falla, intentar parsing manual línea por línea
            parse_statements_manual(body_text)
        }
    }
}

/// Parsear statements manualmente línea por línea (fallback)
fn parse_statements_manual(body_text: &str) -> Result<Vec<Stmt>, Box<dyn std::error::Error>> {
    let mut statements = Vec::new();
    
    // Dividir en líneas y parsear cada una
    let lines: Vec<&str> = body_text.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i].trim();
        
        if line.is_empty() {
            i += 1;
            continue;
        }
        
        // If statement
        if line.starts_with("if ") {
            // Encontrar el bloque completo del if
            let mut if_text = String::new();
            let mut brace_count = 0;
            let mut started = false;
            
            for j in i..lines.len() {
                let current_line = lines[j];
                if_text.push_str(current_line);
                if_text.push('\n');
                
                for ch in current_line.chars() {
                    if ch == '{' {
                        brace_count += 1;
                        started = true;
                    } else if ch == '}' {
                        brace_count -= 1;
                        if started && brace_count == 0 {
                            // Parsear if completo
                            if let Ok((condition, then_body, else_body)) = parse_if_from_text(&if_text) {
                                statements.push(Stmt::If {
                                    condition,
                                    then_body,
                                    else_body,
                                });
                            }
                            i = j + 1;
                            break;
                        }
                    }
                }
                
                if started && brace_count == 0 {
                    break;
                }
            }
            continue;
        }
        
        // Print statement
        if line.starts_with("print ") {
            let expr_text = &line[6..].trim();
            if let Ok(expr) = parse_simple_expr(expr_text) {
                statements.push(Stmt::Print(expr));
            }
            i += 1;
            continue;
        }
        
        // Assignment
        if line.contains(" = ") && !line.starts_with("let ") {
            if let Some(eq_pos) = line.find(" = ") {
                let var_name = line[..eq_pos].trim().to_string();
                let value_text = &line[eq_pos + 3..].trim();
                if let Ok(value_expr) = parse_simple_expr(value_text) {
                    statements.push(Stmt::Expr(Expr::Assign {
                        name: var_name,
                        value: Box::new(value_expr),
                    }));
                }
            }
            i += 1;
            continue;
        }
        
        i += 1;
    }
    
    Ok(statements)
}

