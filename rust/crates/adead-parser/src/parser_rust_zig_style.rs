// Inspiración Rust + Zig para mejorar el parser
// Este módulo contiene técnicas y helpers inspirados en ambos lenguajes

use chumsky::prelude::*;

/// Helper inspirado en Zig: Parser simple y directo para tipos
/// Zig tiene parsers muy simples y directos - aplicamos eso aquí
pub fn tipo_parser() -> impl Parser<char, String, Error = Simple<char>> + Clone {
    // Parsers específicos primero (más específicos tienen precedencia)
    just("int64")
        .map(|_| "int64".to_string())
        .or(just("string")
            .map(|_| "string".to_string()))
        .or(text::ident().map(|s: String| s))
        .padded()
        .labelled("tipo")
}

/// Helper inspirado en Rust: Parser robusto con mejor manejo de errores
/// Rust tiene excelente error handling - aplicamos eso aquí
pub fn parametro_parser() -> impl Parser<char, (String, String), Error = Simple<char>> + Clone {
    text::ident()
        .padded()
        .then_ignore(just(":").padded().labelled("separador ':'"))
        .then(tipo_parser())
        .labelled("parámetro (nombre: tipo)")
}

/// Helper inspirado en Zig: Parser de lista simple y directo
/// Zig usa parsers muy simples para listas - aplicamos eso aquí
pub fn lista_parametros_parser() -> impl Parser<char, Vec<(String, String)>, Error = Simple<char>> + Clone {
    parametro_parser()
        .separated_by(just(",").padded())
        .allow_trailing()
        .or_not()
        .map(|opt| opt.unwrap_or_default())
        .labelled("lista de parámetros")
}

/// Helper inspirado en Rust: Parser con delimitadores explícitos
/// Rust es muy explícito con los delimitadores - aplicamos eso aquí
pub fn parametros_delimitados_parser() -> impl Parser<char, Vec<(String, String)>, Error = Simple<char>> + Clone {
    lista_parametros_parser()
        .delimited_by(
            just("(").padded().labelled("inicio de parámetros '('"),
            just(")").padded().labelled("fin de parámetros ')'")
        )
        .labelled("parámetros entre paréntesis")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tipo_parser() {
        let parser = tipo_parser();
        
        assert_eq!(parser.parse("int64").unwrap(), "int64".to_string());
        assert_eq!(parser.parse("string").unwrap(), "string".to_string());
        assert_eq!(parser.parse("Persona").unwrap(), "Persona".to_string());
    }

    #[test]
    fn test_parametro_parser() {
        let parser = parametro_parser();
        
        let result = parser.parse("nombre: string");
        assert!(result.is_ok());
        let (nombre, tipo) = result.unwrap();
        assert_eq!(nombre, "nombre");
        assert_eq!(tipo, "string");
    }

    #[test]
    fn test_lista_parametros_parser() {
        let parser = lista_parametros_parser();
        
        // Sin parámetros
        let result = parser.parse("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
        
        // Un parámetro
        let result = parser.parse("nombre: string");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
        
        // Múltiples parámetros
        let result = parser.parse("nombre: string, edad: int64");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_parametros_delimitados_parser() {
        let parser = parametros_delimitados_parser();
        
        // Sin parámetros
        let result = parser.parse("()");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
        
        // Un parámetro
        let result = parser.parse("(nombre: string)");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
        
        // Múltiples parámetros
        let result = parser.parse("(nombre: string, edad: int64)");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }
}

