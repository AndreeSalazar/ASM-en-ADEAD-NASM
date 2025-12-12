use adead_common::{ADeadError, Result};
use chumsky::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64),
    String(String),
    Ident(String),
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Assign {
        name: String,
        value: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Print(Expr),
    Let {
        name: String,
        value: Expr,
    },
    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Fn {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Expr(Expr),
    Return(Option<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

pub fn parse(source: &str) -> Result<Program> {
    let parser = program_parser();
    match parser.parse(source) {
        Ok(program) => Ok(program),
        Err(errs) => {
            let first = errs.first().unwrap();
            // chumsky 0.9 uses usize for spans, approximate line/col
            let (line, col) = (1, 1); // Simplified for MVP
            Err(ADeadError::ParseError {
                line,
                col,
                message: format!("{}", first),
            })
        }
    }
}

fn program_parser() -> impl Parser<char, Program, Error = Simple<char>> {
    stmt_parser()
        .repeated()
        .then_ignore(end())
        .map(|stmts| Program {
            statements: stmts,
        })
}

fn stmt_parser() -> impl Parser<char, Stmt, Error = Simple<char>> + Clone {
    recursive(|stmt| {
        let ident = text::ident().padded();
        let expr = expr_parser();

        let print = just("print")
            .padded()
            .ignore_then(expr.clone())
            .map(Stmt::Print);

        let let_stmt = just("let")
            .padded()
            .ignore_then(ident.clone())
            .then_ignore(just("=").padded())
            .then(expr.clone())
            .map(|(name, value)| Stmt::Let { name, value });

        let return_stmt = just("return")
            .padded()
            .ignore_then(expr.clone().or_not())
            .map(Stmt::Return);

        let if_stmt = just("if")
            .padded()
            .ignore_then(expr.clone())
            .then(
                just("{")
                    .padded()
                    .ignore_then(stmt.clone().repeated())
                    .then_ignore(just("}").padded()),
            )
            .then(
                just("else")
                    .padded()
                    .ignore_then(
                        just("{")
                            .padded()
                            .ignore_then(stmt.clone().repeated())
                            .then_ignore(just("}").padded()),
                    )
                    .or_not(),
            )
            .map(|((condition, then_body), else_body)| Stmt::If {
                condition,
                then_body,
                else_body,
            });

        let while_stmt = just("while")
            .padded()
            .ignore_then(expr.clone())
            .then(
                just("{")
                    .padded()
                    .ignore_then(stmt.clone().repeated())
                    .then_ignore(just("}").padded()),
            )
            .map(|(condition, body)| Stmt::While {
                condition,
                body,
            });

        let fn_stmt = just("fn")
            .padded()
            .ignore_then(ident.clone())
            .then(
                just("(")
                    .padded()
                    .ignore_then(
                        ident
                            .clone()
                            .separated_by(just(",").padded())
                            .allow_trailing(),
                    )
                    .then_ignore(just(")").padded()),
            )
            .then(
                just("{")
                    .padded()
                    .ignore_then(stmt.clone().repeated())
                    .then_ignore(just("}").padded()),
            )
            .map(|((name, params), body)| Stmt::Fn {
                name,
                params,
                body,
            });

        // Assignment: ident = expr (as statement)
        let assign_stmt = ident
            .clone()
            .then_ignore(just("=").padded())
            .then(expr.clone())
            .map(|(name, value)| Stmt::Expr(Expr::Assign {
                name,
                value: Box::new(value),
            }));

        let expr_stmt = expr.map(Stmt::Expr);

        print
            .or(let_stmt)
            .or(if_stmt)
            .or(while_stmt)
            .or(fn_stmt)
            .or(return_stmt)
            .or(assign_stmt)
            .or(expr_stmt)
            .padded()
    })
}

fn expr_parser() -> impl Parser<char, Expr, Error = Simple<char>> + Clone {
    recursive(|expr| {
        let number = text::int(10)
            .map(|s: String| s.parse::<i64>().unwrap())
            .map(Expr::Number)
            .labelled("number");

        let string = just('"')
            .ignore_then(none_of('"').repeated())
            .then_ignore(just('"'))
            .collect::<String>()
            .map(Expr::String)
            .labelled("string");

        let ident = text::ident().map(Expr::Ident).labelled("identifier");

        let atom = number
            .or(string)
            .or(ident.clone())
            .or(expr
                .clone()
                .delimited_by(just("(").padded(), just(")").padded()));

        let call = ident
            .clone()
            .then(
                just("(")
                    .padded()
                    .ignore_then(
                        expr.clone()
                            .separated_by(just(",").padded())
                            .allow_trailing(),
                    )
                    .then_ignore(just(")").padded())
                    .or_not(),
            )
            .map(|(callee, args)| match callee {
                Expr::Ident(name) => Expr::Call {
                    name,
                    args: args.unwrap_or_default(),
                },
                _ => unreachable!(),
            })
            .or(atom);

        let unary = call;

        let product = unary
            .clone()
            .then(
                just("*")
                    .to(BinOp::Mul)
                    .or(just("/").to(BinOp::Div))
                    .then(unary.clone())
                    .repeated(),
            )
            .foldl(|l, (op, r)| Expr::BinaryOp {
                op,
                left: Box::new(l),
                right: Box::new(r),
            });

        let sum = product
            .clone()
            .then(
                just("+")
                    .to(BinOp::Add)
                    .or(just("-").to(BinOp::Sub))
                    .then(product.clone())
                    .repeated(),
            )
            .foldl(|l, (op, r)| Expr::BinaryOp {
                op,
                left: Box::new(l),
                right: Box::new(r),
            });

        let comparison = sum
            .clone()
            .then(
                just("==")
                    .to(BinOp::Eq)
                    .or(just("!=").to(BinOp::Ne))
                    .or(just("<=").to(BinOp::Le))
                    .or(just(">=").to(BinOp::Ge))
                    .or(just("<").to(BinOp::Lt))
                    .or(just(">").to(BinOp::Gt))
                    .then(sum.clone())
                    .repeated(),
            )
            .foldl(|l, (op, r)| Expr::BinaryOp {
                op,
                left: Box::new(l),
                right: Box::new(r),
            });

        comparison
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_print() {
        let src = r#"print "Hola Mundo""#;
        let program = parse(src).unwrap();
        assert_eq!(
            program.statements,
            vec![Stmt::Print(Expr::String("Hola Mundo".to_string()))]
        );
    }

    #[test]
    fn test_parse_let() {
        let src = r#"let x = 42"#;
        let program = parse(src).unwrap();
        assert_eq!(
            program.statements,
            vec![Stmt::Let {
                name: "x".to_string(),
                value: Expr::Number(42)
            }]
        );
    }

    #[test]
    fn test_parse_if() {
        let src = r#"
            if 5 > 3 {
                print "yes"
            }
        "#;
        let program = parse(src).unwrap();
        assert!(matches!(&program.statements[0], Stmt::If { .. }));
    }
}

