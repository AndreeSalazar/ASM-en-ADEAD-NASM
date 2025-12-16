/**
 * Build script para adead-parser
 * Enlaza con módulo C++ Optimizer si está disponible
 */

use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap().parent().unwrap().parent().unwrap();
    
    // Buscar biblioteca C++ Optimizer (nueva estructura: CORE/cpp/)
    let cpp_lib_paths = [
        workspace_root.join("CORE").join("cpp").join("build").join("libadead_cpp_optimizer.a"),
        workspace_root.join("CORE").join("cpp").join("build").join("adead_cpp_optimizer.lib"),
        workspace_root.join("CORE").join("cpp").join("adead_cpp_optimizer.lib"),
    ];
    
    let mut cpp_found = false;
    for cpp_path in &cpp_lib_paths {
        if cpp_path.exists() {
            println!("cargo:rustc-link-search=native={}", cpp_path.parent().unwrap().display());
            println!("cargo:rustc-link-lib=static=adead_cpp_optimizer");
            println!("cargo:warning=ADead C++ Optimizer library encontrado: {}", cpp_path.display());
            cpp_found = true;
            println!("cargo:rerun-if-changed={}", cpp_path.display());
            break;
        }
    }
    
    if !cpp_found {
        println!("cargo:warning=ADead C++ Optimizer library no encontrado, compilando sin optimizador C++");
        println!("cargo:warning=Para habilitar, compila el módulo C++ primero: cd CORE/cpp && ./build.sh");
    }
    
    // Compilar stub para ___chkstk_ms si es necesario (para Windows)
    let chkstk_stub = manifest_dir.join("chkstk_stub.c");
    if chkstk_stub.exists() {
        cc::Build::new()
            .file(&chkstk_stub)
            .compile("chkstk_stub");
        println!("cargo:warning=Stub ___chkstk_ms (C) compilado correctamente");
    }
}
