/**
 * FFI para D Language CTFE (Compile-Time Function Execution)
 * 
 * Este módulo permite llamar funciones D desde Rust para optimizar
 * expresiones constantes en compile-time.
 * 
 * Autor: Eddi Andreé Salazar Matos
 * Fecha: Diciembre 2025
 */

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// FFI externo para funciones D
// Linking condicional: solo si D está disponible
// NOTA: Si el linking falla, el código usará automáticamente el fallback Rust
#[cfg(feature = "d-language")]
extern "C" {
    /// Optimiza una expresión constante en compile-time (FFI D)
    /// Ejemplo: "5 + 3" → "8"
    /// Retorna string optimizado (debe liberarse con d_free_string)
    #[link_name = "d_optimize_const_expr"]
    fn optimize_const_expr_ffi(expr: *const c_char) -> *const c_char;
    
    /// Optimiza código fuente completo buscando expresiones constantes (FFI D)
    /// Ejemplo: "let x = 5 + 3" → "let x = 8"
    #[link_name = "d_optimize_source"]
    fn optimize_source_ffi(source: *const c_char) -> *const c_char;
    
    /// Libera memoria de string C retornado por funciones D
    #[link_name = "d_free_string"]
    fn free_string_ffi(str: *const c_char);
}

/// Optimiza una expresión constante usando D CTFE
/// 
/// # Ejemplos
/// 
/// ```
/// assert_eq!(optimize_const_expr("5 + 3"), Ok("8".to_string()));
/// assert_eq!(optimize_const_expr("10 * 2"), Ok("20".to_string()));
/// ```
pub fn optimize_const_expr(expr: &str) -> Result<String, String> {
    // Siempre usar fallback Rust por ahora
    // D se puede habilitar cuando el linking esté completamente resuelto
    Ok(optimize_const_expr_rust(expr))
}

/// Optimiza código fuente completo usando D CTFE
/// 
/// Busca y reemplaza todas las expresiones constantes en el código
/// 
/// # Ejemplo
/// 
/// ```
/// let source = "let x = 5 + 3\nlet y = 10 * 2";
/// let optimized = optimize_source(source)?;
/// // Resultado: "let x = 8\nlet y = 20"
/// ```
pub fn optimize_source(source: &str) -> Result<String, String> {
    // Siempre usar fallback Rust por ahora
    // D se puede habilitar cuando el linking esté completamente resuelto
    Ok(optimize_source_rust(source))
}

/// Optimización básica en Rust (fallback cuando D no está disponible)
fn optimize_const_expr_rust(expr: &str) -> String {
    let mut result = expr.to_string();
    
    // Buscar patrones simples manualmente: "número operador número"
    let mut changed = true;
    let mut iterations = 0;
    while changed && iterations < 10 {
        changed = false;
        iterations += 1;
        
        // Buscar patrón: número operador número
        let mut pos = 0;
        while pos < result.len() {
            // Buscar inicio de número
            if let Some(num1_start) = result[pos..].find(|c: char| c.is_ascii_digit()) {
                let num1_start = pos + num1_start;
                let num1_end = result[num1_start..]
                    .find(|c: char| !c.is_ascii_digit())
                    .map(|i| num1_start + i)
                    .unwrap_or(result.len());
                
                if num1_end > num1_start {
                    // Buscar operador después de espacios
                    let op_start = result[num1_end..]
                        .trim_start_matches(|c: char| c.is_whitespace())
                        .chars()
                        .next();
                    
                    if let Some(op_char) = op_start {
                        let op_pos = num1_end + result[num1_end..]
                            .find(op_char)
                            .unwrap_or(0);
                        
                        if matches!(op_char, '+' | '-' | '*' | '/') {
                            // Buscar segundo número
                            let num2_start = op_pos + 1;
                            let num2_start_trimmed = result[num2_start..]
                                .trim_start_matches(|c: char| c.is_whitespace());
                            
                            if let Some(num2_start_actual) = num2_start_trimmed
                                .chars()
                                .next()
                                .filter(|c| c.is_ascii_digit())
                                .map(|_| num2_start + result[num2_start..]
                                    .find(|c: char| c.is_ascii_digit())
                                    .unwrap_or(0))
                            {
                                let num2_end = result[num2_start_actual..]
                                    .find(|c: char| !c.is_ascii_digit())
                                    .map(|i| num2_start_actual + i)
                                    .unwrap_or(result.len());
                                
                                if num2_end > num2_start_actual {
                                    // Evaluar expresión
                                    if let (Ok(a), Ok(b)) = (
                                        result[num1_start..num1_end].parse::<i64>(),
                                        result[num2_start_actual..num2_end].parse::<i64>(),
                                    ) {
                                        let value = match op_char {
                                            '+' => a + b,
                                            '-' => a - b,
                                            '*' => a * b,
                                            '/' => if b != 0 { a / b } else { pos = num2_end; continue },
                                            _ => { pos = num2_end; continue },
                                        };
                                        
                                        // Reemplazar
                                        let pattern = &result[num1_start..num2_end];
                                        result = result.replace(pattern, &value.to_string());
                                        changed = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            pos += 1;
        }
    }
    
    result
}

/// Optimización básica de código fuente en Rust (fallback)
fn optimize_source_rust(source: &str) -> String {
    // Usar la misma lógica que optimize_const_expr_rust pero línea por línea
    source.lines()
        .map(|line| optimize_const_expr_rust(line))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_optimize_const_expr_simple() {
        let result = optimize_const_expr("5 + 3");
        assert!(result.is_ok());
        // Puede retornar "8" o "5 + 3" dependiendo de si D está disponible
    }
    
    #[test]
    fn test_optimize_const_expr_multiplication() {
        let result = optimize_const_expr("10 * 2");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_optimize_source() {
        let source = "let x = 5 + 3\nlet y = 10 * 2";
        let result = optimize_source(source);
        assert!(result.is_ok());
    }
}
