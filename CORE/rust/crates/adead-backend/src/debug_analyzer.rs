// Sistema de Debug Inteligente - Estilo Python
// Analiza todo el proceso de compilación y genera reportes detallados

use adead_parser::{Expr, Program, Stmt};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DebugInfo {
    pub total_statements: usize,
    pub structs: Vec<String>,
    pub functions: Vec<FunctionInfo>,
    pub other_statements: Vec<StatementInfo>,
    pub struct_definitions: HashMap<String, Vec<String>>,
    pub struct_methods: HashMap<String, Vec<String>>,
    pub variables: Vec<VariableInfo>,
    pub issues: Vec<Issue>,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub is_struct_method: bool,
    pub struct_name: Option<String>,
    pub method_name: Option<String>,
    pub is_constructor: bool,
    pub is_static: bool,
    pub has_self: bool,
    pub params_count: usize,
}

#[derive(Debug, Clone)]
pub struct StatementInfo {
    pub index: usize,
    pub stmt_type: String,
    pub details: String,
    pub parsed_correctly: bool,
}

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub name: String,
    pub var_type: Option<String>,
    pub is_mutable: bool,
    pub is_used: bool,
}

#[derive(Debug, Clone)]
pub struct Issue {
    pub severity: IssueSeverity,
    pub category: String,
    pub message: String,
    pub suggestion: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

pub struct DebugAnalyzer {
    pub enabled: bool,
    pub verbose: bool,
}

impl DebugAnalyzer {
    pub fn new(enabled: bool, verbose: bool) -> Self {
        Self { enabled, verbose }
    }

    /// Analizar programa completo y generar reporte
    pub fn analyze_program(&self, program: &Program) -> DebugInfo {
        // Siempre analizar, incluso si enabled es false (para debugging interno)
        let mut info = DebugInfo {
            total_statements: program.statements.len(),
            structs: Vec::new(),
            functions: Vec::new(),
            other_statements: Vec::new(),
            struct_definitions: HashMap::new(),
            struct_methods: HashMap::new(),
            variables: Vec::new(),
            issues: Vec::new(),
        };
        
        // Debug inicial
        if self.verbose {
            eprintln!("[DEBUG-ANALYZER] Analizando programa con {} statements", program.statements.len());
        }

        // Analizar cada statement
        for (i, stmt) in program.statements.iter().enumerate() {
            match stmt {
                Stmt::Struct { name, fields, .. } => {
                    info.structs.push(name.clone());
                    let field_names: Vec<String> = fields.iter().map(|f| f.name.clone()).collect();
                    info.struct_definitions.insert(name.clone(), field_names);
                }
                Stmt::Fn { name, params, .. } => {
                    let func_info = self.analyze_function(name, params, &info.struct_definitions);
                    info.functions.push(func_info);
                }
                Stmt::Let { name, mutable, value } => {
                    let stmt_info = self.analyze_let_statement(i, name, *mutable, value);
                    info.other_statements.push(stmt_info);
                }
                Stmt::Print(expr) => {
                    let stmt_info = self.analyze_print_statement(i, expr);
                    info.other_statements.push(stmt_info);
                }
                _ => {
                    let stmt_info = StatementInfo {
                        index: i,
                        stmt_type: format!("{:?}", stmt),
                        details: "Other statement".to_string(),
                        parsed_correctly: true,
                    };
                    info.other_statements.push(stmt_info);
                }
            }
        }

        // Detectar problemas automáticamente
        self.detect_issues(&mut info);
        
        // Debug final
        if self.verbose {
            eprintln!("[DEBUG-ANALYZER] Análisis completo: {} structs, {} funciones, {} otros statements, {} problemas detectados", 
                info.structs.len(), info.functions.len(), info.other_statements.len(), info.issues.len());
        }

        info
    }

    /// Analizar función y determinar su tipo
    fn analyze_function(
        &self,
        name: &str,
        params: &[adead_parser::FnParam],
        struct_definitions: &HashMap<String, Vec<String>>,
    ) -> FunctionInfo {
        let mut is_struct_method = false;
        let mut struct_name = None;
        let mut method_name = None;
        let mut is_constructor = false;
        let mut is_static = false;
        let mut has_self = false;

        // Detectar patrón StructName_methodName
        if let Some(underscore_pos) = name.find('_') {
            let potential_struct = &name[..underscore_pos];
            let potential_method = &name[underscore_pos + 1..];

            if struct_definitions.contains_key(potential_struct) {
                is_struct_method = true;
                struct_name = Some(potential_struct.to_string());
                method_name = Some(potential_method.to_string());
                is_constructor = potential_method == "new";
            }
        }

        // Verificar si tiene self
        if let Some(first_param) = params.first() {
            has_self = first_param.name == "self";
            if is_struct_method && !has_self && !is_constructor {
                is_static = true;
            }
        } else if is_struct_method && !is_constructor {
            is_static = true;
        }

        FunctionInfo {
            name: name.to_string(),
            is_struct_method,
            struct_name,
            method_name,
            is_constructor,
            is_static,
            has_self,
            params_count: params.len(),
        }
    }

    /// Analizar statement Let
    fn analyze_let_statement(
        &self,
        index: usize,
        name: &str,
        mutable: bool,
        value: &Expr,
    ) -> StatementInfo {
        let details = match value {
            Expr::Call { module, name: call_name, args } => {
                if let Some(mod_name) = module {
                    format!("Call: {}.{}({} args)", mod_name, call_name, args.len())
                } else {
                    format!("Call: {}({} args)", call_name, args.len())
                }
            }
            Expr::MethodCall { method, args, .. } => {
                format!("MethodCall: {}({} args)", method, args.len())
            }
            Expr::StructLiteral { name: struct_name, .. } => {
                format!("StructLiteral: {}", struct_name)
            }
            _ => format!("{:?}", value),
        };

        StatementInfo {
            index,
            stmt_type: "Let".to_string(),
            details: format!("let {} = {}", name, details),
            parsed_correctly: true,
        }
    }

    /// Analizar statement Print
    fn analyze_print_statement(&self, index: usize, expr: &Expr) -> StatementInfo {
        let details = match expr {
            Expr::Ident(name) => format!("Ident: {}", name),
            Expr::FieldAccess { field, .. } => format!("FieldAccess: .{}", field),
            Expr::MethodCall { method, args, .. } => {
                format!("MethodCall: {}({} args)", method, args.len())
            }
            _ => format!("{:?}", expr),
        };

        StatementInfo {
            index,
            stmt_type: "Print".to_string(),
            details: format!("print {}", details),
            parsed_correctly: true,
        }
    }

    /// Detectar problemas automáticamente
    fn detect_issues(&self, info: &mut DebugInfo) {
        // Detectar si hay statements Let/Print parseados pero no se procesan
        let let_print_count = info
            .other_statements
            .iter()
            .filter(|s| s.stmt_type == "Let" || s.stmt_type == "Print")
            .count();

        // Calcular statements esperados vs detectados
        let expected_other = info.total_statements - info.structs.len() - info.functions.len();
        
        // PROBLEMA CRÍTICO: Si hay statements esperados pero no detectados
        if expected_other > 0 && let_print_count == 0 {
            info.issues.push(Issue {
                severity: IssueSeverity::Error,
                category: "Parsing".to_string(),
                message: format!(
                    "CRÍTICO: Se esperaban {} statements adicionales (Let/Print) pero NO se detectaron. El parser puede no estar parseando correctamente los statements después de funciones.",
                    expected_other
                ),
                suggestion: "Verificar que el parser esté parseando correctamente TODOS los statements, especialmente los que vienen después de definiciones de funciones".to_string(),
                location: Some("stmt_parser en adead-parser".to_string()),
            });
        }
        
        // Si hay statements detectados pero son menos de los esperados
        if expected_other > let_print_count && let_print_count > 0 {
            info.issues.push(Issue {
                severity: IssueSeverity::Warning,
                category: "Parsing".to_string(),
                message: format!(
                    "Se esperaban {} statements adicionales pero solo se detectaron {} statements Let/Print",
                    expected_other, let_print_count
                ),
                suggestion: "Algunos statements pueden no estar siendo parseados correctamente".to_string(),
                location: Some("stmt_parser".to_string()),
            });
        }

        // Detectar métodos estáticos que no se generan
        for func in &info.functions {
            if func.is_static && func.is_struct_method {
                if let Some(ref struct_name) = func.struct_name {
                    if !info.struct_methods.contains_key(struct_name) {
                        info.issues.push(Issue {
                            severity: IssueSeverity::Warning,
                            category: "Code Generation".to_string(),
                            message: format!(
                                "Método estático '{}' de struct '{}' puede no generarse",
                                func.name, struct_name
                            ),
                            suggestion: "Verificar que los métodos estáticos se generen como funciones globales".to_string(),
                            location: Some(format!("fn_{}", func.name)),
                        });
                    }
                }
            }
        }

        // Detectar funciones que parecen métodos pero no tienen struct asociado
        for func in &info.functions {
            if func.name.contains('_') && !func.is_struct_method {
                let parts: Vec<&str> = func.name.split('_').collect();
                if parts.len() >= 2 {
                    let potential_struct = parts[0];
                    if info.struct_definitions.contains_key(potential_struct) {
                        info.issues.push(Issue {
                            severity: IssueSeverity::Info,
                            category: "Naming".to_string(),
                            message: format!(
                                "Función '{}' sigue patrón StructName_method pero no se detecta como método",
                                func.name
                            ),
                            suggestion: "Verificar detección de métodos estáticos".to_string(),
                            location: Some(func.name.clone()),
                        });
                    }
                }
            }
        }
    }

    /// Generar reporte detallado en formato legible (Python Style)
    pub fn generate_report(&self, info: &DebugInfo) -> String {
        if !self.enabled {
            return String::new();
        }

        let mut report = String::new();
        
        report.push_str(&"=".repeat(80));
        report.push_str("\n");
        report.push_str("🔍 ANÁLISIS INTELIGENTE DEL PROGRAMA\n");
        report.push_str(&"=".repeat(80));
        report.push_str("\n\n");

        // Resumen ejecutivo
        report.push_str("📊 RESUMEN EJECUTIVO\n");
        report.push_str(&"-".repeat(80));
        report.push_str("\n");
        report.push_str(&format!("Total de Statements: {}\n", info.total_statements));
        report.push_str(&format!("Structs: {}\n", info.structs.len()));
        report.push_str(&format!("Funciones: {}\n", info.functions.len()));
        report.push_str(&format!("Other Statements: {}\n", info.other_statements.len()));
        report.push_str(&format!("Problemas Detectados: {}\n", info.issues.len()));
        report.push_str("\n");

        // Detalles de Structs
        if !info.structs.is_empty() {
            report.push_str("🏗️  STRUCTS DETECTADOS\n");
            report.push_str(&"-".repeat(80));
            report.push_str("\n");
            for struct_name in &info.structs {
                if let Some(fields) = info.struct_definitions.get(struct_name) {
                    report.push_str(&format!("  • {} ({} campos)\n", struct_name, fields.len()));
                    for field in fields {
                        report.push_str(&format!("    - {}\n", field));
                    }
                }
            }
            report.push_str("\n");
        }

        // Detalles de Funciones
        if !info.functions.is_empty() {
            report.push_str("⚙️  FUNCIONES DETECTADAS\n");
            report.push_str(&"-".repeat(80));
            report.push_str("\n");
            for func in &info.functions {
                let func_type = if func.is_constructor {
                    "🔨 Constructor"
                } else if func.is_static {
                    "📦 Método Estático"
                } else if func.is_struct_method {
                    "🔧 Método de Instancia"
                } else {
                    "🌐 Función Global"
                };

                report.push_str(&format!("  {} {}\n", func_type, func.name));
                report.push_str(&format!("    Parámetros: {}\n", func.params_count));
                if let Some(ref struct_name) = func.struct_name {
                    report.push_str(&format!("    Struct: {}\n", struct_name));
                }
                if let Some(ref method_name) = func.method_name {
                    report.push_str(&format!("    Método: {}\n", method_name));
                }
                report.push_str(&format!("    Tiene self: {}\n", func.has_self));
                report.push_str("\n");
            }
        }

        // Detalles de Other Statements
        if !info.other_statements.is_empty() {
            report.push_str("📝 STATEMENTS EN MAIN\n");
            report.push_str(&"-".repeat(80));
            report.push_str("\n");
            for stmt in &info.other_statements {
                let status = if stmt.parsed_correctly { "✅" } else { "❌" };
                report.push_str(&format!("  {} [{}] {}\n", status, stmt.index, stmt.details));
            }
            report.push_str("\n");
        } else {
            report.push_str("⚠️  STATEMENTS EN MAIN\n");
            report.push_str(&"-".repeat(80));
            report.push_str("\n");
            report.push_str("  ❌ NO HAY STATEMENTS PARA PROCESAR EN MAIN!\n");
            report.push_str("  Esto puede indicar un problema de parsing o procesamiento.\n");
            report.push_str("\n");
        }

        // Problemas Detectados
        if !info.issues.is_empty() {
            report.push_str("🚨 PROBLEMAS DETECTADOS\n");
            report.push_str(&"-".repeat(80));
            report.push_str("\n");
            
            let errors: Vec<&Issue> = info.issues.iter().filter(|i| i.severity == IssueSeverity::Error).collect();
            let warnings: Vec<&Issue> = info.issues.iter().filter(|i| i.severity == IssueSeverity::Warning).collect();
            let infos: Vec<&Issue> = info.issues.iter().filter(|i| i.severity == IssueSeverity::Info).collect();

            if !errors.is_empty() {
                report.push_str("  ❌ ERRORES:\n");
                for issue in errors {
                    report.push_str(&format!("    • [{}] {}\n", issue.category, issue.message));
                    report.push_str(&format!("      💡 Sugerencia: {}\n", issue.suggestion));
                    if let Some(ref loc) = issue.location {
                        report.push_str(&format!("      📍 Ubicación: {}\n", loc));
                    }
                    report.push_str("\n");
                }
            }

            if !warnings.is_empty() {
                report.push_str("  ⚠️  ADVERTENCIAS:\n");
                for issue in warnings {
                    report.push_str(&format!("    • [{}] {}\n", issue.category, issue.message));
                    report.push_str(&format!("      💡 Sugerencia: {}\n", issue.suggestion));
                    if let Some(ref loc) = issue.location {
                        report.push_str(&format!("      📍 Ubicación: {}\n", loc));
                    }
                    report.push_str("\n");
                }
            }

            if !infos.is_empty() {
                report.push_str("  ℹ️  INFORMACIONES:\n");
                for issue in infos {
                    report.push_str(&format!("    • [{}] {}\n", issue.category, issue.message));
                    report.push_str(&format!("      💡 Sugerencia: {}\n", issue.suggestion));
                    report.push_str("\n");
                }
            }
        } else {
            report.push_str("✅ NO SE DETECTARON PROBLEMAS\n");
            report.push_str(&"-".repeat(80));
            report.push_str("\n");
        }

        report.push_str(&"=".repeat(80));
        report.push_str("\n");

        report
    }

    /// Imprimir reporte de forma legible
    pub fn print_report(&self, info: &DebugInfo) {
        if self.enabled {
            let report = self.generate_report(info);
            // Forzar output a stderr (siempre visible)
            eprintln!("\n{}", report);
            eprintln!("\n"); // Línea en blanco para separar
        }
    }
}

impl DebugInfo {
    fn empty() -> Self {
        Self {
            total_statements: 0,
            structs: Vec::new(),
            functions: Vec::new(),
            other_statements: Vec::new(),
            struct_definitions: HashMap::new(),
            struct_methods: HashMap::new(),
            variables: Vec::new(),
            issues: Vec::new(),
        }
    }
}

