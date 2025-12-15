#[cfg(test)]
mod tests {
    use adead_parser::*;

    #[test]
    fn test_parse_print_number() {
        let src = r#"print 42"#;
        let program = parse(src).unwrap();
        assert_eq!(
            program.statements[0],
            Stmt::Print(Expr::Number(42))
        );
    }

    #[test]
    fn test_parse_print_arithmetic() {
        let src = r#"print 2 + 5"#;
        let program = parse(src).unwrap();
        match &program.statements[0] {
            Stmt::Print(Expr::BinaryOp { op, left, right }) => {
                assert_eq!(*op, BinOp::Add);
                assert_eq!(*left.as_ref(), Expr::Number(2));
                assert_eq!(*right.as_ref(), Expr::Number(5));
            }
            _ => panic!("Expected BinaryOp"),
        }
    }

    #[test]
    fn test_parse_print_multiplication() {
        let src = r#"print 10 * 3"#;
        let program = parse(src).unwrap();
        match &program.statements[0] {
            Stmt::Print(Expr::BinaryOp { op, left, right }) => {
                assert_eq!(*op, BinOp::Mul);
            }
            _ => panic!("Expected BinaryOp"),
        }
    }
}

