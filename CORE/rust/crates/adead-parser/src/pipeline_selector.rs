/**
 * Pipeline Selector - ADead Compiler
 * 
 * Selecciona el mejor pipeline para compilar código ADead a NASM.
 * 
 * PIPELINE PRINCIPAL (NASM Directo - PRIORIDAD MÁXIMA):
 *   ADead → Parser (Rust) → NASM Generator (Rust) → NASM → .obj → Linker → .exe
 *   - Genera código ASM virgen y limpio
 *   - Sin dependencia de GCC/Clang para compilación
 *   - Soporta: Arrays, Strings, Control Flow, Funciones, Structs, Classes
 * 
 * PIPELINE FALLBACK (C++):
 *   ADead → Parser (Rust) → C++ Generator → GCC++/Clang++ → Rust Cleaner → ASM → NASM
 *   - Solo para características avanzadas no implementadas en NASM directo
 *   - Requiere GCC/Clang instalado
 * 
 * Autor: Eddi Andreé Salazar Matos
 * Fecha: Diciembre 2025
 */

use std::path::Path;
use std::process::Command;
use std::fs;

/// Características detectadas en el código ADead
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CodeFeatures {
    pub has_while_loops: bool,
    pub has_if_statements: bool,
    pub has_for_loops: bool,
    pub has_functions: bool,
    pub has_arrays: bool,
    pub has_strings: bool,
    pub has_structs: bool,
    pub has_classes: bool,
    pub has_imports: bool,
    pub has_floats: bool,
    pub complexity_score: u32,
}

/// Tipo de pipeline recomendado
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendedPipeline {
    /// PRIORIDAD MÁXIMA: Parser Rust → Backend NASM directo
    /// Genera ASM virgen y limpio sin dependencias externas
    NasmDirect,
    
    /// Fallback: Parser → C++ Generator → GCC++/Clang++ → Rust Cleaner → ASM
    /// Para características que requieren runtime C++
    CppFallback,
    
    /// Fallback alternativo: Parser → C Generator → GCC/Clang → Rust Cleaner → ASM
    CFallback,
}

impl Default for RecommendedPipeline {
    fn default() -> Self {
        // NASM directo es SIEMPRE el default - prioridad máxima
        RecommendedPipeline::NasmDirect
    }
}

impl std::fmt::Display for RecommendedPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecommendedPipeline::NasmDirect => write!(f, "NASM Directo (Prioritario)"),
            RecommendedPipeline::CppFallback => write!(f, "C++ Fallback"),
            RecommendedPipeline::CFallback => write!(f, "C Fallback"),
        }
    }
}

/// Analizar código ADead y detectar características
pub fn analyze_code_features(source: &str) -> CodeFeatures {
    CodeFeatures {
        has_while_loops: source.contains("while"),
        has_if_statements: {
            let words: Vec<&str> = source.split_whitespace().collect();
            words.iter().any(|w| *w == "if" || w.starts_with("if "))
        },
        has_for_loops: source.contains("for ") && source.contains(" in "),
        has_functions: source.contains("fn "),
        has_arrays: source.contains('[') && source.contains(']'),
        has_strings: source.contains('"'),
        has_structs: source.contains("struct "),
        has_classes: source.contains("class "),
        has_imports: source.contains("import "),
        has_floats: {
            // Detectar floats: número.número (pero no rangos como 0..10)
            let mut has_float = false;
            for line in source.lines() {
                if line.contains('.') && !line.contains("..") {
                    // Verificar si hay un patrón de número.número
                    for part in line.split_whitespace() {
                        if part.contains('.') && !part.contains("..") {
                            let chars: Vec<char> = part.chars().collect();
                            for i in 0..chars.len().saturating_sub(2) {
                                if chars[i].is_ascii_digit() 
                                    && chars[i + 1] == '.' 
                                    && chars.get(i + 2).map_or(false, |c| c.is_ascii_digit()) {
                                    has_float = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            has_float
        },
        complexity_score: calculate_complexity(source),
    }
}

/// Calcular score de complejidad del código
fn calculate_complexity(source: &str) -> u32 {
    let mut score = 0u32;
    
    // Control flow
    score += source.matches("while").count() as u32 * 5;
    score += source.matches("if ").count() as u32 * 3;
    score += source.matches("for ").count() as u32 * 5;
    score += source.matches("else").count() as u32 * 2;
    
    // Funciones y estructuras
    score += source.matches("fn ").count() as u32 * 4;
    score += source.matches("struct ").count() as u32 * 6;
    score += source.matches("class ").count() as u32 * 8;
    
    // Profundidad de anidamiento (aproximada por conteo de llaves)
    let open_braces = source.matches('{').count() as u32;
    if open_braces > 3 {
        score += (open_braces - 3) * 2;
    }
    
    // Variables y expresiones
    score += source.matches("let ").count() as u32;
    
    score
}

/// Seleccionar el mejor pipeline según las características
/// 
/// POLÍTICA: NASM DIRECTO SIEMPRE (es el objetivo principal de ADead)
/// 
/// El backend NASM de Rust genera código NASM nativo sin conversiones.
/// Solo usamos fallback C++ si hay una razón específica (actualmente ninguna).
pub fn select_optimal_pipeline(features: &CodeFeatures) -> RecommendedPipeline {
    // PRIORIDAD MÁXIMA: NASM Directo para todo
    // El backend NASM soporta todas las características del lenguaje ADead
    
    // Por ahora, SIEMPRE usar NASM directo
    // El backend soporta: variables, arrays, strings, control flow, 
    // funciones, structs, classes, imports
    
    // Solo consideramos fallback para características futuras no implementadas
    // (actualmente ninguna)
    
    let _ = features; // Suprimir warning de unused
    RecommendedPipeline::NasmDirect
}

/// Generar código ASM usando el pipeline seleccionado
/// 
/// NOTA: El pipeline NASM directo se ejecuta desde adead-cli
/// Esta función es para pipelines fallback (C++/C)
pub fn generate_asm_with_pipeline(
    source: &str,
    pipeline: &RecommendedPipeline,
    output_path: Option<&Path>,
) -> Result<String, String> {
    match pipeline {
        RecommendedPipeline::NasmDirect => {
            // NASM directo se maneja en adead-cli usando adead-backend
            // Evita dependencias circulares entre crates
            Err("Pipeline NASM Directo: use 'adeadc build' o 'adeadc compile'".to_string())
        }
        
        RecommendedPipeline::CppFallback => {
            // ADead → Parser → C++ Generator → GCC++/Clang++ → Rust Cleaner → ASM
            generate_via_cpp(source, output_path)
        }
        
        RecommendedPipeline::CFallback => {
            // ADead → Parser → C Generator → GCC/Clang → Rust Cleaner → ASM
            generate_via_c(source, output_path)
        }
    }
}

/// Generar ASM via pipeline C++
fn generate_via_cpp(source: &str, output_path: Option<&Path>) -> Result<String, String> {
    let program = crate::parse(source)
        .map_err(|e| format!("Parser error: {:?}", e))?;
    
    let cpp_code = crate::cpp_generator::generate_cpp_code(&program);
    
    let temp_path = output_path
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "temp.cpp".to_string());
    
    match compile_cpp_to_asm(&cpp_code, &temp_path) {
        Ok(asm_code) => {
            if is_valid_asm(&asm_code) {
                Ok(crate::clean_asm::clean_asm(&asm_code))
            } else {
                Err(format!(
                    "El compilador C++ generó código inválido.\n\
                    Recomendación: use --backend nasm para compilación directa."
                ))
            }
        }
        Err(e) => {
            Err(format!(
                "Error compilando C++ a ASM: {}\n\
                Verifica que GCC++ o Clang++ esté instalado.\n\
                Alternativa: use --backend nasm para compilación directa.",
                e
            ))
        }
    }
}

/// Generar ASM via pipeline C
fn generate_via_c(source: &str, output_path: Option<&Path>) -> Result<String, String> {
    let program = crate::c_manual_parser::CManualParser::parse_program(source)
        .map_err(|e| format!("Parser error: {:?}", e))?;
    
    let c_code = crate::c_generator::generate_c_code(&program);
    
    let temp_path = output_path
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "temp.c".to_string());
    
    match compile_c_to_asm(&c_code, &temp_path) {
        Ok(asm_code) => {
            if is_valid_asm(&asm_code) {
                Ok(crate::clean_asm::clean_asm(&asm_code))
            } else {
                eprintln!("   ⚠️  ASM generado no es válido, retornando código C");
                Ok(format!("// Código C generado\n{}", c_code))
            }
        }
        Err(e) => {
            eprintln!("   ⚠️  Error compilando C a ASM: {}", e);
            Ok(format!("// Código C generado\n{}", c_code))
        }
    }
}

/// Procesar código ADead inteligentemente (usa NASM directo por defecto)
pub fn process_adead_intelligent(source: &str) -> Result<(RecommendedPipeline, String), String> {
    let features = analyze_code_features(source);
    let pipeline = select_optimal_pipeline(&features);
    
    // NOTA: Esta función siempre devuelve error para NASM directo
    // porque la generación real se hace en adead-cli
    let asm_code = generate_asm_with_pipeline(source, &pipeline, None)?;
    
    Ok((pipeline, asm_code))
}

/// Verificar si el código generado es ASM válido
fn is_valid_asm(asm_code: &str) -> bool {
    // Verificar indicadores de ASM válido
    let has_section = asm_code.contains("section") || asm_code.contains(".text");
    let has_code = asm_code.contains("mov") || asm_code.contains("push") || 
                   asm_code.contains("call") || asm_code.contains("ret");
    let has_label = asm_code.contains("main:") || asm_code.contains("_start:");
    let min_length = asm_code.len() > 50;
    
    (has_section || has_code || has_label) && min_length
}

/// Verificar si un compilador soporta C++20
fn check_cpp20_support(compiler: &str) -> bool {
    let test_code = r#"
#include <version>
#if __cplusplus >= 202002L
int main() { return 0; }
#else
#error "C++20 not supported"
#endif
"#;
    
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join(format!("adead_cpp20_test_{}.cpp", std::process::id()));
    let obj_file = temp_dir.join(format!("adead_cpp20_test_{}.o", std::process::id()));
    
    if fs::write(&test_file, test_code).is_err() {
        return false;
    }
    
    let mut cmd = Command::new(compiler);
    cmd.arg("-std=c++20")
       .arg("-c")
       .arg(&test_file)
       .arg("-o")
       .arg(&obj_file);
    
    if !compiler.contains("++") && (compiler.contains("clang") || compiler.contains("gcc")) {
        cmd.arg("-x").arg("c++");
    }
    
    let output = cmd.output();
    let result = output.is_ok() && output.as_ref().unwrap().status.success();
    
    let _ = fs::remove_file(&test_file);
    let _ = fs::remove_file(&obj_file);
    
    result
}

/// Compilar código C++ a ASM usando GCC++/Clang++
fn compile_cpp_to_asm(cpp_code: &str, _input_path: &str) -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let cpp_file = temp_dir.join(format!("adead_temp_{}.cpp", std::process::id()));
    
    fs::write(&cpp_file, cpp_code)
        .map_err(|e| format!("Error escribiendo archivo C++: {}", e))?;
    
    let compiler = find_cpp_compiler()
        .ok_or_else(|| "No se encontró GCC++ ni Clang++".to_string())?;
    
    let asm_file = cpp_file.with_extension("s");
    
    let mut cmd = Command::new(&compiler);
    cmd.arg("-S");
    
    if !compiler.contains("++") && (compiler.contains("clang") || compiler.contains("gcc")) {
        cmd.arg("-x").arg("c++");
    }
    
    // Seleccionar estándar C++ (C++20 preferido, fallback a C++17)
    let cpp_std = if check_cpp20_support(&compiler) {
        "-std=c++20"
    } else {
        "-std=c++17"
    };
    
    cmd.arg(cpp_std)
       .arg("-O2")                           // Optimización
       .arg("-fno-asynchronous-unwind-tables") // Eliminar unwind tables
       .arg("-fno-exceptions")               // Sin excepciones
       .arg("-fno-stack-protector")          // Sin stack protector
       .arg("-mno-red-zone")                 // Sin red zone
       .arg("-o")
       .arg(&asm_file)
       .arg(&cpp_file);
    
    // Usar sintaxis Intel para ASM
    if compiler.contains("clang") {
        cmd.arg("-mllvm").arg("--x86-asm-syntax=intel");
    } else {
        cmd.arg("-masm=intel");
    }
    
    let output = cmd.output()
        .map_err(|e| format!("Error ejecutando compilador: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = fs::remove_file(&cpp_file);
        return Err(format!("Compilación fallida: {}", stderr));
    }
    
    let asm = fs::read_to_string(&asm_file)
        .map_err(|e| format!("Error leyendo archivo ASM: {}", e))?;
    
    let _ = fs::remove_file(&cpp_file);
    let _ = fs::remove_file(&asm_file);
    
    Ok(asm)
}

/// Compilar código C a ASM usando GCC/Clang
fn compile_c_to_asm(c_code: &str, _input_path: &str) -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let c_file = temp_dir.join(format!("adead_temp_{}.c", std::process::id()));
    
    fs::write(&c_file, c_code)
        .map_err(|e| format!("Error escribiendo archivo C: {}", e))?;
    
    let compiler = find_c_compiler()
        .ok_or_else(|| "No se encontró GCC ni Clang".to_string())?;
    
    let asm_file = c_file.with_extension("s");
    
    let mut cmd = Command::new(&compiler);
    cmd.arg("-S")
       .arg("-O2")
       .arg("-fno-asynchronous-unwind-tables")
       .arg("-fno-exceptions")
       .arg("-fno-stack-protector")
       .arg("-mno-red-zone")
       .arg("-o")
       .arg(&asm_file)
       .arg(&c_file);
    
    // Usar sintaxis Intel para ASM
    if compiler.contains("clang") {
        cmd.arg("-mllvm").arg("--x86-asm-syntax=intel");
    } else {
        cmd.arg("-masm=intel");
    }
    
    let output = cmd.output()
        .map_err(|e| format!("Error ejecutando compilador: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = fs::remove_file(&c_file);
        return Err(format!("Compilación fallida: {}", stderr));
    }
    
    let asm = fs::read_to_string(&asm_file)
        .map_err(|e| format!("Error leyendo archivo ASM: {}", e))?;
    
    let _ = fs::remove_file(&c_file);
    let _ = fs::remove_file(&asm_file);
    
    Ok(asm)
}

/// Buscar compilador C++ disponible (preferencia: C++20)
fn find_cpp_compiler() -> Option<String> {
    let compilers = vec![
        "clang++",
        "g++",
        "clang",
        "gcc",
        #[cfg(target_os = "windows")]
        "C:\\msys64\\mingw64\\bin\\g++.exe",
        #[cfg(target_os = "windows")]
        "C:\\msys64\\clang64\\bin\\clang++.exe",
        #[cfg(target_os = "windows")]
        "C:\\Program Files\\LLVM\\bin\\clang++.exe",
    ];
    
    let mut cpp20_compiler: Option<String> = None;
    let mut any_compiler: Option<String> = None;
    
    for compiler in compilers {
        let exists = if Path::new(compiler).exists() {
            true
        } else {
            Command::new(compiler)
                .arg("--version")
                .output()
                .map_or(false, |o| o.status.success())
        };
        
        if exists {
            if any_compiler.is_none() {
                any_compiler = Some(compiler.to_string());
            }
            if check_cpp20_support(compiler) {
                cpp20_compiler = Some(compiler.to_string());
                break; // Encontramos C++20, no buscar más
            }
        }
    }
    
    cpp20_compiler.or(any_compiler)
}

/// Buscar compilador C disponible
fn find_c_compiler() -> Option<String> {
    let compilers = vec![
        "clang",
        "gcc",
        #[cfg(target_os = "windows")]
        "C:\\msys64\\mingw64\\bin\\gcc.exe",
        #[cfg(target_os = "windows")]
        "C:\\msys64\\clang64\\bin\\clang.exe",
        #[cfg(target_os = "windows")]
        "C:\\Program Files\\LLVM\\bin\\clang.exe",
    ];
    
    for compiler in compilers {
        let exists = if Path::new(compiler).exists() {
            true
        } else {
            Command::new(compiler)
                .arg("--version")
                .output()
                .map_or(false, |o| o.status.success())
        };
        
        if exists {
            return Some(compiler.to_string());
        }
    }
    
    None
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_analyze_simple_code() {
        let source = "print 42";
        let features = analyze_code_features(source);
        assert!(!features.has_while_loops);
        assert!(!features.has_functions);
    }
    
    #[test]
    fn test_analyze_while_loop() {
        let source = "while i < 10 { print i }";
        let features = analyze_code_features(source);
        assert!(features.has_while_loops);
        assert!(features.complexity_score > 0);
    }
    
    #[test]
    fn test_analyze_arrays() {
        let source = "let arr = [1, 2, 3]";
        let features = analyze_code_features(source);
        assert!(features.has_arrays);
    }
    
    #[test]
    fn test_analyze_strings() {
        let source = r#"let s = "hello""#;
        let features = analyze_code_features(source);
        assert!(features.has_strings);
    }
    
    #[test]
    fn test_analyze_functions() {
        let source = "fn test() { return 42 }";
        let features = analyze_code_features(source);
        assert!(features.has_functions);
    }
    
    #[test]
    fn test_analyze_structs() {
        let source = "struct Point { x y }";
        let features = analyze_code_features(source);
        assert!(features.has_structs);
    }
    
    #[test]
    fn test_analyze_classes() {
        let source = "class Rect { fn new() { } }";
        let features = analyze_code_features(source);
        assert!(features.has_classes);
    }
    
    #[test]
    fn test_select_pipeline_always_nasm() {
        // Pipeline debe ser SIEMPRE NASM directo
        let features = CodeFeatures::default();
        assert_eq!(select_optimal_pipeline(&features), RecommendedPipeline::NasmDirect);
    }
    
    #[test]
    fn test_select_pipeline_complex_still_nasm() {
        let features = CodeFeatures {
            complexity_score: 100,
            has_while_loops: true,
            has_if_statements: true,
            has_for_loops: true,
            has_functions: true,
            has_arrays: true,
            has_strings: true,
            has_structs: true,
            has_classes: true,
            has_imports: true,
            has_floats: false,
        };
        // Siempre debe seleccionar NASM directo
        assert_eq!(select_optimal_pipeline(&features), RecommendedPipeline::NasmDirect);
    }
    
    #[test]
    fn test_is_valid_asm() {
        let valid_asm = r#"
section .text
global main
main:
    mov rax, 42
    ret
"#;
        assert!(is_valid_asm(valid_asm));
        
        let invalid_asm = "hello world";
        assert!(!is_valid_asm(invalid_asm));
    }
    
    #[test]
    fn test_pipeline_display() {
        assert_eq!(
            format!("{}", RecommendedPipeline::NasmDirect),
            "NASM Directo (Prioritario)"
        );
    }
}
