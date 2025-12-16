// Helper function para usar pipeline inteligente
fn compile_with_intelligent_pipeline(source: &str, input_path: &str, output_path: &str) -> Result<()> {
    use std::fs;
    
    println!("   🔍 Analizando código ADead y seleccionando pipeline óptimo...");
    
    match adead_parser::pipeline_selector::process_adead_intelligent(source) {
        Ok((pipeline, nasm_code)) => {
            let pipeline_name = match pipeline {
                adead_parser::pipeline_selector::RecommendedPipeline::ParserManualCppC => "Parser Manual → C++ Optimizer → C → GCC/Clang → Rust Cleaner → ASM Virgen",
                adead_parser::pipeline_selector::RecommendedPipeline::ParserManualC => "Parser Manual → C → GCC/Clang → Rust Cleaner → ASM Virgen",
                adead_parser::pipeline_selector::RecommendedPipeline::RustDirect => "Rust → NASM",
            };
            println!("   ✅ Pipeline seleccionado: {}", pipeline_name);
            
            fs::write(output_path, nasm_code)
                .with_context(|| format!("Failed to write output file: {}", output_path))?;
            println!("✅ Compilado: {} -> {}", input_path, output_path);
            Ok(())
        }
        Err(e) => {
            eprintln!("   ❌ Error en pipeline inteligente: {}", e);
            eprintln!("   🔄 Intentando fallback a backend C...");
            compile_with_c_backend(source, input_path, output_path)
        }
    }
}
