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
    /// Parser Manual → C++ Generator → GCC++/Clang++ → Rust Cleaner → ASM Virgen (flujo principal)
    ParserManualCpp,
    /// Parser Manual → C++ Optimizer → C → GCC/Clang → Rust Cleaner → ASM Virgen (fallback)
    ParserManualCppC,
    /// Parser Manual → C → GCC/Clang → Rust Cleaner → ASM Virgen (fallback sin C++)
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
    // Siempre usar Parser Manual + C++ Generator como flujo principal
    // C++ Generator usa std::vector, RAII, constexpr para código más limpio
    RecommendedPipeline::ParserManualCpp
}

/// Generar código ASM usando el pipeline seleccionado
pub fn generate_asm_with_pipeline(
    source: &str,
    pipeline: &RecommendedPipeline,
    output_path: Option<&Path>,
) -> Result<String, String> {
    match pipeline {
        RecommendedPipeline::ParserManualCpp => {
            // ADead → Parser Manual → C++ Generator → GCC++/Clang++ → Rust Cleaner → ASM (flujo principal)
            let program = crate::c_manual_parser::CManualParser::parse_program(source)
                .map_err(|e| format!("Parser manual error: {:?}", e))?;
            
            // Generar código C++ usando C++ Generator
            let cpp_code = crate::cpp_generator::generate_cpp_code(&program);
            
            // Compilar código C++ a ASM usando GCC++/Clang++
            let temp_path = output_path
                .and_then(|p| p.to_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "temp.cpp".to_string());
            
            match compile_cpp_to_asm_for_pipeline(&cpp_code, &temp_path) {
                Ok(asm_code) => {
                    // Verificar que el ASM tiene contenido válido (NASM o GAS)
                    if asm_code.contains("section") || asm_code.contains(".text") || 
                       asm_code.contains(".globl") || asm_code.contains("main:") ||
                       asm_code.contains(".intel_syntax") || asm_code.contains("push") ||
                       asm_code.len() > 100 {
                        // Limpiar ASM usando Rust Cleaner
                        Ok(crate::clean_asm::clean_asm(&asm_code))
                    } else {
                        // ASM no válido - retornar error descriptivo en lugar de código C++
                        Err(format!(
                            "El compilador C++ generó código que no parece ser ASM válido.\n\
                            Longitud del código generado: {} bytes\n\
                            Contenido (primeras 200 líneas):\n{}\n\n\
                            Posibles causas:\n\
                            1. El compilador C++ no está funcionando correctamente\n\
                            2. El código C++ generado tiene errores\n\
                            3. El compilador no soporta las opciones usadas\n\n\
                            Código C++ generado (para diagnóstico):\n{}",
                            asm_code.len(),
                            asm_code.lines().take(200).collect::<Vec<_>>().join("\n"),
                            cpp_code.lines().take(50).collect::<Vec<_>>().join("\n")
                        ))
                    }
                }
                Err(e) => {
                    // Mejorar mensaje de error con información útil
                    let compiler_info = find_cpp_compiler_for_pipeline()
                        .map(|c| format!("Compilador encontrado: {}", c))
                        .unwrap_or_else(|| "No se encontró ningún compilador C++".to_string());
                    
                    Err(format!(
                        "No se pudo compilar C++ a ASM.\n\n\
                        Error: {}\n\n\
                        {}\n\n\
                        Verifica que:\n\
                        1. GCC++ o Clang++ está instalado\n\
                        2. El compilador está en PATH o en una ruta común\n\
                        3. El código C++ generado es válido\n\n\
                        Rutas comunes verificadas:\n\
                        - C:\\Program Files\\LLVM\\bin\\clang++.exe\n\
                        - C:\\msys64\\mingw64\\bin\\g++.exe\n\
                        - C:\\msys64\\clang64\\bin\\clang++.exe\n\n\
                        Código C++ generado (primeras 30 líneas para diagnóstico):\n{}",
                        e,
                        compiler_info,
                        cpp_code.lines().take(30).collect::<Vec<_>>().join("\n")
                    ))
                }
            }
        }
        
        RecommendedPipeline::ParserManualCppC => {
            // ADead → Parser Manual → C++ Optimizer → C → GCC/Clang → ASM (fallback)
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

/// Verificar si un compilador soporta C++20
fn check_cpp20_support(compiler: &str) -> bool {
    // Crear un archivo temporal de prueba C++20 más completo
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
    
    // Intentar compilar con C++20
    let mut cmd = Command::new(compiler);
    cmd.arg("-std=c++20")
       .arg("-c")
       .arg(&test_file)
       .arg("-o")
       .arg(&obj_file);
    
    // Si el compilador es "clang" o "gcc" (sin ++), especificar lenguaje C++
    if !compiler.contains("++") && (compiler.contains("clang") || compiler.contains("gcc")) {
        cmd.arg("-x").arg("c++");
    }
    
    // Ejecutar y verificar resultado
    let output = cmd.output();
    let result = output.is_ok() && output.as_ref().unwrap().status.success();
    
    // Limpiar archivos temporales
    let _ = fs::remove_file(&test_file);
    let _ = fs::remove_file(&obj_file);
    
    result
}

/// Compilar código C++ a ASM usando GCC++/Clang++ (para pipeline ParserManualCpp)
fn compile_cpp_to_asm_for_pipeline(cpp_code: &str, input_path: &str) -> Result<String, String> {
    use std::path::PathBuf;
    
    // Crear archivo C++ temporal
    let temp_dir = std::env::temp_dir();
    let cpp_file = temp_dir.join(format!("adead_temp_{}.cpp", std::process::id()));
    
    fs::write(&cpp_file, cpp_code)
        .map_err(|e| format!("Failed to write C++ file: {}", e))?;
    
    // Buscar compilador C++
    let compiler = find_cpp_compiler_for_pipeline()
        .ok_or_else(|| "No se encontró GCC++ ni Clang++".to_string())?;
    
    // Crear archivo ASM temporal
    let asm_file = cpp_file.with_extension("asm");
    
    // Compilar C++ a ASM
    let mut cmd = Command::new(&compiler);
    cmd.arg("-S");
    
    // Si el compilador es "clang" o "gcc" (sin ++), especificar lenguaje C++
    if !compiler.contains("++") && (compiler.contains("clang") || compiler.contains("gcc")) {
        cmd.arg("-x").arg("c++");
    }
    
    // Detectar soporte C++20 y usar si está disponible
    // C++20 es preferido porque tiene mejores features (ranges, concepts, format)
    let cpp_std = if check_cpp20_support(&compiler) {
        "-std=c++20"  // Usar C++20 si está disponible (preferido)
    } else {
        "-std=c++17"  // Fallback a C++17 si C++20 no está disponible
    };
    
    cmd.arg(cpp_std)
       .arg("-O2")
       .arg("-fno-asynchronous-unwind-tables")
       .arg("-fno-exceptions")
       .arg("-fno-stack-protector")
       .arg("-mno-red-zone")
       .arg("-o")
       .arg(&asm_file)
       .arg(&cpp_file);
    
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
        let stdout = String::from_utf8_lossy(&output.stdout);
        let _ = fs::remove_file(&cpp_file);
        
        // Mejorar mensaje de error con más contexto
        let error_msg = if stderr.is_empty() && !stdout.is_empty() {
            format!("Compilation failed (sin stderr, pero stdout contiene): {}", stdout)
        } else if stderr.len() > 2000 {
            // Truncar stderr muy largo
            format!("Compilation failed: {}... (truncado, {} caracteres totales)", 
                &stderr[..2000], stderr.len())
        } else {
            format!("Compilation failed: {}", stderr)
        };
        
        return Err(format!(
            "{}\n\n\
            Compilador usado: {}\n\
            Archivo C++ temporal: {}\n\
            Comando ejecutado: {} -S {} -O2 ... -o {} {}\n\n\
            Sugerencias:\n\
            1. Verifica que el código C++ generado es válido\n\
            2. Verifica que el compilador soporta C++20/C++17\n\
            3. Intenta compilar manualmente el archivo temporal para más detalles",
            error_msg,
            compiler,
            cpp_file.display(),
            compiler,
            cpp_std,
            asm_file.display(),
            cpp_file.display()
        ));
    }
    
    // Leer ASM generado
    let asm = fs::read_to_string(&asm_file)
        .map_err(|e| format!("Failed to read ASM file: {} (archivo: {})", e, asm_file.display()))?;
    
    // Verificar que el ASM tiene contenido válido antes de retornar
    if asm.is_empty() {
        let _ = fs::remove_file(&cpp_file);
        let _ = fs::remove_file(&asm_file);
        return Err(format!(
            "El compilador generó un archivo ASM vacío.\n\
            Compilador: {}\n\
            Archivo ASM: {}\n\
            Esto puede indicar un problema con el compilador o las opciones usadas.",
            compiler,
            asm_file.display()
        ));
    }
    
    // Verificar que contiene al menos algunas instrucciones ASM básicas
    let has_asm_content = asm.contains("section") || asm.contains(".text") || 
                         asm.contains(".globl") || asm.contains("main:") ||
                         asm.contains(".intel_syntax") || asm.contains("push") ||
                         asm.contains("mov") || asm.contains("call") ||
                         asm.contains("ret");
    
    if !has_asm_content && asm.len() < 100 {
        let _ = fs::remove_file(&cpp_file);
        let _ = fs::remove_file(&asm_file);
        return Err(format!(
            "El archivo ASM generado no parece contener código ASM válido.\n\
            Longitud: {} bytes\n\
            Contenido (primeras 500 caracteres):\n{}\n\n\
            Esto puede indicar que el compilador no generó ASM correctamente.",
            asm.len(),
            &asm.chars().take(500).collect::<String>()
        ));
    }
    
    // Limpiar archivos temporales
    let _ = fs::remove_file(&cpp_file);
    let _ = fs::remove_file(&asm_file);
    
    Ok(asm)
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

/// Buscar compilador C++ para el pipeline (con detección de C++20)
fn find_cpp_compiler_for_pipeline() -> Option<String> {
    // Lista de compiladores a probar (en orden de preferencia)
    let mut compilers_to_try = Vec::new();
    
    // Primero probar compiladores en PATH
    compilers_to_try.push("clang++".to_string());
    compilers_to_try.push("g++".to_string());
    compilers_to_try.push("clang".to_string());
    compilers_to_try.push("gcc".to_string());
    
    // Luego probar rutas comunes de Windows
    #[cfg(target_os = "windows")]
    {
        compilers_to_try.extend(vec![
            "C:\\msys64\\mingw64\\bin\\g++.exe".to_string(),
            "C:\\msys64\\clang64\\bin\\clang++.exe".to_string(),
            "C:\\Program Files\\LLVM\\bin\\clang++.exe".to_string(),
            "C:\\msys64\\mingw64\\bin\\gcc.exe".to_string(),
            "C:\\msys64\\clang64\\bin\\clang.exe".to_string(),
        ]);
    }
    
    // Buscar compilador que funcione (preferir C++20)
    let mut cpp20_compiler: Option<String> = None;
    let mut cpp17_compiler: Option<String> = None;
    
    for compiler in compilers_to_try {
        // Verificar si existe (para rutas absolutas) o está en PATH
        let compiler_exists = if Path::new(&compiler).exists() {
            true
        } else if compiler.contains("++") || compiler.contains("clang") || compiler.contains("gcc") {
            // Para compiladores en PATH, verificar que respondan a --version
            Command::new(&compiler).arg("--version").output().is_ok()
        } else {
            false
        };
        
        if compiler_exists {
            // Verificar soporte C++20 primero (preferido)
            if check_cpp20_support(&compiler) {
                cpp20_compiler = Some(compiler.clone());
                // Continuar buscando por si hay uno mejor, pero guardar este
            } else if cpp17_compiler.is_none() {
                // Guardar como fallback C++17 solo si no tenemos C++20
                cpp17_compiler = Some(compiler);
            }
        }
    }
    
    // Retornar C++20 si está disponible (preferido), sino C++17
    cpp20_compiler.or(cpp17_compiler)
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
        assert_eq!(select_optimal_pipeline(&features), RecommendedPipeline::ParserManualCpp);
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
        assert_eq!(pipeline, RecommendedPipeline::ParserManualCpp);
    }
}

