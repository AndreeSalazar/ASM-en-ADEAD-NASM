// Sistema de Debug para el Parser
// Analiza qué se está parseando y genera reportes detallados

use crate::{Program, Stmt};
use std::collections::HashMap;

pub struct ParserDebugger {
    pub enabled: bool,
    pub verbose: bool,
}

impl ParserDebugger {
    pub fn new(enabled: bool, verbose: bool) -> Self {
        Self { enabled, verbose }
    }

    /// Analizar programa parseado y generar reporte
    pub fn analyze_parsed_program(&self, program: &Program, source: &str) {
        if !self.enabled {
            return;
        }

        eprintln!("\n{}", "=".repeat(80));
        eprintln!("🔍 ANÁLISIS DEL PARSER - PROGRAMA PARSEADO");
        eprintln!("{}", "=".repeat(80));
        eprintln!();

        // Resumen ejecutivo
        eprintln!("📊 RESUMEN EJECUTIVO");
        eprintln!("{}", "-".repeat(80));
        eprintln!("Total de Statements Parseados: {}", program.statements.len());
        eprintln!("Tamaño del código fuente: {} caracteres", source.len());
        eprintln!("Líneas de código: {}", source.lines().count());
        eprintln!();

        // Analizar cada statement
        let mut structs = Vec::new();
        let mut functions = Vec::new();
        let mut let_statements = Vec::new();
        let mut print_statements = Vec::new();
        let mut other_statements = Vec::new();

        for (i, stmt) in program.statements.iter().enumerate() {
            match stmt {
                Stmt::Struct { name, fields, .. } => {
                    structs.push((i, name.clone(), fields.len()));
                }
                Stmt::Fn { name, params, body, .. } => {
                    functions.push((i, name.clone(), params.len(), body.len()));
                }
                Stmt::Let { name, mutable, value } => {
                    let value_type = format!("{:?}", value);
                    let value_preview = if value_type.len() > 50 {
                        format!("{}...", &value_type[..50])
                    } else {
                        value_type
                    };
                    let_statements.push((i, name.clone(), *mutable, value_preview));
                }
                Stmt::Print(expr) => {
                    let expr_type = format!("{:?}", expr);
                    let expr_preview = if expr_type.len() > 50 {
                        format!("{}...", &expr_type[..50])
                    } else {
                        expr_type
                    };
                    print_statements.push((i, expr_preview));
                }
                _ => {
                    other_statements.push((i, format!("{:?}", stmt)));
                }
            }
        }

        // Detalles de Structs
        if !structs.is_empty() {
            eprintln!("🏗️  STRUCTS PARSEADOS");
            eprintln!("{}", "-".repeat(80));
            for (idx, name, field_count) in &structs {
                eprintln!("  [{}] struct {} ({} campos)", idx, name, field_count);
            }
            eprintln!();
        }

        // Detalles de Funciones
        if !functions.is_empty() {
            eprintln!("⚙️  FUNCIONES PARSEADAS");
            eprintln!("{}", "-".repeat(80));
            for (idx, name, param_count, body_len) in &functions {
                eprintln!("  [{}] fn {}({} params, {} statements en body)", idx, name, param_count, body_len);
            }
            eprintln!();
        }

        // Detalles de Let Statements
        if !let_statements.is_empty() {
            eprintln!("📝 STATEMENTS LET PARSEADOS");
            eprintln!("{}", "-".repeat(80));
            for (idx, name, mutable, value_preview) in &let_statements {
                let mut_str = if *mutable { "mut " } else { "" };
                eprintln!("  [{}] let {}{} = {}", idx, mut_str, name, value_preview);
            }
            eprintln!();
        } else {
            eprintln!("⚠️  STATEMENTS LET");
            eprintln!("{}", "-".repeat(80));
            eprintln!("  ❌ NO SE ENCONTRARON STATEMENTS LET!");
            eprintln!("  Esto puede indicar un problema de parsing.");
            eprintln!();
        }

        // Detalles de Print Statements
        if !print_statements.is_empty() {
            eprintln!("🖨️  STATEMENTS PRINT PARSEADOS");
            eprintln!("{}", "-".repeat(80));
            for (idx, expr_preview) in &print_statements {
                eprintln!("  [{}] print {}", idx, expr_preview);
            }
            eprintln!();
        } else {
            eprintln!("⚠️  STATEMENTS PRINT");
            eprintln!("{}", "-".repeat(80));
            eprintln!("  ❌ NO SE ENCONTRARON STATEMENTS PRINT!");
            eprintln!("  Esto puede indicar un problema de parsing.");
            eprintln!();
        }

        // Otros Statements
        if !other_statements.is_empty() {
            eprintln!("📋 OTROS STATEMENTS");
            eprintln!("{}", "-".repeat(80));
            for (idx, stmt_preview) in &other_statements {
                eprintln!("  [{}] {}", idx, stmt_preview);
            }
            eprintln!();
        }

        // Detectar problemas
        let expected_statements = source.lines()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.starts_with("let ") || trimmed.starts_with("print ")
            })
            .count();

        let parsed_statements = let_statements.len() + print_statements.len();

        if expected_statements > parsed_statements {
            eprintln!("🚨 PROBLEMA DETECTADO");
            eprintln!("{}", "-".repeat(80));
            eprintln!("  ❌ Se esperaban {} statements Let/Print pero solo se parsearon {}", 
                expected_statements, parsed_statements);
            eprintln!("  💡 Posibles causas:");
            eprintln!("     - El parser puede estar deteniéndose después de funciones");
            eprintln!("     - Los comentarios pueden estar interfiriendo");
            eprintln!("     - El orden de precedencia puede estar causando problemas");
            eprintln!();
        }

        // Análisis de líneas del código fuente
        if self.verbose {
            eprintln!("📄 ANÁLISIS DEL CÓDIGO FUENTE");
            eprintln!("{}", "-".repeat(80));
            for (i, line) in source.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("let ") || trimmed.starts_with("print ") {
                    eprintln!("  Línea {}: {} [DEBE PARSEARSE]", i + 1, trimmed);
                } else if trimmed.starts_with("fn ") || trimmed.starts_with("struct ") {
                    eprintln!("  Línea {}: {} [DEFINICIÓN]", i + 1, trimmed);
                } else if trimmed.starts_with("#") {
                    eprintln!("  Línea {}: {} [COMENTARIO]", i + 1, trimmed);
                } else if !trimmed.is_empty() {
                    eprintln!("  Línea {}: {} [OTRO]", i + 1, trimmed);
                }
            }
            eprintln!();
        }

        eprintln!("{}", "=".repeat(80));
        eprintln!();
    }

    /// Analizar un statement individual
    pub fn analyze_statement(&self, idx: usize, stmt: &Stmt) {
        if !self.enabled || !self.verbose {
            return;
        }

        eprintln!("[PARSER-DEBUG] Statement {}: {:?}", idx, stmt);
    }
}

