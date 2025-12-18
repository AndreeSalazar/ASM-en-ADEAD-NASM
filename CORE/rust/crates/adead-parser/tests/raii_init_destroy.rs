use adead_parser::parse;

#[test]
fn test_parse_struct_with_init() {
    let src = r#"
        struct Recurso {
            valor: int64
            init(valor: int64) {
                self.valor = valor
            }
        }
    "#;
    let program = parse(src).unwrap();
    if let adead_parser::Stmt::Struct { name, fields, init, destroy, .. } = &program.statements[0] {
        assert_eq!(name, "Recurso");
        assert_eq!(fields.len(), 1);
        assert!(init.is_some(), "Debe tener constructor init");
        assert!(destroy.is_none(), "No debe tener destructor");
        
        if let Some(init_method) = init {
            assert_eq!(init_method.params.len(), 1);
            assert_eq!(init_method.params[0].name, "valor");
            assert_eq!(init_method.body.len(), 1);
        }
    } else {
        panic!("Expected Struct statement");
    }
}

#[test]
fn test_parse_struct_with_destroy() {
    let src = r#"
        struct Recurso {
            valor: int64
            destroy() {
                print "Destruyendo recurso"
            }
        }
    "#;
    let program = parse(src).unwrap();
    if let adead_parser::Stmt::Struct { name, fields, init, destroy, .. } = &program.statements[0] {
        assert_eq!(name, "Recurso");
        assert_eq!(fields.len(), 1);
        assert!(init.is_none(), "No debe tener constructor");
        assert!(destroy.is_some(), "Debe tener destructor destroy");
        
        if let Some(destroy_method) = destroy {
            assert_eq!(destroy_method.params.len(), 0);
            assert_eq!(destroy_method.body.len(), 1);
        }
    } else {
        panic!("Expected Struct statement");
    }
}

#[test]
fn test_parse_struct_with_init_and_destroy() {
    let src = r#"
        struct Recurso {
            valor: int64
            init(valor: int64) {
                self.valor = valor
            }
            destroy() {
                print "Destruyendo recurso"
            }
        }
    "#;
    let program = parse(src).unwrap();
    if let adead_parser::Stmt::Struct { name, fields, init, destroy, .. } = &program.statements[0] {
        assert_eq!(name, "Recurso");
        assert_eq!(fields.len(), 1);
        assert!(init.is_some(), "Debe tener constructor init");
        assert!(destroy.is_some(), "Debe tener destructor destroy");
        
        if let Some(init_method) = init {
            assert_eq!(init_method.params.len(), 1);
        }
        
        if let Some(destroy_method) = destroy {
            assert_eq!(destroy_method.params.len(), 0);
        }
    } else {
        panic!("Expected Struct statement");
    }
}

