// Archivo de debugging para el problema "at 1:1"
// Este archivo nos permite probar parsers de forma aislada
// para entender qué está causando el error "Parse error at 1:1"

use chumsky::prelude::*;

/// Parser de prueba para entender el problema "at 1:1"
/// Prueba diferentes variantes de parsing de parámetros
pub fn test_parameter_parsing_variants() {
    println!("=== Testing Parameter Parsing Variants ===\n");
    
    // Test 1: Parser básico con ident simple
    println!("Test 1: Parser básico ident().padded()");
    let basic_ident = text::ident().padded();
    test_parser(&basic_ident, "nombre");
    test_parser(&basic_ident, "string");
    test_parser(&basic_ident, "int64");
    
    // Test 2: Parser de nombre + tipo básico
    println!("\nTest 2: nombre + tipo (ident + ident)");
    let param_basic = text::ident().padded()
        .then(text::ident().padded())
        .map(|(name, ty)| (name, ty));
    test_parser(&param_basic, "nombre string");
    test_parser(&param_basic, "valor int64");
    
    // Test 3: Parser con just("int64") y just("string")
    println!("\nTest 3: Parser con just() para int64 y string");
    let type_parser_keyword = just("int64")
        .map(|_| "int64".to_string())
        .or(just("string")
            .map(|_| "string".to_string()))
        .or(text::ident().map(|s: String| s))
        .padded();
    test_parser(&type_parser_keyword, "int64");
    test_parser(&type_parser_keyword, "string");
    test_parser(&type_parser_keyword, "nombre");
    
    // Test 4: Parser de parámetro con tipo específico
    println!("\nTest 4: nombre + tipo con parser específico");
    let param_with_type = text::ident().padded()
        .then(
            just("int64")
                .map(|_| "int64".to_string())
                .or(just("string")
                    .map(|_| "string".to_string()))
                .or(text::ident().map(|s: String| s))
                .padded()
        )
        .map(|(name, ty)| (name, ty));
    test_parser(&param_with_type, "nombre string");
    test_parser(&param_with_type, "valor int64");
    
    // Test 5: Parser dentro de paréntesis
    println!("\nTest 5: Parámetro dentro de paréntesis con ignore_then");
    let param_in_parens = just("(")
        .padded()
        .ignore_then(
            text::ident().padded()
                .then(text::ident().padded())
                .map(|(name, ty)| (name, ty))
        )
        .then_ignore(just(")").padded());
    test_parser(&param_in_parens, "(nombre string)");
    test_parser(&param_in_parens, "(valor int64)");
    
    // Test 6: Parser con tipo específico dentro de paréntesis
    println!("\nTest 6: Parámetro con tipo específico dentro de paréntesis");
    let param_type_in_parens = just("(")
        .padded()
        .ignore_then(
            text::ident().padded()
                .then(
                    just("int64")
                        .map(|_| "int64".to_string())
                        .or(just("string")
                            .map(|_| "string".to_string()))
                        .or(text::ident().map(|s: String| s))
                        .padded()
                )
                .map(|(name, ty)| (name, ty))
        )
        .then_ignore(just(")").padded());
    test_parser(&param_type_in_parens, "(nombre string)");
    test_parser(&param_type_in_parens, "(valor int64)");
    
    // Test 7: Parser con separated_by (múltiples parámetros)
    println!("\nTest 7: Múltiples parámetros con separated_by");
    let multiple_params = just("(")
        .padded()
        .ignore_then(
            text::ident().padded()
                .then(
                    just("int64")
                        .map(|_| "int64".to_string())
                        .or(just("string")
                            .map(|_| "string".to_string()))
                        .or(text::ident().map(|s: String| s))
                        .padded()
                )
                .map(|(name, ty)| (name, ty))
                .separated_by(just(",").padded())
                .allow_trailing()
        )
        .then_ignore(just(")").padded());
    test_parser(&multiple_params, "(nombre string)");
    test_parser(&multiple_params, "(nombre string, edad int64)");
    test_parser(&multiple_params, "()");  // Sin parámetros
    
    // Test 8: Parser completo de método init(nombre string)
    println!("\nTest 8: Parser completo de método init(nombre string)");
    let method_parser = just("init")
        .padded()
        .then(
            just("(")
                .padded()
                .ignore_then(
                    text::ident().padded()
                        .then(
                            just("int64")
                                .map(|_| "int64".to_string())
                                .or(just("string")
                                    .map(|_| "string".to_string()))
                                .or(text::ident().map(|s: String| s))
                                .padded()
                        )
                        .map(|(name, ty)| (name, ty))
                        .separated_by(just(",").padded())
                        .allow_trailing()
                        .or_not()
                        .map(|opt| opt.unwrap_or_default())
                )
                .then_ignore(just(")").padded())
        )
        .map(|(method, params)| (method, params));
    test_parser(&method_parser, "init(nombre string)");
    test_parser(&method_parser, "init()");
    test_parser(&method_parser, "init(valor int64)");
    
    println!("\n=== Tests completados ===");
}

/// Función auxiliar para probar un parser y mostrar resultados
fn test_parser<P, O>(parser: &P, input: &str)
where
    P: Parser<char, O, Error = Simple<char>>,
    O: std::fmt::Debug,
{
    match parser.parse(input) {
        Ok(result) => println!("  ✅ '{}' -> {:?}", input, result),
        Err(errs) => {
            println!("  ❌ '{}' -> ERROR", input);
            for err in errs {
                println!("      Error: {}", err);
                // Intentar obtener información sobre la posición
                let span = err.span();
                println!("      Span: {:?}", span);
                if span.start < input.len() {
                    let start = span.start.min(input.len());
                    let end = (span.end.min(input.len())).max(start);
                    println!("      Texto: '{}'", &input[start..end]);
                }
            }
        }
    }
}

/// Parser simplificado para probar el caso específico de encapsulacion.ad
pub fn test_encapsulacion_case() {
    println!("\n=== Testing caso específico de encapsulacion.ad ===\n");
    
    let input = "pub init(nombre string)";
    
    // Parser paso a paso
    println!("Input completo: '{}'", input);
    
    // Paso 1: "pub"
    println!("\nPaso 1: Parsear 'pub'");
    let pub_parser = just("pub").padded();
    test_parser(&pub_parser, "pub");
    
    // Paso 2: "init"
    println!("\nPaso 2: Parsear 'init'");
    let init_parser = just("init").padded();
    test_parser(&init_parser, "init");
    
    // Paso 3: "(nombre string)"
    println!("\nPaso 3: Parsear '(nombre string)'");
    let params_parser = just("(")
        .padded()
        .ignore_then(
            text::ident().padded()
                .then(
                    just("int64")
                        .map(|_| "int64".to_string())
                        .or(just("string")
                            .map(|_| "string".to_string()))
                        .or(text::ident().map(|s: String| s))
                        .padded()
                )
                .map(|(name, ty)| (name, ty))
        )
        .then_ignore(just(")").padded());
    test_parser(&params_parser, "(nombre string)");
    
    // Paso 4: Combinado
    println!("\nPaso 4: Combinado 'pub init(nombre string)'");
    let combined_parser = just("pub")
        .padded()
        .or_not()
        .then(just("init").padded())
        .then(
            just("(")
                .padded()
                .ignore_then(
                    text::ident().padded()
                        .then(
                            just("int64")
                                .map(|_| "int64".to_string())
                                .or(just("string")
                                    .map(|_| "string".to_string()))
                                .or(text::ident().map(|s: String| s))
                                .padded()
                        )
                        .map(|(name, ty)| (name, ty))
                        .separated_by(just(",").padded())
                        .allow_trailing()
                        .or_not()
                        .map(|opt| opt.unwrap_or_default())
                )
                .then_ignore(just(")").padded())
        );
    test_parser(&combined_parser, input);
    
    // Prueba con el input real del archivo (línea 4)
    println!("\nPaso 5: Input real del archivo");
    let real_input = "    pub init(nombre string)";
    test_parser(&combined_parser, real_input.trim());
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_all_variants() {
        test_parameter_parsing_variants();
        test_encapsulacion_case();
    }
}

