use adead_parser::{parse, Stmt, Visibility, Expr};

#[test]
fn test_parse_struct_with_public_fields() {
    let src = r#"
        struct Persona {
            pub nombre: string
            edad: int64
        }
    "#;
    let program = parse(src).unwrap();
    if let adead_parser::Stmt::Struct { fields, .. } = &program.statements[0] {
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name, "nombre");
        assert_eq!(fields[0].visibility, Visibility::Public);  // Campo público
        assert_eq!(fields[1].name, "edad");
        assert_eq!(fields[1].visibility, Visibility::Private);  // Campo privado (por defecto)
    } else {
        panic!("Expected Struct statement");
    }
}

#[test]
fn test_parse_struct_all_private_by_default() {
    let src = r#"
        struct Persona {
            nombre: string
            edad: int64
        }
    "#;
    let program = parse(src).unwrap();
    if let adead_parser::Stmt::Struct { fields, .. } = &program.statements[0] {
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].visibility, Visibility::Private);  // Privado por defecto
        assert_eq!(fields[1].visibility, Visibility::Private);  // Privado por defecto
    } else {
        panic!("Expected Struct statement");
    }
}

#[test]
fn test_parse_struct_with_public_init() {
    let src = r#"
        struct Recurso {
            valor: int64
            pub init(valor: int64) {
                self.valor = valor
            }
        }
    "#;
    let program = parse(src).unwrap();
    if let adead_parser::Stmt::Struct { init, .. } = &program.statements[0] {
        assert!(init.is_some());
        if let Some(init_method) = init {
            assert_eq!(init_method.visibility, Visibility::Public);
        }
    } else {
        panic!("Expected Struct statement");
    }
}

#[test]
fn test_parse_struct_with_private_destroy() {
    let src = r#"
        struct Recurso {
            valor: int64
            destroy() {
                print "Destruyendo"
            }
        }
    "#;
    let program = parse(src).unwrap();
    if let adead_parser::Stmt::Struct { destroy, .. } = &program.statements[0] {
        assert!(destroy.is_some());
        if let Some(destroy_method) = destroy {
            assert_eq!(destroy_method.visibility, Visibility::Private);  // Privado por defecto
        }
    } else {
        panic!("Expected Struct statement");
    }
}

#[test]
fn test_parse_struct_mixed_visibility() {
    let src = r#"
        struct Banco {
            pub nombre: string
            saldo: int64
            pub init(nombre: string) {
                self.nombre = nombre
                self.saldo = 0
            }
            destroy() {
                print "Cerrando cuenta"
            }
        }
    "#;
    let program = parse(src).unwrap();
    if let adead_parser::Stmt::Struct { fields, init, destroy, .. } = &program.statements[0] {
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].visibility, Visibility::Public);  // nombre es público
        assert_eq!(fields[1].visibility, Visibility::Private);  // saldo es privado
        
        assert!(init.is_some());
        if let Some(init_method) = init {
            assert_eq!(init_method.visibility, Visibility::Public);
        }
        
        assert!(destroy.is_some());
        if let Some(destroy_method) = destroy {
            assert_eq!(destroy_method.visibility, Visibility::Private);
        }
    } else {
        panic!("Expected Struct statement");
    }
}

