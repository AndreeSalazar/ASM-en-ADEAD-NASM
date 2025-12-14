// Build script para linkear con librería Zig y Tree-sitter
// Zig es el parser PRINCIPAL - esto asegura que Rust pueda usar Zig
// Tree-sitter es el parser ROBUSTO - para estructuras complejas

fn main() {
    // COMPILAR TREE-SITTER
    compile_tree_sitter();
    
    // LINKAR CON ZIG (código existente)
    // En Windows, linkear con librerías del sistema necesarias para Zig
    if cfg!(target_os = "windows") {
        // El símbolo ___chkstk_ms es generado por Zig cuando una función necesita
        // más de 4KB de stack (incluso con -fno-stack-check)
        // Este símbolo está en libcmt.lib (C runtime multithreaded)
        
        // Solución: Linkear con libcmt.lib que contiene ___chkstk_ms
        // Rust por defecto usa msvcrt.lib, pero podemos forzar libcmt
        // usando un objeto stub o linkeando directamente
        
        // Intentar linkear libcmt que tiene el símbolo
        // Nota: Puede causar conflictos con msvcrt, pero es necesario
        println!("cargo:rustc-link-lib=msvcrt");
        
        // Alternativa: Crear un stub para ___chkstk_ms si no queremos libcmt
        // Por ahora, intentamos que Rust encuentre el símbolo en msvcrt
    }
    
    // Buscar librería Zig compilada
    if let Ok(zig_lib_path) = std::env::var("ZIG_LIB_PATH") {
        println!("cargo:rustc-link-search=native={}", zig_lib_path);
        let lib_name = if cfg!(target_os = "windows") {
            "adead_zig.lib"
        } else {
            "libadead_zig.a"
        };
        println!("cargo:rerun-if-changed={}/{}", zig_lib_path, lib_name);
        return;
    }
    
    // Buscar desde CARGO_MANIFEST_DIR (más confiable)
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = std::path::Path::new(&manifest_dir);
        // Desde rust/crates/adead-parser/ subir a la raíz: ../../../
        if let Some(root) = manifest_path.parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent()) {
            
            let lib_name = if cfg!(target_os = "windows") {
                "adead_zig.lib"
            } else {
                "libadead_zig.a"
            };
            
            // Intentar zig/adead_zig.lib (directo)
            let zig_dir = root.join("zig");
            let lib_path_direct = zig_dir.join(lib_name);
            if lib_path_direct.exists() {
                println!("cargo:rustc-link-search=native={}", zig_dir.display());
                println!("cargo:rerun-if-changed={}", lib_path_direct.display());
                return;
            }
            
            // Intentar zig/zig-out/lib/adead_zig.lib
            let zig_lib_dir = root.join("zig/zig-out/lib");
            let lib_path = zig_lib_dir.join(lib_name);
            if lib_path.exists() {
                println!("cargo:rustc-link-search=native={}", zig_lib_dir.display());
                println!("cargo:rerun-if-changed={}", lib_path.display());
                return;
            }
            
            // Intentar rust/target/release/adead_zig.lib (copiado manualmente)
            let rust_target = root.join("rust/target/release");
            let rust_lib_path = rust_target.join(lib_name);
            if rust_lib_path.exists() {
                println!("cargo:rustc-link-search=native={}", rust_target.display());
                println!("cargo:rerun-if-changed={}", rust_lib_path.display());
                return;
            }
        }
    }
    
    // Buscar en directorios relativos comunes (desde rust/crates/adead-parser/)
    let lib_name = if cfg!(target_os = "windows") {
        "adead_zig.lib"
    } else {
        "libadead_zig.a"
    };
    
    let possible_paths = [
        ("../../../zig", &lib_name[..]),  // zig/adead_zig.lib
        ("../../../zig/zig-out/lib", &lib_name[..]),  // zig/zig-out/lib/adead_zig.lib
        ("../../../rust/target/release", &lib_name[..]),  // rust/target/release/adead_zig.lib
        ("../../zig", &lib_name[..]),
        ("../zig", &lib_name[..]),
        ("./zig", &lib_name[..]),
    ];
    
    for (path_str, lib) in &possible_paths {
        let path_obj = std::path::Path::new(path_str);
        if path_obj.exists() {
            let lib_path = if path_obj.is_dir() {
                path_obj.join(lib)
            } else if path_obj.is_file() && path_obj.file_name().and_then(|n| n.to_str()) == Some(lib) {
                path_obj.to_path_buf()
            } else {
                continue;
            };
            
            if lib_path.exists() {
                let search_dir = if lib_path.is_file() {
                    lib_path.parent().unwrap()
                } else {
                    path_obj
                };
                println!("cargo:rustc-link-search=native={}", search_dir.display());
                println!("cargo:rerun-if-changed={}", lib_path.display());
                return;
            }
        }
    }
    
    // Si no encontramos, usar path estándar y advertir
    let default_path = "../../../zig/zig-out/lib";
    println!("cargo:warning=Zig library path not found, using default: {}", default_path);
    println!("cargo:warning=Compile Zig first with: cd zig && zig build-lib src/nasm_generator.zig ...");
    println!("cargo:rustc-link-search=native={}", default_path);
}

fn compile_tree_sitter() {
    // Ruta al directorio tree-sitter-adead
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = std::path::Path::new(&manifest_dir);
    
    // Desde rust/crates/adead-parser/ subir a la raíz: ../../../
    if let Some(root) = manifest_path.parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent()) {
        
        let tree_sitter_dir = root.join("tree-sitter-adead");
        let src_dir = tree_sitter_dir.join("src");
        
        if !src_dir.exists() {
            println!("cargo:warning=Tree-sitter source directory not found: {}", src_dir.display());
            return;
        }
        
        // Verificar que parser.c existe
        let parser_c = src_dir.join("parser.c");
        if !parser_c.exists() {
            println!("cargo:warning=parser.c not found. Run 'cd tree-sitter-adead && tree-sitter generate' first");
            return;
        }
        
        // Compilar tree-sitter como biblioteca estática
        let mut build = cc::Build::new();
        build
            .file(src_dir.join("parser.c"))
            .include(&src_dir)
            .include(src_dir.join("tree_sitter"))
            .warnings(false)
            .flag_if_supported("-std=c99");
        
        // Compilar
        build.compile("tree_sitter_adead");
        
        // Registrar función externa para Rust
        println!("cargo:rustc-link-lib=static=tree_sitter_adead");
        
        // Recompilar si cambian los archivos fuente
        println!("cargo:rerun-if-changed={}", parser_c.display());
    }
}
