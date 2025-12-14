use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::process::Command;
use std::path::{Path, PathBuf};
use std::env;

#[derive(Parser)]
#[command(name = "adeadc")]
#[command(about = "ADead compiler: Python-like syntax to NASM", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Paso 1: Compilar .ad -> .asm (Rust parser genera NASM)
    Compile {
        /// Archivo de entrada (.ad)
        #[arg(value_name = "INPUT")]
        input: String,

        /// Archivo de salida (.asm) [opcional: se infiere del nombre]
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<String>,
    },
    /// Paso 2: Ensamblar .asm -> .obj (NASM genera objeto)
    Assemble {
        /// Archivo de entrada (.asm)
        #[arg(value_name = "INPUT")]
        input: String,

        /// Archivo de salida (.obj/.o) [opcional: se infiere del nombre]
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<String>,
    },
    /// Paso 3: Enlazar .obj -> .exe (Linker genera ejecutable)
    Link {
        /// Archivo de entrada (.obj/.o)
        #[arg(value_name = "INPUT")]
        input: String,

        /// Archivo de salida (.exe) [opcional: se infiere del nombre]
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<String>,
    },
    /// Todo el proceso: .ad -> .asm -> .obj -> .exe (y ejecutar)
    Run {
        /// Archivo de entrada (.ad)
        #[arg(value_name = "INPUT")]
        input: String,

        /// Mantener archivos temporales (.asm, .obj)
        #[arg(long)]
        keep_temp: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile { input, output } => {
            println!("🔨 Paso 1: Compilando .ad -> .asm");
            println!("   Entrada: {}", input);
            
            let mut source = fs::read_to_string(&input)
                .with_context(|| format!("Failed to read input file: {}", input))?;
            
            // Remover BOM (Byte Order Mark) si existe
            if source.starts_with('\u{feff}') {
                source = source.trim_start_matches('\u{feff}').to_string();
            }

            let output_path = output.unwrap_or_else(|| {
                input
                    .replace(".ad", ".asm")
                    .replace(".adead", ".asm")
                    .to_string()
            });

            // FLUJO DIRECTO: ADead → Zig → NASM (para floats simples)
            // Detectar si es un programa simple con float: "print 3.14"
            // NOTA: Solo para literales simples, NO para expresiones complejas
            let trimmed_source = source.trim();
            
            if trimmed_source.starts_with("print ") {
                let expr_part = trimmed_source.strip_prefix("print ").unwrap_or("").trim();
                
                // PASO 3: Verificación SIMPLE - solo detectar '+'
                // Si tiene '+', usar Rust directamente (sin verificar nada más)
                if expr_part.contains('+') {
                    println!("   🔒 Expresión compleja detectada (+), usando flujo Rust");
                    // Continuar con el flujo Rust de abajo (no return)
                } else {
                    // Literal simple sin '+' - intentar flujo directo Zig
                    if adead_parser::zig_nasm_generator::can_use_direct_flow(expr_part) {
                        println!("   🚀 Usando flujo directo: Zig → NASM");
                        if let Some(nasm_code) = adead_parser::zig_nasm_generator::generate_nasm_direct(expr_part) {
                            fs::write(&output_path, nasm_code)
                                .with_context(|| format!("Failed to write output file: {}", output_path))?;
                            println!("✅ Compilado (Zig directo): {} -> {}", input, output_path);
                            return Ok(());
                        }
                    }
                }
            }

            // FLUJO HÍBRIDO: Intentar Rust primero, si falla usar Zig directo
            println!("   🔒 Intentando flujo con validación: Zig → Rust → NASM");
            let input_path = Path::new(&input);
            let current_dir = input_path.parent();
            
            // Intentar parsear y generar con Rust primero
            let rust_success = match adead_parser::parse_with_dir(&source, current_dir) {
                Ok(program) => {
                    // Debug: verificar statements parseados
                    println!("   📊 Statements parseados: {}", program.statements.len());
                    for (i, stmt) in program.statements.iter().enumerate() {
                        println!("      [{}] {:?}", i, std::mem::discriminant(stmt));
                    }
                    // Rust parser exitoso, intentar generar código con Rust
                    let mut generator = adead_backend::CodeGenerator::new();
                    match generator.generate(&program) {
                        Ok(asm) => {
                            // Rust exitoso: escribir y retornar
                            if let Err(e) = fs::write(&output_path, asm) {
                                println!("   ⚠️ Error escribiendo archivo: {}", e);
                                false
                            } else {
                                println!("✅ Compilado (Rust): {} -> {}", input, output_path);
                                true
                            }
                        }
                        Err(e) => {
                            // Rust codegen falló, intentar Zig
                            println!("   ⚠️ Rust codegen falló: {}", e);
                            false
                        }
                    }
                }
                Err(e) => {
                    // Rust parser falló, intentar Zig
                    println!("   ⚠️ Rust parser falló: {}", e);
                    false
                }
            };
            
            // Si Rust falló, intentar con Zig directo (para statements como while)
            if !rust_success {
                println!("   🚀 Intentando flujo directo: Zig → NASM");
                match adead_parser::zig_nasm_generator::generate_nasm_direct(&source) {
                    Some(nasm_code) => {
                        fs::write(&output_path, nasm_code)
                            .with_context(|| format!("Failed to write output file: {}", output_path))?;
                        println!("✅ Compilado (Zig directo): {} -> {}", input, output_path);
                    }
                    None => {
                        return Err(anyhow::anyhow!("Falló tanto Rust como Zig. Verifica que el código sea válido."));
                    }
                }
            }
        }
        Commands::Assemble { input, output } => {
            println!("⚙️  Paso 2: Ensamblando .asm -> .obj");
            println!("   Entrada: {}", input);
            
            let output_path = output.unwrap_or_else(|| {
                if cfg!(target_os = "windows") {
                    input.replace(".asm", ".obj")
                } else {
                    input.replace(".asm", ".o")
                }
            });
            
            assemble_asm_to_obj(&input, &output_path)?;
            println!("✅ Ensamblado: {} -> {}", input, output_path);
        }
        Commands::Link { input, output } => {
            println!("🔗 Paso 3: Enlazando .obj -> .exe");
            println!("   Entrada: {}", input);
            
            let output_path = output.unwrap_or_else(|| {
                if cfg!(target_os = "windows") {
                    input.replace(".obj", ".exe").replace(".o", ".exe")
                } else {
                    input.replace(".obj", "").replace(".o", "")
                }
            });
            
            link_obj_to_exe(&input, &output_path)?;
            println!("✅ Enlazado: {} -> {}", input, output_path);
        }
        Commands::Run { input, keep_temp } => {
            println!("🚀 Ejecutando proceso completo:");
            println!("   .ad -> .asm -> .obj -> .exe\n");
            run_program(&input, keep_temp)?;
        }
    }

    Ok(())
}

/// Busca una herramienta en el PATH y ubicaciones comunes
fn find_tool(tool_name: &str, windows_exts: &[&str]) -> Option<PathBuf> {
    // En Windows, priorizar rutas específicas de MSYS2/MingW64 primero
    // porque estas son las más confiables para linking
    if cfg!(target_os = "windows") {
        let msys_paths = vec![
            PathBuf::from(r"C:\msys64\mingw64\bin"),
            PathBuf::from(r"C:\mingw64\bin"),
            PathBuf::from(r"C:\msys64\usr\bin"),
            PathBuf::from(r"C:\msys32\mingw32\bin"),
            PathBuf::from(r"C:\Program Files\mingw-w64\x86_64-8.1.0-posix-seh-rt_v6-rev0\mingw64\bin"),
            PathBuf::from(r"C:\Program Files (x86)\mingw-w64\x86_64-8.1.0-posix-seh-rt_v6-rev0\mingw64\bin"),
        ];

        // PRIMERO buscar en rutas de MSYS2/MingW (más confiables)
        for dir in &msys_paths {
            for ext in windows_exts {
                let full_path = if !ext.is_empty() {
                    dir.join(format!("{}{}", tool_name, ext))
                } else {
                    dir.join(tool_name)
                };
                if full_path.exists() {
                    return Some(full_path);
                }
            }
        }

        // También buscar en ubicaciones estándar de Program Files
        let program_paths = vec![
            PathBuf::from(r"C:\Program Files\mingw-w64\bin"),
            PathBuf::from(r"C:\Program Files (x86)\mingw-w64\bin"),
        ];

        for dir in &program_paths {
            for ext in windows_exts {
                let full_path = if !ext.is_empty() {
                    dir.join(format!("{}{}", tool_name, ext))
                } else {
                    dir.join(tool_name)
                };
                if full_path.exists() {
                    return Some(full_path);
                }
            }
        }
    }

    // Luego buscar en el PATH del sistema
    if let Ok(path) = env::var("PATH") {
        for dir in env::split_paths(&path) {
            for ext in windows_exts {
                let full_path = if cfg!(target_os = "windows") && !ext.is_empty() {
                    dir.join(format!("{}{}", tool_name, ext))
                } else {
                    dir.join(tool_name)
                };
                if full_path.exists() {
                    return Some(full_path);
                }
            }
        }
    }

    None
}

/// Obtiene la ruta de NASM o usa el nombre si está en PATH
fn get_nasm_path() -> String {
    if let Some(path) = find_tool("nasm", &[".exe", ""]) {
        path.to_string_lossy().to_string()
    } else {
        "nasm".to_string()
    }
}

/// Obtiene la ruta de un linker disponible
fn find_linker() -> Option<(String, Vec<String>)> {
    if cfg!(target_os = "windows") {
        // En Windows, buscar gcc de MSYS2/MingW64 primero (más confiable)
        
        // Método 1: gcc de MSYS2/MingW64 (PREFERIDO)
        // Buscar primero en ubicaciones específicas de MSYS2
        let msys_gcc_paths = vec![
            r"C:\msys64\mingw64\bin\gcc.exe",
            r"C:\mingw64\bin\gcc.exe",
        ];
        
        for gcc_path_str in msys_gcc_paths {
            let gcc_path = PathBuf::from(gcc_path_str);
            if gcc_path.exists() {
                // Verificar que realmente funciona
                if Command::new(&gcc_path).arg("--version").output().is_ok() {
                    // IMPORTANTE: Separar -Wl, flags correctamente
                    // gcc necesita -Wl,flag como un solo argumento, NO como dos
                    let args = vec![
                        "-nostdlib".to_string(),
                        "-lkernel32".to_string(),
                        "-Wl,--entry=main".to_string(),
                        "-Wl,--subsystem=console".to_string(),
                    ];
                    return Some((gcc_path.to_string_lossy().to_string(), args));
                }
            }
        }
        
        // Método 2: Buscar gcc en PATH usando find_tool (fallback)
        if let Some(gcc_path) = find_tool("gcc", &[".exe", ""]) {
            // Verificar que funciona antes de usarlo
            if Command::new(&gcc_path).arg("--version").output().is_ok() {
                let args = vec![
                    "-nostdlib".to_string(),
                    "-lkernel32".to_string(),
                    "-Wl,--entry=main".to_string(),
                    "-Wl,--subsystem=console".to_string(),
                ];
                return Some((gcc_path.to_string_lossy().to_string(), args));
            }
        }
        
        // Método 2: ld directamente (alternativa)
        if let Some(ld_path) = find_tool("ld", &[".exe", ""]) {
            // Buscar libkernel32.a
            let kernel32_paths = vec![
                r"C:\msys64\mingw64\lib\libkernel32.a",
                r"C:\mingw64\lib\libkernel32.a",
                r"C:\msys64\mingw64\x86_64-w64-mingw32\lib\libkernel32.a",
            ];
            
            let mut args = vec![
                "--entry=main".to_string(),
                "--subsystem=console".to_string(),
            ];
            
            for path in kernel32_paths {
                if Path::new(path).exists() {
                    args.push(path.to_string());
                    break;
                }
            }
            
            // Si no encontramos kernel32.a, usar -lkernel32 y dejar que ld lo busque
            if !args.iter().any(|a| a.contains("kernel32")) {
                args.push("-lkernel32".to_string());
            }
            
            return Some((ld_path.to_string_lossy().to_string(), args));
        }
    } else {
        // Linux: buscar gcc primero
        if let Some(gcc_path) = find_tool("gcc", &[".exe", ""]) {
            let args = vec!["-nostdlib".to_string(), "-Wl,--entry=_start".to_string()];
            return Some((gcc_path.to_string_lossy().to_string(), args));
        }
        
        // Linux: buscar ld
        if let Some(ld_path) = find_tool("ld", &[".exe", ""]) {
            return Some((ld_path.to_string_lossy().to_string(), vec![]));
        }
    }

    None
}

/// Ensambla un archivo .asm a .obj/.o
fn assemble_asm_to_obj(asm_file: &str, obj_file: &str) -> Result<()> {
    use std::path::Path;
    
    let nasm_format = if cfg!(target_os = "windows") {
        "-fwin64"
    } else {
        "-felf64"
    };

    let nasm_path = get_nasm_path();
    println!("   Usando NASM: {}", nasm_path);

    // Convertir rutas a absolutas para evitar problemas con working directory
    let asm_path = Path::new(asm_file);
    let obj_path = Path::new(obj_file);
    
    // Si la ruta no es absoluta, convertirla
    let asm_abs = if asm_path.is_absolute() {
        asm_path.to_path_buf()
    } else {
        std::env::current_dir()?
            .join(asm_path)
            .canonicalize()
            .with_context(|| format!("No se pudo encontrar el archivo ASM: {}", asm_file))?
    };
    
    let obj_abs = if obj_path.is_absolute() {
        obj_path.to_path_buf()
    } else {
        std::env::current_dir()?
            .join(obj_path)
    };
    
    // Asegurar que el directorio de salida existe
    if let Some(obj_dir) = obj_abs.parent() {
        fs::create_dir_all(obj_dir)?;
    }

    let asm_str = asm_abs.to_string_lossy().to_string();
    let obj_str = obj_abs.to_string_lossy().to_string();

    let nasm_status = Command::new(&nasm_path)
        .args([nasm_format, &asm_str, "-o", &obj_str])
        .status()
        .with_context(|| format!("Failed to run nasm at {}. Is NASM installed?", nasm_path))?;

    if !nasm_status.success() {
        // Capturar stderr para mostrar errores
        let output = Command::new(&nasm_path)
            .args([nasm_format, &asm_str, "-o", &obj_str])
            .output()
            .unwrap_or_else(|_| std::process::Output {
                status: nasm_status,
                stdout: vec![],
                stderr: vec![],
            });
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("❌ Error de NASM:");
        if !stderr.is_empty() {
            eprintln!("{}", stderr);
        }
        anyhow::bail!("NASM assembly failed");
    }

    Ok(())
}

/// Enlaza un archivo .obj/.o a .exe
fn link_obj_to_exe(obj_file: &str, exe_file: &str) -> Result<()> {
    use std::path::Path;
    
    // Convertir rutas a absolutas para evitar problemas con working directory
    let obj_path = Path::new(obj_file);
    let exe_path = Path::new(exe_file);
    
    // Si la ruta no es absoluta, convertirla
    let obj_abs_raw = if obj_path.is_absolute() {
        obj_path.to_path_buf()
    } else {
        std::env::current_dir()?
            .join(obj_path)
            .canonicalize()
            .with_context(|| format!("No se pudo encontrar el archivo objeto: {}", obj_file))?
    };
    
    // IMPORTANTE: Remover el prefijo \\?\ de Windows (gcc no lo entiende)
    let obj_abs = if obj_abs_raw.to_string_lossy().starts_with(r"\\?\") {
        PathBuf::from(obj_abs_raw.to_string_lossy().strip_prefix(r"\\?\").unwrap())
    } else {
        obj_abs_raw
    };
    
    let exe_abs_raw = if exe_path.is_absolute() {
        // Si es absoluta, remover \\?\ también
        let path_buf = PathBuf::from(exe_path);
        if path_buf.to_string_lossy().starts_with(r"\\?\") {
            PathBuf::from(path_buf.to_string_lossy().strip_prefix(r"\\?\").unwrap())
        } else {
            path_buf
        }
    } else {
        // Para el ejecutable, construir desde el directorio del objeto
        obj_abs.parent()
            .unwrap_or(Path::new("."))
            .join(exe_path.file_name().unwrap_or(exe_path.as_os_str()))
    };
    
    let exe_abs = exe_abs_raw;
    
    // Asegurar que el directorio de salida existe
    if let Some(exe_dir) = exe_abs.parent() {
        fs::create_dir_all(exe_dir)?;
    }
    
    let link_result = if let Some((linker_path, linker_args)) = find_linker() {
        let linker_path_buf = PathBuf::from(&linker_path);
        if !linker_path_buf.exists() {
            return Err(anyhow::anyhow!("Linker no encontrado en: {}", linker_path));
        }
        
        println!("   Usando linker: {}", linker_path);
        let mut cmd = Command::new(&linker_path);
        
        // CRÍTICO: gcc necesita encontrar DLLs (libgcc_s_seh-1.dll, etc.)
        // Asegurar que el directorio de MSYS2/bin esté en el PATH
        let gcc_dir = PathBuf::from(&linker_path).parent()
            .unwrap_or(Path::new("."))
            .to_path_buf();
        let gcc_dir_str = gcc_dir.to_string_lossy().to_string();
        
        // Obtener PATH actual y agregar directorio de gcc si no está
        let current_path = std::env::var("PATH").unwrap_or_default();
        let new_path = if current_path.contains(&gcc_dir_str) {
            current_path
        } else {
            format!("{};{}", gcc_dir_str, current_path)
        };
        cmd.env("PATH", &new_path);
        
        // También preservar otras variables importantes de MSYS2
        for (key, value) in std::env::vars() {
            if key.starts_with("MSYSTEM") || key.starts_with("MINGW") {
                cmd.env(&key, &value);
            }
        }
        
        // Configurar working directory al directorio del objeto
        // Asegurar que el working directory no tenga \\?\ (gcc no lo maneja bien)
        let obj_dir_raw = obj_abs.parent().unwrap_or(Path::new("."));
        let obj_dir_str = obj_dir_raw.to_string_lossy();
        let obj_dir = if obj_dir_str.starts_with(r"\\?\") {
            Path::new(&obj_dir_str[4..]) // Remover "\\?\"
        } else {
            obj_dir_raw
        };
        cmd.current_dir(obj_dir);
        
        let is_ld = linker_path.to_lowercase().contains("ld") && !linker_path.to_lowercase().contains("gcc");
        
        if cfg!(target_os = "windows") {
            if is_ld {
                for arg in &linker_args {
                    cmd.arg(arg);
                }
                // Usar rutas absolutas para ld
                cmd.arg("-o").arg(&exe_abs);
                cmd.arg(&obj_abs);
            } else {
                // gcc: usar nombres de archivo relativos al working directory (más confiable)
                let obj_name = obj_abs.file_name().unwrap().to_string_lossy().to_string();
                let exe_name = exe_abs.file_name().unwrap().to_string_lossy().to_string();
                
                // IMPORTANTE: Construir argumentos en el orden correcto
                // Primero el archivo objeto, luego -o, luego el ejecutable, luego las opciones
                cmd.arg(&obj_name)
                   .arg("-o")
                   .arg(&exe_name);
                for arg in &linker_args {
                    cmd.arg(arg);
                }
                
                // DEBUG: Mostrar comando exacto que se va a ejecutar
                eprintln!("\n🔍 DEBUG: Comando exacto a ejecutar:");
                eprintln!("   Working dir: {:?}", obj_dir);
                eprintln!("   Programa: {}", linker_path);
                eprintln!("   Args: {} -o {} {}", obj_name, exe_name, linker_args.join(" "));
            }
        } else {
            if is_ld {
                for arg in &linker_args {
                    cmd.arg(arg);
                }
                cmd.arg("-o").arg(&exe_abs);
                cmd.arg(&obj_abs);
            } else {
                cmd.arg(&obj_abs).arg("-o").arg(&exe_abs);
                for arg in &linker_args {
                    cmd.arg(arg);
                }
            }
        }
        
        // IMPORTANTE: Capturar output para poder mostrar stderr/stdout
        // Esto nos permite diagnosticar problemas
        cmd.stderr(std::process::Stdio::piped())
           .stdout(std::process::Stdio::piped())
           .output()
    } else {
        anyhow::bail!("No se encontró un linker. Instala MinGW/MSYS2 o binutils.");
    };

    match link_result {
        Ok(output) => {
            // SIEMPRE mostrar stderr/stdout para diagnóstico
            let stderr_str = String::from_utf8_lossy(&output.stderr);
            let stdout_str = String::from_utf8_lossy(&output.stdout);
            
            if !output.status.success() {
                let exit_code = output.status.code().unwrap_or(-1);
                
                eprintln!("\n❌ Error en linking (exit code: {})", exit_code);
                
                // SIEMPRE mostrar stderr y stdout (incluso si están vacíos para diagnóstico)
                eprintln!("\n📋 STDERR (errores de gcc):");
                if stderr_str.trim().is_empty() {
                    eprintln!("(vacío - no hay errores en stderr)");
                } else {
                    eprintln!("{}", stderr_str);
                }
                eprintln!("\n📋 STDOUT (salida de gcc):");
                if stdout_str.trim().is_empty() {
                    eprintln!("(vacío - no hay salida estándar)");
                } else {
                    eprintln!("{}", stdout_str);
                }
                
                // Mostrar información de diagnóstico
                let linker_info = find_linker();
                let linker_path_display = linker_info.as_ref()
                    .map(|(p, _)| p.as_str())
                    .unwrap_or("No encontrado");
                let linker_args_display = linker_info.as_ref()
                    .map(|(_, args)| args.join(" "))
                    .unwrap_or_else(|| "N/A".to_string());
                
                eprintln!("\n📋 Información del comando:");
                eprintln!("   Linker: {}", linker_path_display);
                eprintln!("   Objeto: {} ({})", obj_abs.display(), 
                    if obj_abs.exists() { 
                        format!("existe, {} bytes", std::fs::metadata(&obj_abs).map(|m| m.len()).unwrap_or(0))
                    } else { 
                        "NO existe".to_string() 
                    });
                eprintln!("   Ejecutable esperado: {}", exe_abs.display());
                let obj_dir = obj_abs.parent().unwrap_or(Path::new("."));
                eprintln!("   Working directory: {}", obj_dir.display());
                
                // Mostrar el comando exacto que se intentó ejecutar
                let obj_name = obj_abs.file_name().unwrap().to_string_lossy();
                let exe_name = exe_abs.file_name().unwrap().to_string_lossy();
                eprintln!("\n💡 Comando que se intentó ejecutar:");
                eprintln!("   cd {}", obj_dir.display());
                eprintln!("   {} {} -o {} {}", linker_path_display, obj_name, exe_name, linker_args_display);
                
                // Intentar diagnóstico adicional: verificar si el objeto es válido
                if obj_abs.exists() {
                    if let Ok(metadata) = std::fs::metadata(&obj_abs) {
                        if metadata.len() == 0 {
                            eprintln!("\n⚠️  ADVERTENCIA: El archivo objeto está vacío (0 bytes)");
                            eprintln!("   Esto puede indicar un error en el paso de ensamblado (assemble)");
                        }
                    }
                }
                
                anyhow::bail!("Linking failed with exit code: {}", exit_code);
            } else {
                // Éxito: también mostrar stderr si hay advertencias (pero no fatal)
                if !stderr_str.trim().is_empty() {
                    eprintln!("\n⚠️  Advertencias del linker:");
                    eprintln!("{}", stderr_str);
                }
            }
        }
        Err(e) => {
            eprintln!("\n❌ Error ejecutando linker: {}", e);
            anyhow::bail!("Error ejecutando linker: {}", e);
        }
    }

    // Verificar que el ejecutable se creó (usar ruta absoluta)
    if !exe_abs.exists() {
        // Esperar un poco por si el sistema tarda
        use std::thread;
        use std::time::Duration;
        thread::sleep(Duration::from_millis(500));
        
        if !exe_abs.exists() {
            anyhow::bail!(
                "El ejecutable no se creó: {}\n\
                 Se esperaba en: {}",
                exe_file,
                exe_abs.display()
            );
        }
    }

    Ok(())
}

/// Función legacy: mantiene compatibilidad (assemble + link)
fn assemble_and_link(asm_file: &str) -> Result<()> {
    let (obj_file, exe_file) = if cfg!(target_os = "windows") {
        let obj = asm_file.replace(".asm", ".obj");
        let exe = asm_file.replace(".asm", ".exe");
        (obj, exe)
    } else {
        let obj = asm_file.replace(".asm", ".o");
        let exe = asm_file.replace(".asm", "");
        (obj, exe)
    };

    // Assemble with NASM - buscar la ruta correcta
    let nasm_format = if cfg!(target_os = "windows") {
        "-fwin64"
    } else {
        "-felf64"
    };

    let nasm_path = get_nasm_path();
    println!("🔍 Usando NASM: {}", nasm_path);

    let nasm_status = Command::new(&nasm_path)
        .args([nasm_format, asm_file, "-o", &obj_file])
        .status()
        .with_context(|| format!("Failed to run nasm at {}. Is NASM installed?", nasm_path))?;

    if !nasm_status.success() {
        anyhow::bail!("NASM assembly failed");
    }

    // Try different linkers - buscar automáticamente
    let link_success = if let Some((linker_path, linker_args)) = find_linker() {
        let mut cmd = Command::new(&linker_path);
        cmd.arg(&obj_file).arg("-o").arg(&exe_file);
        for arg in &linker_args {
            cmd.arg(arg);
        }
        
        match cmd.status() {
            Ok(status) => status.success(),
            Err(_) => false,
        }
    } else {
        // Fallback: intentar con nombres simples
        if cfg!(target_os = "windows") {
            let gcc_result = Command::new("gcc")
                .args([&obj_file, "-o", &exe_file, "-nostdlib", "-Wl,--entry=_start"])
                .status();
            
            if let Ok(status) = gcc_result {
                status.success()
            } else {
                Command::new("ld")
                    .args([&obj_file, "-o", &exe_file])
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
            }
        } else {
            Command::new("ld")
                .args([&obj_file, "-o", &exe_file])
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
    };

    if !link_success {
        if cfg!(target_os = "windows") {
            // Check what's available
            let has_gcc = Command::new("gcc").arg("--version").status().is_ok();
            let has_ld = Command::new("ld").arg("--version").status().is_ok();
            
            let mut msg = format!(
                "❌ Linking failed. NASM compiló correctamente a {}, pero no se encontró un linker.\n\n",
                obj_file
            );
            
            if !has_gcc && !has_ld {
                msg.push_str(&format!(
                    "📦 Opciones para instalar un linker:\n\
                     1. MinGW/MSYS2 (recomendado): https://www.msys2.org/\n\
                        - Instala MSYS2, luego ejecuta: pacman -S mingw-w64-x86_64-gcc\n\
                     2. WinLibs (portable): https://winlibs.com/\n\
                     3. O instala WSL para un entorno Linux completo\n\n"
                ));
            }
            
            msg.push_str(&format!(
                "💡 Alternativa: Puedes enlazar manualmente con:\n\
                 gcc {} -o {} -nostdlib -Wl,--entry=_start\n\n\
                 ⚠️  Nota: El código generado usa syscalls de Linux.\n\
                 Para ejecutar necesitas WSL o un entorno compatible con Linux.",
                obj_file, exe_file
            ));
            
            anyhow::bail!("{}", msg);
        } else {
            anyhow::bail!("Linking failed. Make sure binutils (ld) is installed: sudo apt-get install binutils");
        }
    }

    println!("Assembled and linked: {}", exe_file);
    let run_cmd = if cfg!(target_os = "windows") {
        exe_file.clone()
    } else {
        format!("./{}", exe_file)
    };
    println!("Run with: {}", run_cmd);

    Ok(())
}

fn run_program(input_file: &str, keep_temp: bool) -> Result<()> {
    use std::path::{Path, PathBuf};
    use std::fs;

    // Obtener el directorio y nombre base del archivo de entrada
    let input_path = Path::new(input_file);
    let base_name = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("adead_output");
    
    // Si el archivo está en una carpeta "ejemplos", guardar en "compilados" del mismo nivel
    let output_dir: PathBuf = if let Some(parent) = input_path.parent() {
        if let Some(parent_name) = parent.file_name().and_then(|s| s.to_str()) {
            if parent_name == "ejemplos" {
                // Si está en "ejemplos", guardar en "compilados" del mismo nivel padre
                if let Some(grandparent) = parent.parent() {
                    grandparent.join("compilados")
                } else {
                    parent.to_path_buf()
                }
            } else {
                // Si no está en "ejemplos", guardar en el mismo directorio
                parent.to_path_buf()
            }
        } else {
            parent.to_path_buf()
        }
    } else {
        PathBuf::from(".")
    };
    
    // Asegurar que el directorio de salida existe
    if let Err(e) = fs::create_dir_all(&output_dir) {
        eprintln!("⚠️  Warning: No se pudo crear el directorio de salida: {}", e);
    }
    
    let asm_file = output_dir.join(format!("{}.asm", base_name));
    // Usar formato correcto según el sistema operativo
    let obj_file = output_dir.join(if cfg!(target_os = "windows") {
        format!("{}.obj", base_name)  // COFF para Windows
    } else {
        format!("{}.o", base_name)    // ELF para Linux
    });
    let exe_file = output_dir.join(if cfg!(target_os = "windows") {
        format!("{}.exe", base_name)
    } else {
        base_name.to_string()
    });
    
    // Convertir a strings para usar en comandos
    let asm_file_str = asm_file.to_string_lossy().to_string();
    let obj_file_str = obj_file.to_string_lossy().to_string();
    let exe_file_str = exe_file.to_string_lossy().to_string();

    // Step 1: Compile to ASM
    println!("🔨 Compiling {}...", input_file);
    let mut source = fs::read_to_string(input_file)
        .with_context(|| format!("Failed to read input file: {}", input_file))?;
    
    // Remover BOM (Byte Order Mark) si existe
    if source.starts_with('\u{feff}') {
        source = source.trim_start_matches('\u{feff}').to_string();
    }

    // Pasar directorio del archivo de entrada para resolución de imports (Sprint 1.3)
    let input_path = Path::new(input_file);
    let current_dir = input_path.parent();

    let program = adead_parser::parse_with_dir(&source, current_dir)
        .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

    let mut generator = adead_backend::CodeGenerator::new();
    let asm = generator
        .generate(&program)
        .map_err(|e| anyhow::anyhow!("Code generation error: {}", e))?;

    fs::write(&asm_file, asm)
        .with_context(|| format!("Failed to write ASM file: {}", asm_file_str))?;
    
    println!("✅ ASM generado: {}", asm_file_str);

    // Step 2: Assemble
    println!("⚙️  Paso 2: Ensamblando .asm -> .obj");
    assemble_asm_to_obj(&asm_file_str, &obj_file_str)?;
    println!("✅ Objeto generado: {}", obj_file_str);

    // Step 3: Link - buscar linker automáticamente
    println!("🔗 Paso 3: Enlazando .obj -> .exe");
    link_obj_to_exe(&obj_file_str, &exe_file_str)?;
    
    // Verificar tamaño del ejecutable
    if let Ok(metadata) = fs::metadata(&exe_file) {
        println!("✅ Ejecutable generado: {} ({} bytes)", exe_file_str, metadata.len());
    } else {
        println!("✅ Ejecutable generado: {}", exe_file_str);
    }

    // Step 4: Execute
    if !exe_file.exists() {
        anyhow::bail!(
            "❌ Error: El ejecutable no se generó correctamente.\n\
             Se esperaba en: {}\n\
             Verifica los errores del linking arriba.",
            exe_file.display()
        );
    }
    
    let final_exe = exe_file.clone();
    
    let exe_size = fs::metadata(&final_exe)
        .map(|m| m.len())
        .unwrap_or(0);
    
    if exe_size == 0 {
        anyhow::bail!(
            "❌ Error: El ejecutable existe pero está vacío (0 bytes).\n\
             Algo salió mal en el linking."
        );
    }
    
    println!("✅ Ejecutable generado: {} ({} bytes)", final_exe.display(), exe_size);
    println!("🚀 Ejecutando...\n");
    println!("{}", "─".repeat(50));
    
    // Ejecutar de forma robusta, usando ruta absoluta y manejando espacios
    let exe_path = fs::canonicalize(&final_exe)
        .unwrap_or_else(|_| final_exe.clone());
    
    // En Windows, asegurar que usamos la ruta correcta y el working directory
    let exec_status = Command::new(&exe_path)
        .current_dir(&output_dir)
        .status()
        .with_context(|| format!(
            "Failed to execute program: {}\n\
            Working directory: {}\n\
            Verifica que el archivo existe y tiene permisos de ejecución.",
            exe_path.display(),
            output_dir.display()
        ))?;

    println!("{}", "─".repeat(50));
    
    // Cleanup: Solo eliminar archivos temporales, NUNCA el .exe (producto final)
    if !keep_temp {
        // Solo eliminar .asm y .obj (archivos temporales)
        let _ = fs::remove_file(&asm_file);
        let _ = fs::remove_file(&obj_file);
        // NO eliminar exe_file - es el producto final que el usuario necesita
    } else {
        println!("\n📁 Archivos generados (conservados):");
        if asm_file.exists() {
            println!("  - {}", asm_file.display());
        }
        if obj_file.exists() {
            println!("  - {}", obj_file.display());
        }
    }
    
    // Asegurar que el ejecutable está en la ubicación esperada (relativa)
    if !exe_file.exists() && final_exe.exists() {
        let _ = fs::copy(&final_exe, &exe_file).ok();
    }
    
    // Usar exe_file como el archivo final para mostrar al usuario
    let display_exe = if exe_file.exists() { &exe_file } else { &final_exe };
    
    println!("\n✅ ¡Completado! Ejecutable guardado en:");
    println!("   {}", display_exe.display());
    println!("\n💡 Puedes ejecutarlo directamente:");
    if cfg!(target_os = "windows") {
        // En Windows, mostrar ambas formas: directa y con comillas (para espacios)
        println!("   {}", display_exe.display());
        println!("\n   O con comillas (si hay espacios en la ruta):");
        println!("   & \"{}\"", display_exe.display());
    } else {
        println!("   ./{}", display_exe.display());
    }

    std::process::exit(exec_status.code().unwrap_or(0));
}


