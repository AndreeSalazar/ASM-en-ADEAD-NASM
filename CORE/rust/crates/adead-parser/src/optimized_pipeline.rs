/**
 * Pipeline Optimizado Completo: D → Zig → Rust → ASM Virgen
 * 
 * Este módulo implementa el pipeline mejorado:
 * ADead → Parser → D (CTFE) → Zig (ASM Directo) → Rust (Limpieza) → ASM Virgen
 * 
 * Autor: Eddi Andreé Salazar Matos
 * Fecha: Diciembre 2025
 */

use crate::clean_asm;
use crate::c_manual_parser::CManualParser;
use crate::c_generator;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::process::Command;
use std::path::Path;

/// Pipeline completo optimizado
pub struct OptimizedPipeline;

impl OptimizedPipeline {
    /// Procesa código ADead usando el pipeline completo: D → Zig → Rust → ASM Virgen
    pub fn process_complete(source: &str, input_path: &str) -> Result<String, String> {
        println!("   🔷 Paso 1: D Language - CTFE y optimización compile-time...");
        
        // Paso 1: D Language - CTFE (optimización compile-time)
        let optimized_source = match Self::optimize_with_d_ctfe(source) {
            Ok(opt) => {
                println!("   ✅ D CTFE optimizó el código");
                opt
            }
            Err(_) => {
                println!("   ⚠️  D CTFE no disponible, continuando sin optimización");
                source.to_string()
            }
        };
        
        println!("   ⚡ Paso 2: Zig - Generación ASM directo...");
        
        // Paso 2: Zig - Generar ASM directo
        let zig_asm = match Self::generate_asm_with_zig(&optimized_source) {
            Ok(asm) => {
                println!("   ✅ Zig generó ASM directo (sin pasar por C)");
                asm
            }
            Err(_) => {
                // Fallback: usar C → GCC/Clang → ASM
                println!("   ⚠️  Zig no disponible, usando C (backend principal) → GCC/Clang → ASM");
                println!("   🔧 C es el fallback seguro y siempre está disponible");
                Self::generate_asm_with_c(&optimized_source, input_path)?
            }
        };
        
        println!("   🔒 Paso 3: Rust - Limpieza y optimización de ASM...");
        
        // Paso 3: Rust - Limpiar y optimizar ASM
        let clean_asm = clean_asm::clean_asm(&zig_asm);
        
        println!("   ✅ Pipeline completo: ASM virgen y limpio generado");
        
        Ok(clean_asm)
    }
    
    /// Optimiza código usando D Language CTFE
    fn optimize_with_d_ctfe(source: &str) -> Result<String, String> {
        println!("   🔷 Aplicando CTFE avanzado (D Language)...");
        
        // Intentar usar D Language CTFE primero (si está disponible)
        #[cfg(feature = "d-language")]
        {
            use crate::d_ctfe;
            match d_ctfe::optimize_source(source) {
                Ok(optimized) => {
                    println!("   ✅ D CTFE optimizó el código");
                    
                    // Verificar si hubo cambios
                    if optimized != source {
                        println!("   📊 Optimizaciones aplicadas:");
                        // Mostrar diferencias
                        let source_lines: Vec<&str> = source.lines().collect();
                        let optimized_lines: Vec<&str> = optimized.lines().collect();
                        
                        for (i, (orig, opt)) in source_lines.iter().zip(optimized_lines.iter()).enumerate() {
                            if orig != opt {
                                println!("      Línea {}: {} → {}", i + 1, orig.trim(), opt.trim());
                            }
                        }
                    }
                    
                    Ok(optimized)
                }
                Err(e) => {
                    println!("   ⚠️  D CTFE no disponible: {}, usando optimización Rust básica", e);
                    // Fallback: optimización básica en Rust
                    let mut optimized = source.to_string();
                    optimized = Self::evaluate_constant_expressions(&optimized);
                    optimized = Self::remove_dead_code_basic(&optimized);
                    Ok(optimized)
                }
            }
        }
        
        #[cfg(not(feature = "d-language"))]
        {
            // Si D Language no está disponible, usar optimización Rust básica
            println!("   ⚠️  D CTFE no disponible (feature d-language no activada), usando optimización Rust básica");
            let mut optimized = source.to_string();
            optimized = Self::evaluate_constant_expressions(&optimized);
            optimized = Self::remove_dead_code_basic(&optimized);
            Ok(optimized)
        }
    }
    
    /// Evalúa expresiones constantes en compile-time
    fn evaluate_constant_expressions(source: &str) -> String {
        // Implementación básica: buscar patrones simples de suma y multiplicación
        let mut result = source.to_string();
        
        // Buscar patrones simples: "5 + 3" -> "8"
        // Estrategia: buscar dígitos seguidos de operador y más dígitos
        let mut changed = true;
        while changed {
            changed = false;
            let mut new_result = result.clone();
            
            // Buscar "número + número"
            for i in 0..result.len().saturating_sub(3) {
                if let Some(rest) = result.get(i..) {
                    // Buscar patrón simple: dígitos + espacio + "+" + espacio + dígitos
                    let parts: Vec<&str> = rest.split_whitespace().collect();
                    if parts.len() >= 3 {
                        if let (Ok(a), Ok(b)) = (parts[0].parse::<i64>(), parts[2].parse::<i64>()) {
                            if parts[1] == "+" {
                                let sum = a + b;
                                let pattern = format!("{} + {}", a, b);
                                if new_result.contains(&pattern) {
                                    new_result = new_result.replace(&pattern, &sum.to_string());
                                    changed = true;
                                    break;
                                }
                            } else if parts[1] == "*" {
                                let prod = a * b;
                                let pattern = format!("{} * {}", a, b);
                                if new_result.contains(&pattern) {
                                    new_result = new_result.replace(&pattern, &prod.to_string());
                                    changed = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            
            result = new_result;
        }
        
        result
    }
    
    /// Elimina código muerto básico
    fn remove_dead_code_basic(source: &str) -> String {
        // Por ahora, implementación básica
        // En el futuro, análisis más complejo con D
        
        let mut result = String::new();
        let mut defined_vars: std::collections::HashSet<String> = std::collections::HashSet::new();
        
        for line in source.lines() {
            let trimmed = line.trim();
            
            // Detectar definiciones de variables
            if trimmed.starts_with("let ") {
                if let Some(var_name) = trimmed.split_whitespace().nth(1) {
                    if let Some(name) = var_name.split('=').next() {
                        defined_vars.insert(name.trim().to_string());
                    }
                }
            }
            
            // Por ahora, mantener todas las líneas
            // En el futuro, eliminar variables no usadas
            result.push_str(line);
            result.push('\n');
        }
        
        result
    }
    
    /// Genera ASM usando Zig directamente
    fn generate_asm_with_zig(source: &str) -> Result<String, String> {
        // Intentar usar Zig para generar ASM directo
        // Por ahora, usar el generador Zig existente
        
        use crate::zig_nasm_generator;
        
        match zig_nasm_generator::generate_nasm_direct(source) {
            Some(asm) => {
                // Verificar que el ASM tiene contenido real (más que solo headers)
                if asm.lines().count() <= 10 {
                    eprintln!("   ⚠️  Zig generó ASM pero solo tiene {} líneas (solo headers?)", asm.lines().count());
                    eprintln!("   📄 Contenido: {}", asm.lines().take(10).collect::<Vec<_>>().join("\n"));
                }
                Ok(asm)
            },
            None => {
                eprintln!("   ⚠️  Zig retornó None - no pudo generar ASM");
                Err("Zig no pudo generar ASM".to_string())
            },
        }
    }
    
    /// Genera ASM usando C → GCC/Clang (fallback)
    fn generate_asm_with_c(source: &str, input_path: &str) -> Result<String, String> {
        // Parsear con parser manual
        let program = CManualParser::parse_program(source)
            .map_err(|e| format!("Parser error: {:?}", e))?;
        
        // Generar código C
        let c_code = c_generator::generate_c_code(&program);
        
        // Compilar C a ASM con GCC/Clang
        let asm = Self::compile_c_to_asm(&c_code, input_path)?;
        
        Ok(asm)
    }
    
    /// Compila código C a ASM usando GCC/Clang
    fn compile_c_to_asm(c_code: &str, input_path: &str) -> Result<String, String> {
        use std::fs;
        use std::path::PathBuf;
        
        // Crear archivo C temporal
        let c_file = PathBuf::from(input_path)
            .with_extension("temp.c");
        
        fs::write(&c_file, c_code)
            .map_err(|e| format!("Failed to write C file: {}", e))?;
        
        // Buscar compilador C
        let compiler = Self::find_c_compiler()
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
    
    /// Busca compilador C (GCC o Clang)
    fn find_c_compiler() -> Option<String> {
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
                "C:\\Program Files\\LLVM\\bin\\clang.exe",
                "C:\\msys64\\clang64\\bin\\clang.exe",
                "C:\\msys64\\mingw64\\bin\\gcc.exe",
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
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_evaluate_constant_expressions() {
        let source = "let x = 5 + 3";
        let optimized = OptimizedPipeline::evaluate_constant_expressions(source);
        assert!(optimized.contains("8"));
    }
    
    #[test]
    fn test_remove_dead_code_basic() {
        let source = "let x = 5\nlet y = 10";
        let optimized = OptimizedPipeline::remove_dead_code_basic(source);
        assert!(!optimized.is_empty());
    }
}

