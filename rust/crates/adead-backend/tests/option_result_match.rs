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
    let asm = gen.generate(&program).unwrap();
    
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

