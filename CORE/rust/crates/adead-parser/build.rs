/**
 * Build script para adead-parser
 * Enlaza con módulo D Language y biblioteca Zig si están disponibles
 */

use std::env;
use std::path::PathBuf;

fn main() {
    // Buscar el objeto compilado de D (ruta relativa desde crate root)
    // Nueva estructura: CORE/d/ en lugar de d/
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap().parent().unwrap().parent().unwrap();
    let d_build_dir = workspace_root.join("CORE").join("d").join("build");
    let d_obj_path = d_build_dir.join("adead_d.obj");
    let d_lib_path = d_build_dir.join("adead_d.lib");
    
    // IMPORTANTE: Aunque el objeto D existe, las funciones NO están completamente implementadas
    // Por lo tanto, NUNCA linkear el objeto D automáticamente, incluso con --features d-language
    // Las funciones externas solo se declaran cuando la feature está activa, pero no se linkean
    // Esto permite que el código compile, pero las funciones retornarán None (stubs)
    // Cuando D esté completamente funcional, se puede habilitar el linking aquí
    
    if d_obj_path.exists() {
        println!("cargo:warning=ADead D module encontrado en: {}", d_obj_path.display());
        println!("cargo:warning=NOTA: D existe pero las funciones no están completamente implementadas");
        println!("cargo:warning=El sistema usará stubs de D (funciones que retornan None)");
        println!("cargo:warning=Para habilitar D en el futuro, implementa todas las funciones en d/src/");
    } else {
        println!("cargo:warning=ADead D module no encontrado, usando stubs");
        println!("cargo:warning=Para habilitar, compila el módulo D primero: cd CORE/d && ./build.ps1");
    }
    
    // Linkear el objeto D si la feature está habilitada
    let d_feature_enabled = env::var("CARGO_FEATURE_D_LANGUAGE").is_ok();
    if d_feature_enabled {
        // Preferir biblioteca estática si existe, sino usar objeto
        if d_lib_path.exists() {
            println!("cargo:rustc-link-search=native={}", d_build_dir.display());
            println!("cargo:rustc-link-lib=static=adead_d");
            println!("cargo:warning=ADead D library linkeada: {}", d_lib_path.display());
        } else if d_obj_path.exists() {
            println!("cargo:rustc-link-search=native={}", d_build_dir.display());
            // Linkear objeto directamente
            let d_obj_abs = std::fs::canonicalize(&d_obj_path).unwrap_or(d_obj_path.clone());
            println!("cargo:rustc-link-arg={}", d_obj_abs.display());
            println!("cargo:warning=ADead D object linkeado: {}", d_obj_abs.display());
        } else {
            println!("cargo:warning=Feature d-language activada pero objeto/biblioteca D no encontrado");
            println!("cargo:warning=Compila el módulo D: cd CORE/d && ./build.ps1");
        }
    }
    
    // Recompilar si cambia el módulo D
    println!("cargo:rerun-if-changed={}", d_obj_path.display());
    
    // Buscar biblioteca Zig (nueva estructura: CORE/zig/)
    let zig_lib_paths = [
        workspace_root.join("CORE").join("zig").join("zig-out").join("lib").join("adead_zig.lib"),
        workspace_root.join("CORE").join("zig").join("adead_zig.lib"),
        workspace_root.join("CORE").join("zig").join("zig-out").join("lib").join("adead_zig.a"),
        workspace_root.join("CORE").join("zig").join("adead_zig.a"),
    ];
    
    let mut zig_found = false;
    for zig_path in &zig_lib_paths {
        if zig_path.exists() {
            println!("cargo:rustc-link-search=native={}", zig_path.parent().unwrap().display());
            println!("cargo:rustc-link-lib=static=adead_zig");
            // Linkear con libucrt para resolver ___chkstk_ms en Windows
            #[cfg(target_os = "windows")]
            {
                println!("cargo:rustc-link-lib=ucrt");
            }
            println!("cargo:warning=ADead Zig library encontrado: {}", zig_path.display());
            zig_found = true;
            println!("cargo:rerun-if-changed={}", zig_path.display());
            break;
        }
    }
    
    if !zig_found {
        println!("cargo:warning=ADead Zig library no encontrado, compilando sin soporte Zig");
        println!("cargo:warning=Para habilitar, compila Zig primero: cd CORE/zig && zig build-lib src/main.zig -target x86_64-windows -fno-stack-check -lc -O ReleaseFast --name adead_zig");
        // Hacer el linking opcional con una feature flag
        println!("cargo:rustc-cfg=feature=\"no-zig\"");
    }
    
    // Tree-sitter eliminado - ahora usamos Parser Manual Especializado
    // No se compila parser.c de Tree-sitter
    
    // Compilar stub para ___chkstk_ms (necesario cuando se linkea código Zig)
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    
    // Intentar primero con .asm (más confiable para Windows)
    let chkstk_stub_asm = manifest_dir.join("chkstk_stub.asm");
    if chkstk_stub_asm.exists() {
        // Compilar con ml64 (Microsoft Macro Assembler)
        let output = std::process::Command::new("ml64")
            .args(&[
                "/c", "/Fo", &format!("{}chkstk_stub.obj", manifest_dir.join("..").join("..").join("target").join("release").join("build").join("adead-parser-").to_string_lossy()),
                &chkstk_stub_asm.to_string_lossy()
            ])
            .output();
        
        if output.is_ok() {
            println!("cargo:warning=Stub ___chkstk_ms (ASM) compilado correctamente");
        }
    }
    
    // Fallback: usar C stub
    let chkstk_stub = manifest_dir.join("chkstk_stub.c");
    if chkstk_stub.exists() {
        cc::Build::new()
            .file(&chkstk_stub)
            .compile("chkstk_stub");
        println!("cargo:warning=Stub ___chkstk_ms (C) compilado correctamente");
    }
}
