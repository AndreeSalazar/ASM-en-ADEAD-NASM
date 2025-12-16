use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

// Helper function para usar pipeline inteligente
fn compile_with_intelligent_pipeline(source: &str, input_path: &str, output_path: &str) -> Result<()> {
    println!("   🔍 Analizando código ADead y seleccionando pipeline óptimo...");
    
    match adead_parser::pipeline_selector::process_adead_intelligent(source) {
        Ok((pipeline, nasm_code)) => {
            let pipeline_name = match pipeline {
                adead_parser::pipeline_selector::RecommendedPipeline::ParserManualCpp => "Parser Manual → C++ Generator → GCC++/Clang++ → Rust Cleaner → ASM Virgen",
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

// Fallback function para compilar con backend C
fn compile_with_c_backend(source: &str, input_path: &str, output_path: &str) -> Result<()> {
    use std::path::Path;
    
    println!("   🔄 Usando pipeline C como fallback...");
    
    let pipeline = adead_parser::pipeline_selector::RecommendedPipeline::ParserManualC;
    let asm_code = adead_parser::pipeline_selector::generate_asm_with_pipeline(
        source,
        &pipeline,
        Some(Path::new(output_path)),
    )
    .map_err(|e| anyhow::anyhow!("Pipeline C error: {}", e))?;
    
    fs::write(output_path, asm_code)
        .with_context(|| format!("Failed to write output file: {}", output_path))?;
    println!("✅ Compilado (C backend): {} -> {}", input_path, output_path);
    Ok(())
}

#[derive(Parser)]
#[command(name = "adeadc")]
#[command(about = "ADead Compiler - Compila código ADead a ASM", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compila código ADead a ASM
    Compile {
        /// Archivo de entrada (.ad)
        input: PathBuf,
        
        /// Backend a usar (cpp, c, auto)
        #[arg(long, default_value = "auto")]
        backend: String,
        
        /// Archivo de salida
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Archivo de salida (alias de -o)
        #[arg(short = 'o')]
        out: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Compile { input, backend, output, out } => {
            let output_path = output.as_ref()
                .or(out.as_ref())
                .map(|p| p.clone())
                .unwrap_or_else(|| {
                    let mut path = input.clone();
                    path.set_extension("asm");
                    path
                });
            
            let source = fs::read_to_string(input)
                .with_context(|| format!("Failed to read input file: {}", input.display()))?;
            
            match backend.as_str() {
                "cpp" | "c++" => {
                    // Usar pipeline C++ directamente
                    let pipeline = adead_parser::pipeline_selector::RecommendedPipeline::ParserManualCpp;
                    let asm_code = adead_parser::pipeline_selector::generate_asm_with_pipeline(
                        &source,
                        &pipeline,
                        Some(output_path.as_path()),
                    )
                    .map_err(|e| anyhow::anyhow!("Pipeline C++ error: {}", e))?;
                    
                    fs::write(&output_path, asm_code)
                        .with_context(|| format!("Failed to write output file: {}", output_path.display()))?;
                    println!("✅ Compilado (C++ backend): {} -> {}", input.display(), output_path.display());
                }
                "c" => {
                    // Usar pipeline C directamente
                    compile_with_c_backend(&source, &input.display().to_string(), &output_path.display().to_string())?;
                }
                "auto" | _ => {
                    // Usar pipeline inteligente (detecta automáticamente)
                    compile_with_intelligent_pipeline(&source, &input.display().to_string(), &output_path.display().to_string())?;
                }
            }
        }
    }
    
    Ok(())
}
