use adead_common::{ADeadError, Result};
use chumsky::prelude::*;

// ZIG ES EL PARSER PRINCIPAL - Rust solo hace seguridad y codegen
// Módulo FFI para parser Zig (PRINCIPAL)
mod zig_ffi_parser;

// Parser Rust solo como último recurso si Zig falla
mod zig_struct_parser;

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
    // Ownership y Borrowing (O0.2)
    Borrow {
        expr: Box<Expr>,
        mutable: bool,  // false = &T, true = &mut T
    },
    Deref(Box<Expr>),  // *expr para dereferenciar
    
    // Option y Result Types (O0.4)
    Some(Box<Expr>),           // Some(value)
    None,                      // None (para Option)
    Ok(Box<Expr>),             // Ok(value)
    Err(Box<Expr>),            // Err(error)
    Match {                     // match expr { pattern => body, ... }
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    // Structs/Clases (Fase 1.2 - O1)
    StructLiteral {             // StructName { field1: value1, field2: value2 }
        name: String,
        fields: Vec<(String, Expr)>,  // (field_name, value)
    },
    FieldAccess {               // expr.field_name
        object: Box<Expr>,
        field: String,
    },
    MethodCall {                // expr.method_name(args)
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
}

/// Par├ímetro de funci├│n con informaci├│n de borrowing
#[derive(Debug, Clone, PartialEq)]
pub struct FnParam {
    pub name: String,
    pub borrow_type: BorrowType,  // Tipo de borrowing del par├ímetro
}

/// Tipo de borrowing para par├ímetros de funci├│n
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorrowType {
    Owned,      // Valor owned (por defecto)
    Borrowed,   // &T - referencia inmutable
    MutBorrowed, // &mut T - referencia mutable
}

/// Patr├│n para match expressions (O0.4)
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Some,       // Some(_)
    None,       // None
    Ok,         // Ok(_)
    Err,        // Err(_)
    Ident(String), // Variable binding: x
    LiteralNumber(i64), // 42
    LiteralString(String), // "hello"
    Wildcard,   // _ (catch-all)
}

/// Brazo de match expression (O0.4)
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Box<Expr>,
}

/// Nivel de visibilidad (O5 - Encapsulaci├│n)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Visibility {
    Private,  // Privado (por defecto) - solo visible en el m├│dulo actual
    Public,   // P├║blico - visible desde cualquier lugar
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Private  // Privado por defecto (m├ís seguro)
    }
}

/// Campo de struct (Fase 1.2 - O3, O5 - Encapsulaci├│n)
#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub visibility: Visibility,  // O5 - Visibilidad del campo
    pub mutable: bool,  // true = mut field, false = inmutable (por defecto)
    pub name: String,
    pub ty: Option<String>,  // Tipo opcional (None = inferido)
}

/// M├®todo de struct (O2 - Constructores y Destructores, O5 - Encapsulaci├│n)
#[derive(Debug, Clone, PartialEq)]
pub struct StructMethod {
    pub visibility: Visibility,  // O5 - Visibilidad del m├®todo
    pub params: Vec<FnParam>,  // Par├ímetros del m├®todo
    pub body: Vec<Stmt>,        // Cuerpo del m├®todo
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
        mutable: bool,  // true = let mut, false = let (inmutable)
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
        params: Vec<FnParam>,  // Cambiado para soportar borrowing
        body: Vec<Stmt>,
    },
    // Structs/Clases (Fase 1.2 - O1, O2 - RAII)
    Struct {
        name: String,
        fields: Vec<StructField>,
        init: Option<StructMethod>,      // Constructor (O2)
        destroy: Option<StructMethod>,    // Destructor (O2.1 - Drop Trait)
    },
    Expr(Expr),
    Return(Option<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

pub fn parse(source: &str) -> Result<Program> {
    // PRE-PROCESADOR: Extraer structs del input antes del parsing principal
    // Esto evita problemas con take_until que no funciona correctamente dentro del parser
    
    let (preprocessed_source, extracted_structs) = preprocess_extract_structs(source)?;
    
    let parser = program_parser();
    match parser.parse(preprocessed_source.as_str()) {
        Ok(mut program) => {
            // Insertar los structs extraídos al inicio del programa
            for stmt in extracted_structs {
                program.statements.insert(0, stmt);
            }
            Ok(program)
        }
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

/// Pre-procesador: Extrae structs del input y los reemplaza con marcadores
fn preprocess_extract_structs(source: &str) -> Result<(String, Vec<Stmt>)> {
    let mut result = String::new();
    let mut extracted_structs = Vec::new();
    
    // Convertir a bytes para trabajar con posiciones
    let source_bytes = source.as_bytes();
    let mut byte_pos = 0;
    
    while byte_pos < source_bytes.len() {
        // Buscar "struct " usando búsqueda de bytes
        if let Some(struct_start_byte) = source[byte_pos..].find("struct ") {
            let abs_start_byte = byte_pos + struct_start_byte;
            
            // Verificar que "struct" está al inicio de línea o después de whitespace
            let is_at_start = abs_start_byte == 0;
            let is_after_newline = abs_start_byte > 0 && {
                let before_bytes = &source_bytes[..abs_start_byte];
                if let Some(last_char) = source[..abs_start_byte].chars().last() {
                    last_char == '\n' || last_char == '\r'
                } else {
                    false
                }
            };
            
            if is_at_start || is_after_newline {
                // Encontramos un struct, intentar extraerlo usando posición de byte
                match extract_struct(source, abs_start_byte) {
                    Ok((struct_stmt, _struct_content, end_byte_pos)) => {
                        extracted_structs.push(struct_stmt);
                        // Copiar todo hasta el struct
                        result.push_str(&source[byte_pos..abs_start_byte]);
                        // Reemplazar el struct con una línea vacía (para mantener estructura)
                        result.push_str("\n");
                        // Continuar después del struct
                        byte_pos = end_byte_pos;
                        continue;
                    }
                    Err(_) => {
                        // Si falla, dejar el struct en el input y continuar
                        result.push_str(&source[byte_pos..=abs_start_byte]);
                        byte_pos = abs_start_byte + 1;
                        continue;
                    }
                }
            }
        }
        
        // No encontramos struct desde esta posición, avanzar
        if byte_pos < source_bytes.len() {
            // Copiar el byte actual como carácter
            if let Some(ch) = source[byte_pos..].chars().next() {
                result.push(ch);
                byte_pos += ch.len_utf8();
            } else {
                byte_pos += 1;
            }
        } else {
            break;
        }
    }
    
    // Si no encontramos structs, devolver el input original
    if extracted_structs.is_empty() {
        Ok((source.to_string(), Vec::new()))
    } else {
        Ok((result, extracted_structs))
    }
}

/// Extraer un struct del input, retornando el Stmt parseado, el contenido, y la posición final (en bytes)
fn extract_struct(source: &str, start_byte_pos: usize) -> Result<(Stmt, String, usize)> {
    let rest = &source[start_byte_pos..];
    
    // Buscar "struct " seguido del nombre
    if !rest.starts_with("struct ") {
        return Err(ADeadError::ParseError {
            line: 1,
            col: 1,
            message: "Expected 'struct'".to_string(),
        });
    }
    
    // Encontrar el nombre del struct (hasta whitespace o nueva línea)
    let name_start = 7; // "struct ".len()
    let name_end = rest[name_start..]
        .find(|c: char| c.is_whitespace() || c == '\n')
        .unwrap_or(rest.len() - name_start);
    
    let struct_name = rest[name_start..name_start + name_end].trim().to_string();
    if struct_name.is_empty() {
        return Err(ADeadError::ParseError {
            line: 1,
            col: 1,
            message: "Expected struct name".to_string(),
        });
    }
    
    // Buscar el "end" final del struct
    // Estrategia: buscar todos los "end" y encontrar el que está seguido de un keyword
    let keywords = ["print", "let", "if", "while", "fn", "struct", "return"];
    let mut search_pos = name_start + name_end;
    let mut last_valid_end_pos = None;
    
    while let Some(pos) = rest[search_pos..].find("end") {
        let abs_pos = search_pos + pos;
        let after_end = abs_pos + 3;
        
        if after_end < rest.len() {
            let after_end_str = &rest[after_end..];
            let trimmed = after_end_str.trim_start_matches([' ', '\t']);
            
            // Verificar si viene nueva línea seguida de keyword o fin de input
            if trimmed.starts_with('\n') || trimmed.starts_with("\r\n") {
                let after_nl = trimmed.trim_start_matches(['\n', '\r']);
                let after_nl_trimmed = after_nl.trim_start();
                
                // Verificar si viene un keyword o es fin de input
                let is_final = if after_nl_trimmed.is_empty() {
                    true // Fin de input
                } else {
                    keywords.iter().any(|kw| after_nl_trimmed.starts_with(kw))
                };
                
                if is_final {
                    last_valid_end_pos = Some(abs_pos);
                    break; // Encontramos el final
                }
            }
        }
        
        search_pos = abs_pos + 3;
        if search_pos >= rest.len() {
            break;
        }
    }
    
    let end_pos = last_valid_end_pos.ok_or_else(|| ADeadError::ParseError {
        line: 1,
        col: 1,
        message: "No se encontró 'end' final para el struct".to_string(),
    })?;
    
    // Extraer el contenido del struct completo (incluyendo el "end" final)
    // rest ya contiene desde "struct Nombre" hasta después del último "end"
    let full_struct = rest[..end_pos + 3].to_string(); // +3 para incluir "end"
    
    // ZIG ES EL PARSER PRINCIPAL - Intentar parsear con Zig primero
    use zig_ffi_parser::parse_struct_with_zig_ffi;
    if let Ok(stmt) = parse_struct_with_zig_ffi(&full_struct) {
        let end_byte_pos = start_byte_pos + end_pos + 3;
        return Ok((stmt, full_struct, end_byte_pos));
    }
    
    // Fallback: Solo si Zig falla completamente, usar parser Rust
    // (Esto no debería pasar si Zig está bien configurado)
    match zig_struct_parser::parse_struct_from_string(&full_struct) {
        Ok((parsed_name, fields, init, destroy)) => {
            let stmt = Stmt::Struct {
                name: parsed_name,
                fields,
                init,
                destroy,
            };
            // Convertir la posición relativa a absoluta en bytes
            // end_pos es relativo a 'rest', así que sumamos start_byte_pos
            // +3 para incluir "end"
            let end_byte_pos = start_byte_pos + end_pos + 3;
            Ok((stmt, full_struct, end_byte_pos))
        }
        Err(e) => Err(ADeadError::ParseError {
            line: 1,
            col: 1,
            message: format!("Error parseando struct: {}", e),
        })
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
            .then(just("mut").padded().or_not())  // Opcional "mut"
            .then(ident.clone())
            .then_ignore(just("=").padded())
            .then(expr.clone())
            .map(|(((_, mutable), name), value)| Stmt::Let {
                mutable: mutable.is_some(),  // true si hay "mut", false si no
                name,
                value,
            });

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

        // Parser para par├ímetros de funci├│n (soporta borrowing)
        let fn_param = just("&")
            .padded()
            .then(just("mut").padded().or_not())
            .then(ident.clone())
            .map(|((_, mutable), name)| FnParam {
                name,
                borrow_type: if mutable.is_some() {
                    BorrowType::MutBorrowed
                } else {
                    BorrowType::Borrowed
                },
            })
            .or(ident.clone().map(|name| FnParam {
                name,
                borrow_type: BorrowType::Owned,
            }));

        let fn_stmt = just("fn")
            .padded()
            .ignore_then(ident.clone())
            .then(
                just("(")
                    .padded()
                    .ignore_then(
                        fn_param
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

        // Struct definition (Fase 1.2 - O1, O5 - Encapsulaci├│n)
        let struct_field = just("pub")
            .padded()
            .or_not()
            .then(just("mut").padded().or_not())
            .then(ident.clone())
            .then(
                just(":")
                    .padded()
                    .ignore_then(text::ident())
                    .or_not(),
            )
            .map(|(((visibility, mutable), name), ty)| StructField {
                visibility: if visibility.is_some() { Visibility::Public } else { Visibility::Private },
                mutable: mutable.is_some(),
                name,
                ty,
            });

        // Parser para m├®todos de struct (init, destroy) - O5: soporta pub
        let struct_method = just("pub")
            .padded()
            .or_not()
            .then(just("init")
                .padded()
                .map(|_| "init".to_string())
                .or(just("destroy")
                    .padded()
                    .map(|_| "destroy".to_string())))
            .then(
                just("(")
                    .padded()
                    .ignore_then(
                        fn_param
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
            .map(|(((visibility, method_name), params), body)| {
                let vis = if visibility.is_some() { Visibility::Public } else { Visibility::Private };
                (method_name, StructMethod { visibility: vis, params, body })
            });

        // INTEGRACIÓN ZIG: Parser de structs usando parser Zig-style
        // SOLUCIÓN SIMPLIFICADA: Capturar caracteres línea por línea hasta encontrar
        // el patrón "end" seguido de nueva línea y luego un keyword de statement
        // o fin de input. Esto identifica el "end" final del struct.
        
        // INTEGRACIÓN ZIG: Parser de structs usando parser Zig-style
        // SOLUCIÓN DEFINITIVA: Capturar caracteres con take_until hasta "end\n"
        // y luego procesar el string manualmente para encontrar el "end" correcto
        
        let struct_stmt = just("struct")
            .padded()
            .ignore_then(ident.clone())
            .then(
                // ESTRATEGIA: Capturar caracteres hasta encontrar "end\n" (cualquier end)
                // Luego procesar manualmente el string para encontrar el "end" FINAL
                // que está seguido de un keyword (print, let, etc.)
                
                take_until(
                    just("end")
                        .then(just("\n").or(just("\r\n")))
                )
                .map(|(chars, _)| {
                    chars.into_iter().collect::<String>()
                })
                .try_map(|content: String, span| {
                    // Procesar manualmente el string para encontrar el "end" FINAL
                    // que está seguido de "\n" + keyword (print, let, etc.)
                    let keywords = ["print", "let", "if", "while", "fn", "struct", "return"];
                    
                    // Buscar todas las ocurrencias de "end" y verificar cuál es el final
                    let mut search_pos = 0;
                    let mut last_valid_end_pos = None;
                    
                    while let Some(pos) = content[search_pos..].find("end") {
                        let abs_pos = search_pos + pos;
                        let after_end = abs_pos + 3;
                        
                        if after_end < content.len() {
                            let rest = &content[after_end..];
                            // Eliminar whitespace después de "end"
                            let trimmed = rest.trim_start_matches([' ', '\t']);
                            
                            // Verificar si viene nueva línea seguida de keyword
                            if trimmed.starts_with('\n') || trimmed.starts_with("\r\n") {
                                let after_nl = trimmed.trim_start_matches(['\n', '\r']);
                                let after_nl_trimmed = after_nl.trim_start();
                                
                                // Verificar si viene un keyword
                                for keyword in &keywords {
                                    if after_nl_trimmed.starts_with(keyword) {
                                        last_valid_end_pos = Some(abs_pos);
                                        break;
                                    }
                                }
                            }
                        }
                        
                        search_pos = abs_pos + 3;
                        if search_pos >= content.len() {
                            break;
                        }
                    }
                    
                    // Si encontramos un "end" válido, usar solo hasta ese punto
                    if let Some(end_pos) = last_valid_end_pos {
                        Ok(content[..end_pos].to_string())
                    } else {
                        // No encontramos patrón, buscar el último "end" (final del archivo)
                        if let Some(last_end) = content.rfind("end") {
                            Ok(content[..last_end].to_string())
                        } else {
                            Err(Simple::custom(span, "No se encontró 'end' para cerrar el struct"))
                        }
                    }
                })
            )
            .then_ignore(just("end").padded())
            .try_map(|(name, content), span| {
                // Reconstruir el struct completo para el parser Zig
                let full_struct = format!("struct {}\n{}end", name, content.trim());
                
                // Parsear con parser Zig-style
                match zig_struct_parser::parse_struct_from_string(&full_struct) {
                    Ok((parsed_name, fields, init, destroy)) => {
                        Ok(Stmt::Struct {
                            name: parsed_name,
                            fields,
                            init,
                            destroy,
                        })
                    }
                    Err(e) => {
                        Err(Simple::custom(span, format!("Zig parser error: {}", e)))
                    }
                }
            })
            .labelled("struct statement (Zig parser)");

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

        // IMPORTANTE: struct_stmt debe estar ANTES de expr_stmt para tener precedencia
        // Si expr_stmt está primero, intentará parsear "struct" como una expresión
        struct_stmt
            .or(print)
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

        // Borrowing: &expr o &mut expr
        let borrow = just("&")
            .padded()
            .then(just("mut").padded().or_not())
            .then(expr.clone())
            .map(|((_, mutable), expr)| Expr::Borrow {
                expr: Box::new(expr),
                mutable: mutable.is_some(),
            })
            .labelled("borrow");

        // Dereferencing: *expr
        let deref = just("*")
            .padded()
            .ignore_then(expr.clone())
            .map(|e| Expr::Deref(Box::new(e)))
            .labelled("deref");

        // Option/Result constructors (O0.4)
        let some = just("Some")
            .padded()
            .ignore_then(
                expr.clone()
                    .delimited_by(just("(").padded(), just(")").padded())
            )
            .map(|e| Expr::Some(Box::new(e)))
            .labelled("Some");

        let none = just("None")
            .padded()
            .map(|_| Expr::None)
            .labelled("None");

        let ok = just("Ok")
            .padded()
            .ignore_then(
                expr.clone()
                    .delimited_by(just("(").padded(), just(")").padded())
            )
            .map(|e| Expr::Ok(Box::new(e)))
            .labelled("Ok");

        let err = just("Err")
            .padded()
            .ignore_then(
                expr.clone()
                    .delimited_by(just("(").padded(), just(")").padded())
            )
            .map(|e| Expr::Err(Box::new(e)))
            .labelled("Err");

        // Struct literal (Fase 1.2 - O1)
        // StructName { field1: value1, field2: value2 }
        let struct_literal = text::ident()
            .then(
                just("{")
                    .padded()
                    .ignore_then(
                        text::ident()
                            .then_ignore(just(":").padded())
                            .then(expr.clone())
                            .map(|(field_name, value)| (field_name, value))
                            .separated_by(just(",").padded())
                            .allow_trailing(),
                    )
                    .then_ignore(just("}").padded()),
            )
            .map(|(struct_name, fields)| {
                Expr::StructLiteral {
                    name: struct_name,
                    fields,
                }
            })
            .labelled("struct literal");

        let atom = number
            .or(string)
            .or(borrow)  // Borrow debe ir ANTES de ident para que &x se parse como Borrow, no como Call
            .or(deref)
            .or(some.clone())
            .or(none.clone())
            .or(ok.clone())
            .or(err.clone())
            .or(struct_literal)
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
                    .then_ignore(just(")").padded()),
            )
            .map(|(callee, args)| match callee {
                Expr::Ident(name) => Expr::Call {
                    name,
                    args,
                },
                _ => unreachable!(),
            })
            .or(atom);

        // Match expression (O0.4)
        let pattern = recursive(|_pattern| {
            let some_pattern = just("Some").padded().to(Pattern::Some);
            let none_pattern = just("None").padded().to(Pattern::None);
            let ok_pattern = just("Ok").padded().to(Pattern::Ok);
            let err_pattern = just("Err").padded().to(Pattern::Err);
            let wildcard = just("_").padded().to(Pattern::Wildcard);
            let ident_pattern = text::ident().map(Pattern::Ident);
            let number_pattern = text::int(10)
                .map(|s: String| s.parse::<i64>().unwrap())
                .map(Pattern::LiteralNumber);
            let string_pattern = just('"')
                .ignore_then(none_of('"').repeated())
                .then_ignore(just('"'))
                .collect::<String>()
                .map(Pattern::LiteralString);
            
            some_pattern
                .or(none_pattern)
                .or(ok_pattern)
                .or(err_pattern)
                .or(wildcard)
                .or(number_pattern)
                .or(string_pattern)
                .or(ident_pattern)
                .labelled("pattern")
        });

        let match_arm = pattern
            .then_ignore(just("=>").padded())
            .then(expr.clone())
            .map(|(pat, body)| MatchArm {
                pattern: pat,
                body: Box::new(body),
            })
            .labelled("match arm");

        let match_expr = just("match")
            .padded()
            .ignore_then(expr.clone())
            .then(
                just("{")
                    .padded()
                    .ignore_then(
                        match_arm
                            .separated_by(just(",").padded())
                            .allow_trailing()
                    )
                    .then_ignore(just("}").padded())
            )
            .map(|(expr, arms)| Expr::Match {
                expr: Box::new(expr),
                arms,
            })
            .labelled("match");

        let unary = call
            .or(match_expr);

        // Aplicar field/method access despu├®s de call/match (Fase 1.2 - O1, O4)
        let with_access = unary
            .then(
                just(".")
                    .padded()
                    .ignore_then(text::ident())
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
                    .repeated(),
            )
            .foldl(|obj, (name, args)| {
                if let Some(args) = args {
                    // Method call
                    Expr::MethodCall {
                        object: Box::new(obj),
                        method: name,
                        args,
                    }
                } else {
                    // Field access
                    Expr::FieldAccess {
                        object: Box::new(obj),
                        field: name,
                    }
                }
            });

        let product = with_access
            .clone()
            .then(
                just("*")
                    .to(BinOp::Mul)
                    .or(just("/").to(BinOp::Div))
                    .then(with_access.clone())
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
                mutable: false,  // Inmutable por defecto
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

    #[test]
    fn test_parse_borrow() {
        let src = r#"let r = &x"#;
        let program = parse(src).unwrap();
        if let Stmt::Let { value, .. } = &program.statements[0] {
            assert!(matches!(value, Expr::Borrow { mutable: false, .. }));
        } else {
            panic!("Expected Let statement");
        }
    }

    #[test]
    fn test_parse_mut_borrow() {
        let src = r#"let r = &mut x"#;
        let program = parse(src).unwrap();
        if let Stmt::Let { value, .. } = &program.statements[0] {
            assert!(matches!(value, Expr::Borrow { mutable: true, .. }));
        } else {
            panic!("Expected Let statement");
        }
    }

    #[test]
    fn test_parse_deref() {
        let src = r#"let val = *ptr"#;
        let program = parse(src).unwrap();
        if let Stmt::Let { value, .. } = &program.statements[0] {
            assert!(matches!(value, Expr::Deref(_)));
        } else {
            panic!("Expected Let statement");
        }
    }

    #[test]
    fn test_parse_fn_with_borrow_param() {
        let src = r#"
            fn imprimir(&texto) {
                print texto
            }
        "#;
        let program = parse(src).unwrap();
        if let Stmt::Fn { params, .. } = &program.statements[0] {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].name, "texto");
            assert_eq!(params[0].borrow_type, BorrowType::Borrowed);
        } else {
            panic!("Expected Fn statement");
        }
    }

    #[test]
    fn test_parse_fn_with_mut_borrow_param() {
        let src = r#"
            fn modificar(&mut valor) {
                valor = 10
            }
        "#;
        let program = parse(src).unwrap();
        if let Stmt::Fn { params, .. } = &program.statements[0] {
            assert_eq!(params.len(), 1);
            assert_eq!(params[0].name, "valor");
            assert_eq!(params[0].borrow_type, BorrowType::MutBorrowed);
        } else {
            panic!("Expected Fn statement");
        }
    }

    // ========== Tests para Option/Result (O0.4) ==========
    
    #[test]
    fn test_parse_some() {
        let src = r#"let x = Some(42)"#;
        let program = parse(src).unwrap();
        if let Stmt::Let { value, .. } = &program.statements[0] {
            assert!(matches!(value, Expr::Some(_)));
            if let Expr::Some(inner) = value {
                assert!(matches!(inner.as_ref(), Expr::Number(42)));
            }
        } else {
            panic!("Expected Let statement");
        }
    }

    #[test]
    fn test_parse_none() {
        let src = r#"let x = None"#;
        let program = parse(src).unwrap();
        if let Stmt::Let { value, .. } = &program.statements[0] {
            assert!(matches!(value, Expr::None));
        } else {
            panic!("Expected Let statement");
        }
    }

    #[test]
    fn test_parse_ok() {
        let src = r#"let x = Ok(10)"#;
        let program = parse(src).unwrap();
        if let Stmt::Let { value, .. } = &program.statements[0] {
            assert!(matches!(value, Expr::Ok(_)));
            if let Expr::Ok(inner) = value {
                assert!(matches!(inner.as_ref(), Expr::Number(10)));
            }
        } else {
            panic!("Expected Let statement");
        }
    }

    #[test]
    fn test_parse_err() {
        let src = r#"let x = Err("error")"#;
        let program = parse(src).unwrap();
        if let Stmt::Let { value, .. } = &program.statements[0] {
            assert!(matches!(value, Expr::Err(_)));
            if let Expr::Err(inner) = value {
                assert!(matches!(inner.as_ref(), Expr::String(_)));
            }
        } else {
            panic!("Expected Let statement");
        }
    }

    #[test]
    fn test_parse_match_simple() {
        let src = r#"
            match x {
                Some => 1,
                None => 0
            }
        "#;
        let program = parse(src).unwrap();
        if let Stmt::Expr(Expr::Match { expr, arms }) = &program.statements[0] {
            assert!(matches!(expr.as_ref(), Expr::Ident(_)));
            assert_eq!(arms.len(), 2);
            assert!(matches!(arms[0].pattern, Pattern::Some));
            assert!(matches!(arms[1].pattern, Pattern::None));
        } else {
            panic!("Expected Match expression");
        }
    }

    #[test]
    fn test_parse_match_with_bindings() {
        let src = r#"
            match resultado {
                Ok => "success",
                Err => "error",
                _ => "unknown"
            }
        "#;
        let program = parse(src).unwrap();
        if let Stmt::Expr(Expr::Match { arms, .. }) = &program.statements[0] {
            assert_eq!(arms.len(), 3);
            assert!(matches!(arms[0].pattern, Pattern::Ok));
            assert!(matches!(arms[1].pattern, Pattern::Err));
            assert!(matches!(arms[2].pattern, Pattern::Wildcard));
        } else {
            panic!("Expected Match expression");
        }
    }

    #[test]
    fn test_parse_match_with_literals() {
        let src = r#"
            match x {
                0 => "zero",
                1 => "one",
                _ => "other"
            }
        "#;
        let program = parse(src).unwrap();
        if let Stmt::Expr(Expr::Match { arms, .. }) = &program.statements[0] {
            assert_eq!(arms.len(), 3);
            assert!(matches!(arms[0].pattern, Pattern::LiteralNumber(0)));
            assert!(matches!(arms[1].pattern, Pattern::LiteralNumber(1)));
            assert!(matches!(arms[2].pattern, Pattern::Wildcard));
        } else {
            panic!("Expected Match expression");
        }
    }

    #[test]
    fn test_parse_nested_some() {
        let src = r#"let x = Some(Some(42))"#;
        let program = parse(src).unwrap();
        if let Stmt::Let { value, .. } = &program.statements[0] {
            assert!(matches!(value, Expr::Some(_)));
            if let Expr::Some(inner) = value {
                assert!(matches!(inner.as_ref(), Expr::Some(_)));
            }
        } else {
            panic!("Expected Let statement");
        }
    }

    // ========== Tests para Structs (Fase 1.2) ==========
    
    #[test]
    fn test_parse_struct_definition() {
        let src = r#"
            struct Persona {
                nombre: string,
                edad: int64
            }
        "#;
        let program = parse(src).unwrap();
        if let Stmt::Struct { name, fields, init, destroy } = &program.statements[0] {
            assert_eq!(name, "Persona");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].name, "nombre");
            assert_eq!(fields[0].mutable, false);  // Inmutable por defecto
            assert_eq!(fields[0].visibility, Visibility::Private);  // Privado por defecto (O5)
            assert_eq!(fields[1].name, "edad");
            assert_eq!(fields[1].visibility, Visibility::Private);  // Privado por defecto (O5)
            assert!(init.is_none());  // Sin constructor
            assert!(destroy.is_none());  // Sin destructor
        } else {
            panic!("Expected Struct statement");
        }
    }

    #[test]
    fn test_parse_struct_with_mutable_fields() {
        let src = r#"
            struct Contador {
                mut valor: int64
            }
        "#;
        let program = parse(src).unwrap();
        if let Stmt::Struct { fields, init, destroy, .. } = &program.statements[0] {
            assert_eq!(fields[0].mutable, true);  // Campo mutable
            assert_eq!(fields[0].visibility, Visibility::Private);  // Privado por defecto (O5)
            assert!(init.is_none());  // Sin constructor
            assert!(destroy.is_none());  // Sin destructor
        } else {
            panic!("Expected Struct statement");
        }
    }

    #[test]
    fn test_parse_struct_literal() {
        let src = r#"
            let p = Persona {
                nombre: "Juan",
                edad: 25
            }
        "#;
        let program = parse(src).unwrap();
        if let Stmt::Let { value, .. } = &program.statements[0] {
            assert!(matches!(value, Expr::StructLiteral { .. }));
            if let Expr::StructLiteral { name, fields } = value {
                assert_eq!(name, "Persona");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "nombre");
                assert_eq!(fields[1].0, "edad");
            }
        } else {
            panic!("Expected Let statement");
        }
    }

    #[test]
    fn test_parse_field_access() {
        let src = r#"
            let nombre = p.nombre
        "#;
        let program = parse(src).unwrap();
        if let Stmt::Let { value, .. } = &program.statements[0] {
            assert!(matches!(value, Expr::FieldAccess { .. }));
            if let Expr::FieldAccess { object, field } = value {
                assert!(matches!(object.as_ref(), Expr::Ident(_)));
                assert_eq!(field, "nombre");
            }
        } else {
            panic!("Expected Let statement");
        }
    }

    #[test]
    fn test_parse_method_call() {
        let src = r#"
            let resultado = objeto.metodo(10, 20)
        "#;
        let program = parse(src).unwrap();
        if let Stmt::Let { value, .. } = &program.statements[0] {
            assert!(matches!(value, Expr::MethodCall { .. }));
            if let Expr::MethodCall { object, method, args } = value {
                assert!(matches!(object.as_ref(), Expr::Ident(_)));
                assert_eq!(method, "metodo");
                assert_eq!(args.len(), 2);
            }
        } else {
            panic!("Expected Let statement");
        }
    }

    #[test]
    fn test_parse_chained_field_access() {
        let src = r#"
            let valor = objeto.campo.subcampo
        "#;
        let program = parse(src).unwrap();
        if let Stmt::Let { value, .. } = &program.statements[0] {
            assert!(matches!(value, Expr::FieldAccess { .. }));
            if let Expr::FieldAccess { object, field } = value {
                assert_eq!(field, "subcampo");
                // El object deber├¡a ser otro FieldAccess
                assert!(matches!(object.as_ref(), Expr::FieldAccess { .. }));
            }
        } else {
            panic!("Expected Let statement");
        }
    }
}

