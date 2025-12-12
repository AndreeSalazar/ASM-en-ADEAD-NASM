// FFI Bridge entre Rust y Zig para parsing de parámetros
// Rust usa funciones de Zig a través de FFI para parsing eficiente

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

/// Wrapper Rust para parsear parámetros con Zig
/// Parsea "nombre: tipo, otro: tipo" y retorna Vec<(nombre, tipo)>
pub fn parse_params_with_zig(input: &str) -> Result<Vec<(String, String)>, String> {
    // Por ahora, parsear en Rust hasta que Zig esté completamente integrado
    // Esta función será reemplazada cuando Zig esté compilado y linkeado
    
    if input.trim().is_empty() {
        return Ok(Vec::new());
    }
    
    let mut result = Vec::new();
    
    // Dividir por comas
    for param_str in input.split(',') {
        let trimmed = param_str.trim();
        if trimmed.is_empty() {
            continue;
        }
        
        // Buscar el separador ':'
        if let Some(colon_pos) = trimmed.find(':') {
            let name = trimmed[..colon_pos].trim().to_string();
            let tipo = trimmed[colon_pos + 1..].trim().to_string();
            
            if !name.is_empty() && !tipo.is_empty() {
                result.push((name, tipo));
            }
        }
    }
    
    Ok(result)
}

#[cfg(feature = "zig-integration")]
#[link(name = "adead_zig", kind = "static")]
extern "C" {
    /// Parsea parámetros usando Zig (cuando esté compilado)
    fn parse_params_simple(
        input_ptr: *const c_char,
        output_buffer: *mut u8,
        output_buffer_len: usize,
    ) -> c_int;
}

#[cfg(feature = "zig-integration")]
pub fn parse_params_with_zig_ffi(input: &str) -> Result<Vec<(String, String)>, String> {
    let c_input = CString::new(input).map_err(|e| format!("Invalid string: {}", e))?;
    
    // Buffer para el resultado
    let mut output_buffer = vec![0u8; 1024];
    
    unsafe {
        let count = parse_params_simple(
            c_input.as_ptr(),
            output_buffer.as_mut_ptr(),
            output_buffer.len(),
        );
        
        if count < 0 {
            return Err("Zig parsing failed".to_string());
        }
        
        // Parsear el resultado del buffer
        let output_str = CStr::from_ptr(output_buffer.as_ptr() as *const c_char)
            .to_str()
            .map_err(|e| format!("Invalid UTF-8: {}", e))?;
        
        // Convertir "name1:type1,name2:type2" a Vec<(String, String)>
        let mut result = Vec::new();
        for param_str in output_str.split(',') {
            if let Some(colon_pos) = param_str.find(':') {
                let name = param_str[..colon_pos].to_string();
                let tipo = param_str[colon_pos + 1..].to_string();
                result.push((name, tipo));
            }
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_params_rust() {
        let result = parse_params_with_zig("nombre: string").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], ("nombre".to_string(), "string".to_string()));
    }

    #[test]
    fn test_parse_params_multiple() {
        let result = parse_params_with_zig("nombre: string, edad: int64").unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("nombre".to_string(), "string".to_string()));
        assert_eq!(result[1], ("edad".to_string(), "int64".to_string()));
    }

    #[test]
    fn test_parse_params_empty() {
        let result = parse_params_with_zig("").unwrap();
        assert_eq!(result.len(), 0);
    }
}
