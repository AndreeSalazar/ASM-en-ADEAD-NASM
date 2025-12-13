//! Resolución de módulos para import statements
//! Sprint 1.3 - Import básico

use crate::{parse, Program};
use adead_common::{ADeadError, Result};
use std::path::{Path, PathBuf};

/// Resolver el path de un módulo importado
/// Busca el archivo `nombre_modulo.ad` en múltiples ubicaciones
/// 
/// Estrategia de búsqueda:
/// 1. `nombre_modulo.ad` en directorio actual
/// 2. `modules/nombre_modulo.ad` en directorio actual
/// 3. `nombre_modulo/nombre_modulo.ad` (estructura de módulo)
pub fn resolve_module_path(module_name: &str, current_dir: Option<&Path>) -> Result<PathBuf> {
    // Validar nombre del módulo
    if module_name.is_empty() {
        return Err(ADeadError::ParseError {
            line: 1,
            col: 1,
            message: "El nombre del módulo no puede estar vacío".to_string(),
        });
    }
    
    // Validar caracteres permitidos en nombre de módulo
    if !module_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(ADeadError::ParseError {
            line: 1,
            col: 1,
            message: format!(
                "Nombre de módulo inválido: '{}'. Solo se permiten letras, números y guiones bajos",
                module_name
            ),
        });
    }
    
    let base_dir = current_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    // Intentar 1: nombre_modulo.ad en directorio actual
    let path1 = base_dir.join(format!("{}.ad", module_name));
    if path1.exists() && path1.is_file() {
        return Ok(path1);
    }

    // Intentar 2: nombre_modulo.ad en ./modules/
    let path2 = base_dir.join("modules").join(format!("{}.ad", module_name));
    if path2.exists() && path2.is_file() {
        return Ok(path2);
    }

    // Intentar 3: nombre_modulo/nombre_modulo.ad (estructura de módulo)
    let path3 = base_dir.join(module_name).join(format!("{}.ad", module_name));
    if path3.exists() && path3.is_file() {
        return Ok(path3);
    }

    // No encontrado - mensaje de error detallado
    let searched_paths = vec![
        path1,
        path2,
        path3,
    ];
    
    let paths_str: Vec<String> = searched_paths
        .iter()
        .map(|p| p.display().to_string())
        .collect();
    
    Err(ADeadError::ParseError {
        line: 1,
        col: 1,
        message: format!(
            "No se pudo encontrar el módulo '{}'.\n\
            Buscado en:\n  - {}\n  - {}\n  - {}\n\n\
            Asegúrate de que el archivo existe y está en una de estas ubicaciones.",
            module_name,
            paths_str.get(0).unwrap_or(&"N/A".to_string()),
            paths_str.get(1).unwrap_or(&"N/A".to_string()),
            paths_str.get(2).unwrap_or(&"N/A".to_string())
        ),
    })
}

/// Parsear un módulo importado
pub fn parse_module_file(path: &Path) -> Result<Program> {
    // Leer el archivo
    let content = std::fs::read_to_string(path).map_err(|e| {
        ADeadError::RuntimeError {
            message: format!("Error leyendo módulo '{}': {}", path.display(), e),
        }
    })?;

    // Parsear el contenido
    parse(&content)
}

/// Resolver y parsear un módulo
pub fn resolve_and_parse(module_name: &str, current_dir: Option<&Path>) -> Result<Program> {
    let module_path = resolve_module_path(module_name, current_dir)?;
    parse_module_file(&module_path)
}

