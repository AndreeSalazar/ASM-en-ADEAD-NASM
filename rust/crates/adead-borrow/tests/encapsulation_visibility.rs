use adead_borrow::BorrowChecker;
use adead_parser::parse;

#[test]
fn test_check_struct_with_public_and_private_fields() {
    let src = r#"
        struct Persona {
            pub nombre: string
            edad: int64
        }
        let p = Persona { nombre: "Juan", edad: 25 }
        let nombre = p.nombre
    "#;
    let program = parse(src).unwrap();
    let mut checker = BorrowChecker::new();
    // Debe pasar porque estamos accediendo a un campo público
    let result = checker.check(&program);
    assert!(result.is_ok(), "Acceso a campo público debe ser permitido");
}

#[test]
fn test_check_field_access_through_variable() {
    let src = r#"
        struct Banco {
            pub nombre: string
            saldo: int64
        }
        let mut banco = Banco { nombre: "Mi Banco", saldo: 0 }
        let nombre = banco.nombre
    "#;
    let program = parse(src).unwrap();
    let mut checker = BorrowChecker::new();
    let result = checker.check(&program);
    assert!(result.is_ok(), "Debe pasar: acceso a campo público");
}

#[test]
fn test_check_struct_with_methods() {
    let src = r#"
        struct Contador {
            mut valor: int64
            pub init(valor_inicial: int64) {
                self.valor = valor_inicial
            }
            pub fn incrementar(&mut self) {
                self.valor = self.valor + 1
            }
        }
        let mut c = Contador { valor: 0 }
        c.incrementar()
    "#;
    let program = parse(src).unwrap();
    let mut checker = BorrowChecker::new();
    let result = checker.check(&program);
    // Por ahora, la verificación básica debe pasar
    // La verificación completa de visibilidad entre módulos se hará cuando tengamos módulos
    assert!(result.is_ok(), "Debe pasar: acceso a método");
}

#[test]
fn test_check_multiple_structs() {
    let src = r#"
        struct A {
            pub valor_a: int64
        }
        struct B {
            pub valor_b: int64
        }
        let a = A { valor_a: 10 }
        let b = B { valor_b: 20 }
        let va = a.valor_a
        let vb = b.valor_b
    "#;
    let program = parse(src).unwrap();
    let mut checker = BorrowChecker::new();
    let result = checker.check(&program);
    assert!(result.is_ok(), "Debe pasar: múltiples structs con campos públicos");
}

#[test]
fn test_check_struct_with_init_and_destroy() {
    let src = r#"
        struct Recurso {
            id: int64
            pub init(id_val: int64) {
                self.id = id_val
            }
            destroy() {
                print "Destruyendo"
            }
        }
        let r = Recurso { id: 1 }
    "#;
    let program = parse(src).unwrap();
    let mut checker = BorrowChecker::new();
    let result = checker.check(&program);
    assert!(result.is_ok(), "Debe pasar: struct con init y destroy con diferentes visibilidades");
}

