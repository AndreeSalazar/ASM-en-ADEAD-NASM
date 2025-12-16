/**
 * Pipeline Paralelo: Compilación paralela de múltiples archivos con caching
 * 
 * Este módulo implementa:
 * - Compilación paralela de múltiples archivos .ad
 * - Caching de resultados intermedios (AST, ASM optimizado, etc.)
 * - Hash SHA256 para verificar cambios en archivos
 * 
 * Autor: Eddi Andreé Salazar Matos
 * Fecha: Diciembre 2025
 */

// use crate::optimized_pipeline::OptimizedPipeline;  // Removido - usar pipeline_selector en su lugar
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Resultado de la compilación de un archivo
#[derive(Debug, Clone)]
pub struct CompilationResult {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub success: bool,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Cache entry para almacenar resultados intermedios
#[derive(Debug, Clone)]
struct CacheEntry {
    file_hash: String,
    output_path: PathBuf,
    timestamp: SystemTime,
    asm_content: String,
}

/// Pipeline paralelo con caching
pub struct ParallelPipeline {
    cache_dir: PathBuf,
    cache: Arc<Mutex<HashMap<PathBuf, CacheEntry>>>,
}

impl ParallelPipeline {
    /// Crear un nuevo pipeline paralelo
    pub fn new(cache_dir: Option<PathBuf>) -> Self {
        let cache_dir = cache_dir.unwrap_or_else(|| {
            // Directorio de cache por defecto: .adead_cache en el directorio actual
            PathBuf::from(".adead_cache")
        });
        
        // Crear directorio de cache si no existe
        if let Err(e) = fs::create_dir_all(&cache_dir) {
            eprintln!("⚠️  Warning: No se pudo crear directorio de cache: {}", e);
        }
        
        Self {
            cache_dir,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Calcular hash SHA256 de un archivo
    fn calculate_file_hash(path: &Path) -> Result<String, String> {
        use std::io::Read;
        
        let mut file = fs::File::open(path)
            .map_err(|e| format!("No se pudo abrir archivo {}: {}", path.display(), e))?;
        
        let mut hasher = sha2::Sha256::new();
        let mut buffer = [0u8; 8192];
        
        loop {
            let bytes_read = file.read(&mut buffer)
                .map_err(|e| format!("Error leyendo archivo {}: {}", path.display(), e))?;
            
            if bytes_read == 0 {
                break;
            }
            
            use sha2::Digest;
            hasher.update(&buffer[..bytes_read]);
        }
        
        use sha2::Digest;
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }
    
    /// Calcular hash SHA256 de contenido de string
    fn calculate_content_hash(content: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let hash = hasher.finalize();
        format!("{:x}", hash)
    }
    
    /// Obtener entrada del cache si existe y es válida
    fn get_cache_entry(&self, input_path: &Path, current_hash: &str) -> Option<String> {
        // Normalizar la ruta para comparación consistente
        let normalized_path = match input_path.canonicalize() {
            Ok(p) => p,
            Err(_) => input_path.to_path_buf(),
        };
        
        let cache = self.cache.lock().unwrap();
        if let Some(entry) = cache.get(&normalized_path) {
            // Verificar que el hash coincida
            if entry.file_hash == current_hash {
                // Verificar que el archivo de salida existe
                if entry.output_path.exists() {
                    // Leer contenido del archivo cacheado
                    if let Ok(content) = fs::read_to_string(&entry.output_path) {
                        println!("   💾 Cache hit para: {}", normalized_path.display());
                        return Some(content);
                    }
                }
            }
        }
        None
    }
    
    /// Guardar entrada en el cache
    fn save_cache_entry(&self, input_path: PathBuf, file_hash: String, output_path: PathBuf, asm_content: String) {
        // Normalizar la ruta para almacenamiento consistente
        let normalized_path = match input_path.canonicalize() {
            Ok(p) => p,
            Err(_) => input_path,
        };
        
        let entry = CacheEntry {
            file_hash,
            output_path: output_path.clone(),
            timestamp: SystemTime::now(),
            asm_content: asm_content.clone(),
        };
        
        let mut cache = self.cache.lock().unwrap();
        cache.insert(normalized_path, entry);
        
        // También guardar en disco para persistencia entre ejecuciones
        let _cache_file = self.cache_dir.join("cache.json");
        // Por simplicidad, solo guardamos en memoria por ahora
        // TODO: Implementar persistencia en disco si es necesario
    }
    
    /// Compilar un archivo individual (usado internamente por compile_parallel)
    fn compile_single_file(
        &self,
        input_path: PathBuf,
        output_path: PathBuf,
    ) -> CompilationResult {
        let start_time = SystemTime::now();
        
        // Resolver ruta absoluta del archivo de entrada
        let input_path_abs = if input_path.is_absolute() {
            input_path.clone()
        } else {
            // Resolver ruta relativa desde el directorio actual
            match std::env::current_dir() {
                Ok(current_dir) => current_dir.join(&input_path),
                Err(_) => input_path.clone(),
            }
        };
        
        // Normalizar la ruta (canonicalize) para consistencia en cache
        let input_path_canonical = match input_path_abs.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                // Si no se puede canonicalizar, verificar existencia primero
                if !input_path_abs.exists() {
                    return CompilationResult {
                        input_path: input_path.clone(),
                        output_path: output_path.clone(),
                        success: false,
                        error: Some(format!("Archivo no encontrado: {} (ruta resuelta: {})", 
                            input_path.display(), input_path_abs.display())),
                        duration_ms: 0,
                    };
                }
                input_path_abs.clone()
            }
        };
        
        // Verificar que el archivo existe
        if !input_path_canonical.exists() {
            return CompilationResult {
                input_path: input_path.clone(),
                output_path: output_path.clone(),
                success: false,
                error: Some(format!("Archivo no encontrado: {} (ruta resuelta: {})", 
                    input_path.display(), input_path_canonical.display())),
                duration_ms: 0,
            };
        }
        
        // Calcular hash del archivo
        let file_hash = match Self::calculate_file_hash(&input_path_canonical) {
            Ok(hash) => hash,
            Err(e) => {
                return CompilationResult {
                    input_path: input_path.clone(),
                    output_path: output_path.clone(),
                    success: false,
                    error: Some(format!("Error calculando hash: {}", e)),
                    duration_ms: 0,
                };
            }
        };
        
        // Verificar cache (usar ruta canónica para consistencia)
        if let Some(cached_asm) = self.get_cache_entry(&input_path_canonical, &file_hash) {
            // Usar contenido cacheado
            match fs::write(&output_path, &cached_asm) {
                Ok(_) => {
                    let duration = start_time.elapsed()
                        .unwrap_or_default()
                        .as_millis() as u64;
                    
                    return CompilationResult {
                        input_path: input_path.clone(),
                        output_path: output_path.clone(),
                        success: true,
                        error: None,
                        duration_ms: duration,
                    };
                }
                Err(e) => {
                    // Cache hit pero error al escribir, continuar con compilación
                    eprintln!("⚠️  Warning: Cache hit pero error al escribir: {}", e);
                }
            }
        }
        
        // Leer archivo fuente (usar ruta canónica)
        let source = match fs::read_to_string(&input_path_canonical) {
            Ok(content) => content,
            Err(e) => {
                return CompilationResult {
                    input_path: input_path.clone(),
                    output_path: output_path.clone(),
                    success: false,
                    error: Some(format!("Error leyendo archivo: {}", e)),
                    duration_ms: 0,
                };
            }
        };
        
        // Asegurar que el directorio de salida existe
        if let Some(parent) = output_path.parent() {
            // Verificar si el path es un archivo existente (no un directorio)
            if parent.exists() {
                if !parent.is_dir() {
                    return CompilationResult {
                        input_path: input_path.clone(),
                        output_path: output_path.clone(),
                        success: false,
                        error: Some(format!("Error: La ruta de salida es un archivo, no un directorio: {} (usa un nombre diferente para el directorio de salida)", parent.display())),
                        duration_ms: 0,
                    };
                }
                // Si es un directorio y existe, está bien, continuar
            } else {
                // El directorio no existe, crearlo
                if let Err(e) = fs::create_dir_all(parent) {
                    return CompilationResult {
                        input_path: input_path.clone(),
                        output_path: output_path.clone(),
                        success: false,
                        error: Some(format!("Error creando directorio de salida: {} (verifica permisos y que no existe un archivo con ese nombre)", e)),
                        duration_ms: 0,
                    };
                }
            }
        }
        
        // Compilar usando pipeline inteligente
        let input_path_str = input_path_canonical.to_string_lossy().to_string();
        let asm_content = match crate::pipeline_selector::process_adead_intelligent(&source) {
            Ok((_pipeline, asm)) => asm,
            Err(e) => {
                return CompilationResult {
                    input_path: input_path.clone(),
                    output_path: output_path.clone(),
                    success: false,
                    error: Some(format!("Error en pipeline inteligente: {}", e)),
                    duration_ms: start_time.elapsed().unwrap_or_default().as_millis() as u64,
                };
            }
        };
        
        // Guardar resultado
        match fs::write(&output_path, &asm_content) {
            Ok(_) => {
                // Guardar en cache (usar ruta canónica)
                self.save_cache_entry(input_path_canonical.clone(), file_hash, output_path.clone(), asm_content);
                
                let duration = start_time.elapsed()
                    .unwrap_or_default()
                    .as_millis() as u64;
                
                CompilationResult {
                    input_path: input_path.clone(),
                    output_path: output_path.clone(),
                    success: true,
                    error: None,
                    duration_ms: duration,
                }
            }
            Err(e) => {
                CompilationResult {
                    input_path: input_path.clone(),
                    output_path: output_path.clone(),
                    success: false,
                    error: Some(format!("Error escribiendo archivo de salida: {}", e)),
                    duration_ms: start_time.elapsed().unwrap_or_default().as_millis() as u64,
                }
            }
        }
    }
    
    /// Compilar múltiples archivos en paralelo
    pub fn compile_parallel(
        &self,
        inputs: Vec<PathBuf>,
        output_dir: Option<PathBuf>,
    ) -> Vec<CompilationResult> {
        
        println!("🚀 Compilando {} archivo(s) en paralelo...", inputs.len());
        
        // Resolver directorio de salida (absoluto)
        let output_dir = output_dir.map(|p| {
            if p.is_absolute() {
                p
            } else {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(p)
            }
        }).unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        });
        
        // Preparar rutas de entrada y salida (resolver rutas absolutas)
        let inputs_with_outputs: Vec<(PathBuf, PathBuf)> = inputs
            .into_iter()
            .map(|input| {
                // Resolver ruta absoluta del archivo de entrada
                let input_abs = if input.is_absolute() {
                    input.clone()
                } else {
                    std::env::current_dir()
                        .unwrap_or_else(|_| PathBuf::from("."))
                        .join(&input)
                };
                
                let output_name = input_abs.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| format!("{}.asm", s))
                    .unwrap_or_else(|| "output.asm".to_string());
                let output = output_dir.join(output_name);
                (input_abs, output)
            })
            .collect();
        
        // Compilar en paralelo usando threads
        // Usamos scoped threads (disponible desde Rust 1.63+)
        use std::thread;
        let results: Vec<CompilationResult> = thread::scope(|s| {
            let handles: Vec<_> = inputs_with_outputs
                .into_iter()
                .map(|(input, output)| {
                    s.spawn(|| {
                        self.compile_single_file(input, output)
                    })
                })
                .collect();
            
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });
        
        // Mostrar resumen
        let successful = results.iter().filter(|r| r.success).count();
        let failed = results.len() - successful;
        let total_time: u64 = results.iter().map(|r| r.duration_ms).sum();
        
        println!("\n📊 Resumen de compilación paralela:");
        println!("   ✅ Exitosas: {}", successful);
        println!("   ❌ Fallidas: {}", failed);
        println!("   ⏱️  Tiempo total: {} ms", total_time);
        
        if failed > 0 {
            println!("\n❌ Archivos con errores:");
            for result in &results {
                if !result.success {
                    if let Some(error) = &result.error {
                        println!("   - {}: {}", result.input_path.display(), error);
                    }
                }
            }
        }
        
        results
    }
    
    /// Limpiar cache (eliminar entradas obsoletas)
    pub fn clear_cache(&self) -> Result<usize, String> {
        let mut cache = self.cache.lock().unwrap();
        let count = cache.len();
        cache.clear();
        Ok(count)
    }
    
    /// Obtener estadísticas del cache
    pub fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.lock().unwrap();
        let total_entries = cache.len();
        
        // Contar entradas válidas (donde el archivo de salida existe)
        let valid_entries = cache.values()
            .filter(|entry| entry.output_path.exists())
            .count();
        
        (total_entries, valid_entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_content_hash() {
        let content1 = "print 42\n";
        let content2 = "print 100\n";
        
        let hash1 = ParallelPipeline::calculate_content_hash(content1);
        let hash2 = ParallelPipeline::calculate_content_hash(content1);
        let hash3 = ParallelPipeline::calculate_content_hash(content2);
        
        // El hash debe ser consistente para el mismo contenido
        assert_eq!(hash1, hash2);
        
        // El hash debe cambiar para contenido diferente
        assert_ne!(hash1, hash3);
        
        // El hash debe tener 64 caracteres (SHA256 en hex)
        assert_eq!(hash1.len(), 64);
    }
}

