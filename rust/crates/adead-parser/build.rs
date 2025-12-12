// Build script para linkear con librería Zig
// Zig es el parser PRINCIPAL - esto asegura que Rust pueda usar Zig

fn main() {
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
    
    // Buscar en directorios relativos comunes (desde rust/crates/adead-parser/)
    let possible_paths = [
        "../../../zig/zig-out/lib",
        "../../zig/zig-out/lib",
        "../zig/zig-out/lib",
        "./zig/zig-out/lib",
    ];
    
    for path in &possible_paths {
        let path_obj = std::path::Path::new(path);
        // En Windows, Zig genera .lib en lugar de .a
        let lib_name = if cfg!(target_os = "windows") {
            "adead_zig.lib"
        } else {
            "libadead_zig.a"
        };
        if path_obj.exists() && path_obj.join(lib_name).exists() {
            println!("cargo:rustc-link-search=native={}", path);
            println!("cargo:rerun-if-changed={}/{}", path, lib_name);
            return;
        }
    }
    
    // Si no encontramos, usar path estándar y advertir
    let default_path = "../../../zig/zig-out/lib";
    println!("cargo:warning=Zig library path not found, using default: {}", default_path);
    println!("cargo:warning=Compile Zig first with: cd zig && zig build");
    println!("cargo:rustc-link-search=native={}", default_path);
}

