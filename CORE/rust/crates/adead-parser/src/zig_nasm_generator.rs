// Wrapper FFI para generar NASM directamente desde Zig
// Flujo: ADead → Zig → NASM (ASM) directo
// Para casos simples que no requieren validación de Rust

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

// FFI externo para llamar a Zig generate_nasm_ffi
// Linking condicional: solo si Zig está disponible
#[cfg(not(feature = "no-zig"))]
#[link(name = "adead_zig", kind = "static")]
extern "C" {
    #[link_name = "generate_nasm_ffi"]
    fn generate_nasm_ffi(
        input_ptr: *const c_char,
        input_len: usize,
        output_buffer: *mut u8,
        output_buffer_len: usize,
    ) -> c_int;
}

// Stub cuando Zig no está disponible
#[cfg(feature = "no-zig")]
unsafe extern "C" fn generate_nasm_ffi(
    _input_ptr: *const c_char,
    _input_len: usize,
    _output_buffer: *mut u8,
    _output_buffer_len: usize,
) -> c_int {
    -1 // Error: Zig no disponible
}

/// Generar código NASM directamente desde Zig (flujo directo)
/// Retorna el código NASM completo o None si falla
/// 
/// Este flujo es más eficiente para:
/// - Floats simples: print 3.14
/// - Expresiones aritméticas puras: 2 + 5, 3.14 * 2.0
/// - Statements completos: while loops, if statements, etc.
/// - No requiere validación de Rust
pub fn generate_nasm_direct(expr_str: &str) -> Option<String> {
    #[cfg(feature = "no-zig")] {
        // Zig no disponible
        return None;
    }
    
    #[cfg(not(feature = "no-zig"))] {
        // Limpiar string de entrada
        let trimmed = expr_str.trim();
        if trimmed.is_empty() {
            return None;
        }

        // Convertir a CString para FFI
        let c_input = match CString::new(trimmed) {
            Ok(s) => s,
            Err(_) => return None,
        };

        // Buffer para resultado (tamaño razonable: 16KB para código NASM)
        const BUFFER_SIZE: usize = 16384;
        let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        // Llamar a Zig generator
        let result_len = unsafe {
            generate_nasm_ffi(
                c_input.as_ptr(),
                trimmed.len(),
                buffer.as_mut_ptr(),
                BUFFER_SIZE,
            )
        };

        // Verificar resultado
        if result_len < 0 {
            // Error en Zig generator - códigos de error específicos
            let error_msg = match result_len {
                -1 => "Error general en Zig",
                -2 => "Error en parsing de statements",
                -3 => "No se parseó ningún statement",
                -4 => "Error en generación de código",
                -5 => "Ningún statement parseado (parser falló)",
                -6 => "Error al generar código completo",
                -7 => "Solo headers generados (text_section vacío)",
                -8 => "text_section vacío antes de generateCompleteCode",
                -9 => "text_section tenía contenido pero buffer final es pequeño",
                -10 => "generateStatement no agregó código a text_section",
                -11 => "Statements parseados pero text_section vacío",
                _ => "Error desconocido en Zig",
            };
            eprintln!("   ⚠️  Zig error ({}): {}", result_len, error_msg);
            
            // CRÍTICO: Si text_section está vacío, usar fallback a C
            if result_len == -7 || result_len == -8 || result_len == -10 || result_len == -11 {
                eprintln!("   🔧 Usando fallback a C debido a text_section vacío");
            }
            
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

        Some(result_str.to_string())
    }
}

/// Determinar si una expresión puede usar el flujo directo Zig → NASM
/// 
/// Criterios para flujo directo:
/// - Solo literales simples (números o floats individuales)
/// - NO expresiones binarias complejas (usa Rust para esas)
/// - Sin variables, funciones, o código complejo
/// 
/// NOTA: Expresiones como "3.14 + 2.5" deben usar Rust porque
/// el flujo directo Zig aún tiene problemas con expresiones complejas
pub fn can_use_direct_flow(expr_str: &str) -> bool {
    let trimmed = expr_str.trim();
    
    // SIEMPRE usar Zig para while loops - es más confiable que Tree-sitter
    if trimmed.contains("while") || trimmed.contains("if") {
        return true; // Forzar uso de Zig para estructuras complejas
    }
    
    // Verificar que no contiene caracteres que indiquen código complejo
    // Variables, funciones, etc.
    let complex_chars = ['(', ')', '{', '}', '[', ']', '&', '*', '=', '!', '<', '>'];
    if trimmed.chars().any(|c| complex_chars.contains(&c)) {
        // Puede tener paréntesis para agrupación, pero verificar más
        // Si tiene '=' es asignación, no usar flujo directo (excepto para while/if)
        if trimmed.contains('=') || trimmed.contains('{') || trimmed.contains('[') {
            return false;
        }
    }
    
    // Verificar que contiene al menos un número o float
    if !trimmed.chars().any(|c| c.is_ascii_digit()) {
        return false;
    }
    
    // ⚠️ IMPORTANTE: Si contiene operadores aritméticos (+, -, *, /), usar Rust
    // El flujo directo Zig solo funciona para literales simples por ahora
    // CRÍTICO: Verificar operadores ANTES de cualquier otra cosa
    // Verificar primero el operador más común: '+'
    if trimmed.contains('+') {
        return false; // Cualquier expresión con '+' debe usar Rust
    }
    
    // Verificar otros operadores binarios (pero no negativos al inicio)
    let has_operator = if trimmed.starts_with('-') {
        // Puede ser un número negativo, verificar si hay operador después del signo
        let rest = &trimmed[1..];
        rest.contains('-') ||  // Segundo '-' después del primero es operador
        rest.contains('*') || 
        rest.contains('/')
    } else {
        // Verificar operadores binarios (-, *, /)
        // IMPORTANTE: Para '-', solo considerarlo operador si no está al inicio
        trimmed.contains('-') || 
        trimmed.contains('*') || 
        trimmed.contains('/')
    };
    
    if has_operator {
        return false; // Usar Rust para expresiones binarias
    }
    
    // Solo literales simples (números o floats individuales)
    // Ejemplo: "3.14" o "42" o "-5.2" - OK para flujo directo
    // Ejemplo: "3.14 + 2.5" - NO, usar Rust
    true
}

