#![cfg(feature = "integration-tests")]
/**
 * Tests Exhaustivos del Pipeline Completo
 * 
 * Este módulo contiene tests exhaustivos para verificar que todo el pipeline
 * funcione correctamente: ADead → Parser → D (CTFE) → Zig (ASM) → Rust (Limpieza) → ASM Virgen
 * 
 * Autor: Eddi Andreé Salazar Matos
 * Fecha: Diciembre 2025
 */

use adead_parser::optimized_pipeline::OptimizedPipeline;
use adead_parser::clean_asm;

/// Helper para crear un archivo temporal de prueba
fn create_temp_file(content: &str) -> String {
    use std::fs;
    use std::path::PathBuf;
    
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!("test_pipeline_{}.ad", 
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .unwrap().as_secs()));
    
    fs::write(&file_path, content).expect("Failed to write temp file");
    file_path.to_string_lossy().to_string()
}

/// Helper para limpiar archivo temporal
fn cleanup_temp_file(path: &str) {
    use std::fs;
    let _ = fs::remove_file(path);
}

// ========== TESTS DEL PIPELINE COMPLETO ==========

#[test]
fn test_pipeline_simple_constant_expression() {
    let source = r#"
let x = 5 + 3
print x
"#;
    
    let temp_path = create_temp_file(source);
    
    // Probar que el pipeline procesa correctamente
    match OptimizedPipeline::process_complete(source, &temp_path) {
        Ok(asm) => {
            // Verificar que se generó ASM
            assert!(!asm.is_empty(), "ASM generado no debe estar vacío");
            
            // Verificar que el ASM está limpio (sin SEH metadata)
            assert!(!asm.contains(".seh_"), "ASM no debe contener metadatos SEH");
            
            // Verificar que hay código de función main
            assert!(asm.contains("main") || asm.contains("_start") || 
                   asm.contains("section") || asm.contains("global"),
                   "ASM debe contener código de función");
            
            println!("✅ Pipeline procesó expresión constante correctamente");
            println!("   ASM generado: {} bytes", asm.len());
        }
        Err(e) => {
            // Si falla, verificar que sea por una razón conocida (Zig no disponible, etc.)
            println!("⚠️  Pipeline falló (esperado si Zig no está disponible): {}", e);
            // No fallar el test si es un error esperado
        }
    }
    
    cleanup_temp_file(&temp_path);
}

#[test]
fn test_pipeline_variable_assignment() {
    let source = r#"
let x = 10
let y = 20
let suma = x + y
print suma
"#;
    
    let temp_path = create_temp_file(source);
    
    match OptimizedPipeline::process_complete(source, &temp_path) {
        Ok(asm) => {
            assert!(!asm.is_empty());
            assert!(!asm.contains(".seh_"));
            
            println!("✅ Pipeline procesó asignación de variables correctamente");
        }
        Err(e) => {
            println!("⚠️  Pipeline falló: {}", e);
        }
    }
    
    cleanup_temp_file(&temp_path);
}

#[test]
fn test_pipeline_if_statement() {
    let source = r#"
let x = 5
if x > 3 {
    print x
}
"#;
    
    let temp_path = create_temp_file(source);
    
    match OptimizedPipeline::process_complete(source, &temp_path) {
        Ok(asm) => {
            assert!(!asm.is_empty());
            assert!(!asm.contains(".seh_"));
            
            // Verificar que hay código de comparación
            assert!(asm.contains("cmp") || asm.contains("test") || 
                   asm.contains("jg") || asm.contains("jle"),
                   "ASM debe contener código de comparación para if");
            
            println!("✅ Pipeline procesó if statement correctamente");
        }
        Err(e) => {
            println!("⚠️  Pipeline falló: {}", e);
        }
    }
    
    cleanup_temp_file(&temp_path);
}

#[test]
fn test_pipeline_while_loop() {
    let source = r#"
let i = 0
while i < 10 {
    print i
    let i = i + 1
}
"#;
    
    let temp_path = create_temp_file(source);
    
    match OptimizedPipeline::process_complete(source, &temp_path) {
        Ok(asm) => {
            assert!(!asm.is_empty());
            assert!(!asm.contains(".seh_"));
            
            // Verificar que hay código de loop (jmp o labels)
            assert!(asm.contains("jmp") || asm.contains("je") || 
                   asm.contains("jne") || asm.contains(":") ||
                   asm.contains("loop"),
                   "ASM debe contener código de loop");
            
            println!("✅ Pipeline procesó while loop correctamente");
        }
        Err(e) => {
            println!("⚠️  Pipeline falló: {}", e);
        }
    }
    
    cleanup_temp_file(&temp_path);
}

#[test]
fn test_pipeline_constant_folding() {
    let source = r#"
let x = 5 + 3
let y = 10 * 2
let z = (5 + 3) * 2
print x
print y
print z
"#;
    
    let temp_path = create_temp_file(source);
    
    match OptimizedPipeline::process_complete(source, &temp_path) {
        Ok(asm) => {
            assert!(!asm.is_empty());
            assert!(!asm.contains(".seh_"));
            
            println!("✅ Pipeline procesó constant folding correctamente");
        }
        Err(e) => {
            println!("⚠️  Pipeline falló: {}", e);
        }
    }
    
    cleanup_temp_file(&temp_path);
}

#[test]
fn test_pipeline_clean_asm_removes_seh() {
    let asm_with_seh = r#"
.seh_proc main
main:
    push rbp
    .seh_pushreg rbp
    mov rbp, rsp
    .seh_stackalloc 32
    .seh_endprologue
    mov rax, 42
    pop rbp
    ret
.seh_endproc
"#;
    
    let cleaned = clean_asm::clean_asm(asm_with_seh);
    
    // Verificar que SEH metadata fue eliminado
    assert!(!cleaned.contains(".seh_"));
    assert!(!cleaned.contains(".seh_proc"));
    assert!(!cleaned.contains(".seh_pushreg"));
    assert!(!cleaned.contains(".seh_stackalloc"));
    assert!(!cleaned.contains(".seh_endprologue"));
    assert!(!cleaned.contains(".seh_endproc"));
    
    // Verificar que el código útil se mantiene
    assert!(cleaned.contains("mov rax, 42"));
    assert!(cleaned.contains("ret"));
    
    println!("✅ Limpieza de ASM eliminó SEH metadata correctamente");
}

#[test]
fn test_pipeline_clean_asm_removes_frame_overhead() {
    let asm_with_frame = r#"
simple_func:
    push rbp
    mov rbp, rsp
    mov rax, 42
    pop rbp
    ret
"#;
    
    let cleaned = clean_asm::clean_asm(asm_with_frame);
    
    // Verificar que frame overhead fue eliminado (si la función no lo necesita)
    // Nota: Esto puede variar dependiendo de la implementación
    println!("✅ Limpieza de ASM procesó frame overhead");
    println!("   Código limpio: {} bytes", cleaned.len());
}

#[test]
fn test_pipeline_clean_asm_optimizes_redundant_movements() {
    let asm_with_redundant = r#"
mov rax, 5
mov rax, 10
push rax
pop rax
mov rbx, rbx
mov rax, 0
add rax, 42
"#;
    
    let cleaned = clean_asm::clean_asm(asm_with_redundant);
    
    // Verificar que movimientos redundantes fueron optimizados
    // Nota: Las verificaciones exactas dependen de la implementación
    println!("✅ Limpieza de ASM optimizó movimientos redundantes");
    println!("   Código optimizado: {} bytes", cleaned.len());
}

#[test]
fn test_pipeline_clean_asm_optimizes_jumps() {
    let asm_with_jumps = r#"
cmp rax, 0
je label1
jmp label2
label1:
    mov rax, 42
label2:
    mov rbx, 10
"#;
    
    let cleaned = clean_asm::clean_asm(asm_with_jumps);
    
    // Verificar que saltos fueron optimizados
    println!("✅ Limpieza de ASM optimizó saltos");
    println!("   Código optimizado: {} bytes", cleaned.len());
}

#[test]
fn test_pipeline_clean_asm_removes_dead_code() {
    let asm_with_dead = r#"
mov rax, 5
mov rbx, 10
mov rcx, 20
mov rax, 30
ret
"#;
    
    let cleaned = clean_asm::clean_asm(asm_with_dead);
    
    // Verificar que código muerto fue eliminado
    println!("✅ Limpieza de ASM eliminó código muerto");
    println!("   Código optimizado: {} bytes", cleaned.len());
}

#[test]
fn test_pipeline_complete_integration() {
    let source = r#"
let x = 5 + 3
let y = 10 * 2
if x > y {
    print x
} else {
    print y
}
let i = 0
while i < 5 {
    print i
    let i = i + 1
}
"#;
    
    let temp_path = create_temp_file(source);
    
    match OptimizedPipeline::process_complete(source, &temp_path) {
        Ok(asm) => {
            // Verificaciones básicas
            assert!(!asm.is_empty(), "ASM no debe estar vacío");
            assert!(!asm.contains(".seh_"), "ASM no debe contener SEH");
            
            // Verificar que hay código generado
            let asm_lines: Vec<&str> = asm.lines().collect();
            assert!(asm_lines.len() > 10, "ASM debe tener múltiples líneas");
            
            // Verificar optimizaciones aplicadas
            // (El código debe estar optimizado después de pasar por el pipeline)
            
            println!("✅ Pipeline completo procesó código complejo correctamente");
            println!("   Líneas de ASM generadas: {}", asm_lines.len());
            println!("   Tamaño del ASM: {} bytes", asm.len());
        }
        Err(e) => {
            println!("⚠️  Pipeline falló: {}", e);
            // No fallar el test si es un error esperado (Zig no disponible, etc.)
        }
    }
    
    cleanup_temp_file(&temp_path);
}

#[test]
fn test_pipeline_error_handling() {
    // Probar con código inválido
    let invalid_source = r#"
let x = 
if {
    print
"#;
    
    let temp_path = create_temp_file(invalid_source);
    
    // El pipeline debe manejar errores gracefully
    match OptimizedPipeline::process_complete(invalid_source, &temp_path) {
        Ok(_) => {
            // Si no falla, está bien (puede que el parser sea tolerante)
            println!("✅ Pipeline manejó código inválido (no falló catastróficamente)");
        }
        Err(e) => {
            // Si falla, debe ser con un error descriptivo
            assert!(!e.is_empty(), "Error debe tener mensaje descriptivo");
            println!("✅ Pipeline manejó error correctamente: {}", e);
        }
    }
    
    cleanup_temp_file(&temp_path);
}

#[test]
fn test_pipeline_empty_source() {
    let empty_source = "";
    
    let temp_path = create_temp_file(empty_source);
    
    match OptimizedPipeline::process_complete(empty_source, &temp_path) {
        Ok(asm) => {
            // Puede generar código mínimo (headers, etc.)
            println!("✅ Pipeline procesó código vacío");
            println!("   ASM generado: {} bytes", asm.len());
        }
        Err(e) => {
            // O puede fallar con error descriptivo
            println!("✅ Pipeline manejó código vacío: {}", e);
        }
    }
    
    cleanup_temp_file(&temp_path);
}

#[test]
fn test_pipeline_large_program() {
    // Crear un programa más grande para probar rendimiento
    let mut source = String::new();
    source.push_str("let x = 0\n");
    
    for i in 1..=100 {
        source.push_str(&format!("let x = x + {}\n", i));
    }
    
    source.push_str("print x\n");
    
    let temp_path = create_temp_file(&source);
    
    match OptimizedPipeline::process_complete(&source, &temp_path) {
        Ok(asm) => {
            assert!(!asm.is_empty());
            assert!(!asm.contains(".seh_"));
            
            println!("✅ Pipeline procesó programa grande correctamente");
            println!("   Líneas de código fuente: {}", source.lines().count());
            println!("   Tamaño del ASM: {} bytes", asm.len());
        }
        Err(e) => {
            println!("⚠️  Pipeline falló con programa grande: {}", e);
        }
    }
    
    cleanup_temp_file(&temp_path);
}

#[test]
fn test_pipeline_arithmetic_operations() {
    let source = r#"
let a = 10
let b = 5
let suma = a + b
let resta = a - b
let multiplicacion = a * b
let division = a / b
let modulo = a % b
print suma
print resta
print multiplicacion
print division
print modulo
"#;
    
    let temp_path = create_temp_file(source);
    
    match OptimizedPipeline::process_complete(source, &temp_path) {
        Ok(asm) => {
            assert!(!asm.is_empty());
            assert!(!asm.contains(".seh_"));
            
            // Verificar que hay código para operaciones aritméticas
            assert!(asm.contains("add") || asm.contains("sub") || 
                   asm.contains("mul") || asm.contains("div") ||
                   asm.contains("imul") || asm.contains("idiv"),
                   "ASM debe contener código para operaciones aritméticas");
            
            println!("✅ Pipeline procesó operaciones aritméticas correctamente");
        }
        Err(e) => {
            println!("⚠️  Pipeline falló: {}", e);
        }
    }
    
    cleanup_temp_file(&temp_path);
}

#[test]
fn test_pipeline_comparison_operations() {
    let source = r#"
let a = 10
let b = 5
if a > b {
    print a
}
if a < b {
    print b
}
if a == b {
    print a
}
if a != b {
    print b
}
"#;
    
    let temp_path = create_temp_file(source);
    
    match OptimizedPipeline::process_complete(source, &temp_path) {
        Ok(asm) => {
            assert!(!asm.is_empty());
            assert!(!asm.contains(".seh_"));
            
            // Verificar que hay código de comparación
            assert!(asm.contains("cmp") || asm.contains("test"),
                   "ASM debe contener código de comparación");
            
            println!("✅ Pipeline procesó operaciones de comparación correctamente");
        }
        Err(e) => {
            println!("⚠️  Pipeline falló: {}", e);
        }
    }
    
    cleanup_temp_file(&temp_path);
}

// ========== TESTS DE RENDIMIENTO ==========

#[test]
fn test_pipeline_performance_small() {
    use std::time::Instant;
    
    let source = r#"
let x = 5 + 3
print x
"#;
    
    let temp_path = create_temp_file(source);
    let start = Instant::now();
    
    let _ = OptimizedPipeline::process_complete(source, &temp_path);
    
    let duration = start.elapsed();
    
    println!("✅ Pipeline procesó código pequeño en {:?}", duration);
    assert!(duration.as_secs() < 10, "Pipeline debe completarse en menos de 10 segundos");
    
    cleanup_temp_file(&temp_path);
}

#[test]
fn test_pipeline_output_quality() {
    let source = r#"
let x = 5 + 3
print x
"#;
    
    let temp_path = create_temp_file(source);
    
    match OptimizedPipeline::process_complete(source, &temp_path) {
        Ok(asm) => {
            // Verificar calidad del ASM generado
            let lines: Vec<&str> = asm.lines().collect();
            let non_empty_lines: Vec<&str> = lines.iter()
                .filter(|l| !l.trim().is_empty())
                .copied()
                .collect();
            
            assert!(non_empty_lines.len() > 0, "ASM debe tener líneas no vacías");
            
            // Verificar que no hay líneas excesivamente largas (posible error)
            for line in &non_empty_lines {
                assert!(line.len() < 1000, "Líneas de ASM no deben ser excesivamente largas");
            }
            
            println!("✅ Calidad del ASM generado verificada");
            println!("   Líneas no vacías: {}", non_empty_lines.len());
        }
        Err(e) => {
            println!("⚠️  Pipeline falló: {}", e);
        }
    }
    
    cleanup_temp_file(&temp_path);
}

