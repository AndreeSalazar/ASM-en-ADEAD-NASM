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
use std::io::{self, Write};

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
        
        /// Archivo de salida (.exe)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Linker a usar: auto (default), zig, gcc, clang
        #[arg(long, default_value = "auto")]
        linker: String,
        
        /// Solo ensamblar a .obj (no linkear)
        #[arg(long)]
        assemble_only: bool,
    },
    
    /// Linkea archivos .obj a .exe
    Link {
        /// Archivos .obj a linkear
        #[arg(required = true)]
        obj_files: Vec<PathBuf>,
        
        /// Archivo de salida (.exe)
        #[arg(short, long, required = true)]
        output: PathBuf,
        
        /// Linker a usar: auto (default), zig, gcc, clang
        #[arg(long, default_value = "auto")]
        linker: String,
    },
    
    /// Ensambla archivo .asm a .obj
    Assemble {
        /// Archivo de entrada (.asm)
        input: PathBuf,
        
        /// Archivo de salida (.obj)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() {
    // Forzar flush inmediato de stderr y stdout para debugging
    let _ = io::stderr().flush();
    let _ = io::stdout().flush();
    
    eprintln!("[CLI-DEBUG] Iniciando CLI...");
    io::stderr().flush().ok();
    
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Compile { input, backend, output } => {
            eprintln!("[CLI-DEBUG] Comando: compile, input: {:?}, backend: {}", input, backend);
            io::stderr().flush().ok();
            
            if let Err(e) = cmd_compile(input, backend, output.clone()) {
                eprintln!("[CLI-ERROR] Error en compile: {}", e);
                io::stderr().flush().ok();
                std::process::exit(1);
            }
        }
        
        Commands::Build { input, backend, output, linker, assemble_only } => {
            eprintln!("[CLI-DEBUG] Comando: build, input: {:?}, backend: {}", input, backend);
            io::stderr().flush().ok();
            
            if let Err(e) = cmd_build(input, backend, output.clone(), linker, *assemble_only) {
                eprintln!("[CLI-ERROR] Error en build: {}", e);
                io::stderr().flush().ok();
                std::process::exit(1);
            }
        }
        
        Commands::Link { obj_files, output, linker } => {
            eprintln!("[CLI-DEBUG] Comando: link, obj_files: {:?}, output: {:?}", obj_files, output);
            io::stderr().flush().ok();
            
            if let Err(e) = cmd_link(obj_files, output, linker) {
                eprintln!("[CLI-ERROR] Error en link: {}", e);
                io::stderr().flush().ok();
                std::process::exit(1);
            }
        }
        
        Commands::Assemble { input, output } => {
            eprintln!("[CLI-DEBUG] Comando: assemble, input: {:?}", input);
            io::stderr().flush().ok();
            
            if let Err(e) = cmd_assemble(input, output.clone()) {
                eprintln!("[CLI-ERROR] Error en assemble: {}", e);
                io::stderr().flush().ok();
                std::process::exit(1);
    }
}
    }
    
    eprintln!("[CLI-DEBUG] CLI terminado exitosamente");
    io::stderr().flush().ok();
}

/// Comando: compile - Compila .ad a .asm
fn cmd_compile(input: &PathBuf, backend: &str, output: Option<PathBuf>) -> Result<()> {
    let output_path = output.unwrap_or_else(|| {
        let mut path = input.clone();
        path.set_extension("asm");
        path
    });
    
    eprintln!("[CLI-DEBUG] Leyendo archivo: {:?}", input);
    io::stderr().flush().ok();
    
    let source = fs::read_to_string(input)
        .with_context(|| format!("Error leyendo archivo: {}", input.display()))?;
    
    eprintln!("[CLI-DEBUG] Archivo leído: {} caracteres", source.len());
    io::stderr().flush().ok();
    
    println!("🔄 Compilando: {} → {}", input.display(), output_path.display());
    io::stdout().flush().ok();
    
    match backend {
        "nasm" | "direct" | "auto" => {
            // PRIORIDAD: Backend NASM directo
            eprintln!("[CLI-DEBUG] Usando backend NASM directo");
            io::stderr().flush().ok();
            
            compile_nasm_direct(&source, &output_path)?;
            
            println!("✅ Compilado (NASM directo): {}", output_path.display());
            io::stdout().flush().ok();
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
    assemble_only: bool,
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
    let obj_file = output.unwrap_or_else(|| {
        let mut path = input.clone();
        path.set_extension("obj");
        path
    });
    
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
    eprintln!("[CLI-DEBUG] Iniciando parse...");
    io::stderr().flush().ok();
    
    // Parsear código ADead
    let program = adead_parser::parse(source)
        .map_err(|e| {
            eprintln!("[CLI-ERROR] Error de parser: {:?}", e);
            io::stderr().flush().ok();
            anyhow::anyhow!("Error de parser: {:?}", e)
        })?;
    
    eprintln!("[CLI-DEBUG] Parse exitoso, iniciando generación NASM...");
    io::stderr().flush().ok();
    
    // Generar NASM usando el backend directo
    let mut generator = adead_backend::CodeGenerator::new();
    let nasm_code = generator.generate(&program)
        .map_err(|e| {
            eprintln!("[CLI-ERROR] Error generando NASM: {:?}", e);
            io::stderr().flush().ok();
            anyhow::anyhow!("Error generando NASM: {:?}", e)
        })?;
    
    eprintln!("[CLI-DEBUG] Generación NASM exitosa, escribiendo archivo...");
    io::stderr().flush().ok();
    
    // Escribir archivo ASM
    fs::write(output_path, nasm_code)
        .with_context(|| format!("Error escribiendo: {}", output_path.display()))?;
    
    eprintln!("[CLI-DEBUG] Archivo escrito exitosamente");
    io::stderr().flush().ok();
    
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

fn parse_linker_type(linker: &str) -> Option<LinkerType> {
    match linker.to_lowercase().as_str() {
        "zig" => Some(LinkerType::Zig),
        "gcc" => Some(LinkerType::Gcc),
        "clang" => Some(LinkerType::Clang),
        "auto" | _ => None, // Auto-detect
    }
}
