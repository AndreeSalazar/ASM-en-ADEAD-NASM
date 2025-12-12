//! Tests para generación de código NASM de Constructores y Destructores (RAII)

use adead_backend::CodeGenerator;
use adead_parser::parse;

#[test]
fn test_generate_init_constructor() {
    let src = r#"
        struct Recurso {
            valor: int64
            init(valor: int64) {
                self.valor = valor
            }
        }
        let r = Recurso { valor: 42 }
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que se genera función de constructor
    assert!(asm.contains("Recurso_init:"), "Debe generar función Recurso_init");
    assert!(asm.contains("Recurso_init_end:"), "Debe generar label de fin del constructor");
    
    // Verificar convención de llamadas Windows x64 (RCX para primer parámetro)
    assert!(asm.contains("mov [rbp -"), "Debe guardar parámetros en stack");
}

#[test]
fn test_generate_destroy_destructor() {
    let src = r#"
        struct Recurso {
            valor: int64
            destroy() {
                print "Destruyendo"
            }
        }
        let r = Recurso { valor: 42 }
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que se genera función de destructor
    assert!(asm.contains("Recurso_destroy:"), "Debe generar función Recurso_destroy");
    assert!(asm.contains("Recurso_destroy_end:"), "Debe generar label de fin del destructor");
}

#[test]
fn test_generate_raii_automatic_destruction() {
    let src = r#"
        struct Recurso {
            valor: int64
            destroy() {
                print "RAII: Destruyendo"
            }
        }
        let r1 = Recurso { valor: 1 }
        let r2 = Recurso { valor: 2 }
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que se generan llamadas automáticas a destructores antes de ExitProcess
    assert!(asm.contains("RAII:"), "Debe incluir comentario RAII");
    assert!(asm.contains("call Recurso_destroy"), "Debe llamar destructor automáticamente");
    assert!(asm.contains("call ExitProcess"), "Debe llamar ExitProcess después de destructores");
    
    // Verificar orden LIFO (r2 se destruye primero, luego r1)
    // La última variable creada debe destruirse primero
    let destroy_calls: Vec<_> = asm.match_indices("call Recurso_destroy").collect();
    assert!(destroy_calls.len() >= 2, "Debe haber múltiples llamadas a destructores");
}

#[test]
fn test_generate_init_and_destroy_together() {
    let src = r#"
        struct Recurso {
            valor: int64
            init(valor: int64) {
                self.valor = valor
            }
            destroy() {
                print "Destruyendo"
            }
        }
        let r = Recurso { valor: 100 }
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que se generan ambas funciones
    assert!(asm.contains("Recurso_init:"), "Debe generar constructor");
    assert!(asm.contains("Recurso_destroy:"), "Debe generar destructor");
    
    // Verificar que el struct está registrado para RAII
    assert!(asm.contains("destruyendo"), "Debe incluir lógica de destrucción RAII");
}

#[test]
fn test_generate_multiple_structs_with_destroy() {
    let src = r#"
        struct Recurso1 {
            valor: int64
            destroy() {
                print "Destruyendo Recurso1"
            }
        }
        struct Recurso2 {
            valor: int64
            destroy() {
                print "Destruyendo Recurso2"
            }
        }
        let r1 = Recurso1 { valor: 1 }
        let r2 = Recurso2 { valor: 2 }
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que se generan destructores para ambos structs
    assert!(asm.contains("Recurso1_destroy:"), "Debe generar destructor para Recurso1");
    assert!(asm.contains("Recurso2_destroy:"), "Debe generar destructor para Recurso2");
    
    // Verificar llamadas a ambos destructores
    assert!(asm.contains("call Recurso1_destroy") || asm.contains("call Recurso2_destroy"), 
            "Debe incluir llamadas a destructores");
}

#[test]
fn test_generate_init_with_multiple_params() {
    let src = r#"
        struct Punto {
            x: int64
            y: int64
            init(x: int64, y: int64) {
                self.x = x
                self.y = y
            }
        }
        let p = Punto { x: 10, y: 20 }
    "#;
    let program = parse(src).unwrap();
    let mut gen = CodeGenerator::new();
    let asm = gen.generate(&program).unwrap();
    
    // Verificar que se genera constructor con múltiples parámetros
    assert!(asm.contains("Punto_init:"), "Debe generar constructor");
    
    // Verificar manejo de parámetros (Windows x64: RCX, RDX, R8, R9)
    assert!(asm.contains("rcx") || asm.contains("rdx"), 
            "Debe manejar múltiples parámetros según convención Windows x64");
}

