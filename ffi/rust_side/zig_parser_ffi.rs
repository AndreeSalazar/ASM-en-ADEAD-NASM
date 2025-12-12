// FFI Bridge: Rust llama a Zig para parsing completo
// Zig hace TODO el parsing, Rust solo valida y genera código

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// Estructuras C-compatibles (deben coincidir con Zig)
#[repr(C)]
pub struct ZigStructField {
    pub visibility: bool,
    pub mutable: bool,
    pub name: *const c_char,
    pub ty: *const c_char,
}

#[repr(C)]
pub struct ZigParam {
    pub name: *const c_char,
    pub ty: *const c_char,
}

#[repr(C)]
pub struct ZigMethodStmt {
    pub visibility: bool,
    pub name: *const c_char,
    pub params: *const ZigParam,
    pub params_len: usize,
    pub body: *const c_char,
}

#[repr(C)]
pub struct ZigStructStmt {
    pub name: *const c_char,
    pub fields: *const ZigStructField,
    pub fields_len: usize,
    pub init_method: *const ZigMethodStmt,
    pub destroy_method: *const ZigMethodStmt,
}

// Por ahora, implementación sin FFI real (simula Zig)
// Cuando Zig esté compilado y linkeado, usar las funciones externas

/// Wrapper Rust para parsear struct con Zig
/// Por ahora usa parsing Rust, luego será FFI real
pub fn parse_struct_with_zig(input: &str) -> Result<ZigStructStmt, String> {
    // TODO: Llamar a parse_struct_ffi cuando Zig esté linkeado
    // Por ahora, retornar error
    Err("Zig parser not yet linked. Use Rust parser for now.".to_string())
}

#[cfg(feature = "zig-integration")]
#[link(name = "adead_zig", kind = "static")]
extern "C" {
    /// Parsear un struct completo usando Zig
    fn parse_struct_ffi(
        input_ptr: *const c_char,
        input_len: usize,
    ) -> *mut ZigStructStmt;
}

#[cfg(feature = "zig-integration")]
pub fn parse_struct_with_zig_ffi(input: &str) -> Result<ZigStructStmt, String> {
    let c_input = CString::new(input).map_err(|e| format!("Invalid string: {}", e))?;
    
    unsafe {
        let zig_stmt_ptr = parse_struct_ffi(
            c_input.as_ptr(),
            input.len(),
        );
        
        if zig_stmt_ptr.is_null() {
            return Err("Zig parsing failed".to_string());
        }
        
        // Convertir puntero a estructura
        let zig_stmt = &*zig_stmt_ptr;
        Ok(*zig_stmt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Solo funciona cuando Zig está compilado
    fn test_parse_struct_zig() {
        let input = "struct Banco\n    saldo: int64\nend";
        let result = parse_struct_with_zig(input);
        assert!(result.is_ok());
    }
}
