//! ADead Compiler CLI
//! 
//! Compilador de ADead a ejecutables nativos via NASM.
//! 
//! Pipeline Principal (NASM Directo):
//!   ADead → Parser (Rust) → NASM Generator (Rust) → ASM → NASM → .obj → Linker → .exe
//! 
//! Pipeline Fallback (C++):
//!   ADead → Parser → C++ Generator → GCC++ → Rust Cleaner → ASM → NASM → .obj → Linker → .exe
//! 
//! Autor: Eddi Andreé Salazar Matos
//! Fecha: Diciembre 2025

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

mod linker;
use linker::{LinkerType, compile_and_link, link_objs_to_exe, assemble_asm_to_obj};

// ============================================================================
// CLI DEFINITION
// ============================================================================

#[derive(Parser)]
#[command(name = "adeadc")]
#[command(author = "Eddi Andreé Salazar Matos")]
#[command(version = "0.9.0")]
#[command(about = "ADead Compiler - Compila código ADead a ejecutables nativos", long_about = None)]
#[command(after_help = "Pipeline: ADead → Parser → NASM → ASM Virgen → Linker → .exe\n\nEjemplos:\n  adeadc compile programa.ad -o programa.asm\n  adeadc build programa.ad -o programa.exe\n  adeadc build programa.ad --linker zig")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compila código ADead a ASM (NASM x86_64)
    Compile {
        /// Archivo de entrada (.ad)
        input: PathBuf,
        
        /// Backend a usar: nasm (default), cpp, c
        #[arg(long, default_value = "nasm")]
        backend: String,
        
        /// Archivo de salida (.asm)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Compila y linkea código ADead a ejecutable (.exe)
    Build {
        /// Archivo de entrada (.ad)
        input: PathBuf,
        
        /// Backend a usar: nasm (default), cpp, c
        #[arg(long, default_value = "nasm")]
        backend: String,
        
        /// Archivo ejecutable de salida
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Linker a usar: zig (recomendado), gcc, clang, auto
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
        
        /// Linker a usar: zig (recomendado), gcc, clang, auto
        #[arg(long, default_value = "auto")]
        linker: String,
    },
    
    /// Ensambla archivo .asm a .obj usando NASM
    Assemble {
        /// Archivo de entrada (.asm)
        input: PathBuf,
        
        /// Archivo objeto de salida (.obj)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

// ============================================================================
// MAIN
// ============================================================================

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Compile { input, backend, output } => {
            cmd_compile(input, backend, output.clone())
        }
        
        Commands::Build { input, backend, output, linker, assemble_only } => {
            cmd_build(input, backend, output.clone(), linker, *assemble_only)
        }
        
        Commands::Link { obj_files, output, linker } => {
            cmd_link(obj_files, output, linker)
        }
        
        Commands::Assemble { input, output } => {
            cmd_assemble(input, output.clone())
        }
    }
}

// ============================================================================
// COMMANDS
// ============================================================================

/// Comando: compile - Compila .ad a .asm
fn cmd_compile(input: &PathBuf, backend: &str, output: Option<PathBuf>) -> Result<()> {
    let output_path = output.unwrap_or_else(|| {
        let mut path = input.clone();
        path.set_extension("asm");
        path
    });
    
    let source = fs::read_to_string(input)
        .with_context(|| format!("Error leyendo archivo: {}", input.display()))?;
    
    println!("🔄 Compilando: {} → {}", input.display(), output_path.display());
    
    match backend {
        "nasm" | "direct" | "auto" => {
            // PRIORIDAD: Backend NASM directo
            compile_nasm_direct(&source, &output_path)?;
            println!("✅ Compilado (NASM directo): {}", output_path.display());
        }
        "cpp" | "c++" => {
            // Fallback: Pipeline C++
            compile_cpp_fallback(&source, &output_path)?;
            println!("✅ Compilado (C++ fallback): {}", output_path.display());
        }
        "c" => {
            // Fallback: Pipeline C
            compile_c_fallback(&source, &output_path)?;
            println!("✅ Compilado (C fallback): {}", output_path.display());
        }
        _ => {
            // Default: NASM directo
            compile_nasm_direct(&source, &output_path)?;
            println!("✅ Compilado (NASM directo): {}", output_path.display());
        }
    }
    
    Ok(())
}

/// Comando: build - Compila y linkea .ad a .exe
fn cmd_build(
    input: &PathBuf, 
    backend: &str, 
    output: Option<PathBuf>, 
    linker: &str, 
    assemble_only: bool
) -> Result<()> {
    let linker_type = parse_linker_type(linker);
    
    if assemble_only {
        // Solo ensamblar: .ad → .asm → .obj
        let asm_file = input.with_extension("asm");
        let obj_file = input.with_extension("obj");
        
        let source = fs::read_to_string(input)
            .with_context(|| format!("Error leyendo archivo: {}", input.display()))?;
        
        println!("📝 Compilando: {} → {}", input.display(), asm_file.display());
        compile_nasm_direct(&source, &asm_file)?;
        
        println!("🔧 Ensamblando: {} → {}", asm_file.display(), obj_file.display());
        assemble_asm_to_obj(&asm_file, &obj_file)?;
        
        println!("✅ Objeto generado: {}", obj_file.display());
    } else {
        // Build completo: .ad → .asm → .obj → .exe
        let exe_file = compile_and_link(input, output, backend, linker_type)?;
        println!("✅ Build completo: {} → {}", input.display(), exe_file.display());
    }
    
    Ok(())
}

/// Comando: link - Linkea .obj a .exe
fn cmd_link(obj_files: &[PathBuf], output: &PathBuf, linker: &str) -> Result<()> {
    let linker_type = parse_linker_type(linker);
    
    println!("🔗 Linkeando {} archivo(s) → {}", obj_files.len(), output.display());
    link_objs_to_exe(obj_files, output, linker_type)?;
    println!("✅ Ejecutable generado: {}", output.display());
    
    Ok(())
}

/// Comando: assemble - Ensambla .asm a .obj
fn cmd_assemble(input: &PathBuf, output: Option<PathBuf>) -> Result<()> {
    let obj_file = output.unwrap_or_else(|| input.with_extension("obj"));
    
    println!("🔧 Ensamblando: {} → {}", input.display(), obj_file.display());
    assemble_asm_to_obj(input, &obj_file)?;
    println!("✅ Objeto generado: {}", obj_file.display());
    
    Ok(())
}

// ============================================================================
// COMPILATION BACKENDS
// ============================================================================

/// Compila usando backend NASM directo (PRIORIDAD)
/// Genera ASM virgen y limpio sin dependencias externas
fn compile_nasm_direct(source: &str, output_path: &PathBuf) -> Result<()> {
    // Parsear código ADead
    let program = adead_parser::parse(source)
        .map_err(|e| anyhow::anyhow!("Error de parser: {:?}", e))?;
    
    // Generar NASM usando el backend directo
    let mut generator = adead_backend::CodeGenerator::new();
    let nasm_code = generator.generate(&program)
        .map_err(|e| anyhow::anyhow!("Error generando NASM: {:?}", e))?;
    
    // Escribir archivo ASM
    fs::write(output_path, nasm_code)
        .with_context(|| format!("Error escribiendo: {}", output_path.display()))?;
    
    Ok(())
}

/// Compila usando pipeline C++ (fallback)
fn compile_cpp_fallback(source: &str, output_path: &PathBuf) -> Result<()> {
    use std::path::Path;
    
    let pipeline = adead_parser::pipeline_selector::RecommendedPipeline::CppFallback;
    let asm_code = adead_parser::pipeline_selector::generate_asm_with_pipeline(
        source,
        &pipeline,
        Some(Path::new(&output_path.display().to_string())),
    )
    .map_err(|e| anyhow::anyhow!("Error pipeline C++: {}", e))?;
    
    fs::write(output_path, asm_code)
        .with_context(|| format!("Error escribiendo: {}", output_path.display()))?;
    
    Ok(())
}

/// Compila usando pipeline C (fallback)
fn compile_c_fallback(source: &str, output_path: &PathBuf) -> Result<()> {
    use std::path::Path;
    
    let pipeline = adead_parser::pipeline_selector::RecommendedPipeline::CFallback;
    let asm_code = adead_parser::pipeline_selector::generate_asm_with_pipeline(
        source,
        &pipeline,
        Some(Path::new(&output_path.display().to_string())),
    )
    .map_err(|e| anyhow::anyhow!("Error pipeline C: {}", e))?;
    
    fs::write(output_path, asm_code)
        .with_context(|| format!("Error escribiendo: {}", output_path.display()))?;
    
    Ok(())
}

// ============================================================================
// HELPERS
// ============================================================================

/// Parsea el tipo de linker desde string
fn parse_linker_type(linker: &str) -> Option<LinkerType> {
    match linker.to_lowercase().as_str() {
        "zig" => Some(LinkerType::Zig),
        "gcc" => Some(LinkerType::Gcc),
        "clang" => Some(LinkerType::Clang),
        "auto" | _ => None, // Auto-detect
    }
}
