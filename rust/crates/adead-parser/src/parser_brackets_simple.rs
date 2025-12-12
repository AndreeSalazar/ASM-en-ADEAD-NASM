// Parser simplificado para Opción 10: Corchetes [] para parámetros de métodos
// Este archivo contiene una versión simplificada y más clara del parser
// que usa corchetes en lugar de paréntesis para métodos de struct

use chumsky::prelude::*;

/// Parser simplificado para parámetros usando corchetes []
/// Formato: [nombre is tipo, nombre2 is tipo2]
pub fn params_brackets_parser() -> impl Parser<char, Vec<(String, String)>, Error = Simple<char>> + Clone {
    let ident = text::ident().padded();
    
    let param = ident.clone()
        .then_ignore(just("is").padded())
        .then(text::ident().padded())
        .map(|(name, ty)| (name, ty));
    
    param
        .separated_by(just(",").padded())
        .allow_trailing()
        .delimited_by(just("[").padded(), just("]").padded())
}

/// Parser simplificado para métodos de struct con corchetes
/// Formato: init[params] ... end
pub fn method_brackets_parser() -> impl Parser<char, (String, Vec<(String, String)>), Error = Simple<char>> + Clone {
    let ident = text::ident().padded();
    
    let method_name = just("init")
        .padded()
        .map(|_| "init".to_string())
        .or(just("destroy")
            .padded()
            .map(|_| "destroy".to_string()));
    
    let param = ident.clone()
        .then_ignore(just("is").padded())
        .then(text::ident().padded())
        .map(|(name, ty)| (name, ty));
    
    method_name
        .then(
            param
                .separated_by(just(",").padded())
                .allow_trailing()
                .delimited_by(just("[").padded(), just("]").padded())
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_params_brackets_simple() {
        let parser = params_brackets_parser();
        let result = parser.parse("[valor is int64]");
        assert!(result.is_ok(), "Should parse: {:?}", result.err());
        let params = result.unwrap();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].0, "valor");
        assert_eq!(params[0].1, "int64");
    }

    #[test]
    fn test_params_brackets_empty() {
        let parser = params_brackets_parser();
        let result = parser.parse("[]");
        assert!(result.is_ok(), "Should parse empty: {:?}", result.err());
        let params = result.unwrap();
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_params_brackets_multiple() {
        let parser = params_brackets_parser();
        let result = parser.parse("[x is int64, y is string]");
        assert!(result.is_ok(), "Should parse multiple: {:?}", result.err());
        let params = result.unwrap();
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].0, "x");
        assert_eq!(params[0].1, "int64");
        assert_eq!(params[1].0, "y");
        assert_eq!(params[1].1, "string");
    }

    #[test]
    fn test_method_brackets_simple() {
        let parser = method_brackets_parser();
        let result = parser.parse("init[valor is int64]");
        assert!(result.is_ok(), "Should parse method: {:?}", result.err());
        let (name, params) = result.unwrap();
        assert_eq!(name, "init");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].0, "valor");
    }

    #[test]
    fn test_method_brackets_empty() {
        let parser = method_brackets_parser();
        let result = parser.parse("destroy[]");
        assert!(result.is_ok(), "Should parse empty method: {:?}", result.err());
        let (name, params) = result.unwrap();
        assert_eq!(name, "destroy");
        assert_eq!(params.len(), 0);
    }
}

