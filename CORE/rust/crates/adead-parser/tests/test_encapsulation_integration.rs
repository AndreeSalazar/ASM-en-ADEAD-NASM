/// Tests de integración para encapsulación
/// Verifica que el parser puede manejar casos más complejos de encapsulación

use adead_parser::{parse, Stmt, Visibility, Expr};

#[test]
fn test_parse_struct_with_methods_and_fields_visibility() {
    let src = r#"
        struct Banco {
            pub nombre: string
            saldo: int64
            
            pub init(nombre: string) {
                self.nombre = nombre
                self.saldo = 0
            }
            
            pub fn depositar(&mut self, monto: int64) {
                self.saldo = self.saldo + monto
            }
            
            fn obtener_saldo(&self) -> int64 {
                return self.saldo
            }
            
            destroy() {
                print "Cerrando"
            }
        }
    "#;
    let program = parse(src).unwrap();
    if let Stmt::Struct { name, fields, init, destroy, .. } = &program.statements[0] {
        assert_eq!(name, "Banco");
        
        // Verificar campos
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name, "nombre");
        assert_eq!(fields[0].visibility, Visibility::Public);
        assert_eq!(fields[1].name, "saldo");
        assert_eq!(fields[1].visibility, Visibility::Private);
        
        // Verificar init
        assert!(init.is_some());
        if let Some(init_method) = init {
            assert_eq!(init_method.visibility, Visibility::Public);
        }
        
        // Verificar destroy (debe ser privado por defecto)
        assert!(destroy.is_some());
        if let Some(destroy_method) = destroy {
            assert_eq!(destroy_method.visibility, Visibility::Private);
        }
    } else {
        panic!("Expected Struct statement");
    }
}

#[test]
fn test_parse_struct_literal_with_public_fields() {
    let src = r#"
        struct Persona {
            pub nombre: string
            edad: int64
        }
        let p = Persona {
            nombre: "Juan",
            edad: 25
        }
        let nombre = p.nombre
    "#;
    let program = parse(src).unwrap();
    
    // Verificar que el struct se parseó
    if let Stmt::Struct { .. } = &program.statements[0] {
        // OK
    } else {
        panic!("Expected Struct statement");
    }
    
    // Verificar el literal de struct
    if let Stmt::Let { value, .. } = &program.statements[1] {
        if let Expr::StructLiteral { name, fields } = value {
            assert_eq!(name, "Persona");
            assert_eq!(fields.len(), 2);
        } else {
            panic!("Expected StructLiteral");
        }
    } else {
        panic!("Expected Let statement");
    }
    
    // Verificar acceso a campo
    if let Stmt::Let { value, .. } = &program.statements[2] {
        if let Expr::FieldAccess { object, field } = value {
            if let Expr::Ident(var_name) = object.as_ref() {
                assert_eq!(var_name, "p");
            } else {
                panic!("Expected Ident in FieldAccess");
            }
            assert_eq!(field, "nombre");
        } else {
            panic!("Expected FieldAccess");
        }
    } else {
        panic!("Expected Let statement");
    }
}

#[test]
fn test_parse_nested_struct_with_visibility() {
    let src = r#"
        struct Direccion {
            pub calle: string
            numero: int64
        }
        struct Persona {
            pub nombre: string
            direccion: Direccion
        }
        let dir = Direccion { calle: "Av. Principal", numero: 123 }
        let p = Persona { nombre: "Juan", direccion: dir }
        let calle = p.direccion.calle
    "#;
    let program = parse(src).unwrap();
    
    // Debe parsear sin errores
    assert_eq!(program.statements.len(), 5);
    
    // Verificar primer struct
    if let Stmt::Struct { name: name1, fields: fields1, .. } = &program.statements[0] {
        assert_eq!(name1, "Direccion");
        assert_eq!(fields1[0].visibility, Visibility::Public);
        assert_eq!(fields1[1].visibility, Visibility::Private);
    }
    
    // Verificar segundo struct
    if let Stmt::Struct { name: name2, fields: fields2, .. } = &program.statements[1] {
        assert_eq!(name2, "Persona");
        assert_eq!(fields2[0].visibility, Visibility::Public);
        assert_eq!(fields2[1].visibility, Visibility::Private);
    }
}

