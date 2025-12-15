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
 * Arquitectura: Zig + Rust + C + Parser Manual + D Language
 * 
 * Autor: Eddi Andreé Salazar Matos
 * Fecha: Diciembre 2025
 */

use crate::zig_nasm_generator;
use std::path::Path;

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
    /// Parser Manual → C → GCC/Clang → ASM (flujo principal actual)
    ParserManualC,
    /// Zig → NASM directo (máxima eficiencia para casos simples)
    ZigDirect,
    /// Zig → Rust → NASM (eficiente + seguro)
    ZigRust,
    /// D → Zig → NASM (metaprogramming + eficiencia)
    DZig,
    /// D → Zig → Rust → NASM (máxima potencia: metaprogramming + eficiencia + seguridad)
    DZigRust,
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
    // Caso 1: Código muy simple (solo prints, literales)
    if features.complexity_score == 0 && !features.has_expressions {
        return RecommendedPipeline::ZigDirect;
    }
    
    // Caso 2: Estructuras complejas anidadas (while con if) - PRIORIDAD MÁXIMA
    // Si tiene while Y if statements (bloques anidados), usar Parser Manual + C (principal)
    if (features.has_nested_blocks && features.has_while_loops && features.has_if_statements) ||
       (features.has_while_loops && features.has_if_statements) {
        // Para máxima robustez: D → Zig → Rust → NASM
        // Pero si D no está disponible, usar Parser Manual + C (flujo principal)
        #[cfg(feature = "d-language")]
        {
            return RecommendedPipeline::DZigRust;
        }
        #[cfg(not(feature = "d-language"))]
        {
            return RecommendedPipeline::ParserManualC;
        }
    }
    
    // Caso 3: While loops simples (sin if anidado)
    if features.has_while_loops {
        // Usar Parser Manual + C (flujo principal) o Zig si está disponible
        #[cfg(not(feature = "no-zig"))]
        {
            return RecommendedPipeline::ZigDirect;
        }
        #[cfg(feature = "no-zig")]
        {
            // Zig no disponible, usar Parser Manual + C
            return RecommendedPipeline::ParserManualC;
        }
    }
    
    // Caso 4: Expresiones aritméticas con validación necesaria
    if features.has_expressions && features.has_variables {
        return RecommendedPipeline::ZigRust;
    }
    
    // Caso 5: Floats simples
    if features.has_floats && !features.has_arithmetic {
        return RecommendedPipeline::ZigDirect;
    }
    
    // Caso 6: If statements simples
    if features.has_if_statements && !features.has_nested_blocks {
        return RecommendedPipeline::ParserManualC;
    }
    
    // Default: Parser Manual + C (flujo principal)
    RecommendedPipeline::ParserManualC
}

/// Generar código ASM usando el pipeline seleccionado
pub fn generate_asm_with_pipeline(
    source: &str,
    pipeline: &RecommendedPipeline,
    output_path: Option<&Path>,
) -> Result<String, String> {
    match pipeline {
        RecommendedPipeline::ZigDirect => {
            // ADead → Zig → NASM (directo, sin capas)
            zig_nasm_generator::generate_nasm_direct(source)
                .ok_or_else(|| "Zig directo falló".to_string())
        }
        
        RecommendedPipeline::ParserManualC => {
            // ADead → Parser Manual → C → GCC/Clang → ASM (flujo principal actual)
            // Usar parser manual y generador de C
            let program = crate::c_manual_parser::CManualParser::parse_program(source)
                .map_err(|e| format!("Parser manual error: {:?}", e))?;
            let c_code = crate::c_generator::generate_c_code(&program);
            // Retornar código C (GCC/Clang lo compila después)
            Ok(format!("// Código C generado\n{}", c_code))
        }
        
        RecommendedPipeline::ZigRust => {
            // ADead → Zig (parse) → Rust (validación) → NASM
            // Por ahora, usar Zig directo (Rust validación se hace después)
            zig_nasm_generator::generate_nasm_direct(source)
                .ok_or_else(|| "Zig → Rust pipeline falló".to_string())
        }
        
        RecommendedPipeline::DZig => {
            // ADead → D → Zig → NASM
            #[cfg(feature = "d-language")]
            {
                crate::d_zig_asm::compile_adead_to_asm_via_zig(source)
                    .ok_or_else(|| "D → Zig pipeline falló".to_string())
            }
            #[cfg(not(feature = "d-language"))]
            {
                // Fallback a Zig directo si D no está disponible
                zig_nasm_generator::generate_nasm_direct(source)
                    .ok_or_else(|| "D no disponible, Zig falló".to_string())
            }
        }
        
        RecommendedPipeline::DZigRust => {
            // ADead → D (metaprogramming) → Zig → Rust (validación) → NASM
            #[cfg(feature = "d-language")]
            {
                crate::d_zig_asm::compile_adead_to_asm_via_zig(source)
                    .ok_or_else(|| "D → Zig → Rust pipeline falló".to_string())
            }
            #[cfg(not(feature = "d-language"))]
            {
                // Fallback a Zig → Rust si D no está disponible
                zig_nasm_generator::generate_nasm_direct(source)
                    .ok_or_else(|| "D no disponible, Zig → Rust falló".to_string())
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
    let asm_code = generate_asm_with_pipeline(source, &pipeline, None)?;
    
    // 4. Optimizar y limpiar código ASM
    let optimized_asm = optimize_asm(&asm_code)?;
    
    Ok((pipeline, optimized_asm))
}

/// Optimizar código ASM generado
fn optimize_asm(asm: &str) -> Result<String, String> {
    let mut optimized = asm.to_string();
    
    // Eliminar instrucciones redundantes
    // mov rax, rax -> (eliminar)
    optimized = optimized.replace("    mov rax, rax\n", "");
    
    // Optimizar secuencias comunes
    // push rax; pop rax -> (eliminar)
    // (esto requiere análisis más complejo, por ahora básico)
    
    // Limpiar espacios múltiples
    while optimized.contains("  \n") {
        optimized = optimized.replace("  \n", " \n");
    }
    
    // Asegurar formato consistente
    optimized = optimized.replace("\r\n", "\n");
    
    Ok(optimized)
}

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
        assert_eq!(select_optimal_pipeline(&features), RecommendedPipeline::ZigDirect);
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
        assert!(matches!(pipeline, RecommendedPipeline::ParserManualC | RecommendedPipeline::DZigRust));
    }
}

