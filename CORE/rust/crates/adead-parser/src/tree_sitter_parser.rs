// Tree-sitter parser para ADead
// Integración con el parser generado por tree-sitter
// Soporta: Tree-sitter → NASM directo (sin Rust AST)

use tree_sitter::{Language, Parser, Tree};
use std::ffi::OsStr;
use std::path::Path;

// Función externa para obtener el lenguaje (implementada en parser.c)
extern "C" {
    fn tree_sitter_adead() -> *const std::ffi::c_void;
}

pub struct TreeSitterParser {
    parser: Parser,
    language: Language,
}

impl TreeSitterParser {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let language_ptr = tree_sitter_adead();
            if language_ptr.is_null() {
                return Err("tree_sitter_adead() returned null. Tree-sitter may not be compiled correctly.".into());
            }
            
            // Convertir el puntero void a TSLanguage usando from_raw
            // tree-sitter usa un puntero opaco, así que necesitamos castearlo correctamente
            let language = Language::from_raw(language_ptr as *mut _);
            
            let mut parser = Parser::new();
            match parser.set_language(&language) {
                Ok(_) => {},
                Err(e) => {
                    return Err(format!("Failed to set language: {}", e).into());
                }
            }
            
            Ok(Self {
                parser,
                language,
            })
        }
    }

    pub fn parse(&mut self, source: &str) -> Result<Tree, Box<dyn std::error::Error>> {
        self.parser.parse(source, None)
            .ok_or_else(|| "Parsing failed".into())
    }

    pub fn parse_file<P: AsRef<Path>>(&mut self, path: P) -> Result<Tree, Box<dyn std::error::Error>> {
        let source = std::fs::read_to_string(path.as_ref())?;
        self.parse(&source)
    }

    /// Generar NASM directamente desde Tree-sitter AST (sin pasar por Rust AST)
    /// Flujo: ADead Source → Tree-sitter → NASM
    pub fn generate_nasm_direct(&mut self, source: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Por ahora, usar el módulo tree_sitter_nasm cuando esté listo
        // TODO: Implementar generación completa
        let tree = self.parse(source)?;
        
        // Usar generador directo
        let mut generator = crate::tree_sitter_nasm::TreeSitterNASMGenerator::new()
            .map_err(|e| format!("Error creando generador: {}", e))?;
        let nasm = generator.generate(&tree, source);
        
        Ok(nasm)
    }
}

// Conversión de Tree-sitter AST a AST interno de ADead
// TODO: Implementar conversión completa
pub fn convert_tree_sitter_to_adead_ast(_tree: &Tree, _source: &str) -> Result<crate::Program, Box<dyn std::error::Error>> {
    // Por ahora retornar programa vacío
    // TODO: Implementar conversión completa del AST
    Ok(crate::Program {
        statements: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_sitter_parser_basic() {
        let mut parser = TreeSitterParser::new().unwrap();
        let source = r#"
            print "Hello"
            let x = 10
        "#;
        
        let result = parser.parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_tree_sitter_while_loop() {
        let mut parser = TreeSitterParser::new().unwrap();
        let source = r#"
            let x = 0
            while x < 10 {
                print x
                x = x + 1
            }
        "#;
        
        let result = parser.parse(source);
        assert!(result.is_ok(), "While loop should parse correctly");
    }

    #[test]
    fn test_tree_sitter_while_with_nested_if() {
        let mut parser = TreeSitterParser::new().unwrap();
        let source = r#"
            let suma = 1
            let limite = 100
            let intervalo = 10
            
            while suma <= limite {
                if suma % intervalo == 0 {
                    print suma
                }
                suma = suma + 1
            }
        "#;
        
        let result = parser.parse(source);
        assert!(result.is_ok(), "While with nested if should parse correctly");
    }
}

