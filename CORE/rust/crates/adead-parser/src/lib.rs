use adead_common::{ADeadError, Result};
use chumsky::prelude::*;
use std::io::{self, Write};

// ═══════════════════════════════════════════════════════════════════════════
// ADead Parser - Compilador de ADead a NASM (x86_64)
// ═══════════════════════════════════════════════════════════════════════════
//
// PIPELINE PRINCIPAL (NASM Directo):
//   ADead → Parser (Chumsky) → NASM Generator (adead-backend) → NASM → .obj → .exe
//
// PIPELINE FALLBACK (C++):
//   ADead → Parser → C++ Generator → GCC++/Clang++ → ASM Cleaner → NASM
//
// Autor: Eddi Andreé Salazar Matos
// ═══════════════════════════════════════════════════════════════════════════

// Parser manual para estructuras C (usado en pipeline fallback)
pub mod c_manual_parser;
pub mod c_while_if_parser;

// Resolución de módulos (import básico)
pub mod module_resolver;

// Selector de pipeline (prioriza NASM directo)
pub mod pipeline_selector;

// Pipeline paralelo: Compilación paralela con caching
pub mod parallel_pipeline;

// Limpieza y optimización de ASM (incluye conversión GAS→NASM)
pub mod clean_asm;

// C++ Optimizer (placeholder para futuras optimizaciones)
pub mod cpp_optimizer;

// Generadores de código (pipelines fallback)
pub mod c_generator;      // Genera código C desde AST
pub mod c_to_nasm;        // Convierte C AST directamente a NASM
pub mod cpp_generator;    // Genera código C++ desde AST

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64),
    Float(f64),  // Literal flotante: 3.14, 2.5e10, etc.
    Bool(bool),  // Literal booleano: true, false
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
        module: Option<String>,  // None = función local, Some("math") = math.factorial (Sprint 1.3)
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
    PropagateError(Box<Expr>), // expr? - Propaga error automáticamente
    // Structs/Clases (Fase 1.2 - O1)
    StructLiteral {             // StructName { field1: value1, field2: value2 }
        name: String,
        fields: Vec<(String, Expr)>,  // (field_name, value)
    },
    FieldAccess {               // expr.field_name
        object: Box<Expr>,
        field: String,
    },
    FieldAssign {               // expr.field_name = value
        object: Box<Expr>,
        field: String,
        value: Box<Expr>,
    },
    MethodCall {                // expr.method_name(args)
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    SuperCall {                 // super.method_name(args) - llamada a método del padre
        method: String,
        args: Vec<Expr>,
    },
    // Arrays (Sprint 1.2)
    ArrayLiteral(Vec<Expr>),    // [1, 2, 3]
    Index {                     // arr[0]
        array: Box<Expr>,
        index: Box<Expr>,
    },
    // Strings (Sprint 2)
    Slice {                     // s[0:4]
        object: Box<Expr>,
        start: Box<Expr>,
        end: Box<Expr>,
    },
    // Operadores lógicos (Prioridad 2)
    Not(Box<Expr>),             // !expr - Negación lógica
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
    Mod,  // Módulo: a % b
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Operadores lógicos (Prioridad 2)
    And,  // && - AND lógico con short-circuit
    Or,   // || - OR lógico con short-circuit
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
    // For loops (NASM-Universal.md - Patrones para Futuras Implementaciones)
    For {
        var: String,        // Variable de iteración
        start: Expr,        // Inicio del rango
        end: Expr,          // Fin del rango (exclusivo)
        body: Vec<Stmt>,
    },
    // Control de flujo en loops (NASM-Universal.md)
    Break,      // Salir del loop más cercano
    Continue,   // Saltar a la siguiente iteración
    Fn {
        visibility: Visibility,  // Sprint 1.3 - Import básico: pub fn o fn (privada)
        name: String,
        params: Vec<FnParam>,  // Cambiado para soportar borrowing
        body: Vec<Stmt>,
    },
    // Structs/Clases (Fase 1.2 - O1, O2 - RAII)
    Struct {
        name: String,
        parent: Option<String>,  // Herencia: extends Parent
        fields: Vec<StructField>,
        init: Option<StructMethod>,      // Constructor (O2)
        destroy: Option<StructMethod>,    // Destructor (O2.1 - Drop Trait)
        methods: Vec<(String, StructMethod)>, // Métodos de instancia
    },
    Expr(Expr),
    Return(Option<Expr>),
    // Import básico (Sprint 1.3)
    Import(String),  // import nombre_modulo
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

pub fn parse(source: &str) -> Result<Program> {
    parse_with_dir(source, None)
}

/// Parsear con directorio base para resolución de imports
pub fn parse_with_dir(source: &str, current_dir: Option<&std::path::Path>) -> Result<Program> {
    // NOTA: El preprocesador de structs con sintaxis 'end' está deshabilitado
    // Ahora usamos sintaxis con llaves {} que el parser maneja directamente
    
    // DEBUG: Activar análisis del parser
    #[cfg(feature = "parser-debug")]
    {
        use crate::parser_debug::ParserDebugger;
        let debugger = ParserDebugger::new(true, true);
        eprintln!("[PARSER-DEBUG] Iniciando parsing del código fuente...");
    }
    
    let parser = program_parser();
    
    eprintln!("[PARSER-DEBUG] Iniciando parsing del código fuente ({} caracteres)", source.len());
    eprintln!("[PARSER-DEBUG] Primeras líneas del código:");
    for (i, line) in source.lines().take(5).enumerate() {
        eprintln!("[PARSER-DEBUG]   Línea {}: '{}'", i + 1, line);
    }
    io::stderr().flush().ok();
    
    // Intentar parsear con manejo detallado de errores
    eprintln!("[PARSER-DEBUG] Llamando a parser.parse()...");
    io::stderr().flush().ok();
    
    let parse_result = parser.parse(source);
    
    eprintln!("[PARSER-DEBUG] parser.parse() retornó resultado");
    io::stderr().flush().ok();
    
    match parse_result {
        Ok(mut program) => {
            eprintln!("[PARSER-DEBUG] Parse exitoso (sin errores reportados)");
            io::stderr().flush().ok();
            
            // DEBUG: Analizar programa parseado
            #[cfg(feature = "parser-debug")]
            {
                use crate::parser_debug::ParserDebugger;
                let debugger = ParserDebugger::new(true, true);
                debugger.analyze_parsed_program(&program, source);
            }
            
            // Análisis siempre activo (sin feature flag) para debugging
            eprintln!("[PARSER-INFO] Programa parseado: {} statements", program.statements.len());
            let let_count = program.statements.iter().filter(|s| matches!(s, Stmt::Let { .. })).count();
            let print_count = program.statements.iter().filter(|s| matches!(s, Stmt::Print(_))).count();
            let fn_count = program.statements.iter().filter(|s| matches!(s, Stmt::Fn { .. })).count();
            let struct_count = program.statements.iter().filter(|s| matches!(s, Stmt::Struct { .. })).count();
            
            eprintln!("[PARSER-INFO] Desglose: {} structs, {} funciones, {} let, {} print", 
                struct_count, fn_count, let_count, print_count);
            
            // CRÍTICO: Si no se parseó nada, hay un problema grave
            if program.statements.is_empty() {
                eprintln!("[PARSER-ERROR] ⚠️⚠️⚠️  CRÍTICO: El parser retornó éxito pero NO parseó ningún statement!");
                eprintln!("[PARSER-ERROR] Esto indica un problema grave en el parser o en el código fuente.");
                eprintln!("[PARSER-ERROR] Primeras líneas del código fuente:");
                for (i, line) in source.lines().take(10).enumerate() {
                    eprintln!("[PARSER-ERROR]   Línea {}: {}", i + 1, line);
                }
                io::stderr().flush().ok();
            }
            
            // Verificar si hay statements esperados pero no parseados
            let expected_let_print = source.lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    trimmed.starts_with("let ") || trimmed.starts_with("print ")
                })
                .count();
            
            if expected_let_print > (let_count + print_count) {
                eprintln!("[PARSER-WARNING] ⚠️  Se esperaban {} statements Let/Print pero solo se parsearon {}!", 
                    expected_let_print, let_count + print_count);
                eprintln!("[PARSER-WARNING] Posible problema: El parser puede estar deteniéndose después de funciones.");
                io::stderr().flush().ok();
            }
            
            // POST-PROCESADOR: Resolver imports (Sprint 1.3)
            resolve_imports(&mut program, current_dir)?;
            
            Ok(program)
        }
        Err(errs) => {
            eprintln!("[PARSER-ERROR] Error de parsing detectado:");
            for (i, err) in errs.iter().enumerate() {
                eprintln!("[PARSER-ERROR]   Error {}: {}", i + 1, err);
            }
            io::stderr().flush().ok();
            
            // Mostrar contexto del código fuente donde falló
            eprintln!("[PARSER-ERROR] Primeras líneas del código fuente:");
            for (i, line) in source.lines().take(20).enumerate() {
                eprintln!("[PARSER-ERROR]   Línea {}: {}", i + 1, line);
            }
            io::stderr().flush().ok();
            
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

/// Resolver imports en un programa parseado (Sprint 1.3)
/// 
/// Procesa todos los `Stmt::Import` y:
/// 1. Resuelve la ruta del módulo
/// 2. Parsea el módulo
/// 3. Filtra solo funciones públicas
/// 4. Combina statements en el programa principal
fn resolve_imports(program: &mut Program, current_dir: Option<&std::path::Path>) -> Result<()> {
    use std::collections::{HashSet, HashMap};
    
    // Extraer imports y parsear módulos
    let mut imports_to_resolve = Vec::new();
    let mut new_statements = Vec::new();
    let mut module_functions: HashMap<String, Vec<String>> = HashMap::new(); // Para detectar colisiones
    
    for stmt in &program.statements {
        if let Stmt::Import(module_name) = stmt {
            imports_to_resolve.push(module_name.clone());
        }
    }
    
    // Resolver cada import (sin duplicados)
    let mut resolved_modules = HashSet::new();
    for module_name in imports_to_resolve {
        if resolved_modules.contains(&module_name) {
            continue; // Ya procesado - evitar imports duplicados
        }
        resolved_modules.insert(module_name.clone());
        
        // Parsear el módulo
        let module_program = module_resolver::resolve_and_parse(&module_name, current_dir)?;
        
        // Agregar statements del módulo al programa
        // Filtrar solo funciones públicas (Sprint 1.3 - Import básico)
        let mut module_funcs = Vec::new();
        for stmt in module_program.statements {
            match &stmt {
                Stmt::Fn { visibility, name, .. } => {
                    // Solo agregar funciones públicas
                    if *visibility == Visibility::Public {
                        new_statements.push(stmt.clone());
                        module_funcs.push(name.clone());
                    }
                }
                // Otros statements (structs, etc.) se agregan siempre
                _ => {
                    new_statements.push(stmt);
                }
            }
        }
        
        // Registrar funciones del módulo para detección de colisiones
        if !module_funcs.is_empty() {
            module_functions.insert(module_name.clone(), module_funcs);
        }
    }
    
    // Verificar colisiones de nombres (opcional - warning, no error)
    // Por ahora solo verificamos, pero no bloqueamos
    let all_function_names: Vec<String> = program.statements
        .iter()
        .filter_map(|s| {
            if let Stmt::Fn { name, .. } = s {
                Some(name.clone())
            } else {
                None
            }
        })
        .collect();
    
    for (module_name, funcs) in &module_functions {
        for func_name in funcs {
            if all_function_names.contains(func_name) {
                // Función local con mismo nombre que una importada
                // Esto está bien si se usa namespace, pero es un warning
                // Por ahora no hacemos nada, pero se podría agregar warning
            }
        }
    }
    
    // Insertar statements de módulos al inicio (después de structs)
    // Mantener orden: structs primero, luego funciones importadas, luego código local
    for stmt in new_statements.into_iter().rev() {
        program.statements.insert(0, stmt);
    }
    
    Ok(())
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
    
    // Parsear struct usando parser Rust estándar
    // TODO: Implementar parser de structs en Rust (por ahora usar parse básico)
    Err(ADeadError::ParseError {
        line: 1,
        col: 1,
        message: format!("Struct parsing not yet fully implemented. Struct content: {}", full_struct),
    })
}

// Parser para comentarios (# hasta el final de línea)
fn comment() -> impl Parser<char, (), Error = Simple<char>> + Clone {
    just('#')
        .then(none_of("\n\r").repeated())
        .then(text::newline().or_not())
        .ignored()
}

// Parser para whitespace y comentarios
fn ws_and_comments() -> impl Parser<char, (), Error = Simple<char>> + Clone {
    filter(|c: &char| c.is_whitespace())
        .repeated()
        .ignored()
        .then(comment().padded().repeated().ignored())
        .ignored()
}

fn program_parser() -> impl Parser<char, Program, Error = Simple<char>> {
    // Ignorar comentarios al inicio y entre statements
    ws_and_comments()
        .ignore_then(
            stmt_parser()
                .padded()
                .then_ignore(ws_and_comments())
                .repeated()
        )
        .then_ignore(end().or_not())  // Permitir trailing whitespace/newlines
        .try_map(|stmts: Vec<Stmt>, span| {
            // DEBUG: Verificar qué se parseó
            eprintln!("[PARSER-DEBUG] program_parser: Se parsearon {} statements", stmts.len());
            io::stderr().flush().ok();
            
            // Si no se parseó nada pero el span no está vacío, puede haber un problema
            if stmts.is_empty() {
                eprintln!("[PARSER-DEBUG] WARNING: program_parser retornó 0 statements pero el input puede no estar vacío");
                io::stderr().flush().ok();
            }
            
            Ok(Program {
            statements: stmts,
            })
        })
}

// Helper para detectar si una expresión contiene floats
fn contains_float(expr: &Expr) -> bool {
    match expr {
        Expr::Float(_) => true,
        Expr::BinaryOp { left, right, .. } => {
            contains_float(left) || contains_float(right)
        }
        _ => false,
    }
}

fn stmt_parser() -> impl Parser<char, Stmt, Error = Simple<char>> + Clone {
    recursive(|stmt| {
        let ident = text::ident().padded();
        let expr = expr_parser();
        // Los parsers de expr se clonan cuando se necesitan
        let expr_for_print = expr.clone();
        let expr_for_while = expr.clone();
        let expr_for_expr_stmt = expr.clone();

        // Print statement: Usar parser Rust estándar
        let print = just("print")
            .padded()
            .ignore_then(
                // Capturar la expresión como string para decidir qué parser usar
                none_of("\n")
                    .repeated()
                    .at_least(1)
                    .collect::<String>()
                    .padded()
                    .try_map({
                        let expr_clone = expr.clone(); // Clonar el parser de Rust
                        move |expr_str: String, span| {
                            let trimmed = expr_str.trim();
                            // Detectar booleanos primero
                            if trimmed == "true" {
                                Ok(Expr::Bool(true))
                            } else if trimmed == "false" {
                                Ok(Expr::Bool(false))
                            } else {
                                // Usar parser Rust estándar para todas las expresiones
                                expr_clone.clone().parse(trimmed)
                                    .map_err(|_| Simple::custom(span, format!("Parse error: could not parse expression '{}'", trimmed)))
                            }
                        }
                    })
            )
            .map(Stmt::Print)
            .labelled("print statement");

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

        // While statement: Parser robusto para estructuras anidadas
        // CRÍTICO: El parser recursivo ya maneja bloques anidados correctamente
        // El problema puede estar en el orden de precedencia o en cómo se parsea el cierre
        let while_stmt = just("while")
            .padded()
            .ignore_then(
                expr_for_while.clone()
                    .padded()
            )
            .then(
                // Parsear bloque: el parser recursivo ya maneja if/while anidados
                just("{")
                    .padded()
                    .ignore_then(
                        // IMPORTANTE: stmt.clone() ya es recursivo, así que maneja if/while dentro
                        // Usar .padded() para permitir newlines entre statements dentro del bloque
                        stmt.clone()
                            .padded()
                            .repeated()
                            .collect::<Vec<_>>()
                    )
                    .then_ignore(just("}").padded())
            )
            .map(|(condition, body)| Stmt::While {
                condition,
                body,
            })
            .labelled("while statement");

        // For loop: for VAR in START..END { BODY }
        // El backend YA soporta generación NASM para for loops
        let for_stmt = just("for")
            .padded()
            .ignore_then(ident.clone())
            .then_ignore(just("in").padded())
            .then(expr.clone())  // start expression
            .then_ignore(just("..").padded())
            .then(expr.clone())  // end expression
            .then(
                just("{")
                    .padded()
                    .ignore_then(
                        stmt.clone()
                            .padded()
                            .repeated()
                            .collect::<Vec<_>>()
                    )
                    .then_ignore(just("}").padded())
            )
            .map(|(((var, start), end), body)| Stmt::For {
                var,
                start,
                end,
                body,
            })
            .labelled("for statement");

        // Break statement: sale del loop más cercano
        let break_stmt = just("break")
            .padded()
            .map(|_| Stmt::Break)
            .labelled("break statement");

        // Continue statement: salta a la siguiente iteración
        let continue_stmt = just("continue")
            .padded()
            .map(|_| Stmt::Continue)
            .labelled("continue statement");

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

        // Parser específico para el cuerpo de funciones (sin return_stmt en nivel superior)
        // Esto evita que return se parse como statement de nivel superior
        let fn_body_stmt = recursive(|fn_body_stmt| {
            let ident = text::ident().padded();
            let expr = expr_parser();
            let expr_for_print = expr.clone();
            let expr_for_while = expr.clone();
            let expr_for_expr_stmt = expr.clone();

            // Print statement dentro de funciones
            let print = just("print")
                .padded()
                .ignore_then(expr_for_print.clone())
                .map(Stmt::Print)
                .labelled("print statement");

            let let_stmt = just("let")
                .padded()
                .then(just("mut").padded().or_not())
                .then(ident.clone())
                .then_ignore(just("=").padded())
                .then(expr.clone())
                .map(|(((_, mutable), name), value)| Stmt::Let {
                    mutable: mutable.is_some(),
                    name,
                    value,
                });

            let if_stmt = just("if")
                .padded()
                .ignore_then(expr_for_while.clone())
                .then(
                    just("{")
                        .padded()
                        .ignore_then(fn_body_stmt.clone().repeated())
                        .then_ignore(just("}").padded()),
                )
                .then(
                    just("else")
                        .padded()
                        .ignore_then(
                            just("{")
                                .padded()
                                .ignore_then(fn_body_stmt.clone().repeated())
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
                .ignore_then(expr_for_while.clone())
                .then(
                    just("{")
                        .padded()
                        .ignore_then(fn_body_stmt.clone().repeated())
                        .then_ignore(just("}").padded()),
                )
                .map(|(condition, body)| Stmt::While { condition, body });

            let break_stmt = just("break")
                .padded()
                .map(|_| Stmt::Break)
                .labelled("break statement");

            let continue_stmt = just("continue")
                .padded()
                .map(|_| Stmt::Continue)
                .labelled("continue statement");

            let field_assign_stmt = ident
                .clone()
                .then(just('.').ignore_then(ident.clone()))
                .then_ignore(just("=").padded())
                .then(expr.clone())
                .map(|((obj_name, field_name), value)| Stmt::Expr(Expr::FieldAssign {
                    object: Box::new(Expr::Ident(obj_name)),
                    field: field_name,
                    value: Box::new(value),
                }));

            let assign_stmt = ident
                .clone()
                .then_ignore(just("=").padded())
                .then(expr.clone())
                .map(|(name, value)| Stmt::Expr(Expr::Assign {
                    name,
                    value: Box::new(value),
                }));

            let expr_stmt = expr_for_expr_stmt.map(Stmt::Expr);

            // CRÍTICO: return_stmt DEBE estar incluido aquí para funciones
            let return_stmt = just("return")
                .padded()
                .ignore_then(expr.clone().or_not())
                .map(Stmt::Return);

            // Orden de precedencia para el cuerpo de funciones
            while_stmt
                .or(break_stmt)
                .or(continue_stmt)
                .or(if_stmt)
                .or(print)
                .or(let_stmt)
                .or(return_stmt)  // return_stmt está aquí para el cuerpo de funciones
                .or(field_assign_stmt)
                .or(assign_stmt)
                .or(expr_stmt)
                .padded()
        });

        // Parser para funciones con visibilidad opcional (Sprint 1.3 - Import básico)
        let fn_stmt = just("pub")
            .padded()
            .or_not()
            .then(just("fn")
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
                        .ignore_then(
                            fn_body_stmt
                                .padded()
                                .then_ignore(ws_and_comments())
                                .repeated()
                                .collect::<Vec<_>>()
                        )
                        .then_ignore(just("}").padded())
                        .then_ignore(ws_and_comments()),  // CRÍTICO: Consumir whitespace/comentarios después del cierre
                ))
            .map(|(visibility, ((name, params), body))| Stmt::Fn {
                visibility: if visibility.is_some() { 
                    Visibility::Public 
                } else { 
                    Visibility::Private 
                },
                name,
                params,
                body,
            });

        // ═══════════════════════════════════════════════════════════════════════════
        // OOP: STRUCTS Y CLASES (Sintaxis con llaves {})
        // ═══════════════════════════════════════════════════════════════════════════
        
        // Campo de struct: nombre (sin tipo explícito por ahora)
        // Sintaxis: x, y, nombre, etc.
        let struct_field_simple = text::ident()
            .padded()
            .map(|name: String| StructField {
                visibility: Visibility::Public,  // Público por defecto para structs simples
                mutable: true,  // Mutable por defecto
                name,
                ty: None,
            });

        // Struct definition: struct Nombre { campo1 campo2 ... }
        // Los campos pueden estar separados por comas, espacios o newlines
        // IMPORTANTE: Los campos son OPCIONALES (puede haber structs vacíos)
        let struct_stmt = just("struct")
                    .padded()
                    .ignore_then(text::ident())
            .then(
                just("{")
                    .padded()
                    .ignore_then(
                        struct_field_simple
                            .padded()  // Permite whitespace (incluyendo newlines) alrededor de cada campo
                            .repeated()  // Campos opcionales (puede ser vacío)
                            .collect::<Vec<_>>()  // Convertir a Vec explícitamente
                    )
                    .then_ignore(just("}").padded())
            )
            .map(|(name, fields)| {
                eprintln!("[PARSER-DEBUG] struct_stmt: Parseando struct '{}' con {} campos", name, fields.len());
                io::stderr().flush().ok();
                Stmt::Struct {
                name,
                parent: None,
                fields,
                init: None,
                destroy: None,
                methods: Vec::new(),
                }
            })
            .labelled("struct statement");

        // ═══════════════════════════════════════════════════════════════════════════
        // CLASS: Clases con constructor y métodos
        // Sintaxis: class Nombre { fn new(...) { } fn metodo(self) { } }
        // ═══════════════════════════════════════════════════════════════════════════
        
        // Método de clase (incluyendo constructor 'new')
        let class_method = just("fn")
            .padded()
            .ignore_then(ident.clone())
            .then(
                just("(")
                    .padded()
                    .ignore_then(
                        fn_param
                            .separated_by(just(",").padded())
                            .allow_trailing()
                    )
                    .then_ignore(just(")").padded())
            )
            .then(
                just("{")
                    .padded()
                    .ignore_then(stmt.clone().repeated())
                    .then_ignore(just("}").padded())
            )
            .map(|((name, params), body)| (name, params, body));

        // Class definition: class Nombre { fn new(...) { } fn metodo(self) { } }
        let class_stmt = just("class")
            .padded()
            .ignore_then(ident.clone())
            .then(
                just("extends")
                    .padded()
                    .ignore_then(ident.clone())
                    .or_not()
            )
            .then(
                just("{")
                    .padded()
                    .ignore_then(
                        class_method
                            .padded()
                            .repeated()
                    )
                    .then_ignore(just("}").padded())
            )
            .map(|((class_name, parent), methods)| {
                // Extraer constructor (new) y otros métodos
                let mut init_method = None;
                let mut destroy_method = None;
                let mut fields = Vec::new();
                let mut other_methods = Vec::new();
                
                for (method_name, params, body) in methods {
                    if method_name == "new" {
                        // Extraer campos del constructor (self.campo = ...)
                        for stmt in &body {
                            // Buscar FieldAssign: self.campo = valor
                            if let Stmt::Expr(Expr::FieldAssign { object, field, .. }) = stmt {
                                // Verificar que el objeto es "self"
                                if let Expr::Ident(obj_name) = object.as_ref() {
                                    if obj_name == "self" {
                                        fields.push(StructField {
                                            visibility: Visibility::Public,
                                            mutable: true,
                                            name: field.clone(),
                                            ty: None,
                                        });
                                    }
                                }
                            }
                        }
                        init_method = Some(StructMethod {
                            visibility: Visibility::Public,
                            params: params.clone(),
                            body: body.clone(),
                        });
                    } else if method_name == "destroy" {
                        destroy_method = Some(StructMethod {
                            visibility: Visibility::Public,
                            params,
                            body,
                        });
                    } else {
                        // Guardar otros métodos en el struct
                        other_methods.push((method_name, StructMethod {
                            visibility: Visibility::Public,
                            params,
                            body,
                        }));
                    }
                }
                
                Stmt::Struct {
                    name: class_name,
                    parent,
                    fields,
                    init: init_method,
                    destroy: destroy_method,
                    methods: other_methods,
                }
            })
            .labelled("class statement");

        // Field assignment: ident.field = expr (as statement)
        let field_assign_stmt = ident
            .clone()
            .then(just('.').ignore_then(ident.clone()))
            .then_ignore(just("=").padded())
            .then(expr.clone())
            .map(|((obj_name, field_name), value)| Stmt::Expr(Expr::FieldAssign {
                object: Box::new(Expr::Ident(obj_name)),
                field: field_name,
                value: Box::new(value),
            }));
        
        // Assignment: ident = expr (as statement)
        let assign_stmt = ident
            .clone()
            .then_ignore(just("=").padded())
            .then(expr.clone())
            .map(|(name, value)| Stmt::Expr(Expr::Assign {
                name,
                value: Box::new(value),
            }));

        let expr_stmt = expr_for_expr_stmt.map(Stmt::Expr);

        // Import statement (Sprint 1.3)
        let import_stmt = just("import")
            .padded()
            .ignore_then(text::ident())
            .map(Stmt::Import)
            .labelled("import statement");

        // IMPORTANTE: Orden de precedencia crítico para parsing correcto
        // struct_stmt y class_stmt deben estar ANTES de expr_stmt para tener precedencia
        // Import debe estar antes de expr_stmt también para evitar conflictos
        // CRÍTICO: while_stmt e if_stmt deben estar ANTES de assign_stmt y expr_stmt
        // para evitar que se parseen mal las condiciones y bloques
        // El orden correcto es: keywords primero, luego expresiones simples
        // CRÍTICO: while_stmt debe estar PRIMERO para tener máxima precedencia
        // CRÍTICO: fn_stmt debe estar ANTES de return_stmt para evitar que return se parse como statement de nivel superior
        // DEBUG: Agregar debug para identificar qué statement se intenta parsear
        let stmt_with_debug = while_stmt  // PRIMERO: máxima precedencia para while
            .or(for_stmt)     // For loops con rango
            .or(break_stmt)   // Break para salir de loops
            .or(continue_stmt) // Continue para saltar iteración
            .or(if_stmt)      // If tiene alta precedencia
            .or(class_stmt)   // OOP: Clases con métodos
            .or(struct_stmt)  // OOP: Structs simples (DEBE estar antes de fn_stmt)
            .or(import_stmt)
            .or(fn_stmt)      // CRÍTICO: fn_stmt ANTES de return_stmt para que return dentro de funciones se parse correctamente
            .or(print)
            .or(let_stmt)
            .or(return_stmt)  // return_stmt DESPUÉS de fn_stmt para evitar conflictos
            .or(field_assign_stmt)  // Field assignment ANTES de assign_stmt
            .or(assign_stmt)
            .or(expr_stmt)
            .padded()
            .try_map(|stmt: Stmt, span| {
                // DEBUG: Ver qué statement se parseó exitosamente
                let stmt_type = match &stmt {
                    Stmt::Struct { name, .. } => format!("Struct({})", name),
                    Stmt::Fn { name, .. } => format!("Function({})", name),
                    Stmt::Let { name, .. } => format!("Let({})", name),
                    Stmt::Print(_) => "Print".to_string(),
                    Stmt::If { .. } => "If".to_string(),
                    Stmt::While { .. } => "While".to_string(),
                    Stmt::Return(_) => "Return".to_string(),
                    Stmt::Expr(_) => "Expr".to_string(),
                    Stmt::Import(name) => format!("Import({})", name),
                    _ => format!("Other({:?})", stmt),
                };
                eprintln!("[PARSER-DEBUG] stmt_parser: ✅ Se parseó exitosamente: {} (span: {:?})", stmt_type, span);
                io::stderr().flush().ok();
                Ok(stmt)
            });
        
        stmt_with_debug
    })
}

fn expr_parser() -> impl Parser<char, Expr, Error = Simple<char>> + Clone {
        // ZIG ES EL PARSER PRINCIPAL - Intentar parsear con Zig primero para TODAS las expresiones
        recursive(|expr| {
            // Definir literales primero (number, float, string)
        // Parser para literales flotantes (debe venir ANTES de number para que 3.14 se parse como float)
        // Maneja: 3.14, .5, 5., 2.5e10, 1e-5
        // Estrategia: usar then_ignore para consumir el punto explícitamente
        // CRÍTICO: El problema es que text::digits() consume "3" antes de verificar el punto
        // Solución: Usar un parser que verifica el punto ANTES de consumir
        // Estrategia: leer dígitos con lookahead para ver si viene un punto
        // Parser de float: 3.14, 5.0, etc.
        // SOLUCIÓN DEFINITIVA: Usar un parser que lea el patrón completo
        // Importante: text::digits() lee dígitos y se detiene en el primer no-dígito
        // Luego verificamos si ese carácter es un punto, y si es, seguimos leyendo
        // Parser de float: 3.14, 5.0, etc.
        // SOLUCIÓN DEFINITIVA: Usar text::int() que lee dígitos y luego verificar punto
        // text::int() consume dígitos pero se detiene antes de consumir el siguiente carácter
        // Luego verificamos si ese carácter es un punto
        // Float parser: NO debe consumir "0.." como float (eso es rango para for)
        // Solo consume "0.5", "3.14", "5." cuando NO viene seguido de otro punto
        let float_with_int_part = text::int(10)
            .then(
                just('.')
                    .then(just('.').not())  // Rechazar ".." (rango)
                    .then(text::digits(10).or_not())  // Dígitos opcionales después del punto
            )
            .try_map(|(int_part, ((_, _), dec_opt)), span| {
                let dec = dec_opt.unwrap_or_default();
                let float_str = if dec.is_empty() {
                    format!("{}.0", int_part)  // 5. -> 5.0
                } else {
                    format!("{}.{}", int_part, dec)
                };
                float_str.parse::<f64>()
                    .map_err(|_| Simple::custom(span, format!("Invalid float literal: {}", float_str)))
            })
            .map(Expr::Float)
            .labelled("float");
        
        // También manejar .5 (sin parte entera)
        let float_without_int_part = just('.')
            .ignore_then(text::digits(10))  // .5
            .then(
                just('e').or(just('E'))
                    .ignore_then(
                        just('+').or(just('-')).or_not()
                            .then(text::int(10))
                            .map(|(sign, num): (Option<char>, String)| {
                                if sign == Some('-') {
                                    format!("-{}", num)
                                } else {
                                    num
                                }
                            })
                    )
                    .or_not()
            )
            .try_map(|(dec_part, exp_part), span| {
                let float_str = format!("0.{}", dec_part);
                let float_str = match exp_part {
                    Some(exp) => format!("{}e{}", float_str, exp),
                    None => float_str,
                };
                float_str.parse::<f64>()
                    .map_err(|_| Simple::custom(span, "Invalid float literal"))
            })
            .map(Expr::Float)
            .labelled("float");
        
        // Combinar ambos parsers de float
        let float_literal = float_with_int_part
            .or(float_without_int_part);

        // Parser para números enteros (solo enteros puros, sin punto decimal)
        // SOLUCIÓN SIMPLE: Parsear dígitos y verificar que NO viene ".D" (float)
        // Permitimos ".." (rango) porque eso no es un float
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

        // Array literal: [1, 2, 3] (Sprint 1.2)
        let array_literal = just('[')
            .padded()
            .ignore_then(
                expr.clone()
                    .separated_by(just(',').padded())
                    .allow_trailing(),
            )
            .then_ignore(just(']').padded())
            .map(Expr::ArrayLiteral)
            .labelled("array literal");

        // Identificador - filtrar keywords para evitar que "while", "if", etc. se parseen como identificadores
        // NOTA: "self" NO está filtrado para permitir self.campo = valor en clases
        let ident = text::ident()
            .try_map(|s: String, span| {
                // Keywords que NO deben parsearse como identificadores
                let keywords = [
                    "while", "if", "else", "let", "print", "fn", "struct", "return",
                    "true", "false", "Some", "None", "Ok", "Err", "match", "end",
                    "for", "in", "break", "continue",  // For loops y control de flujo
                    "import", "pub", "mut", "class",  // Otros keywords (self NO está aquí)
                ];
                if keywords.contains(&s.as_str()) {
                    Err(Simple::custom(span, format!("'{}' is a keyword and cannot be used as an identifier", s)))
                } else {
                    Ok(Expr::Ident(s))
                }
            })
            .labelled("identifier");

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

        // Negación lógica: !expr (Prioridad 2)
        let not_expr = just("!")
            .padded()
            .ignore_then(expr.clone())
            .map(|e| Expr::Not(Box::new(e)))
            .labelled("not");

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
            .padded()  // Permite whitespace después del nombre del struct
            .then_ignore(just("{").padded())  // Consume la llave de apertura
            .then(
                        text::ident()
                    .padded()
                            .then_ignore(just(":").padded())
                            .then(expr.clone())
                            .map(|(field_name, value)| (field_name, value))
                            .separated_by(just(",").padded())
                    .allow_trailing()
                    )
            .then_ignore(just("}").padded())  // Consume la llave de cierre
            .map(|(struct_name, fields)| {
                Expr::StructLiteral {
                    name: struct_name,
                    fields,
                }
            })
            .labelled("struct literal");

        // Parser para booleanos: true, false
        // CRÍTICO: Debe venir ANTES de ident para que "true" no se parse como identificador
        // Usar just() con .padded() y verificar que no es seguido de caracteres alfanuméricos
        // Alternativa: usar lookahead negativo para asegurar que "true" no es parte de "trueVar"
        let bool_literal = just("true")
            .then(text::ident().not().to(()))  // Asegurar que no es seguido de identificador
            .map(|_| Expr::Bool(true))
            .labelled("true")
            .padded()
            .or(just("false")
                .then(text::ident().not().to(()))  // Asegurar que no es seguido de identificador
                .map(|_| Expr::Bool(false))
                .labelled("false")
                .padded());

        let atom = float_literal.clone()  // Floats PRIMERO para que 3.14 se parse como float, no como 3
            .or(number)  // Numbers después para evitar conflicto con floats
            .or(bool_literal)  // Booleanos después de numbers pero antes de string
            .or(string)
            .or(array_literal)  // Array literal antes de borrow
            .or(borrow)  // Borrow debe ir ANTES de ident para que &x se parse como Borrow, no como Call
            .or(deref)
            .or(not_expr)  // Negación lógica: !expr
            .or(some.clone())
            .or(none.clone())
            .or(ok.clone())
            .or(err.clone())
            .or(struct_literal)
            .or(ident.clone())
            .or(expr
                .clone()
                .delimited_by(just("(").padded(), just(")").padded()));

        // Parser para nombres con namespace: modulo.funcion o solo funcion (Sprint 1.3)
        let qualified_name = text::ident()
            .then(
                just(".")
                    .padded()
                    .ignore_then(text::ident())
                    .or_not()
            )
            .try_map(|(first, second), span| {
                if let Some(second) = second {
                    // modulo.funcion
                    Ok((Some(first), second))
                } else {
                    // solo funcion (sin módulo)
                    // IMPORTANTE: Evitar que Some/Ok/Err se parsen como llamadas a función
                    if first == "Some" || first == "Ok" || first == "Err" {
                        Err(Simple::custom(span, "Reserved constructor used as function name"))
                    } else {
                        Ok((None, first))
                    }
                }
            });

        // Parser para super.metodo(args) - llamada a método del padre
        let super_call = just("super")
            .padded()
            .ignore_then(just(".").padded())
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
            )
            .map(|(method, args)| Expr::SuperCall {
                method,
                args,
            });

        // Parser para method calls: obj.metodo(args)
        // NOTA: ClassName.new() se parsea como Call, no como MethodCall
        // Solo obj.metodo() donde obj es una variable se parsea como MethodCall
        let method_call = text::ident()
            .padded()
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
                )
            )
            .try_map(|(obj_name, (method, args)), span| {
                // Si el método es "new", parsear como Call (constructor), no MethodCall
                if method == "new" {
                    // Es un constructor: ClassName.new() -> Call { module: Some("ClassName"), name: "new", args }
                    Ok(Expr::Call {
                        module: Some(obj_name),
                        name: method,
                        args,
                    })
                } else {
                    // Es un método de instancia: obj.metodo() -> MethodCall
                    Ok(Expr::MethodCall {
                        object: Box::new(Expr::Ident(obj_name)),
                        method,
                        args,
                    })
                }
            });

        let call = qualified_name
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
            .map(|((module, name), args)| Expr::Call {
                module,
                name,
                args,
            })
            .or(atom);

        // Combinar: super_call tiene prioridad, luego method_call, luego call
        // super_call -> SuperCall
        // method_call puede retornar Call (para constructores) o MethodCall (para métodos)
        let call_or_method = super_call
            .or(method_call)
            .or(call);

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

        let unary = call_or_method
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

        // Indexación: arr[0] o Slicing: s[0:4] (Sprint 1.2, Sprint 2)
        let index_or_slice = with_access
            .then(
                just('[')
                    .padded()
                    .ignore_then(expr.clone())
                    .then(
                        // Detectar si viene ':' para slicing
                        just(':')
                            .padded()
                            .ignore_then(expr.clone())
                            .or_not()
                    )
                    .then_ignore(just(']').padded())
                    .repeated(),
            )
            .foldl(|arr, (idx, end_opt)| {
                if let Some(end) = end_opt {
                    // Slicing: s[0:4]
                    Expr::Slice {
                        object: Box::new(arr),
                        start: Box::new(idx),
                        end: Box::new(end),
                    }
                } else {
                    // Indexación: arr[0]
                    Expr::Index {
                        array: Box::new(arr),
                        index: Box::new(idx),
                    }
                }
            });
        
        let with_index = index_or_slice;

        // Operador ? para propagación de errores (expr?)
        let with_propagate = with_index
            .then(just("?").padded().or_not())
            .map(|(expr, has_question)| {
                if has_question.is_some() {
                    Expr::PropagateError(Box::new(expr))
                } else {
                    expr
                }
            });

        let product = with_propagate
            .clone()
            .then(
                just("*")
                    .padded()
                    .to(BinOp::Mul)
                    .or(just("/").padded().to(BinOp::Div))
                    .or(just("%").padded().to(BinOp::Mod))
                    .then(with_propagate.clone())
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
                    .padded()
                    .to(BinOp::Add)
                    .or(just("-").padded().to(BinOp::Sub))
                    .then(product.clone())
                    .repeated(),
            )
            .foldl(|l, (op, r)| Expr::BinaryOp {
                op,
                left: Box::new(l),
                right: Box::new(r),
            });

        // Operadores de comparación
        // IMPORTANTE: Los operadores de dos caracteres (<=, >=, ==, !=) DEBEN ir antes de los de un carácter (<, >)
        let comparison = sum
            .clone()
            .then(
                just("<=").padded().to(BinOp::Le)
                    .or(just(">=").padded().to(BinOp::Ge))
                    .or(just("==").padded().to(BinOp::Eq))
                    .or(just("!=").padded().to(BinOp::Ne))
                    .or(just("<").padded().to(BinOp::Lt))
                    .or(just(">").padded().to(BinOp::Gt))
                    .then(sum.clone())
                    .repeated(),
            )
            .foldl(|l, (op, r)| Expr::BinaryOp {
                op,
                left: Box::new(l),
                right: Box::new(r),
            });

        // Operadores lógicos (Prioridad 2)
        // AND lógico: a && b (mayor precedencia que OR)
        let logical_and = comparison
            .clone()
            .then(
                just("&&")
                    .padded()
                    .to(BinOp::And)
                    .then(comparison.clone())
                    .repeated(),
            )
            .foldl(|l, (op, r)| Expr::BinaryOp {
                op,
                left: Box::new(l),
                right: Box::new(r),
            });

        // OR lógico: a || b (menor precedencia que AND)
        let logical_or = logical_and
            .clone()
            .then(
                just("||")
                    .padded()
                    .to(BinOp::Or)
                    .then(logical_and.clone())
                    .repeated(),
            )
            .foldl(|l, (op, r)| Expr::BinaryOp {
                op,
                left: Box::new(l),
                right: Box::new(r),
            });

        // Parser de expresiones final con todos los operadores
        logical_or
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
        if let Stmt::Struct { name, fields, init, destroy, .. } = &program.statements[0] {
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
    fn test_parse_propagate_error_operator() {
        // Test del operador ? para propagación de errores
        let src = r#"let valor = funcion()?"#;
        let program = parse(src).unwrap();
        if let Stmt::Let { value, .. } = &program.statements[0] {
            assert!(matches!(value, Expr::PropagateError(_)));
            if let Expr::PropagateError(inner) = value {
                // El inner debe ser una llamada a función
                assert!(matches!(inner.as_ref(), Expr::Call { .. }));
            }
        } else {
            panic!("Expected Let statement with PropagateError");
        }
    }

    #[test]
    fn test_parse_propagate_error_with_method_call() {
        // Test de propagación con método: objeto.metodo()?
        let src = r#"let resultado = objeto.metodo()?"#;
        let program = parse(src).unwrap();
        if let Stmt::Let { value, .. } = &program.statements[0] {
            assert!(matches!(value, Expr::PropagateError(_)));
        } else {
            panic!("Expected Let statement with PropagateError");
        }
    }

    #[test]
    fn test_parse_propagate_error_with_ok() {
        // Test de propagación con Ok(): Ok(42)?
        let src = r#"let valor = Ok(42)?"#;
        let program = parse(src).unwrap();
        if let Stmt::Let { value, .. } = &program.statements[0] {
            assert!(matches!(value, Expr::PropagateError(_)));
            if let Expr::PropagateError(inner) = value {
                // El inner debe ser Ok(42)
                assert!(matches!(inner.as_ref(), Expr::Ok(_)));
            }
        } else {
            panic!("Expected Let statement with PropagateError");
        }
    }

    #[test]
    fn test_parse_propagate_error_chained() {
        // Test de múltiples propagaciones: funcion1()? + funcion2()?
        let src = r#"let suma = funcion1()? + funcion2()?"#;
        let program = parse(src).unwrap();
        if let Stmt::Let { value, .. } = &program.statements[0] {
            // Debe ser una BinaryOp con PropagateError en ambos lados
            assert!(matches!(value, Expr::BinaryOp { .. }));
        } else {
            panic!("Expected Let statement with BinaryOp");
        }
    }

    // ========== Tests para Arrays (Sprint 1.2) ==========
    
    #[test]
    fn test_parse_array_literal() {
        // Test de literal de array: [1, 2, 3]
        let src = r#"let arr = [1, 2, 3]"#;
        let program = parse(src).unwrap();
        if let Stmt::Let { value, .. } = &program.statements[0] {
            assert!(matches!(value, Expr::ArrayLiteral(_)));
            if let Expr::ArrayLiteral(elements) = value {
                assert_eq!(elements.len(), 3);
                assert!(matches!(elements[0], Expr::Number(1)));
                assert!(matches!(elements[1], Expr::Number(2)));
                assert!(matches!(elements[2], Expr::Number(3)));
            }
        } else {
            panic!("Expected Let statement with ArrayLiteral");
        }
    }

    #[test]
    fn test_parse_array_literal_empty() {
        // Test de array vacío: []
        let src = r#"let arr = []"#;
        let program = parse(src).unwrap();
        if let Stmt::Let { value, .. } = &program.statements[0] {
            assert!(matches!(value, Expr::ArrayLiteral(_)));
            if let Expr::ArrayLiteral(elements) = value {
                assert_eq!(elements.len(), 0);
            }
        } else {
            panic!("Expected Let statement with empty ArrayLiteral");
        }
    }

    #[test]
    fn test_parse_array_index() {
        // Test de indexación: arr[0]
        let src = r#"let valor = arr[0]"#;
        let program = parse(src).unwrap();
        if let Stmt::Let { value, .. } = &program.statements[0] {
            assert!(matches!(value, Expr::Index { .. }));
            if let Expr::Index { array, index } = value {
                assert!(matches!(array.as_ref(), Expr::Ident(_)));
                assert!(matches!(index.as_ref(), Expr::Number(0)));
            }
        } else {
            panic!("Expected Let statement with Index");
        }
    }

    #[test]
    fn test_parse_array_index_nested() {
        // Test de indexación anidada: arr[i][j]
        let src = r#"let valor = matriz[i][j]"#;
        let program = parse(src).unwrap();
        if let Stmt::Let { value, .. } = &program.statements[0] {
            assert!(matches!(value, Expr::Index { .. }));
            if let Expr::Index { array, index } = value {
                // El array debe ser otro Index
                assert!(matches!(array.as_ref(), Expr::Index { .. }));
                if let Expr::Index { array: inner_array, index: inner_index } = array.as_ref() {
                    assert!(matches!(inner_array.as_ref(), Expr::Ident(_)));
                    assert!(matches!(inner_index.as_ref(), Expr::Ident(_)));
                }
                assert!(matches!(index.as_ref(), Expr::Ident(_)));
            }
        } else {
            panic!("Expected Let statement with nested Index");
        }
    }

    #[test]
    fn test_parse_array_literal_with_expressions() {
        // Test de array con expresiones: [1 + 2, 3 * 4]
        let src = r#"let arr = [1 + 2, 3 * 4]"#;
        let program = parse(src).unwrap();
        if let Stmt::Let { value, .. } = &program.statements[0] {
            assert!(matches!(value, Expr::ArrayLiteral(_)));
            if let Expr::ArrayLiteral(elements) = value {
                assert_eq!(elements.len(), 2);
                assert!(matches!(elements[0], Expr::BinaryOp { .. }));
                assert!(matches!(elements[1], Expr::BinaryOp { .. }));
            }
        } else {
            panic!("Expected Let statement with ArrayLiteral containing expressions");
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
