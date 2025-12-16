use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

mod linker;
use linker::{LinkerType, compile_and_link, link_objs_to_exe, assemble_asm_to_obj, detect_linker};

// Helper function para detectar si el código tiene strings avanzados (usado en linker.rs también)
pub fn has_advanced_strings(source: &str) -> bool {
    // Detectar operaciones de strings avanzadas
    source.contains("\"") ||  // String literals
    source.contains("+") && (source.contains("let") || source.contains("s")) ||  // Concatenación
    source.contains("[") && source.contains(":") ||  // Slicing s[0:4]
    source.contains(".upper()") || source.contains(".lower()") ||  // Métodos de strings
    source.contains("len(")  // len(s)
}

// Helper function para usar backend NASM directo
fn compile_with_nasm_backend(source: &str, input_path: &str, output_path: &str) -> Result<()> {
    println!("   🎯 Usando backend NASM directo (strings avanzados detectados)...");
    
    // Parsear código ADead
    let program = adead_parser::parse(source)
        .map_err(|e| anyhow::anyhow!("Parser error: {:?}", e))?;
    
    // Generar NASM usando el backend directo
    let mut generator = adead_backend::CodeGenerator::new();
    let nasm_code = generator.generate(&program)
        .map_err(|e| anyhow::anyhow!("NASM generation error: {:?}", e))?;
    
    fs::write(output_path, nasm_code)
        .with_context(|| format!("Failed to write output file: {}", output_path))?;
    println!("✅ Compilado (NASM directo): {} -> {}", input_path, output_path);
    Ok(())
}

// Helper function para usar pipeline inteligente
fn compile_with_intelligent_pipeline(source: &str, input_path: &str, output_path: &str) -> Result<()> {
    // Si detecta strings avanzados, usar NASM directo
    if has_advanced_strings(source) {
        return compile_with_nasm_backend(source, input_path, output_path);
    }
    
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
        
        /// Backend a usar (cpp, c, nasm, auto)
        #[arg(long, default_value = "auto")]
        backend: String,
        
        /// Archivo de salida
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Archivo de salida (alias de -o)
        #[arg(short = 'o')]
        out: Option<PathBuf>,
    },
    
    /// Compila y linkea código ADead a ejecutable (.exe)
    Build {
        /// Archivo de entrada (.ad)
        input: PathBuf,
        
        /// Backend a usar (cpp, c, nasm, auto)
        #[arg(long, default_value = "auto")]
        backend: String,
        
        /// Archivo ejecutable de salida
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Linker a usar (zig, gcc, clang, auto)
        #[arg(long, default_value = "auto")]
        linker: String,
        
        /// Solo ensamblar (.asm → .obj), no linkear
        #[arg(long)]
        assemble_only: bool,
    },
    
    /// Linkea archivos .obj a ejecutable (.exe)
    Link {
        /// Archivos objeto (.obj) a linkear
        #[arg(required = true)]
        obj_files: Vec<PathBuf>,
        
        /// Archivo ejecutable de salida
        #[arg(short, long, required = true)]
        output: PathBuf,
        
        /// Linker a usar (zig, gcc, clang, auto)
        #[arg(long, default_value = "auto")]
        linker: String,
    },
    
    /// Ensambla archivo .asm a .obj
    Assemble {
        /// Archivo de entrada (.asm)
        input: PathBuf,
        
        /// Archivo objeto de salida (.obj)
        #[arg(short, long)]
        output: Option<PathBuf>,
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
                "nasm" | "direct" => {
                    // Usar backend NASM directo
                    compile_with_nasm_backend(&source, &input.display().to_string(), &output_path.display().to_string())?;
                }
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
                    // Usar pipeline inteligente (detecta automáticamente strings y usa NASM directo)
                    compile_with_intelligent_pipeline(&source, &input.display().to_string(), &output_path.display().to_string())?;
                }
            }
        }
        
        Commands::Build { input, backend, output, linker, assemble_only } => {
            // Determinar linker preferido
            let linker_type = match linker.as_str() {
                "zig" => Some(LinkerType::Zig),
                "gcc" => Some(LinkerType::Gcc),
                "clang" => Some(LinkerType::Clang),
                "auto" | _ => None,
            };
            
            if *assemble_only {
                // Solo ensamblar: .ad → .asm → .obj
                let asm_file = input.with_extension("asm");
                let source = fs::read_to_string(input)
                    .with_context(|| format!("Error al leer archivo: {}", input.display()))?;
                
                println!("   📝 Compilando {} → {}", input.display(), asm_file.display());
                
                match backend.as_str() {
                    "nasm" | "direct" => {
                        let program = adead_parser::parse(&source)
                            .map_err(|e| anyhow::anyhow!("Parser error: {:?}", e))?;
                        
                        let mut generator = adead_backend::CodeGenerator::new();
                        let nasm_code = generator.generate(&program)
                            .map_err(|e| anyhow::anyhow!("NASM generation error: {:?}", e))?;
                        
                        fs::write(&asm_file, nasm_code)
                            .with_context(|| format!("Error al escribir {}", asm_file.display()))?;
                    }
                    _ => {
                        // Usar pipeline inteligente que detecta automáticamente
                        let pipeline = adead_parser::pipeline_selector::RecommendedPipeline::ParserManualCpp;
                        
                        let asm_code = adead_parser::pipeline_selector::generate_asm_with_pipeline(
                            &source,
                            &pipeline,
                            Some(&asm_file),
                        )
                        .map_err(|e| anyhow::anyhow!("Pipeline error: {}", e))?;
                        
                        fs::write(&asm_file, asm_code)
                            .with_context(|| format!("Error al escribir {}", asm_file.display()))?;
                    }
                }
                
                let obj_file = input.with_extension("obj");
                println!("   🔧 Ensamblando {} → {}", asm_file.display(), obj_file.display());
                assemble_asm_to_obj(&asm_file, &obj_file)?;
                println!("   ✅ Objeto generado: {}", obj_file.display());
            } else {
                // Compilar y linkear completo: .ad → .asm → .obj → .exe
                let exe_file = compile_and_link(input, output.clone(), backend, linker_type)?;
                println!("   ✅ Build completo: {} → {}", input.display(), exe_file.display());
            }
        }
        
        Commands::Link { obj_files, output, linker } => {
            // Determinar linker preferido
            let linker_type = match linker.as_str() {
                "zig" => Some(LinkerType::Zig),
                "gcc" => Some(LinkerType::Gcc),
                "clang" => Some(LinkerType::Clang),
                "auto" | _ => None,
            };
            
            println!("   🔗 Linkeando {} archivo(s) .obj → {}", obj_files.len(), output.display());
            link_objs_to_exe(obj_files, output, linker_type)?;
            println!("   ✅ Ejecutable generado: {}", output.display());
        }
        
        Commands::Assemble { input, output } => {
            let obj_file = output.as_ref()
                .map(|p| p.clone())
                .unwrap_or_else(|| input.with_extension("obj"));
            
            println!("   🔧 Ensamblando {} → {}", input.display(), obj_file.display());
            assemble_asm_to_obj(input, &obj_file)?;
            println!("   ✅ Objeto generado: {}", obj_file.display());
        }
    }
    
    Ok(())
}
