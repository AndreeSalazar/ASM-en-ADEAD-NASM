/**
 * Pipeline Selector Inteligente
 * 
 * Este módulo identifica el tipo de código ADead y selecciona
 * el mejor pipeline (D, Zig, Rust, C, Parser Manual) para procesarlo
 * 
 * Flujo:
 * 1. Analizar código ADead
 * 2. Identificar características (simplicidad, complejidad, tipos)
 * 3. Seleccionar pipeline óptimo
 * 4. Generar ASM puro y limpio
 * 5. Ejecutar
 * 
 * Arquitectura: Parser Manual (Rust) + C++ Optimizer + C Generator (Rust) + Rust Cleaner
 * 
 * Autor: Eddi Andreé Salazar Matos
 * Fecha: Diciembre 2025
 */

use std::path::Path;
use std::process::Command;
use std::fs;

/// Características detectadas en el código ADead
#[derive(Debug, Clone, PartialEq)]
pub struct CodeFeatures {
    pub has_while_loops: bool,
    pub has_if_statements: bool,
    pub has_nested_blocks: bool,
    pub has_variables: bool,
    pub has_expressions: bool,
    pub has_floats: bool,
    pub has_arithmetic: bool,
    pub has_comparisons: bool,
    pub complexity_score: u32,
}

/// Tipo de pipeline recomendado
#[derive(Debug, Clone, PartialEq)]
pub enum RecommendedPipeline {
    /// Parser Manual → C++ Optimizer → C → GCC/Clang → Rust Cleaner → ASM Virgen (flujo principal)
    ParserManualCppC,
    /// Parser Manual → C → GCC/Clang → Rust Cleaner → ASM Virgen (fallback sin C++ Optimizer)
    ParserManualC,
    /// Rust directo (fallback completo)
    RustDirect,
}

/// Analizar código ADead y detectar características
pub fn analyze_code_features(source: &str) -> CodeFeatures {
    let source_lower = source.to_lowercase();
    
    CodeFeatures {
        has_while_loops: source.contains("while"),
        has_if_statements: {
            // Detectar if statements (puede estar dentro de while)
            // Buscar "if" que no sea parte de otra palabra
            let words: Vec<&str> = source.split_whitespace().collect();
            words.iter().any(|w| *w == "if" || w.starts_with("if"))
        },
        has_nested_blocks: {
            let open_braces = source.matches('{').count();
            let close_braces = source.matches('}').count();
            open_braces > 1 && close_braces > 1
        },
        has_variables: source.contains("let") || 
                      source.contains("=") && !source.contains("==") && !source.contains("!="),
        has_expressions: source.contains('+') || source.contains('-') || 
                        source.contains('*') || source.contains('/') || source.contains('%'),
        has_floats: source.contains('.') && source.chars().any(|c| c.is_ascii_digit()),
        has_arithmetic: source.contains('+') || source.contains('-') || 
                       source.contains('*') || source.contains('/'),
        has_comparisons: source.contains("<=") || source.contains(">=") || 
                        source.contains("==") || source.contains("!=") ||
                        source.contains('<') || source.contains('>'),
        complexity_score: calculate_complexity(&source),
    }
}

/// Calcular score de complejidad
fn calculate_complexity(source: &str) -> u32 {
    let mut score = 0;
    
    // Estructuras complejas
    if source.contains("while") { score += 10; }
    if source.contains("if") { score += 5; }
    
    // Anidamiento
    let brace_depth = source.chars().fold((0, 0), |(depth, max), c| {
        let new_depth = match c {
            '{' => depth + 1,
            '}' => depth - 1,
            _ => depth,
        };
        (new_depth, max.max(new_depth))
    }).1;
    score += brace_depth * 3;
    
    // Variables y expresiones
    if source.contains("let") { score += 2; }
    let expr_count = source.matches('+').count() + 
                     source.matches('-').count() + 
                     source.matches('*').count() + 
                     source.matches('/').count();
    score += expr_count;
    
    score as u32
}

/// Seleccionar el mejor pipeline según las características
pub fn select_optimal_pipeline(features: &CodeFeatures) -> RecommendedPipeline {
    // Siempre usar Parser Manual + C++ Optimizer + C como flujo principal
    // C++ Optimizer proporciona optimizaciones compile-time
    RecommendedPipeline::ParserManualCppC
}

/// Generar código ASM usando el pipeline seleccionado
pub fn generate_asm_with_pipeline(
    source: &str,
    pipeline: &RecommendedPipeline,
    output_path: Option<&Path>,
) -> Result<String, String> {
    match pipeline {
        RecommendedPipeline::ParserManualCppC => {
            // ADead → Parser Manual → C++ Optimizer → C → GCC/Clang → ASM (flujo principal)
            let program = crate::c_manual_parser::CManualParser::parse_program(source)
                .map_err(|e| format!("Parser manual error: {:?}", e))?;
            
            // Optimizar AST usando C++ Optimizer (si está disponible)
            let optimized_program = crate::cpp_optimizer::optimize_ast(&program)
                .unwrap_or(program); // Fallback a programa sin optimizar si C++ no está disponible
            
            let c_code = crate::c_generator::generate_c_code(&optimized_program);
            
            // Compilar código C a ASM usando GCC/Clang
            let temp_path = output_path
                .and_then(|p| p.to_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "temp.c".to_string());
            
            match compile_c_to_asm_for_pipeline(&c_code, &temp_path) {
                Ok(asm_code) => {
                    // Verificar que el ASM tiene contenido válido (NASM o GAS)
                    if asm_code.contains("section") || asm_code.contains(".text") || 
                       asm_code.contains(".globl") || asm_code.contains("main:") ||
                       asm_code.len() > 100 {
                        // Limpiar ASM usando Rust Cleaner
                        Ok(crate::clean_asm::clean_asm(&asm_code))
                    } else {
                        Ok(format!("// Código C generado\n{}", c_code))
                    }
                }
                Err(e) => {
                    eprintln!("   ⚠️  No se pudo compilar C a ASM: {}, retornando código C", e);
                    Ok(format!("// Código C generado\n{}", c_code))
                }
            }
        }
        
        RecommendedPipeline::ParserManualC => {
            // ADead → Parser Manual → C → GCC/Clang → ASM (flujo principal actual)
            // Usar parser manual y generador de C
            let program = crate::c_manual_parser::CManualParser::parse_program(source)
                .map_err(|e| format!("Parser manual error: {:?}", e))?;
            let c_code = crate::c_generator::generate_c_code(&program);
            
            // Compilar código C a ASM usando GCC/Clang
            // Usar el output_path o un path temporal
            let temp_path = output_path
                .and_then(|p| p.to_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "temp.c".to_string());
            
            match compile_c_to_asm_for_pipeline(&c_code, &temp_path) {
                Ok(asm_code) => {
                    // Verificar que el ASM tiene contenido válido (NASM o GAS)
                    if asm_code.contains("section") || asm_code.contains(".text") || 
                       asm_code.contains(".globl") || asm_code.contains("main:") ||
                       asm_code.len() > 100 {
                        // Limpiar ASM usando Rust Cleaner
                        Ok(crate::clean_asm::clean_asm(&asm_code))
                    } else {
                        Ok(format!("// Código C generado\n{}", c_code))
                    }
                }
                Err(e) => {
                    eprintln!("   ⚠️  No se pudo compilar C a ASM: {}, retornando código C", e);
                    Ok(format!("// Código C generado\n{}", c_code))
                }
            }
        }
        
        RecommendedPipeline::RustDirect => {
            // ADead → Rust → NASM (fallback completo)
            // Usar parser Rust estándar
            crate::parse_with_dir(source, None)
                .map_err(|e| format!("Parser error: {:?}", e))?;
            // Generar NASM usando el generador estándar
            Err("NASM generation not yet implemented for this pipeline".to_string())
        }
    }
}

/// Procesar código ADead inteligentemente
/// Analiza, selecciona pipeline óptimo, genera ASM limpio
pub fn process_adead_intelligent(source: &str) -> Result<(RecommendedPipeline, String), String> {
    // 1. Analizar características del código
    let mut features = analyze_code_features(source);
    
    // Detección mejorada: verificar si hay if dentro de while
    // Si hay "while" y también "if" en el código, probablemente hay if dentro de while
    if features.has_while_loops && source.contains("if") {
        features.has_if_statements = true;
        features.has_nested_blocks = true; // while con if es anidado
    }
    
    // 2. Seleccionar pipeline óptimo
    let pipeline = select_optimal_pipeline(&features);
    
    // 3. Generar ASM usando el pipeline seleccionado
    // Nota: generate_asm_with_pipeline ya aplica clean_asm internamente
    let asm_code = generate_asm_with_pipeline(source, &pipeline, None)?;
    
    Ok((pipeline, asm_code))
}

/// Compilar código C a ASM usando GCC/Clang (para pipeline ParserManualC)
fn compile_c_to_asm_for_pipeline(c_code: &str, input_path: &str) -> Result<String, String> {
    use std::path::PathBuf;
    
    // Crear archivo C temporal
    let temp_dir = std::env::temp_dir();
    let c_file = temp_dir.join(format!("adead_temp_{}.c", std::process::id()));
    
    fs::write(&c_file, c_code)
        .map_err(|e| format!("Failed to write C file: {}", e))?;
    
    // Buscar compilador C
    let compiler = find_c_compiler_for_pipeline()
        .ok_or_else(|| "No se encontró GCC ni Clang".to_string())?;
    
    // Crear archivo ASM temporal
    let asm_file = c_file.with_extension("asm");
    
    // Compilar C a ASM
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
    
    // Ajustar flags según compilador
    if compiler.contains("clang") {
        cmd.arg("-mllvm").arg("--x86-asm-syntax=intel");
    } else {
        cmd.arg("-masm=intel");
    }
    
    let output = cmd.output()
        .map_err(|e| format!("Failed to execute compiler: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = fs::remove_file(&c_file);
        return Err(format!("Compilation failed: {}", stderr));
    }
    
    // Leer ASM generado
    let asm = fs::read_to_string(&asm_file)
        .map_err(|e| format!("Failed to read ASM file: {}", e))?;
    
    // Limpiar archivos temporales
    let _ = fs::remove_file(&c_file);
    let _ = fs::remove_file(&asm_file);
    
    Ok(asm)
}

/// Buscar compilador C para el pipeline
fn find_c_compiler_for_pipeline() -> Option<String> {
    // Priorizar Clang sobre GCC
    if Command::new("clang").arg("--version").output().is_ok() {
        return Some("clang".to_string());
    }
    if Command::new("gcc").arg("--version").output().is_ok() {
        return Some("gcc".to_string());
    }
    
    // Buscar en ubicaciones comunes (Windows)
    #[cfg(target_os = "windows")]
    {
        let common_paths = vec![
            "C:\\msys64\\mingw64\\bin\\gcc.exe",
            "C:\\msys64\\clang64\\bin\\clang.exe",
            "C:\\Program Files\\LLVM\\bin\\clang.exe",
        ];
        
        for path in common_paths {
            if Path::new(path).exists() {
                if Command::new(path).arg("--version").output().is_ok() {
                    return Some(path.to_string());
                }
            }
        }
    }
    
    None
}

// Nota: La optimización de ASM se hace en clean_asm::clean_asm()
// Esta función ya no es necesaria porque clean_asm se aplica en generate_asm_with_pipeline

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_analyze_simple_code() {
        let source = "print 42";
        let features = analyze_code_features(source);
        assert_eq!(features.complexity_score, 0);
        assert!(!features.has_while_loops);
    }
    
    #[test]
    fn test_analyze_while_loop() {
        let source = "while i < 10 { print i }";
        let features = analyze_code_features(source);
        assert!(features.has_while_loops);
        assert!(features.has_comparisons);
        assert!(features.complexity_score > 0);
    }
    
    #[test]
    fn test_select_pipeline_simple() {
        let features = CodeFeatures {
            complexity_score: 0,
            has_while_loops: false,
            has_if_statements: false,
            has_nested_blocks: false,
            has_variables: false,
            has_expressions: false,
            has_floats: false,
            has_arithmetic: false,
            has_comparisons: false,
        };
        assert_eq!(select_optimal_pipeline(&features), RecommendedPipeline::ParserManualCppC);
    }
    
    #[test]
    fn test_select_pipeline_complex() {
        let features = CodeFeatures {
            complexity_score: 25,
            has_while_loops: true,
            has_if_statements: true,
            has_nested_blocks: true,
            has_variables: true,
            has_expressions: true,
            has_floats: false,
            has_arithmetic: true,
            has_comparisons: true,
        };
        let pipeline = select_optimal_pipeline(&features);
        assert_eq!(pipeline, RecommendedPipeline::ParserManualCppC);
    }
}

