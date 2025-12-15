//! Integración con Agner Fog's objconv - Optimización avanzada x86

use std::process::Command;
use std::fs;
use std::path::PathBuf;

/// Optimizador usando objconv (opcional)
pub struct ObjconvOptimizer {
    objconv_path: Option<PathBuf>,
    enabled: bool,
}

impl ObjconvOptimizer {
    pub fn new() -> Self {
        // Buscar objconv en PATH o rutas comunes
        let objconv_path = Self::find_objconv();
        Self {
            objconv_path,
            enabled: true,
        }
    }

    /// Busca objconv en el sistema
    fn find_objconv() -> Option<PathBuf> {
        // Intentar encontrar objconv en PATH
        if let Ok(output) = Command::new("objconv").arg("--version").output() {
            if output.status.success() {
                return Some(PathBuf::from("objconv"));
            }
        }

        // Rutas comunes en Windows
        let common_paths = [
            "C:\\objconv\\objconv.exe",
            "C:\\tools\\objconv\\objconv.exe",
            ".\\objconv.exe",
        ];

        for path in &common_paths {
            if PathBuf::from(path).exists() {
                return Some(PathBuf::from(path));
            }
        }

        None
    }

    /// Optimiza ASM usando objconv (si está disponible)
    pub fn optimize(&self, asm: &str) -> Result<String, String> {
        if !self.enabled {
            return Err("Objconv no está habilitado".to_string());
        }

        let objconv_path = match &self.objconv_path {
            Some(p) => p,
            None => return Err("Objconv no encontrado en el sistema".to_string()),
        };

        // Crear archivos temporales
        let temp_dir = std::env::temp_dir();
        let input_asm = temp_dir.join("clean_code_input.asm");
        let output_obj = temp_dir.join("clean_code_output.obj");
        let output_asm = temp_dir.join("clean_code_output.asm");

        // Escribir ASM de entrada
        fs::write(&input_asm, asm)
            .map_err(|e| format!("Error escribiendo ASM temporal: {}", e))?;

        // Paso 1: Ensamblar ASM a objeto usando NASM
        let nasm_result = Command::new("nasm")
            .arg("-f")
            .arg("win64") // o elf64 para Linux
            .arg("-o")
            .arg(&output_obj)
            .arg(&input_asm)
            .output();

        if let Err(_) = nasm_result {
            // NASM no disponible, usar objconv directamente en ASM
            return self.optimize_asm_direct(objconv_path, &input_asm, &output_asm);
        }

        // Paso 2: Optimizar objeto con objconv
        let objconv_result = Command::new(objconv_path)
            .arg("-fyasm") // Formato YASM/NASM
            .arg(&output_obj)
            .arg(&output_asm)
            .output();

        match objconv_result {
            Ok(output) if output.status.success() => {
                // Leer ASM optimizado
                fs::read_to_string(&output_asm)
                    .map_err(|e| format!("Error leyendo ASM optimizado: {}", e))
            }
            Ok(_) => {
                // objconv falló, retornar original
                Err("Objconv falló, usando ASM original".to_string())
            }
            Err(_) => {
                // objconv no disponible, retornar original
                Err("Objconv no disponible".to_string())
            }
        }
    }

    /// Optimiza ASM directamente sin ensamblar (fallback)
    fn optimize_asm_direct(
        &self,
        _objconv_path: &PathBuf,
        _input: &PathBuf,
        _output: &PathBuf,
    ) -> Result<String, String> {
        // objconv puede trabajar directamente con ASM en algunos casos
        // Por ahora, retornar error para usar el método normal
        Err("Objconv requiere NASM para funcionar correctamente".to_string())
    }

    /// Verifica si objconv está disponible
    pub fn is_available(&self) -> bool {
        self.objconv_path.is_some()
    }
}

impl Default for ObjconvOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

