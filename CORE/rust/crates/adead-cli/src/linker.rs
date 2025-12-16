// Linker integration para ADead
// Soporta Zig y GCC/Clang para linkear .obj → .exe

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Tipo de linker disponible
#[derive(Debug, Clone, PartialEq)]
pub enum LinkerType {
    Zig,
    Gcc,
    Clang,
    None,
}

/// Detectar qué linker está disponible en el sistema
pub fn detect_linker() -> LinkerType {
    // Intentar Zig primero (recomendado)
    if Command::new("zig")
        .arg("version")
        .output()
        .is_ok()
    {
        return LinkerType::Zig;
    }
    
    // Intentar GCC
    if Command::new("g++")
        .arg("--version")
        .output()
        .is_ok()
    {
        return LinkerType::Gcc;
    }
    
    // Intentar Clang
    if Command::new("clang++")
        .arg("--version")
        .output()
        .is_ok()
    {
        return LinkerType::Clang;
    }
    
    LinkerType::None
}

/// Ensamblar archivo .asm a .obj usando NASM
pub fn assemble_asm_to_obj(asm_file: &Path, obj_file: &Path) -> Result<()> {
    // Verificar que NASM está disponible
    let nasm_output = Command::new("nasm")
        .arg("--version")
        .output()
        .context("NASM no encontrado. Por favor instala NASM y agrégalo al PATH.")?;
    
    if !nasm_output.status.success() {
        anyhow::bail!("NASM no está funcionando correctamente");
    }
    
    // Ensamblar .asm → .obj
    let output = Command::new("nasm")
        .arg("-f")
        .arg("win64")
        .arg(asm_file)
        .arg("-o")
        .arg(obj_file)
        .output()
        .with_context(|| format!("Error al ensamblar {} con NASM", asm_file.display()))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Error al ensamblar con NASM: {}", error_msg);
    }
    
    if !obj_file.exists() {
        anyhow::bail!("Archivo .obj no fue generado: {}", obj_file.display());
    }
    
    Ok(())
}

/// Linkear archivos .obj a .exe usando Zig
pub fn link_with_zig(obj_files: &[PathBuf], exe_file: &Path) -> Result<()> {
    // Verificar que Zig está disponible
    let zig_version = Command::new("zig")
        .arg("version")
        .output()
        .context("Zig no encontrado. Por favor instala Zig y agrégalo al PATH.")?;
    
    if !zig_version.status.success() {
        anyhow::bail!("Zig no está funcionando correctamente");
    }
    
    // Construir comando: zig build-exe -target x86_64-windows -lc -femit-bin=programa.exe obj1.obj obj2.obj ...
    // NOTA: -femit-bin requiere el signo = para el path: -femit-bin=path
    let mut cmd = Command::new("zig");
    cmd.arg("build-exe");
    
    // OPTIMIZACIÓN AGRESIVA: Flags para reducir tamaño del ejecutable al máximo
    // Opciones de Zig primero
    cmd.arg("-target")
        .arg("x86_64-windows-gnu")  // Usar gnu para mejor compatibilidad
        .arg("-O")
        .arg("ReleaseSmall")  // Optimización para tamaño mínimo
        .arg("-fstrip")  // Eliminar símbolos (sintaxis correcta de Zig)
        .arg("-fsingle-threaded")  // Sin threading overhead (sintaxis correcta)
        .arg("-fno-unwind-tables")  // Sin unwind tables (reduce tamaño)
        .arg("-lc");  // Linkear con C runtime
    
    // -femit-bin necesita el signo = para el path
    // Convertir a ruta absoluta para evitar problemas con espacios y rutas relativas
    let exe_file_abs = exe_file.canonicalize()
        .unwrap_or_else(|_| exe_file.to_path_buf());
    let emit_bin_arg = format!("-femit-bin={}", exe_file_abs.display());
    cmd.arg(&emit_bin_arg);
    
    // Agregar todos los archivos .obj después de las opciones
    for obj_file in obj_files {
        if !obj_file.exists() {
            anyhow::bail!("Archivo .obj no encontrado: {}", obj_file.display());
        }
        cmd.arg(obj_file);
    }
    
    // Ejecutar linking
    let output = cmd
        .output()
        .with_context(|| format!("Error al linkear con Zig"))?;
    
    // Si falla con -lc, intentar sin -lc
    if !output.status.success() {
        let mut cmd_retry = Command::new("zig");
        cmd_retry.arg("build-exe");
        
        // OPTIMIZACIÓN AGRESIVA: Flags para reducir tamaño (retry sin -lc)
        // Opciones de Zig primero
        cmd_retry.arg("-target")
            .arg("x86_64-windows-gnu")
            .arg("-O")
            .arg("ReleaseSmall")
            .arg("-fstrip")  // Eliminar símbolos
            .arg("-fsingle-threaded")  // Sin threading overhead
            .arg("-fno-unwind-tables");  // Sin unwind tables
        
        // -femit-bin necesita el signo = para el path
        // Convertir a ruta absoluta para evitar problemas con espacios y rutas relativas
        let exe_file_abs = exe_file.canonicalize()
            .unwrap_or_else(|_| exe_file.to_path_buf());
        let emit_bin_arg = format!("-femit-bin={}", exe_file_abs.display());
        cmd_retry.arg(&emit_bin_arg);
        
        // Agregar archivos .obj después
        for obj_file in obj_files {
            cmd_retry.arg(obj_file);
        }
        
        let output_retry = cmd_retry
            .output()
            .with_context(|| format!("Error al linkear con Zig (sin -lc)"))?;
        
        if !output_retry.status.success() {
            let error_msg = String::from_utf8_lossy(&output_retry.stderr);
            anyhow::bail!("Error al linkear con Zig: {}", error_msg);
        }
    }
    
    // Verificar que el archivo fue generado (puede estar en la ruta absoluta o relativa)
    if !exe_file.exists() {
        // Intentar con ruta absoluta
        let exe_file_abs = exe_file.canonicalize()
            .unwrap_or_else(|_| exe_file.to_path_buf());
        if !exe_file_abs.exists() {
            anyhow::bail!("Archivo .exe no fue generado en: {} ni en: {}", exe_file.display(), exe_file_abs.display());
        }
    }
    
    Ok(())
}

/// Linkear archivos .obj a .exe usando GCC
pub fn link_with_gcc(obj_files: &[PathBuf], exe_file: &Path) -> Result<()> {
    let mut cmd = Command::new("g++");
    
    // OPTIMIZACIÓN AGRESIVA: Flags para reducir tamaño del ejecutable al máximo
    // -nostdlib: No incluir stdlib de C (ya usamos solo kernel32)
    // -s: Strip symbols (equivalente a --strip-all)
    // -Wl,--strip-all: Eliminar todos los símbolos de debug
    // -Wl,--gc-sections: Eliminar secciones no usadas
    // -Wl,--file-alignment=16: Alineación mínima (reduce padding)
    // -Wl,--section-alignment=16: Alineación de secciones mínima
    // -Wl,--no-seh: Deshabilitar Structured Exception Handling (reduce overhead)
    cmd.arg("-nostdlib")
        .arg("-s")
        .arg("-Wl,--strip-all,--gc-sections,--file-alignment=16,--section-alignment=16,--no-seh");
    
    // Agregar todos los archivos .obj
    for obj_file in obj_files {
        if !obj_file.exists() {
            anyhow::bail!("Archivo .obj no encontrado: {}", obj_file.display());
        }
        cmd.arg(obj_file);
    }
    
    cmd.arg("-o").arg(exe_file);
    
    let output = cmd
        .output()
        .with_context(|| format!("Error al linkear con GCC"))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Error al linkear con GCC: {}", error_msg);
    }
    
    if !exe_file.exists() {
        anyhow::bail!("Archivo .exe no fue generado: {}", exe_file.display());
    }
    
    Ok(())
}

/// Linkear archivos .obj a .exe usando Clang
pub fn link_with_clang(obj_files: &[PathBuf], exe_file: &Path) -> Result<()> {
    let mut cmd = Command::new("clang++");
    
    // OPTIMIZACIÓN AGRESIVA: Flags para reducir tamaño del ejecutable al máximo
    // -nostdlib: No incluir stdlib de C (ya usamos solo kernel32)
    // -s: Strip symbols (equivalente a --strip-all)
    // -Wl,--strip-all: Eliminar todos los símbolos de debug
    // -Wl,--gc-sections: Eliminar secciones no usadas
    // -Wl,--file-alignment=16: Alineación mínima (reduce padding)
    // -Wl,--section-alignment=16: Alineación de secciones mínima
    // -Wl,--no-seh: Deshabilitar Structured Exception Handling (reduce overhead)
    cmd.arg("-nostdlib")
        .arg("-s")
        .arg("-Wl,--strip-all,--gc-sections,--file-alignment=16,--section-alignment=16,--no-seh");
    
    // Agregar todos los archivos .obj
    for obj_file in obj_files {
        if !obj_file.exists() {
            anyhow::bail!("Archivo .obj no encontrado: {}", obj_file.display());
        }
        cmd.arg(obj_file);
    }
    
    cmd.arg("-o").arg(exe_file);
    
    let output = cmd
        .output()
        .with_context(|| format!("Error al linkear con Clang"))?;
    
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Error al linkear con Clang: {}", error_msg);
    }
    
    if !exe_file.exists() {
        anyhow::bail!("Archivo .exe no fue generado: {}", exe_file.display());
    }
    
    Ok(())
}

/// Linkear archivos .obj a .exe usando el linker disponible
pub fn link_objs_to_exe(obj_files: &[PathBuf], exe_file: &Path, preferred_linker: Option<LinkerType>) -> Result<()> {
    let linker = preferred_linker.unwrap_or_else(detect_linker);
    
    match linker {
        LinkerType::Zig => {
            println!("   🔗 Linkeando con Zig...");
            link_with_zig(obj_files, exe_file)
        }
        LinkerType::Gcc => {
            println!("   🔗 Linkeando con GCC...");
            link_with_gcc(obj_files, exe_file)
        }
        LinkerType::Clang => {
            println!("   🔗 Linkeando con Clang...");
            link_with_clang(obj_files, exe_file)
        }
        LinkerType::None => {
            anyhow::bail!("No se encontró ningún linker disponible (Zig, GCC o Clang). Por favor instala uno de ellos.");
        }
    }
}

/// Compilar y linkear completo: .ad → .asm → .obj → .exe
pub fn compile_and_link(
    source_file: &Path,
    output_exe: Option<PathBuf>,
    backend: &str,
    linker_preference: Option<LinkerType>,
) -> Result<PathBuf> {
    use std::fs;
    
    // Paso 1: Compilar .ad → .asm
    let asm_file = source_file.with_extension("asm");
    let source = fs::read_to_string(source_file)
        .with_context(|| format!("Error al leer archivo: {}", source_file.display()))?;
    
    println!("   📝 Compilando {} → {}", source_file.display(), asm_file.display());
    
    match backend {
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
            match adead_parser::pipeline_selector::process_adead_intelligent(&source) {
                Ok((_pipeline, asm_code)) => {
                    fs::write(&asm_file, asm_code)
                        .with_context(|| format!("Error al escribir {}", asm_file.display()))?;
                }
                Err(e) => {
                    anyhow::bail!("Pipeline error: {}", e);
                }
            }
        }
    }
    
    // Paso 2: Ensamblar .asm → .obj
    let obj_file = source_file.with_extension("obj");
    println!("   🔧 Ensamblando {} → {}", asm_file.display(), obj_file.display());
    assemble_asm_to_obj(&asm_file, &obj_file)?;
    
    // Paso 3: Linkear .obj → .exe
    let exe_file = output_exe.unwrap_or_else(|| source_file.with_extension("exe"));
    println!("   🔗 Linkeando {} → {}", obj_file.display(), exe_file.display());
    link_objs_to_exe(&[obj_file], &exe_file, linker_preference)?;
    
    println!("   ✅ Ejecutable generado: {}", exe_file.display());
    
    Ok(exe_file)
}

/// Helper para detectar strings avanzados
fn has_advanced_strings(source: &str) -> bool {
    source.contains("\"") ||
    (source.contains("+") && (source.contains("let") || source.contains("s"))) ||
    (source.contains("[") && source.contains(":")) ||
    source.contains(".upper()") || source.contains(".lower()") ||
    source.contains("len(")
}

