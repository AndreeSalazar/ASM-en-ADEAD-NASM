//! # Clean Code - Post-Procesador Optimizador de ASM
//!
//! Transforma ASM generado "sucio" (con overhead de C/GCC) en ASM virgen puro
//! directo al CPU mediante limpieza quirúrgica y optimizaciones peephole.

mod cleaner;
mod peephole;
mod dead_code;
mod constant_propagation;
mod strength_reduction;
mod objconv_integration;
mod data_flow;

pub use cleaner::AsmCleaner;
pub use cleaner::CleanResult;
pub use cleaner::OptimizationLevel;

/// Limpia ASM sucio y lo transforma en ASM virgen puro
///
/// # Ejemplo
///
/// ```rust
/// use clean_code::AsmCleaner;
///
/// let dirty_asm = r#"
///     mov rax, rax
///     mov rbx, rbx
///     jmp label1
/// label1:
///     mov rax, 5
/// "#;
///
/// let cleaner = AsmCleaner::new();
/// let clean_asm = cleaner.clean(dirty_asm).unwrap();
/// ```
pub fn clean_asm(asm: &str) -> Result<String, String> {
    let cleaner = AsmCleaner::new();
    cleaner.clean(asm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_redundant_mov() {
        let dirty = "mov rax, rax\nmov rbx, rbx\n";
        let clean = clean_asm(dirty).unwrap();
        assert!(!clean.contains("mov rax, rax"));
        assert!(!clean.contains("mov rbx, rbx"));
    }

    #[test]
    fn test_clean_jmp_to_next() {
        let dirty = "jmp label1\nlabel1:\nmov rax, 5\n";
        let clean = clean_asm(dirty).unwrap();
        assert!(!clean.contains("jmp label1"));
    }
}

