// Parser de structs usando Zig
// Este módulo convierte el resultado de Zig a AST Rust

use crate::{StructField, StructMethod, Visibility, FnParam, BorrowType, Stmt};
use std::ffi::CStr;

/// Convertir ZigStructStmt (de FFI) a AST Rust
/// Por ahora, parsear directamente desde el string usando lógica similar a Zig
pub fn parse_struct_from_string(input: &str) -> Result<(String, Vec<StructField>, Option<StructMethod>, Option<StructMethod>), String> {
    let lines: Vec<&str> = input.lines().collect();
    if lines.is_empty() {
        return Err("Empty input".to_string());
    }
    
    // Parsear "struct Nombre"
    let first_line = lines[0].trim();
    if !first_line.starts_with("struct ") {
        return Err("Expected 'struct'".to_string());
    }
    
    let name = first_line[7..].trim().to_string();
    if name.is_empty() {
        return Err("Expected struct name".to_string());
    }
    
    let mut fields = Vec::new();
    let mut init_method: Option<StructMethod> = None;
    let mut destroy_method: Option<StructMethod> = None;
    
    let mut i = 1;
    while i < lines.len() {
        let line = lines[i].trim();
        
        // Verificar si es "end"
        if line == "end" {
            break;
        }
        
        // Intentar parsear método primero (más específico)
        if let Some((method_name, method)) = try_parse_method(&lines, &mut i) {
            if method_name == "init" {
                init_method = Some(method);
            } else if method_name == "destroy" {
                destroy_method = Some(method);
            }
            continue;
        }
        
        // Si no es método, parsear campo
        if let Ok(field) = parse_field(line) {
            fields.push(field);
        }
        
        i += 1;
    }
    
    Ok((name, fields, init_method, destroy_method))
}

/// Intentar parsear un método
fn try_parse_method(lines: &[&str], i: &mut usize) -> Option<(String, StructMethod)> {
    if *i >= lines.len() {
        return None;
    }
    
    let line = lines[*i].trim();
    let saved_i = *i;
    
    // Opcional "pub"
    let (is_public, rest) = if line.starts_with("pub ") {
        (true, &line[4..])
    } else {
        (false, line)
    };
    
    // Debe ser "init" o "destroy"
    let method_name = if rest.starts_with("init(") {
        "init"
    } else if rest.starts_with("destroy(") {
        "destroy"
    } else {
        *i = saved_i;
        return None;
    };
    
    // Parsear parámetros
    let params_start = if method_name == "init" { 5 } else { 8 };
    let params_end = rest[params_start..].find(')')?;
    let params_str = &rest[params_start..params_start + params_end];
    
    let params = parse_params(params_str);
    
    // Buscar "end" del método
    *i += 1;
    let mut body_lines = Vec::new();
    while *i < lines.len() {
        let line = lines[*i].trim();
        if line == "end" {
            *i += 1;
            break;
        }
        body_lines.push(line);
        *i += 1;
    }
    
    // Por ahora, body vacío (luego parsear statements)
    Some((method_name.to_string(), StructMethod {
        visibility: if is_public { Visibility::Public } else { Visibility::Private },
        params,
        body: vec![], // TODO: Parsear body_lines como statements
    }))
}

/// Parsear parámetros "nombre: tipo, otro: tipo"
fn parse_params(params_str: &str) -> Vec<FnParam> {
    if params_str.trim().is_empty() {
        return Vec::new();
    }
    
    let mut result = Vec::new();
    for param_str in params_str.split(',') {
        let trimmed = param_str.trim();
        if let Some(colon_pos) = trimmed.find(':') {
            let name = trimmed[..colon_pos].trim().to_string();
            let _ty = trimmed[colon_pos + 1..].trim().to_string();
            if !name.is_empty() {
                result.push(FnParam {
                    name,
                    borrow_type: BorrowType::Owned,
                });
            }
        }
    }
    result
}

/// Parsear un campo "pub nombre: tipo" o "nombre: tipo"
fn parse_field(line: &str) -> Result<StructField, String> {
    let trimmed = line.trim();
    
    // Opcional "pub"
    let (is_public, rest) = if trimmed.starts_with("pub ") {
        (true, &trimmed[4..])
    } else {
        (false, trimmed)
    };
    
    // Opcional "mut"
    let (is_mutable, rest) = if rest.starts_with("mut ") {
        (true, &rest[4..])
    } else {
        (false, rest)
    };
    
    // Buscar ":"
    if let Some(colon_pos) = rest.find(':') {
        let name = rest[..colon_pos].trim().to_string();
        let ty = rest[colon_pos + 1..].trim().to_string();
        
        if name.is_empty() {
            return Err("Expected field name".to_string());
        }
        
        Ok(StructField {
            visibility: if is_public { Visibility::Public } else { Visibility::Private },
            mutable: is_mutable,
            name,
            ty: Some(ty),
        })
    } else {
        Err("Expected ':' in field definition".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_field() {
        let field = parse_field("saldo: int64").unwrap();
        assert_eq!(field.name, "saldo");
        assert_eq!(field.ty, Some("int64".to_string()));
        assert_eq!(field.visibility, Visibility::Private);
        
        let pub_field = parse_field("pub nombre: string").unwrap();
        assert_eq!(pub_field.name, "nombre");
        assert_eq!(pub_field.visibility, Visibility::Public);
    }

    #[test]
    fn test_parse_struct_simple() {
        let input = "struct Banco\n    saldo: int64\nend";
        let (name, fields, init, destroy) = parse_struct_from_string(input).unwrap();
        assert_eq!(name, "Banco");
        assert_eq!(fields.len(), 1);
        assert!(init.is_none());
        assert!(destroy.is_none());
    }

    #[test]
    fn test_parse_struct_with_method() {
        let input = "struct Banco\n    pub init(nombre: string)\n    end\nend";
        let (name, fields, init, destroy) = parse_struct_from_string(input).unwrap();
        assert_eq!(name, "Banco");
        assert!(init.is_some());
        assert_eq!(init.unwrap().params.len(), 1);
    }
}

