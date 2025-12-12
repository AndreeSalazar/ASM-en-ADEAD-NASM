use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::process::Command;

#[derive(Parser)]
#[command(name = "adeadc")]
#[command(about = "ADead compiler: Python-like syntax to NASM", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile ADead source to NASM assembly
    Compile {
        /// Input file (.ad)
        #[arg(value_name = "INPUT")]
        input: String,

        /// Output file
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<String>,

        /// Also assemble and link to executable
        #[arg(short, long)]
        run: bool,

        /// Optimization level (0, 1, 2)
        #[arg(short = 'O', default_value = "0")]
        opt_level: u8,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile {
            input,
            output,
            run,
            opt_level: _,
        } => {
            let source = fs::read_to_string(&input)
                .with_context(|| format!("Failed to read input file: {}", input))?;

            let program = adead_parser::parse(&source)
                .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

            let mut generator = adead_backend::CodeGenerator::new();
            let asm = generator
                .generate(&program)
                .map_err(|e| anyhow::anyhow!("Code generation error: {}", e))?;

            let output_path = output.unwrap_or_else(|| {
                input
                    .replace(".ad", ".asm")
                    .replace(".adead", ".asm")
                    .to_string()
            });

            fs::write(&output_path, asm)
                .with_context(|| format!("Failed to write output file: {}", output_path))?;

            println!("Compiled {} to {}", input, output_path);

            if run {
                assemble_and_link(&output_path)?;
            }
        }
    }

    Ok(())
}

fn assemble_and_link(asm_file: &str) -> Result<()> {
    let obj_file = asm_file.replace(".asm", ".o");
    let exe_file = asm_file.replace(".asm", "");

    // Assemble with NASM
    let nasm_status = Command::new("nasm")
        .args(["-felf64", asm_file, "-o", &obj_file])
        .status()
        .context("Failed to run nasm. Is NASM installed?")?;

    if !nasm_status.success() {
        anyhow::bail!("NASM assembly failed");
    }

    // Link with ld
    let ld_status = Command::new("ld")
        .args([&obj_file, "-o", &exe_file])
        .status()
        .context("Failed to run ld. Is binutils installed?")?;

    if !ld_status.success() {
        anyhow::bail!("Linking failed");
    }

    println!("Assembled and linked: {}", exe_file);
    println!("Run with: ./{}", exe_file);

    Ok(())
}

