#[cfg(test)]
mod tests {
    use adead_parser::parse;
    use adead_parser::{Stmt, StructField, StructMethod, Visibility, FnParam, BorrowType};

    #[test]
    fn test_struct_with_method_brackets() {
        let source = r#"
struct Test
    valor is int64
    init[valor is int64]
        self.valor = valor
    end
end
"#;
        
        let result = parse(source);
        assert!(result.is_ok(), "Should parse successfully: {:?}", result.err());
        
        let program = result.unwrap();
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Stmt::Struct { name, fields, init, destroy } => {
                assert_eq!(name, "Test");
                assert_eq!(fields.len(), 1);
                assert!(init.is_some());
                assert!(destroy.is_none());
                
                // Verificar campo
                let field = &fields[0];
                assert_eq!(field.name, "valor");
                
                // Verificar init
                let init_method = init.as_ref().unwrap();
                assert_eq!(init_method.params.len(), 1);
                assert_eq!(init_method.params[0].name, "valor");
                assert_eq!(init_method.params[0].borrow_type, BorrowType::Owned);
            }
            _ => panic!("Expected Struct statement"),
        }
    }

    #[test]
    fn test_method_empty_brackets() {
        let source = r#"
struct Test
    valor is int64
    destroy[]
        print "destroyed"
    end
end
"#;
        
        let result = parse(source);
        assert!(result.is_ok(), "Should parse successfully: {:?}", result.err());
        
        let program = result.unwrap();
        match &program.statements[0] {
            Stmt::Struct { destroy, .. } => {
                assert!(destroy.is_some());
                let destroy_method = destroy.as_ref().unwrap();
                assert_eq!(destroy_method.params.len(), 0);
            }
            _ => panic!("Expected Struct statement"),
        }
    }

    #[test]
    fn test_method_multiple_params() {
        let source = r#"
struct Test
    valor is int64
    init[x is int64, y is int64]
        self.valor = x
    end
end
"#;
        
        let result = parse(source);
        assert!(result.is_ok(), "Should parse successfully: {:?}", result.err());
        
        let program = result.unwrap();
        match &program.statements[0] {
            Stmt::Struct { init, .. } => {
                assert!(init.is_some());
                let init_method = init.as_ref().unwrap();
                assert_eq!(init_method.params.len(), 2);
                assert_eq!(init_method.params[0].name, "x");
                assert_eq!(init_method.params[1].name, "y");
            }
            _ => panic!("Expected Struct statement"),
        }
    }
}

