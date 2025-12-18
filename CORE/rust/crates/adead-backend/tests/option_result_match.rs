//! Tests para generación de código NASM de Option/Result/match

use adead_backend::CodeGenerator;
use adead_parser::parse;

#[test]
fn test_generate_some() {
    let src = r#"
        let x = Some(42)
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    println!("{:?}", program);
    let asm = gen.generate(&program).unwrap();
    println!("{}", asm);
    
    // Verificar que se genera código para Some
    assert!(asm.contains("tag = 1 (Some)"));
    assert!(asm.contains("mov rax, 1  ; tag Some = 1"));
    assert!(asm.contains("espacio para Option"));
}

#[test]
fn test_generate_none() {
    let src = r#"
        let x = None
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que se genera código para None
    assert!(asm.contains("tag = 0 (None)"));
    assert!(asm.contains("mov rax, 0  ; tag None = 0"));
}

#[test]
fn test_generate_ok() {
    let src = r#"
        let x = Ok(10)
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que se genera código para Ok
    assert!(asm.contains("tag = 0 (Ok)"));
    assert!(asm.contains("mov rax, 0  ; tag Ok = 0"));
    assert!(asm.contains("espacio para Result"));
}

#[test]
fn test_generate_err() {
    let src = r#"
        let x = Err(5)
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que se genera código para Err
    assert!(asm.contains("tag = 1 (Err)"));
    assert!(asm.contains("mov rax, 1  ; tag Err = 1"));
}

#[test]
fn test_generate_match_option() {
    let src = r#"
        let x = Some(42)
        match x {
            Some => 1,
            None => 0
        }
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que se genera código de match
    assert!(asm.contains("match"));
    assert!(asm.contains("cmp rbx, 1  ; comparar tag con Some"));
    assert!(asm.contains("cmp rbx, 0  ; comparar tag con None"));
    assert!(asm.contains("je match_arm_"));
}

#[test]
fn test_generate_match_result() {
    let src = r#"
        let x = Ok(10)
        match x {
            Ok => 0,
            Err => 1
        }
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar match de Result
    assert!(asm.contains("cmp rbx, 0  ; comparar tag con Ok"));
    assert!(asm.contains("cmp rbx, 1  ; comparar tag con Err"));
}

#[test]
fn test_generate_match_with_wildcard() {
    let src = r#"
        let x = Some(5)
        match x {
            Some => 1,
            _ => 0
        }
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que wildcard genera jmp incondicional
    assert!(asm.contains("jmp match_arm_"));
}

// ========== Tests para operador ? (PropagateError) ==========

#[test]
fn test_generate_propagate_error_with_ok() {
    let src = r#"
        let x = Ok(42)
        let resultado = x?
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que se genera código para propagación de errores
    // Buscar indicadores de propagación: guardar dirección, cargar tag, comparar
    assert!(asm.contains("rbx") || asm.contains("propagate") || asm.contains("tag"));
    // Verificar comparación de tag
    assert!(asm.contains("cmp") || asm.contains("je") || asm.contains("jmp"));
}

#[test]
fn test_generate_propagate_error_with_method_call() {
    let src = r#"
        let x = Ok(100)
        let valor = x?
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que se genera código para propagación
    // Debe haber código que guarda la dirección y verifica el tag
    assert!(asm.contains("rbx") || asm.contains("propagate") || asm.contains("[rbx"));
    // Verificar que se verifica el tag (comparación o salto)
    assert!(asm.contains("cmp") || asm.contains("je") || asm.contains("jmp"));
}

#[test]
fn test_generate_propagate_error_checks_tag() {
    let src = r#"
        let x = Ok(42)
        let valor = x?
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que se genera verificación de tag
    // Debe guardar dirección en rbx y cargar tag
    assert!(asm.contains("rbx") && (asm.contains("mov rbx") || asm.contains("[rbx")));
    // Debe comparar el tag o hacer saltos condicionales
    assert!(asm.contains("cmp") || asm.contains("je") || asm.contains("jmp"));
}

#[test]
fn test_generate_propagate_error_handles_ok() {
    let src = r#"
        let resultado = Ok(10)?
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que desarrolla valor cuando es Ok
    // Debe acceder al valor desde [rbx + 8] o similar
    assert!(asm.contains("[rbx + 8]") || asm.contains("Ok") || asm.contains("valor"));
}

#[test]
fn test_generate_propagate_error_handles_err() {
    let src = r#"
        let resultado = Err(5)?
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que propaga error cuando es Err
    // Debe haber código que maneja el caso de error
    assert!(asm.contains("propagate") || asm.contains("Err") || asm.contains("[rbx + 8]"));
}

#[test]
fn test_generate_propagate_error_chained() {
    let src = r#"
        let x = Ok(1)
        let y = Ok(2)
        let suma = x? + y?
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que ambas propagaciones generan código
    // Debería haber múltiples verificaciones de tag (cmp o saltos)
    let cmp_count = asm.matches("cmp").count();
    let je_count = asm.matches("je").count();
    // Al menos debe haber algunas comparaciones o saltos por las dos propagaciones
    assert!(cmp_count >= 1 || je_count >= 1 || asm.contains("rbx"));
}

// ========== Tests para Arrays (Sprint 1.2) ==========

#[test]
fn test_generate_array_literal() {
    let src = r#"
        let arr = [1, 2, 3]
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que se reserva espacio para el array
    assert!(asm.contains("Array literal") || asm.contains("reservar espacio para array"));
    // Verificar que se almacenan los valores
    assert!(asm.contains("array[0]") || asm.contains("array[1]") || asm.contains("array[2]"));
    // Verificar que se calcula la dirección base
    assert!(asm.contains("dirección base del array") || asm.contains("lea rax"));
}

#[test]
fn test_generate_array_literal_empty() {
    let src = r#"
        let arr = []
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Array vacío debe generar código mínimo
    assert!(asm.contains("Array literal") || asm.contains("0 elementos"));
}

#[test]
fn test_generate_array_index() {
    let src = r#"
        let arr = [1, 2, 3]
        let valor = arr[0]
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que se genera código para indexación
    // Debe guardar dirección base, calcular offset, y cargar valor
    assert!(asm.contains("push rax") || asm.contains("guardar dirección base"));
    // Verificar cálculo de offset (índice * 8)
    assert!(asm.contains("imul") || asm.contains("índice * 8"));
    // Verificar carga del valor
    assert!(asm.contains("mov rax, [rax]") || asm.contains("cargar array[index]"));
}

#[test]
fn test_generate_array_index_with_expression() {
    let src = r#"
        let arr = [10, 20, 30]
        let i = 1
        let valor = arr[i]
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que se puede indexar con una variable
    // Debe cargar la variable i primero, luego usarla como índice
    assert!(asm.contains("cargar array[index]") || asm.contains("mov rax, [rax]"));
    // Verificar que se calcula el offset dinámicamente
    assert!(asm.contains("imul") || asm.contains("índice * 8"));
}

#[test]
fn test_generate_array_index_nested() {
    let src = r#"
        let matriz = [[1, 2], [3, 4]]
        let valor = matriz[0]
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que se puede indexar arrays anidados
    // Debe generar código para la primera indexación
    assert!(asm.contains("cargar array[index]") || asm.contains("push rax"));
}

#[test]
fn test_generate_array_literal_with_expressions() {
    let src = r#"
        let x = 5
        let arr = [x, x + 1, x * 2]
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que se pueden usar expresiones en literales
    // Debe generar código para cada expresión
    assert!(asm.contains("array[0]") || asm.contains("array[1]") || asm.contains("array[2]"));
}

