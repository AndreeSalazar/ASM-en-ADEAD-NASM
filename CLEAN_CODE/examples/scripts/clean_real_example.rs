//! Ejemplo real: Limpiar ASM generado por Clang

use clean_code::{AsmCleaner, OptimizationLevel};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧹 CLEAN CODE - Limpiando ASM real de Clang\n");

    // Leer ASM sucio
    let dirty_asm = fs::read_to_string("examples/test_array_CLANG_dirty.asm")?;
    let original_lines = dirty_asm.lines().count();
    let original_size = dirty_asm.len();

    println!("📊 Estadísticas Originales:");
    println!("   Líneas: {}", original_lines);
    println!("   Tamaño: {} bytes\n", original_size);

    // Limpiar con diferentes niveles
    println!("🔧 Aplicando limpieza...\n");

    // Nivel EXTREMO
    let cleaner_extreme = AsmCleaner::with_level(OptimizationLevel::Extreme);
    let clean_extreme = cleaner_extreme.clean(&dirty_asm)?;
    let extreme_lines = clean_extreme.lines().count();
    let extreme_size = clean_extreme.len();

    // Nivel Avanzado
    let cleaner_advanced = AsmCleaner::with_level(OptimizationLevel::Advanced);
    let clean_advanced = cleaner_advanced.clean(&dirty_asm)?;
    let advanced_lines = clean_advanced.lines().count();
    let advanced_size = clean_advanced.len();

    // Nivel Básico
    let cleaner_basic = AsmCleaner::new();
    let clean_basic = cleaner_basic.clean(&dirty_asm)?;
    let basic_lines = clean_basic.lines().count();
    let basic_size = clean_basic.len();

    // Guardar resultados
    fs::write("examples/test_array_CLANG_cleaned_basic.asm", &clean_basic)?;
    fs::write("examples/test_array_CLANG_cleaned_advanced.asm", &clean_advanced)?;
    fs::write("examples/test_array_CLANG_cleaned_extreme.asm", &clean_extreme)?;

    // Mostrar resultados
    println!("✅ Resultados:\n");
    println!("┌─────────────┬──────────┬──────────┬─────────────┐");
    println!("│ Nivel       │ Líneas   │ Tamaño   │ Reducción   │");
    println!("├─────────────┼──────────┼──────────┼─────────────┤");
    println!("│ Original    │ {:8} │ {:8} │ {:10} │", original_lines, original_size, "0%");
    
    let basic_reduction = ((original_lines - basic_lines) as f64 / original_lines as f64) * 100.0;
    println!("│ Básico      │ {:8} │ {:8} │ {:9.1}% │", basic_lines, basic_size, basic_reduction);
    
    let advanced_reduction = ((original_lines - advanced_lines) as f64 / original_lines as f64) * 100.0;
    println!("│ Avanzado    │ {:8} │ {:8} │ {:9.1}% │", advanced_lines, advanced_size, advanced_reduction);
    
    let extreme_reduction = ((original_lines - extreme_lines) as f64 / original_lines as f64) * 100.0;
    println!("│ EXTREMO 🔥  │ {:8} │ {:8} │ {:9.1}% │", extreme_lines, extreme_size, extreme_reduction);
    println!("└─────────────┴──────────┴──────────┴─────────────┘\n");

    println!("📁 Archivos generados:");
    println!("   - examples/test_array_CLANG_cleaned_basic.asm");
    println!("   - examples/test_array_CLANG_cleaned_advanced.asm");
    println!("   - examples/test_array_CLANG_cleaned_extreme.asm\n");

    Ok(())
}

