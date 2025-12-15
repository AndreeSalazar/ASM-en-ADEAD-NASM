//! Tests profundos para sistema de imports (Sprint 1.3)

use adead_parser::{parse_with_dir, Program, Stmt};
use std::path::Path;

#[test]
fn test_import_statement_parsing() {
    let source = "import math";
    let program = parse_with_dir(source, None).unwrap();
    
    assert_eq!(program.statements.len(), 1);
    if let Stmt::Import(module_name) = &program.statements[0] {
        assert_eq!(module_name, "math");
    } else {
        panic!("Expected Import statement");
    }
}

#[test]
fn test_import_multiple_modules() {
    let source = r#"
import math
import utils
import strings
"#;
    let program = parse_with_dir(source, None).unwrap();
    
    let imports: Vec<_> = program.statements
        .iter()
        .filter_map(|s| {
            if let Stmt::Import(name) = s {
                Some(name.clone())
            } else {
                None
            }
        })
        .collect();
    
    assert_eq!(imports.len(), 3);
    assert!(imports.contains(&"math".to_string()));
    assert!(imports.contains(&"utils".to_string()));
    assert!(imports.contains(&"strings".to_string()));
}

#[test]
fn test_qualified_function_call() {
    let source = "math.factorial(5)";
    let program = parse_with_dir(source, None).unwrap();
    
    // Debería parsear como expresión
    assert!(!program.statements.is_empty());
}

#[test]
fn test_public_vs_private_functions() {
    let source = r#"
pub fn public_func() {
    print "public"
}

fn private_func() {
    print "private"
}
"#;
    let program = parse_with_dir(source, None).unwrap();
    
    // Verificar que ambas funciones se parsean
    let functions: Vec<_> = program.statements
        .iter()
        .filter_map(|s| {
            if let Stmt::Fn { visibility, name, .. } = s {
                Some((visibility.clone(), name.clone()))
            } else {
                None
            }
        })
        .collect();
    
    assert_eq!(functions.len(), 2);
    
    // Verificar visibilidad
    let public = functions.iter().find(|(_, n)| n == "public_func");
    let private = functions.iter().find(|(_, n)| n == "private_func");
    
    assert!(public.is_some());
    assert!(private.is_some());
    
    // Verificar que public es Public y private es Private
    use adead_parser::Visibility;
    if let Some((vis, _)) = public {
        assert_eq!(*vis, Visibility::Public);
    }
    if let Some((vis, _)) = private {
        assert_eq!(*vis, Visibility::Private);
    }
}

#[test]
fn test_module_resolver_path_construction() {
    use adead_parser::module_resolver::resolve_module_path;
    
    // Test con directorio actual
    let current_dir = Path::new(".");
    let result = resolve_module_path("math", Some(current_dir));
    
    // No debería fallar (aunque el archivo puede no existir)
    // Solo verificamos que la función funciona
    let _ = result;
}

// Tests de integración (requieren archivos reales)
#[cfg(feature = "integration-tests")]
mod integration {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;
    
    fn create_temp_module(name: &str, content: &str) -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join(format!("{}.ad", name));
        fs::write(&file_path, content).unwrap();
        (dir, file_path)
    }
    
    #[test]
    fn test_resolve_and_import_module() {
        let (temp_dir, _math_file) = create_temp_module("math", "pub fn add(a: int64, b: int64) { return a + b }");
        
        let source = "import math";
        let program = parse_with_dir(source, Some(temp_dir.path())).unwrap();
        
        // Verificar que se importó
        assert!(program.statements.iter().any(|s| {
            if let Stmt::Fn { name, .. } = s {
                name == "add"
            } else {
                false
            }
        }));
    }
    
    #[test]
    fn test_only_public_functions_imported() {
        let (temp_dir, _module_file) = create_temp_module("test_module", r#"
pub fn public_func() {
    print "public"
}

fn private_func() {
    print "private"
}
"#);
        
        let source = "import test_module";
        let program = parse_with_dir(source, Some(temp_dir.path())).unwrap();
        
        // Verificar que solo public_func está disponible
        let functions: Vec<_> = program.statements
            .iter()
            .filter_map(|s| {
                if let Stmt::Fn { name, .. } = s {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect();
        
        assert!(functions.contains(&"public_func".to_string()));
        assert!(!functions.contains(&"private_func".to_string()));
    }
    
    #[test]
    fn test_module_not_found_error() {
        let temp_dir = TempDir::new().unwrap();
        let source = "import nonexistent";
        
        let result = parse_with_dir(source, Some(temp_dir.path()));
        assert!(result.is_err());
    }
}

