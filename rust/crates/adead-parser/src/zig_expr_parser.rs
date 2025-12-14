// Integración FFI con Zig para parsear expresiones aritméticas
// Zig se encarga del parsing eficiente, Rust solo convierte a AST

use crate::{Expr, BinOp};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

// FFI externo para llamar a Zig
// Biblioteca Zig compilada y lista para usar
#[link(name = "adead_zig", kind = "static")]
extern "C" {
    #[link_name = "parse_expr_ffi"]
    fn parse_expr_ffi(
        input_ptr: *const c_char,
        input_len: usize,
        output_buffer: *mut u8,
        output_buffer_len: usize,
    ) -> c_int;
}

/// Parsear expresión aritmética usando Zig (PARSER PRINCIPAL)
/// Retorna la expresión parseada o None si falla
pub fn parse_expr_with_zig(expr_str: &str) -> Option<Expr> {
    // Limpiar string de entrada (trim whitespace)
    let trimmed = expr_str.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Zig AHORA soporta floats completamente - permitir que Zig los parsee
    // Zig ya tiene readFloat() implementado y serializa como "FLOAT:3.14"

    // Convertir a CString para FFI
    let c_input = match CString::new(trimmed) {
        Ok(s) => s,
        Err(_) => return None,
    };

    // Buffer para resultado (tamaño razonable: 4KB)
    const BUFFER_SIZE: usize = 4096;
    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

    // Llamar a Zig parser
    let result_len = unsafe {
        parse_expr_ffi(
            c_input.as_ptr(),
            trimmed.len(),
            buffer.as_mut_ptr(),
            BUFFER_SIZE,
        )
    };

    // Verificar resultado
    if result_len < 0 {
        // Error en Zig parser - fallback silencioso
        return None;
    }

    // Convertir buffer a string Rust
    let result_str = match CStr::from_bytes_with_nul(&buffer[..(result_len as usize + 1)]) {
        Ok(s) => match s.to_str() {
            Ok(ss) => ss,
            Err(_) => return None,
        },
        Err(_) => return None,
    };

    // Parsear el resultado serializado de Zig
    parse_zig_result(result_str)
}

/// Convertir resultado serializado de Zig a Expr de Rust
/// Formato: "NUMBER:42" o "BINOP:ADD:left:right" (recursivo)
/// Parseo recursivo que maneja expresiones anidadas
fn parse_zig_result(result: &str) -> Option<Expr> {
    parse_zig_result_recursive(result, 0).map(|(expr, _)| expr)
}

/// Parsear recursivamente, retornando (Expr, pos_final)
fn parse_zig_result_recursive(s: &str, start: usize) -> Option<(Expr, usize)> {
    let rest = &s[start..];
    
    if rest.starts_with("NUMBER:") {
        // Encontrar el número (hasta ':' siguiente o fin)
        let num_start = start + 7;
        let num_end = s[num_start..]
            .find(':')
            .map(|pos| num_start + pos)
            .unwrap_or(s.len());
        
        let num_str = &s[num_start..num_end];
        let num: i64 = num_str.parse().ok()?;
        return Some((Expr::Number(num), num_end));
    }

    if rest.starts_with("FLOAT:") {
        // Encontrar el float (hasta ':' siguiente o fin)
        let float_start = start + 6;
        // Buscar el siguiente ':' o el fin del string
        let float_end = s[float_start..]
            .find(':')
            .map(|pos| float_start + pos)
            .unwrap_or(s.len());
        
        let float_str = &s[float_start..float_end];
        // Parsear float - puede ser en formato normal o científico
        let float_val: f64 = float_str.parse().ok()?;
        return Some((Expr::Float(float_val), float_end));
    }

    if rest.starts_with("IDENT:") {
        // Encontrar el identificador (hasta ':' siguiente o fin)
        let ident_start = start + 6;
        let ident_end = s[ident_start..]
            .find(':')
            .map(|pos| ident_start + pos)
            .unwrap_or(s.len());
        
        let ident_str = &s[ident_start..ident_end];
        return Some((Expr::Ident(ident_str.to_string()), ident_end));
    }

    if rest.starts_with("BINOP:") {
        // Formato: BINOP:OP:LEFT:RIGHT
        // Encontrar el operador
        let op_start = start + 6;
        let op_end = match s[op_start..].find(':') {
            Some(pos) => op_start + pos,
            None => return None,
        };
        
        let op_str = &s[op_start..op_end];
        let op = match op_str {
            "ADD" => BinOp::Add,
            "SUB" => BinOp::Sub,
            "MUL" => BinOp::Mul,
            "DIV" => BinOp::Div,
            "EQ" => BinOp::Eq,
            "NE" => BinOp::Ne,
            "LT" => BinOp::Lt,
            "LE" => BinOp::Le,
            "GT" => BinOp::Gt,
            "GE" => BinOp::Ge,
            _ => return None,
        };

        // Parsear left (desde op_end + 1)
        let left_start = op_end + 1;
        let (left, left_end) = parse_zig_result_recursive(s, left_start)?;

        // Parsear right (desde left_end + 1, saltando el ':')
        let right_start = left_end + 1;
        if right_start >= s.len() {
            return None;
        }
        
        let (right, right_end) = parse_zig_result_recursive(s, right_start)?;

        return Some((
            Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
            right_end,
        ));
    }

    None
}

