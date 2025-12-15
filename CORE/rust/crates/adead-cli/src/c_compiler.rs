// Utilidades para encontrar y usar compiladores C (GCC/Clang)

use std::process::Command;
use std::path::PathBuf;

/// Buscar compilador C (GCC o Clang) en el sistema
pub fn find_c_compiler() -> Option<String> {
    // Lista de compiladores a buscar (en orden de preferencia)
    let compilers = vec!["gcc", "clang", "x86_64-w64-mingw32-gcc", "x86_64-pc-msys-gcc"];
    
    // Buscar en PATH
    for compiler in &compilers {
        if let Ok(output) = Command::new(compiler).arg("--version").output() {
            if output.status.success() {
                return Some(compiler.to_string());
            }
        }
    }
    
    // Buscar en ubicaciones comunes de Windows/MSYS2
    #[cfg(target_os = "windows")]
    {
        let common_paths = vec![
            "C:\\msys64\\mingw64\\bin\\gcc.exe",
            "C:\\msys64\\usr\\bin\\gcc.exe",
            "C:\\mingw64\\bin\\gcc.exe",
            "C:\\mingw\\bin\\gcc.exe",
            "C:\\Program Files\\mingw-w64\\x86_64-8.1.0-posix-seh-rt_v6-rev0\\mingw64\\bin\\gcc.exe",
        ];
        
        for path in &common_paths {
            if PathBuf::from(path).exists() {
                return Some(path.to_string());
            }
        }
        
        // Buscar clang en ubicaciones comunes
        let clang_paths = vec![
            "C:\\Program Files\\LLVM\\bin\\clang.exe",
            "C:\\Program Files (x86)\\LLVM\\bin\\clang.exe",
        ];
        
        for path in &clang_paths {
            if PathBuf::from(path).exists() {
                return Some(path.to_string());
            }
        }
    }
    
    None
}

/// Verificar si un compilador está disponible
pub fn check_compiler(compiler: &str) -> bool {
    Command::new(compiler)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

