use adead_common::Result;
use adead_parser::{BinOp, Expr, Pattern, Program, Stmt, StructMethod};
use std::collections::HashMap;

mod memory_pool;
mod optimizer;
mod stdlib;
mod register_optimizer;
mod dependency_graph;
mod usage_analyzer;
mod debug_analyzer;
use optimizer::CodeOptimizer;
use stdlib::StdLib;
use dependency_graph::DependencyGraph;
use usage_analyzer::UsageAnalyzer;
use debug_analyzer::DebugAnalyzer;

/// Contexto para loops (NASM-Universal.md - Break/Continue)
/// Permite manejar break y continue dentro de loops anidados
struct LoopContext {
    break_label: String,    // Label para saltar cuando se ejecuta break
    continue_label: String, // Label para saltar cuando se ejecuta continue
}

pub struct CodeGenerator {
    data_section: Vec<String>,
    text_section: Vec<String>,
    string_counter: usize,
    label_counter: usize,
    variables: HashMap<String, i64>, // simple: track stack offsets
    stack_offset: i64,
    structs_with_destroy: HashMap<String, bool>, // Track structs that have destroy methods (O2.1)
    variables_to_destroy: Vec<(String, String)>, // (variable_name, struct_name) - RAII tracking
    source_lines: Vec<(usize, String)>, // (line_number, source_line) para debug symbols
    current_line: usize, // Línea actual del código fuente
    loop_stack: Vec<LoopContext>, // Stack de contextos de loop para break/continue
    struct_definitions: HashMap<String, Vec<String>>, // Track struct field names for offset calculation
    variable_types: HashMap<String, String>, // Track variable types (for struct field access)
    current_struct: Option<String>, // Struct actual que se está procesando (para super.metodo())
    struct_parents: HashMap<String, Option<String>>, // Mapeo de struct -> parent (para herencia)
    struct_methods: HashMap<String, Vec<String>>, // Track métodos de cada struct (para vtables)
    vtable_counter: usize, // Contador para nombres únicos de vtables
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            data_section: Vec::new(),
            text_section: Vec::new(),
            string_counter: 0,
            label_counter: 0,
            variables: HashMap::new(),
            stack_offset: 0,
            structs_with_destroy: HashMap::new(),
            variables_to_destroy: Vec::new(),
            source_lines: Vec::new(),
            current_line: 0,
            loop_stack: Vec::new(),
            struct_definitions: HashMap::new(),
            variable_types: HashMap::new(),
            current_struct: None,
            struct_parents: HashMap::new(),
            struct_methods: HashMap::new(),
            vtable_counter: 0,
        }
    }
    
    /// Agregar comentario de debug con origen ADead
    fn add_debug_comment(&mut self, comment: &str) {
        if self.current_line > 0 {
            self.text_section.push(format!("    ; ADead: line {} - {}", self.current_line, comment));
        } else {
            self.text_section.push(format!("    ; ADead: {}", comment));
        }
    }

    pub fn generate(&mut self, program: &Program) -> Result<String> {
        // Detectar si estamos en Windows (usaremos API de Windows)
        #[cfg(target_os = "windows")]
        {
            self.generate_windows(program)
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.generate_linux(program)
        }
    }

    fn generate_linux(&mut self, program: &Program) -> Result<String> {
        self.text_section.push("section .text".to_string());
        self.text_section.push("global _start".to_string());
        self.text_section.push("_start:".to_string());
        self.text_section.push("    ; Setup stack frame".to_string());
        self.text_section.push("    mov rbp, rsp".to_string());
        self.text_section.push("    sub rsp, 1024  ; allocate stack space".to_string());

        for stmt in &program.statements {
            self.generate_stmt(stmt)?;
        }

        // Exit syscall
        self.text_section.push("    ; Exit".to_string());
        self.text_section.push("    mov rax, 60".to_string());
        self.text_section.push("    mov rdi, 0".to_string());
        self.text_section.push("    syscall".to_string());

        self.finish_generation()
    }

    fn generate_windows(&mut self, program: &Program) -> Result<String> {
        // ============================================
        // DEBUG INTELIGENTE: Análisis Completo del Programa
        // ============================================
        // Activar debug inteligente (siempre activo para análisis completo)
        eprintln!("\n[DEBUG] Iniciando análisis inteligente del programa...");
        let debug_analyzer = DebugAnalyzer::new(true, true);
        let debug_info = debug_analyzer.analyze_program(program);
        eprintln!("[DEBUG] Análisis completo: {} statements, {} structs, {} funciones, {} otros statements", 
            debug_info.total_statements, debug_info.structs.len(), debug_info.functions.len(), debug_info.other_statements.len());
        
        // Imprimir reporte detallado estilo Python
        let report = debug_analyzer.generate_report(&debug_info);
        if !report.is_empty() {
            // Forzar output a ambos streams para máxima visibilidad
            println!("\n{}", report);
            eprintln!("\n{}", report);
            // También escribir a archivo para debugging
            if let Ok(mut debug_file) = std::fs::File::create("debug_analysis.txt") {
                use std::io::Write;
                let _ = writeln!(debug_file, "{}", report);
            }
            // Forzar flush inmediato
            use std::io::Write;
            std::io::stdout().flush().ok();
            std::io::stderr().flush().ok();
        } else {
            eprintln!("[DEBUG] WARNING: El reporte está vacío!");
        }
        
        // Windows x64 calling convention:
        // - Parameters: RCX, RDX, R8, R9 (first 4 integer params)
        // - Shadow space: 32 bytes must be reserved before each call
        // - Return: RAX
        // - Callee-saved: RBX, RBP, RDI, RSI, R12-R15, XMM6-XMM15
        self.text_section.push("default rel".to_string());  // Usar direccionamiento relativo para mejor compatibilidad
        self.text_section.push("section .text".to_string());
        self.text_section.push("extern GetStdHandle".to_string());
        self.text_section.push("extern WriteFile".to_string());
        self.text_section.push("extern ExitProcess".to_string());
        self.text_section.push("extern VirtualAlloc".to_string());
        self.text_section.push("extern VirtualFree".to_string());
        self.text_section.push("global main".to_string());
        
        // ============================================
        // DEAD CODE ELIMINATION: Análisis Estático
        // ============================================
        // Analizar el programa para detectar qué funciones del runtime se usan
        let mut deps = DependencyGraph::new();
        UsageAnalyzer::analyze_program(program, &mut deps);
        
        // ============================================
        // RUNTIME BOUNDARY: Funciones Helper del Runtime
        // ============================================
        // Estas funciones son parte del runtime de ADead
        // NO son código generado del usuario, son helpers del sistema
        // SOLO se generan si se usan (dead code elimination)
        
        // Generar sistema de panic solo si se usa
        let uses_panic = deps.uses_panic();
        let uses_arrays = deps.uses_arrays();
        let uses_strings = deps.uses_strings();
        
        if uses_panic {
            self.generate_panic_system();
        }
        
        // Generar funciones helper de Array solo si se usan
        if uses_arrays {
            self.generate_array_helpers_nasm_selective(&deps);
        }
        
        // Generar funciones helper de String solo si se usan
        if uses_strings {
            self.generate_string_helpers_nasm_selective(&deps);
        }
        
        // ============================================
        // RUNTIME BOUNDARY: Librería Estándar (Stdlib)
        // ============================================
        // Funciones predefinidas disponibles en todos los programas
        // Parte del runtime, no código generado del usuario
        
        // Generar librería estándar (funciones predefinidas)
        let stdlib_code = StdLib::generate_stdlib_nasm();
        for line in stdlib_code {
            self.text_section.push(line);
        }
        
        // ============================================
        // RUNTIME BOUNDARY END: Código Generado del Usuario
        // ============================================
        
        // Separar statements por tipo para procesar en orden correcto
        // Orden: 1) Structs (para registrar tipos), 2) Funciones, 3) Resto
        let mut structs = Vec::new();
        let mut user_functions = Vec::new();
        let mut other_statements = Vec::new();
        
        // DEBUG: Contar statements totales
        eprintln!("[DEBUG] Total statements parseados: {}", program.statements.len());
        
        for (i, stmt) in program.statements.iter().enumerate() {
            match stmt {
                Stmt::Struct { name, .. } => {
                    eprintln!("[DEBUG] Statement {}: Struct '{}'", i, name);
                    structs.push(stmt);
                }
                Stmt::Fn { name, .. } => {
                    eprintln!("[DEBUG] Statement {}: Function '{}'", i, name);
                    user_functions.push(stmt);
                }
                Stmt::Let { name, .. } => {
                    eprintln!("[DEBUG] Statement {}: Let '{}' -> other_statements", i, name);
                    other_statements.push(stmt);
                }
                Stmt::Print(_) => {
                    eprintln!("[DEBUG] Statement {}: Print -> other_statements", i);
                    other_statements.push(stmt);
                }
                _ => {
                    eprintln!("[DEBUG] Statement {}: Other -> other_statements", i);
                    other_statements.push(stmt);
                }
            }
        }
        
        eprintln!("[DEBUG] Structs: {}, Functions: {}, Other: {}", structs.len(), user_functions.len(), other_statements.len());
        
        // 1. Registrar structs primero (sin generar código, solo registrar tipos)
        for stmt in &structs {
            if let Stmt::Struct { name, fields, .. } = stmt {
                let field_names: Vec<String> = fields.iter().map(|f| f.name.clone()).collect();
                self.struct_definitions.insert(name.clone(), field_names);
            }
        }
        
        // 2. Asociar funciones globales con structs (si siguen patrón StructName_method)
        // Esto permite usar fn StructName_method(self, ...) como métodos
        let mut struct_methods_from_functions: HashMap<String, Vec<(String, StructMethod)>> = HashMap::new();
        for stmt in &user_functions {
            if let Stmt::Fn { name, params, body, .. } = stmt {
                // Detectar patrón: StructName_methodName
                if let Some(underscore_pos) = name.find('_') {
                    let struct_name = &name[..underscore_pos];
                    let method_name = &name[underscore_pos + 1..];
                    
                    // Verificar si el struct existe
                    if self.struct_definitions.contains_key(struct_name) {
                        // Si es 'new', es un constructor (puede o no tener 'self')
                        // Si tiene 'self' como primer parámetro, es un método de instancia
                        // Si NO tiene 'self', es un método estático
                        let is_constructor = method_name == "new";
                        let is_instance_method = params.first().map(|p| p.name == "self").unwrap_or(false);
                        let is_static_method = !is_constructor && !is_instance_method;
                        
                        // Registrar TODOS los métodos (constructores, instancia y estáticos)
                        if is_constructor || is_instance_method || is_static_method {
                            let method = StructMethod {
                                visibility: adead_parser::Visibility::Public,
                                params: params.clone(),
                                body: body.clone(),
                            };
                            struct_methods_from_functions
                                .entry(struct_name.to_string())
                                .or_insert_with(Vec::new)
                                .push((method_name.to_string(), method));
                        }
                    }
                }
            }
        }
        
        // 3. Generar funciones de usuario (que no son métodos de struct)
        for stmt in &user_functions {
            if let Stmt::Fn { name, params, .. } = stmt {
                // Detectar si es método de struct
                let is_struct_method = if let Some(underscore_pos) = name.find('_') {
                    let struct_name = &name[..underscore_pos];
                    if self.struct_definitions.contains_key(struct_name) {
                        // Verificar si es constructor o método de instancia (con self)
                        let method_name = &name[underscore_pos + 1..];
                        let is_constructor = method_name == "new";
                        let is_instance_method = params.first().map(|p| p.name == "self").unwrap_or(false);
                        // Si es constructor o método de instancia, NO generar aquí (se generará con el struct)
                        // Si es método estático (sin self), SÍ generar aquí como función global
                        is_constructor || is_instance_method
                    } else {
                        false
                    }
                } else {
                    false
                };
                
                if !is_struct_method {
                    // Generar función global (no es método de struct)
            self.generate_stmt_windows(stmt)?;
                }
            } else {
                // No es función, generar normalmente
                self.generate_stmt_windows(stmt)?;
            }
        }
        
        // 4. Generar structs (código de métodos y constructores)
        // Si hay métodos desde funciones globales, agregarlos a los structs
        for stmt in &structs {
            match stmt {
                Stmt::Struct { name, parent, fields, init, destroy, methods } => {
                    let mut all_methods = methods.clone();
                    if let Some(additional_methods) = struct_methods_from_functions.get(name) {
                        // Agregar métodos adicionales desde funciones globales
                        for (method_name, method) in additional_methods {
                            all_methods.push((method_name.clone(), method.clone()));
                        }
                    }
                    let modified_stmt = Stmt::Struct {
                        name: name.clone(),
                        parent: parent.clone(),
                        fields: fields.clone(),
                        init: init.clone(),
                        destroy: destroy.clone(),
                        methods: all_methods,
                    };
                    self.generate_stmt_windows(&modified_stmt)?;
                }
                _ => {
                    self.generate_stmt_windows(stmt)?;
                }
            }
        }
        
        self.text_section.push("main:".to_string());
        
        // Setup stack frame (Windows x64)
        // Reserve space for: shadow space (32) + local variables (32 for stdout handle + bytes written)
        // Windows requires stack to be aligned to 16 bytes
        self.text_section.push("    ; Setup stack frame (Windows x64)".to_string());
        self.text_section.push("    push rbp".to_string());
        self.text_section.push("    mov rbp, rsp".to_string());
        self.text_section.push("    ; Align stack to 16 bytes (Windows x64 requirement)".to_string());
        self.text_section.push("    and rsp, -16".to_string());
        self.text_section.push("    sub rsp, 64  ; Allocate space for shadow space (32) + local vars (32)".to_string());

        // Obtener handle de stdout (Windows)
        // STD_OUTPUT_HANDLE = -11
        self.text_section.push("    ; Get stdout handle (STD_OUTPUT_HANDLE = -11)".to_string());
        self.text_section.push("    mov ecx, -11".to_string());
        self.text_section.push("    call GetStdHandle".to_string());
        self.text_section.push("    mov [rbp+16], rax  ; save stdout handle at [rbp+16]".to_string());

        // Generar otros statements (no funciones) dentro del main
        // El debug inteligente ya detectó problemas arriba si los hay
        for stmt in other_statements.iter() {
            self.generate_stmt_windows(stmt)?;
        }

        // RAII: Llamar destructores antes de salir (O2.1 - Drop Trait)
        // Llamar destructores en orden inverso (LIFO - Last In First Out)
        for (var_name, struct_name) in self.variables_to_destroy.iter().rev() {
            if let Some(&offset) = self.variables.get(var_name) {
                self.text_section.push(format!("    ; RAII: destruyendo {} (tipo {})", var_name, struct_name));
                // Cargar dirección del struct
                self.text_section.push(format!("    mov rcx, [rbp - {}]  ; cargar dirección de {}", offset + 8, var_name));
                // Llamar destructor
                self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                self.text_section.push(format!("    call {}_destroy", struct_name));
                self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
            }
        }

        // Exit process with code 0 (success)
        // VOID ExitProcess(UINT uExitCode)
        // uExitCode in RCX (ECX for 32-bit value is fine)
        self.text_section.push("    ; Exit process".to_string());
        self.text_section.push("    mov ecx, 0  ; Exit code 0 (success)".to_string());
        self.text_section.push("    call ExitProcess".to_string());
        // ExitProcess nunca retorna, pero por si acaso:
        self.text_section.push("    ; Should never reach here (ExitProcess terminates process)".to_string());

        self.text_section.push("    leave".to_string());
        self.text_section.push("    ret".to_string());

        self.finish_generation()
    }

    fn finish_generation(&mut self) -> Result<String> {
        // OPTIMIZACIÓN: Aplicar optimizaciones al código generado
        let optimizer = CodeOptimizer::new();
        let text_code = self.text_section.join("\n");
        
        // Eliminar código muerto y limpiar formato
        let cleaned_text = optimizer.clean_asm(&text_code);
        let optimized_lines: Vec<String> = cleaned_text.lines().map(|s| s.to_string()).collect();
        
        let mut output = String::new();
        
        // Header con información del compilador
        output.push_str("; ═══════════════════════════════════════════════════════════════\n");
        output.push_str("; ADead Compiler - Generated NASM x86_64 (Windows)\n");
        output.push_str("; ═══════════════════════════════════════════════════════════════\n\n");
        
        // Data section primero (necesario para Windows)
        if !self.data_section.is_empty() {
            output.push_str("; ─────────────────────────────────────────────────────────────────\n");
            output.push_str("; DATA SECTION\n");
            output.push_str("; ─────────────────────────────────────────────────────────────────\n");
            output.push_str("section .data\n");
            for line in &self.data_section {
                output.push_str(line);
                output.push('\n');
            }
            output.push('\n');
        }
        
        // Text section (optimizado y limpio)
        output.push_str("; ─────────────────────────────────────────────────────────────────\n");
        output.push_str("; CODE SECTION\n");
        output.push_str("; ─────────────────────────────────────────────────────────────────\n");
        for line in &optimized_lines {
            output.push_str(line);
            output.push('\n');
        }
        Ok(output)
    }

    fn generate_stmt_windows(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Print(expr) => {
                self.add_debug_comment("print statement");
                self.text_section.push("    ; print".to_string());
                match expr {
                    Expr::Bool(b) => {
                        // Bool literal: convertir a string en compile-time
                        let bool_str = if *b { "true" } else { "false" };
                        let label = self.add_string_data(bool_str);
                        self.text_section.push("    ; Prepare WriteFile call for bool".to_string());
                        self.text_section.push("    mov rcx, [rbp+16]  ; stdout handle".to_string());
                        self.text_section.push(format!("    lea rdx, [rel {}]  ; buffer pointer", label));
                        self.text_section.push(format!("    mov r8, {}_len  ; number of bytes to write", label));
                        self.text_section.push("    lea r9, [rbp+24]  ; lpNumberOfBytesWritten (local var)".to_string());
                        self.text_section.push("    mov qword [rsp+32], 0  ; lpOverlapped = NULL (5th param in shadow space)".to_string());
                        self.text_section.push("    call WriteFile".to_string());
                    }
                    Expr::FString { parts } => {
                        // F-string: imprimir cada parte secuencialmente
                        use adead_parser::FStringPart;
                        
                        self.text_section.push(format!("    ; Print f-string con {} partes", parts.len()));
                        
                        for part in parts {
                            match part {
                                FStringPart::Literal(s) => {
                                    // Imprimir literal directamente
                                    let label = self.add_string_data(s);
                                    self.text_section.push(format!("    ; Print literal: \"{}\"", s.escape_default()));
                                    self.text_section.push("    mov rcx, [rbp+16]  ; stdout".to_string());
                                    self.text_section.push(format!("    lea rdx, [rel {}]", label));
                                    self.text_section.push(format!("    mov r8, {}_len", label));
                                    self.text_section.push("    lea r9, [rbp+24]".to_string());
                                    self.text_section.push("    mov qword [rsp+32], 0".to_string());
                                    self.text_section.push("    call WriteFile".to_string());
                                }
                                FStringPart::Expr(inner_expr) => {
                                    // Evaluar expresión y convertir a string, luego imprimir
                                    self.text_section.push("    ; Print expresión interpolada".to_string());
                                    self.generate_expr_windows(inner_expr)?;
                                    // RAX tiene el valor numérico
                                    
                                    // Buffer temporal en stack para conversión
                                    let buf_offset = self.stack_offset;
                                    self.stack_offset += 24;
                                    self.text_section.push("    sub rsp, 24".to_string());
                                    self.text_section.push(format!("    lea rdi, [rbp - {}]", buf_offset + 8));
                                    self.text_section.push("    mov rbx, rdi  ; inicio buffer".to_string());
                                    self.text_section.push("    mov rcx, 10".to_string());
                                    
                                    // Caso especial: 0
                                    let not_zero = self.new_label("fstr_nz");
                                    let convert_done = self.new_label("fstr_cvt_done");
                                    self.text_section.push("    cmp rax, 0".to_string());
                                    self.text_section.push(format!("    jne {}", not_zero));
                                    self.text_section.push("    mov byte [rdi], '0'".to_string());
                                    self.text_section.push("    inc rdi".to_string());
                                    self.text_section.push(format!("    jmp {}", convert_done));
                                    
                                    // Convertir número a dígitos (al revés)
                                    self.text_section.push(format!("{}:", not_zero));
                                    let digit_loop = self.new_label("fstr_digit");
                                    self.text_section.push(format!("{}:", digit_loop));
                                    self.text_section.push("    xor rdx, rdx".to_string());
                                    self.text_section.push("    div rcx".to_string());
                                    self.text_section.push("    add dl, '0'".to_string());
                                    self.text_section.push("    mov [rdi], dl".to_string());
                                    self.text_section.push("    inc rdi".to_string());
                                    self.text_section.push("    cmp rax, 0".to_string());
                                    self.text_section.push(format!("    jne {}", digit_loop));
                                    
                                    // Revertir dígitos in-place
                                    self.text_section.push("    mov rsi, rbx  ; inicio".to_string());
                                    self.text_section.push("    mov r8, rdi   ; fin+1".to_string());
                                    self.text_section.push("    dec r8        ; fin".to_string());
                                    let rev_loop = self.new_label("fstr_rev");
                                    let rev_end = self.new_label("fstr_rev_end");
                                    self.text_section.push(format!("{}:", rev_loop));
                                    self.text_section.push("    cmp rsi, r8".to_string());
                                    self.text_section.push(format!("    jge {}", rev_end));
                                    self.text_section.push("    mov al, [rsi]".to_string());
                                    self.text_section.push("    mov cl, [r8]".to_string());
                                    self.text_section.push("    mov [rsi], cl".to_string());
                                    self.text_section.push("    mov [r8], al".to_string());
                                    self.text_section.push("    inc rsi".to_string());
                                    self.text_section.push("    dec r8".to_string());
                                    self.text_section.push(format!("    jmp {}", rev_loop));
                                    self.text_section.push(format!("{}:", rev_end));
                                    
                                    self.text_section.push(format!("{}:", convert_done));
                                    // Calcular longitud: rdi - rbx
                                    self.text_section.push("    sub rdi, rbx  ; longitud".to_string());
                                    self.text_section.push("    mov r8, rdi   ; longitud para WriteFile".to_string());
                                    
                                    // WriteFile
                                    self.text_section.push("    mov rcx, [rbp+16]  ; stdout".to_string());
                                    self.text_section.push("    mov rdx, rbx       ; buffer".to_string());
                                    self.text_section.push("    lea r9, [rbp+24]".to_string());
                                    self.text_section.push("    mov qword [rsp+32], 0".to_string());
                                    self.text_section.push("    call WriteFile".to_string());
                                }
                            }
                        }
                    }
                    Expr::Float(f) => {
                        // Float literal: convertir a string en compile-time
                        // PASO 12: Usar función helper para formateo inteligente
                        let use_precise_version = false; // Cambiar a true para versión completa
                        let float_str = self.format_float_smart(*f, use_precise_version);
                        let float_str = if float_str.is_empty() { "0".to_string() } else { float_str };
                        let label = self.add_string_data(&float_str);
                        self.text_section.push("    ; Prepare WriteFile call for float".to_string());
                        self.text_section.push("    mov rcx, [rbp+16]  ; stdout handle".to_string());
                            self.text_section.push(format!("    lea rdx, [rel {}]  ; buffer pointer", label));
                            self.text_section.push(format!("    mov r8, {}_len  ; number of bytes to write", label));
                        self.text_section.push("    lea r9, [rbp+24]  ; lpNumberOfBytesWritten (local var)".to_string());
                        self.text_section.push("    mov qword [rsp+32], 0  ; lpOverlapped = NULL (5th param in shadow space)".to_string());
                        self.text_section.push("    call WriteFile".to_string());
                    }
                    Expr::String(s) => {
                        // Print string literal: crear estructura String dinámica y luego imprimir
                        // Generar código para crear String desde literal
                        self.generate_expr_windows(expr)?;
                        // RAX ahora contiene puntero al String struct
                        
                        // Cargar String->data y String->length para WriteFile
                        self.text_section.push("    push rax  ; guardar puntero al String".to_string());
                        self.text_section.push("    mov rdx, [rax + 0]  ; String->data".to_string());
                        self.text_section.push("    mov r8, [rax + 8]  ; String->length".to_string());
                        
                        // Preparar WriteFile call
                        self.text_section.push("    ; Prepare WriteFile call for String struct".to_string());
                        self.text_section.push("    mov rcx, [rbp+16]  ; stdout handle".to_string());
                        // RDX ya está listo (String->data)
                        // R8 ya está listo (String->length)
                        self.text_section.push("    lea r9, [rbp+24]  ; lpNumberOfBytesWritten (local var)".to_string());
                        self.text_section.push("    mov qword [rsp+32], 0  ; lpOverlapped = NULL (5th param in shadow space)".to_string());
                        self.text_section.push("    call WriteFile".to_string());
                        self.text_section.push("    pop rax  ; restaurar puntero al String (no usado después)".to_string());
                    }
                    Expr::Number(n) => {
                        // Print número: simplificado - crear string en tiempo de compilación (Sprint 2.1)
                        // Estrategia simple: convertir número a string y usar como string literal
                        let num_str = format!("{}", n);
                        let label = self.add_string_data(&num_str);
                        
                        // Preparar WriteFile call (igual que Expr::String)
                        self.text_section.push("    ; Prepare WriteFile call for number".to_string());
                        self.text_section.push("    mov rcx, [rbp+16]  ; stdout handle".to_string());
                            self.text_section.push(format!("    lea rdx, [rel {}]  ; buffer pointer", label));
                            self.text_section.push(format!("    mov r8, {}_len  ; number of bytes to write", label));
                        self.text_section.push("    lea r9, [rbp+24]  ; lpNumberOfBytesWritten (local var)".to_string());
                        self.text_section.push("    mov qword [rsp+32], 0  ; lpOverlapped = NULL (5th param in shadow space)".to_string());
                        self.text_section.push("    call WriteFile".to_string());
                    }
                    Expr::Ident(name) => {
                        // Verificar tipo de variable: string, bool, o numérica
                        if self.is_string_expr(expr) {
                            // Variable que contiene String struct
                            if let Some(&offset) = self.variables.get(name) {
                                self.text_section.push(format!("    mov rax, [rbp - {}]  ; cargar puntero al String struct desde variable {}", offset + 8, name));
                                self.text_section.push("    mov rdx, [rax + 0]  ; String->data".to_string());
                                self.text_section.push("    mov r8, [rax + 8]  ; String->length".to_string());
                            } else {
                                return Err(adead_common::ADeadError::RuntimeError {
                                    message: format!("undefined variable: {} in print statement", name),
                                });
                            }
                            self.text_section.push("    ; Prepare WriteFile call for String variable".to_string());
                            self.text_section.push("    mov rcx, [rbp+16]  ; stdout handle".to_string());
                            self.text_section.push("    lea r9, [rbp+24]  ; lpNumberOfBytesWritten".to_string());
                            self.text_section.push("    mov qword [r9], 0  ; inicializar".to_string());
                            self.text_section.push("    mov qword [rsp+32], 0  ; lpOverlapped = NULL".to_string());
                            self.text_section.push("    call WriteFile".to_string());
                        } else if self.is_bool_expr(expr) {
                            // Variable booleana: imprimir "true" o "false"
                            if let Some(&offset) = self.variables.get(name) {
                                let true_label = self.add_string_data("true");
                                let false_label = self.add_string_data("false");
                                let print_bool = self.new_label("print_bool");
                                let print_bool_end = self.new_label("print_bool_end");
                                
                                self.text_section.push(format!("    ; Print bool variable {}", name));
                                self.text_section.push(format!("    mov rax, [rbp - {}]  ; cargar valor bool", offset + 8));
                                self.text_section.push("    cmp rax, 0".to_string());
                                self.text_section.push(format!("    je {}", print_bool));
                                // True
                                self.text_section.push("    mov rcx, [rbp+16]  ; stdout".to_string());
                                self.text_section.push(format!("    lea rdx, [rel {}]", true_label));
                                self.text_section.push(format!("    mov r8, {}_len", true_label));
                                self.text_section.push("    lea r9, [rbp+24]".to_string());
                                self.text_section.push("    mov qword [rsp+32], 0".to_string());
                                self.text_section.push("    call WriteFile".to_string());
                                self.text_section.push(format!("    jmp {}", print_bool_end));
                                // False
                                self.text_section.push(format!("{}:", print_bool));
                                self.text_section.push("    mov rcx, [rbp+16]  ; stdout".to_string());
                                self.text_section.push(format!("    lea rdx, [rel {}]", false_label));
                                self.text_section.push(format!("    mov r8, {}_len", false_label));
                                self.text_section.push("    lea r9, [rbp+24]".to_string());
                                self.text_section.push("    mov qword [rsp+32], 0".to_string());
                                self.text_section.push("    call WriteFile".to_string());
                                self.text_section.push(format!("{}:", print_bool_end));
                            } else {
                                return Err(adead_common::ADeadError::RuntimeError {
                                    message: format!("undefined variable: {} in print statement", name),
                                });
                            }
                        } else {
                            // Variable numérica: evaluar expresión y convertir a string
                            // Usar la misma lógica que para expresiones numéricas complejas
                            self.generate_expr_windows(expr)?;
                            // RAX ahora contiene el resultado numérico
                            
                            // Reservar espacio para buffer en stack (20 bytes es suficiente para int64)
                            let buffer_offset = self.stack_offset;
                            self.stack_offset += 24; // Buffer + align
                            
                            // Guardar resultado en rbx temporalmente
                            self.text_section.push("    mov rbx, rax  ; guardar resultado".to_string());
                            
                            // Convertir número en RBX a string en buffer [rbp - buffer_offset]
                            // Función helper inline para convertir int64 a string
                            let conv_label = self.new_label("int_to_str_runtime");
                            self.text_section.push(format!("    lea rdx, [rbp - {}]  ; dirección del buffer", buffer_offset + 8));
                            self.text_section.push("    mov rax, rbx  ; número a convertir".to_string());
                            self.text_section.push("    push rbx  ; guardar resultado".to_string());
                            self.text_section.push("    push rdx  ; guardar dirección buffer".to_string());
                            self.text_section.push(format!("    call {}", conv_label));
                            
                            // RAX ahora contiene la longitud del string
                            // RDX tiene la dirección del buffer (preservada por la función helper)
                            self.text_section.push("    mov r8, rax  ; guardar longitud en r8 (tercer parámetro de WriteFile)".to_string());
                            
                            // Preparar WriteFile call
                            self.text_section.push("    ; Prepare WriteFile call for numeric variable".to_string());
                            self.text_section.push("    mov rcx, [rbp+16]  ; stdout handle (primer parámetro)".to_string());
                            // RDX ya tiene la dirección del buffer (segundo parámetro)
                            // R8 ya tiene la longitud (tercer parámetro)
                            self.text_section.push("    lea r9, [rbp+24]  ; lpNumberOfBytesWritten (cuarto parámetro)".to_string());
                            self.text_section.push("    mov qword [r9], 0  ; inicializar lpNumberOfBytesWritten".to_string());
                            self.text_section.push("    mov qword [rsp+32], 0  ; lpOverlapped = NULL (quinto parámetro en shadow space)".to_string());
                            self.text_section.push("    call WriteFile".to_string());
                            
                            // Restaurar stack
                            self.text_section.push("    pop rdx  ; restaurar dirección buffer".to_string());
                            self.text_section.push("    pop rbx  ; restaurar resultado".to_string());
                            self.stack_offset -= 24; // Liberar buffer
                            
                            // Generar función helper para convertir int64 a string (runtime)
                            self.text_section.push(format!("    jmp {}_end", conv_label));
                            self.text_section.push(format!("{}:", conv_label));
                            self.text_section.push("    ; Función helper: convertir int64 a string decimal (runtime)".to_string());
                            self.text_section.push("    ; Entrada: RAX = número, RDX = dirección del buffer".to_string());
                            self.text_section.push("    ; Salida: RAX = longitud del string".to_string());
                            self.text_section.push("    push rbp".to_string());
                            self.text_section.push("    mov rbp, rsp".to_string());
                            self.text_section.push("    push rbx".to_string());
                            self.text_section.push("    push rcx".to_string());
                            // CRÍTICO: Guardar dirección buffer original en r8 (registro no volátil) ANTES de cualquier modificación
                            self.text_section.push("    mov r8, rdx  ; guardar dirección buffer en r8 (preservar para retorno)".to_string());
                            // Guardar también en stack para restaurar después
                            self.text_section.push("    push rdx  ; guardar dirección buffer original en stack también".to_string());
                            // Usar rsi como registro de trabajo durante procesamiento
                            self.text_section.push("    mov rsi, rdx  ; copiar a rsi para usar durante procesamiento".to_string());
                            
                            // Manejar negativo
                            let pos_label = self.new_label("num_pos_rt");
                            let end_label = self.new_label("num_end_rt");
                            self.text_section.push("    cmp rax, 0".to_string());
                            self.text_section.push(format!("    jge {}", pos_label));
                            self.text_section.push("    mov byte [rdx], '-'".to_string());
                            self.text_section.push("    inc rdx".to_string());
                            self.text_section.push("    neg rax".to_string());
                            self.text_section.push(format!("{}:", pos_label));
                            
                            // CRÍTICO: Usar rsi (que tiene la dirección original del buffer) en lugar de rdx
                            // porque rdx puede haber sido modificado si el número era negativo
                            self.text_section.push("    mov rbx, rsi  ; inicio buffer (usar rsi que tiene dirección original)".to_string());
                            self.text_section.push("    mov rcx, 10  ; divisor".to_string());
                            
                            // Caso especial: 0
                            let not_zero_label = self.new_label("not_zero_rt");
                            self.text_section.push("    cmp rax, 0".to_string());
                            self.text_section.push(format!("    jne {}", not_zero_label));
                            self.text_section.push("    mov byte [rsi], '0'".to_string());
                            self.text_section.push("    inc rsi".to_string());
                            self.text_section.push(format!("    jmp {}", end_label));
                            self.text_section.push(format!("{}:", not_zero_label));
                            
                            // Loop: dividir y obtener dígitos
                            let loop_label = self.new_label("digit_loop_rt");
                            self.text_section.push(format!("{}:", loop_label));
                            self.text_section.push("    xor rdx, rdx  ; limpiar para división".to_string());
                            self.text_section.push("    div rcx  ; rax = rax/10, rdx = rax%10".to_string());
                            self.text_section.push("    add dl, '0'  ; convertir a ASCII".to_string());
                            self.text_section.push("    mov [rbx], dl  ; guardar dígito".to_string());
                            self.text_section.push("    inc rbx".to_string());
                            self.text_section.push("    cmp rax, 0".to_string());
                            self.text_section.push(format!("    jne {}", loop_label));
                            
                            self.text_section.push(format!("{}:", end_label));
                            self.text_section.push("    mov byte [rbx], 0xA  ; newline".to_string());
                            self.text_section.push("    inc rbx".to_string());
                            
                            // Revertir string (dígitos están al revés)
                            let rev_start = self.new_label("rev_start_rt");
                            let rev_loop = self.new_label("rev_loop_rt");
                            let rev_done = self.new_label("rev_done_rt");
                            self.text_section.push(format!("{}:", rev_start));
                            // Usar r8 que tiene la dirección original del buffer (guardada al inicio, nunca modificado)
                            self.text_section.push("    mov rcx, r8  ; inicio para el loop de reverso (r8 tiene dirección original)".to_string());
                            self.text_section.push("    mov rdx, rbx  ; fin para el loop de reverso".to_string());
                            self.text_section.push("    dec rdx  ; excluir newline del reverso".to_string());
                            self.text_section.push("    cmp rcx, rdx".to_string());
                            self.text_section.push(format!("    jge {}", rev_done));
                            // CRÍTICO: Guardar rbx (fin del string) antes del loop de reversión
                            // porque el loop modificará rbx
                            self.text_section.push("    push rbx  ; guardar fin del string antes de reversión".to_string());
                            
                            self.text_section.push(format!("{}:", rev_loop));
                            self.text_section.push("    mov al, [rcx]  ; byte desde inicio".to_string());
                            self.text_section.push("    mov bl, [rdx]  ; byte desde fin (rbx temporal, se restaurará después)".to_string());
                            self.text_section.push("    mov [rcx], bl".to_string());
                            self.text_section.push("    mov [rdx], al".to_string());
                            self.text_section.push("    inc rcx".to_string());
                            self.text_section.push("    dec rdx".to_string());
                            self.text_section.push("    cmp rcx, rdx".to_string());
                            self.text_section.push(format!("    jl {}", rev_loop));
                            self.text_section.push(format!("{}:", rev_done));
                            
                            // Restaurar rbx (fin del string) después del loop
                            self.text_section.push("    pop rbx  ; restaurar fin del string después de reversión".to_string());
                            
                            // Calcular longitud: rbx (fin) - dirección inicial del buffer
                            self.text_section.push("    mov rax, rbx  ; fin del string (incluye newline)".to_string());
                            self.text_section.push("    sub rax, r8  ; longitud = fin - inicio (r8 tiene la dirección original)".to_string());
                            
                            // Restaurar registros (en orden inverso del push)
                            // Stack tiene: [rbp] [rbx] [rcx] [rdx_buffer] <- top
                            // Hacer pop rdx para balancear el stack primero
                            self.text_section.push("    pop rdx  ; balancear stack (descartar valor del stack)".to_string());
                            self.text_section.push("    pop rcx  ; restaurar rcx original".to_string());
                            self.text_section.push("    pop rbx".to_string());
                            self.text_section.push("    pop rbp".to_string());
                            
                            // CRÍTICO: Restaurar la dirección del buffer en RDX DESPUÉS de todos los pop
                            // El caller necesita RDX con la dirección del buffer para WriteFile
                            self.text_section.push("    mov rdx, r8  ; restaurar dirección buffer en rdx para el caller (r8 nunca se modificó)".to_string());
                            
                            // RAX tiene la longitud, RDX tiene la dirección del buffer - ambos para el caller
                            self.text_section.push("    ret".to_string());
                            self.text_section.push(format!("{}_end:", conv_label));
                        }
                    }
                    _ => {
                        // Evaluar expresión numérica (ej: 2 + 5, 3.14 + 2.5) y convertir a string
                        // Intentar evaluación compile-time primero para expresiones float simples
                        if self.is_float_expr(expr) {
                            if let Some(float_result) = self.eval_const_expr(expr) {
                                // Expresión float constante: evaluar en compile-time y imprimir directamente
                                // PASO 12: Usar función helper para formateo inteligente
                                // Versión optimizada (false): formato limpio para legibilidad
                                // Versión precisa (true): mostrar toda la precisión cuando sea necesario
                                let use_precise_version = false; // Cambiar a true para versión completa
                                let float_str = self.format_float_smart(float_result, use_precise_version);
                                let float_str = if float_str.is_empty() { "0".to_string() } else { float_str };
                                let label = self.add_string_data(&float_str);
                                self.text_section.push("    ; Prepare WriteFile call for float expression (compile-time evaluated)".to_string());
                                self.text_section.push("    mov rcx, [rbp+16]  ; stdout handle".to_string());
                                self.text_section.push(format!("    lea rdx, [rel {}]  ; buffer pointer", label));
                                self.text_section.push(format!("    mov r8, {}_len  ; number of bytes to write", label));
                                self.text_section.push("    lea r9, [rbp+24]  ; lpNumberOfBytesWritten (local var)".to_string());
                                self.text_section.push("    mov qword [rsp+32], 0  ; lpOverlapped = NULL (5th param in shadow space)".to_string());
                                self.text_section.push("    call WriteFile".to_string());
                                return Ok(());
                            } else {
                                // Expresión float compleja que no puede evaluarse en compile-time
                                // TODO: Implementar función helper float_to_str_runtime
                                return Err(adead_common::ADeadError::RuntimeError {
                                    message: format!("Printing complex float expressions with variables not yet implemented. Use simple constant expressions or assign to variable first: let x = expr; print x"),
                                });
                            }
                        } else {
                            // Expresión entera: RAX contendrá el resultado después de generate_expr_windows
                            self.generate_expr_windows(expr)?;
                            // RAX ahora contiene el resultado numérico
                            
                            // Reservar espacio para buffer en stack (20 bytes es suficiente para int64)
                            let buffer_offset = self.stack_offset;
                            self.stack_offset += 24; // Buffer + align
                            
                            // Guardar resultado en rbx temporalmente
                            self.text_section.push("    mov rbx, rax  ; guardar resultado".to_string());
                            
                            // Convertir número en RBX a string en buffer [rbp - buffer_offset]
                            // Función helper inline para convertir int64 a string
                            let conv_label = self.new_label("int_to_str_runtime");
                                self.text_section.push(format!("    lea rdx, [rbp - {}]  ; dirección del buffer", buffer_offset + 8));
                            self.text_section.push("    mov rax, rbx  ; número a convertir".to_string());
                            self.text_section.push("    push rbx  ; guardar resultado".to_string());
                            self.text_section.push("    push rdx  ; guardar dirección buffer".to_string());
                                self.text_section.push(format!("    call {}", conv_label));
                            // RAX ahora contiene la longitud del string
                            // RDX tiene la dirección del buffer (preservada por la función helper usando r8)
                            // CRÍTICO: La función helper retorna RDX con la dirección del buffer
                            // No necesitamos mover desde r8 porque RDX ya está correcto después del ret
                            
                            // IMPORTANTE: Guardar RAX (longitud) y RDX (buffer) inmediatamente después del call
                            // porque ambos se necesitan para WriteFile
                            self.text_section.push("    mov r8, rax  ; guardar longitud en r8 (tercer parámetro de WriteFile)".to_string());
                            // RDX ya tiene la dirección del buffer (segundo parámetro) - preservado por la función helper
                            // No tocamos RDX aquí porque la función helper ya lo restauró correctamente
                            
                            // Preparar WriteFile call (Windows x64 calling convention)
                            // BOOL WriteFile(
                            //   HANDLE hFile,                    // RCX
                            //   LPCVOID lpBuffer,                // RDX (ya está correcto)
                            //   DWORD nNumberOfBytesToWrite,     // R8 (longitud, ya guardada arriba)
                            //   LPDWORD lpNumberOfBytesWritten,  // R9
                            //   LPOVERLAPPED lpOverlapped        // [rsp+32] (shadow space)
                            // )
                            self.text_section.push("    ; Prepare WriteFile call for numeric expression".to_string());
                            self.text_section.push("    mov rcx, [rbp+16]  ; stdout handle (primer parámetro)".to_string());
                            // RDX ya tiene la dirección del buffer (segundo parámetro) - no modificar
                            // R8 ya tiene la longitud (tercer parámetro) - no modificar
                            self.text_section.push("    lea r9, [rbp+24]  ; lpNumberOfBytesWritten (cuarto parámetro)".to_string());
                            self.text_section.push("    mov qword [r9], 0  ; inicializar lpNumberOfBytesWritten".to_string());
                            self.text_section.push("    mov qword [rsp+32], 0  ; lpOverlapped = NULL (quinto parámetro en shadow space)".to_string());
                            self.text_section.push("    call WriteFile".to_string());
                            
                            // Restaurar stack después de WriteFile
                            self.text_section.push("    pop rdx  ; restaurar dirección buffer".to_string());
                            self.text_section.push("    pop rbx  ; restaurar resultado".to_string());
                            self.stack_offset -= 24; // Liberar buffer
                            
                            // Generar función helper para convertir int64 a string (runtime)
                            self.text_section.push(format!("    jmp {}_end", conv_label));
                            self.text_section.push(format!("{}:", conv_label));
                            self.text_section.push("    ; Función helper: convertir int64 a string decimal (runtime)".to_string());
                            self.text_section.push("    ; Entrada: RAX = número, RDX = dirección del buffer".to_string());
                            self.text_section.push("    ; Salida: RAX = longitud del string".to_string());
                            self.text_section.push("    push rbp".to_string());
                            self.text_section.push("    mov rbp, rsp".to_string());
                            self.text_section.push("    push rbx".to_string());
                            self.text_section.push("    push rcx".to_string());
                            // CRÍTICO: Guardar dirección buffer original en r8 (registro no volátil) ANTES de cualquier modificación
                            self.text_section.push("    mov r8, rdx  ; guardar dirección buffer en r8 (preservar para retorno)".to_string());
                            // Guardar también en stack para restaurar después
                            self.text_section.push("    push rdx  ; guardar dirección buffer original en stack también".to_string());
                            // Usar rsi como registro de trabajo durante procesamiento
                            self.text_section.push("    mov rsi, rdx  ; copiar a rsi para usar durante procesamiento".to_string());
                            
                            // Manejar negativo
                            let pos_label = self.new_label("num_pos_rt");
                            let end_label = self.new_label("num_end_rt");
                            self.text_section.push("    cmp rax, 0".to_string());
                            self.text_section.push(format!("    jge {}", pos_label));
                            self.text_section.push("    mov byte [rdx], '-'".to_string());
                            self.text_section.push("    inc rdx".to_string());
                            self.text_section.push("    neg rax".to_string());
                            self.text_section.push(format!("{}:", pos_label));
                            
                            // CRÍTICO: Usar rsi (que tiene la dirección original del buffer) en lugar de rdx
                            // porque rdx puede haber sido modificado si el número era negativo
                            self.text_section.push("    mov rbx, rsi  ; inicio buffer (usar rsi que tiene dirección original)".to_string());
                            self.text_section.push("    mov rcx, 10  ; divisor".to_string());
                            
                            // Caso especial: 0
                            let not_zero_label = self.new_label("not_zero_rt");
                            self.text_section.push("    cmp rax, 0".to_string());
                            self.text_section.push(format!("    jne {}", not_zero_label));
                            self.text_section.push("    mov byte [rsi], '0'".to_string());
                            self.text_section.push("    inc rsi".to_string());
                            self.text_section.push(format!("    jmp {}", end_label));
                            self.text_section.push(format!("{}:", not_zero_label));
                            
                            // Loop: dividir y obtener dígitos
                            let loop_label = self.new_label("digit_loop_rt");
                            self.text_section.push(format!("{}:", loop_label));
                            self.text_section.push("    xor rdx, rdx  ; limpiar para división".to_string());
                            self.text_section.push("    div rcx  ; rax = rax/10, rdx = rax%10".to_string());
                            self.text_section.push("    add dl, '0'  ; convertir a ASCII".to_string());
                            self.text_section.push("    mov [rbx], dl  ; guardar dígito".to_string());
                            self.text_section.push("    inc rbx".to_string());
                            self.text_section.push("    cmp rax, 0".to_string());
                            self.text_section.push(format!("    jne {}", loop_label));
                            
                            self.text_section.push(format!("{}:", end_label));
                            self.text_section.push("    mov byte [rbx], 0xA  ; newline".to_string());
                            self.text_section.push("    inc rbx".to_string());
                            
                            // Revertir string (dígitos están al revés)
                            let rev_start = self.new_label("rev_start_rt");
                            let rev_loop = self.new_label("rev_loop_rt");
                            let rev_done = self.new_label("rev_done_rt");
                            self.text_section.push(format!("{}:", rev_start));
                            // Usar r8 que tiene la dirección original del buffer (guardada al inicio, nunca modificado)
                            self.text_section.push("    mov rcx, r8  ; inicio para el loop de reverso (r8 tiene dirección original)".to_string());
                            self.text_section.push("    mov rdx, rbx  ; fin para el loop de reverso".to_string());
                            self.text_section.push("    dec rdx  ; excluir newline del reverso".to_string());
                            self.text_section.push("    cmp rcx, rdx".to_string());
                            self.text_section.push(format!("    jge {}", rev_done));
                            // CRÍTICO: Guardar rbx (fin del string) antes del loop de reversión
                            // porque el loop modificará rbx
                            self.text_section.push("    push rbx  ; guardar fin del string antes de reversión".to_string());
                            
                            self.text_section.push(format!("{}:", rev_loop));
                            self.text_section.push("    mov al, [rcx]  ; byte desde inicio".to_string());
                            self.text_section.push("    mov bl, [rdx]  ; byte desde fin (rbx temporal, se restaurará después)".to_string());
                            self.text_section.push("    mov [rcx], bl".to_string());
                            self.text_section.push("    mov [rdx], al".to_string());
                            self.text_section.push("    inc rcx".to_string());
                            self.text_section.push("    dec rdx".to_string());
                            self.text_section.push("    cmp rcx, rdx".to_string());
                            self.text_section.push(format!("    jl {}", rev_loop));
                            self.text_section.push(format!("{}:", rev_done));
                            
                            // Restaurar rbx (fin del string) después del loop
                            self.text_section.push("    pop rbx  ; restaurar fin del string después de reversión".to_string());
                            
                            // Calcular longitud: rbx (fin) - dirección inicial del buffer
                            self.text_section.push("    mov rax, rbx  ; fin del string (incluye newline)".to_string());
                            self.text_section.push("    sub rax, r8  ; longitud = fin - inicio (r8 tiene la dirección original)".to_string());
                            
                            // Restaurar registros (en orden inverso del push)
                            // Stack tiene: [rbp] [rbx] [rcx] [rdx_buffer] <- top
                            // Hacer pop rdx para balancear el stack primero
                            self.text_section.push("    pop rdx  ; balancear stack (descartar valor del stack)".to_string());
                            self.text_section.push("    pop rcx  ; restaurar rcx original".to_string());
                            self.text_section.push("    pop rbx".to_string());
                            self.text_section.push("    pop rbp".to_string());
                            
                            // CRÍTICO: Restaurar la dirección del buffer en RDX DESPUÉS de todos los pop
                            // El caller necesita RDX con la dirección del buffer para WriteFile
                            self.text_section.push("    mov rdx, r8  ; restaurar dirección buffer en rdx para el caller (r8 nunca se modificó)".to_string());
                            
                            // RAX tiene la longitud, RDX tiene la dirección del buffer - ambos para el caller
                            self.text_section.push("    ret".to_string());
                            self.text_section.push(format!("{}_end:", conv_label));
                        }
                    }
                }
            },
            Stmt::Let { mutable, name, value } => {
                self.add_debug_comment(&format!("let {} = ...", if *mutable { format!("mut {}", name) } else { name.clone() }));
                // Detectar tipo de valor y registrar
                let struct_name = if let Expr::Call { module: Some(class_name), name: method_name, .. } = value {
                    if method_name == "new" && self.struct_definitions.contains_key(class_name) {
                        // Es un constructor, registrar tipo
                        if self.structs_with_destroy.contains_key(class_name) {
                            self.variables_to_destroy.push((name.clone(), class_name.clone()));
                        }
                        self.variable_types.insert(name.clone(), class_name.clone());
                        Some(class_name.clone())
                    } else {
                        None
                    }
                } else if let Expr::StructLiteral { name: struct_name, .. } = value {
                    // Struct literal
                    if self.structs_with_destroy.contains_key(struct_name) {
                        self.variables_to_destroy.push((name.clone(), struct_name.clone()));
                    }
                    self.variable_types.insert(name.clone(), struct_name.clone());
                    Some(struct_name.clone())
                } else if matches!(value, Expr::Bool(_)) {
                    // Booleano: registrar tipo
                    self.variable_types.insert(name.clone(), "bool".to_string());
                    None
                } else {
                    None
                };
                
                self.generate_expr_windows(value)?;
                
                // Si es un struct con constructor, llamarlo aquí
                // Por ahora, solo guardamos la dirección del struct
                
                // Store in stack (simplified: just track as variable)
                // Note: mutability is tracked but NASM code is the same (difference is in borrow checker)
                let offset = if let Some(&existing_offset) = self.variables.get(name) {
                    existing_offset
                } else {
                    let offset = self.stack_offset;
                    self.stack_offset += 8;
                    self.variables.insert(name.clone(), offset);
                    offset
                };
                self.text_section
                    .push(format!("    mov [rbp - {}], rax  ; variable {} ({})", 
                        offset + 8, name, if *mutable { "mutable" } else { "immutable" }));
            }
            Stmt::MultiLet { mutable, names, values } => {
                // Múltiple asignación Python-style: let a, b = 1, 2
                self.add_debug_comment(&format!("multi-let {:?} = ...", names));
                
                // Evaluar cada valor y asignarlo a su variable correspondiente
                for (i, name) in names.iter().enumerate() {
                    // Si hay menos valores que nombres, usar el último valor
                    let value = if i < values.len() { &values[i] } else { &values[values.len() - 1] };
                    
                    self.generate_expr_windows(value)?;
                    
                    let offset = if let Some(&existing_offset) = self.variables.get(name) {
                        existing_offset
                    } else {
                        let offset = self.stack_offset;
                        self.stack_offset += 8;
                        self.variables.insert(name.clone(), offset);
                        offset
                    };
                    self.text_section
                        .push(format!("    mov [rbp - {}], rax  ; variable {} ({})", 
                            offset + 8, name, if *mutable { "mutable" } else { "immutable" }));
                }
            }
            Stmt::Import(_module_name) => {
                // Import statements son procesados en tiempo de compilación
                // No generan código directamente, solo información de módulos
                // TODO: Implementar resolución de módulos
                self.text_section.push(format!("    ; import {} (pendiente resolución)", _module_name));
            }
            Stmt::Expr(expr) => {
                self.generate_expr_windows(expr)?;
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
            } => {
                self.add_debug_comment("if statement");
                self.generate_expr_windows(condition)?;
                let else_label = self.new_label("else");
                let end_label = self.new_label("endif");
                
                // Check condition: if 0, jump to else/end
                self.text_section.push("    cmp rax, 0".to_string());
                if else_body.is_some() {
                    self.text_section.push(format!("    je {}", else_label));
                } else {
                    self.text_section.push(format!("    je {}", end_label));
                }

                // Then body
                for s in then_body {
                    self.generate_stmt_windows(s)?;
                }
                self.text_section.push(format!("    jmp {}", end_label));
                
                // Else body
                if let Some(else_body) = else_body {
                    self.text_section.push(format!("{}:", else_label));
                    for s in else_body {
                        self.generate_stmt_windows(s)?;
                    }
                }
                self.text_section.push(format!("{}:", end_label));
            }
            Stmt::While { condition, body } => {
                self.add_debug_comment("while loop");
                let loop_start = self.new_label("while_start");
                let loop_end = self.new_label("while_end");
                let loop_continue = loop_start.clone(); // continue salta al inicio
                
                // Push loop context para break/continue
                self.loop_stack.push(LoopContext {
                    break_label: loop_end.clone(),
                    continue_label: loop_continue,
                });
                
                    self.text_section.push(format!("{}:", loop_start));
                self.generate_expr_windows(condition)?;
                self.text_section.push("    cmp rax, 0".to_string());
                self.text_section.push(format!("    je {}", loop_end));
                
                for s in body {
                    self.generate_stmt_windows(s)?;
                }
                self.text_section.push(format!("    jmp {}", loop_start));
                self.text_section.push(format!("{}:", loop_end));
                
                // Pop loop context
                self.loop_stack.pop();
            }
            Stmt::For { var, start, end, body } => {
                self.add_debug_comment(&format!("for {} in range", var));
                let loop_start = self.new_label("for_start");
                let loop_end = self.new_label("for_end");
                let loop_continue = self.new_label("for_continue");
                
                // Push loop context para break/continue
                self.loop_stack.push(LoopContext {
                    break_label: loop_end.clone(),
                    continue_label: loop_continue.clone(),
                });
                
                // Evaluar start y guardar en variable
                self.generate_expr_windows(start)?;
                let var_offset = self.stack_offset;
                self.stack_offset += 8;
                self.variables.insert(var.clone(), var_offset);
                self.text_section.push(format!("    mov [rbp - {}], rax  ; {} (loop counter)", var_offset + 8, var));
                
                // Evaluar end y guardar en registro temporal (r13 preservado)
                self.generate_expr_windows(end)?;
                self.text_section.push("    mov r13, rax  ; end value (preserved)".to_string());
                
                // Loop start
                self.text_section.push(format!("{}:", loop_start));
                
                // Comparar variable con end
                self.text_section.push(format!("    mov rax, [rbp - {}]  ; cargar {}", var_offset + 8, var));
                self.text_section.push("    cmp rax, r13  ; comparar con end".to_string());
                self.text_section.push(format!("    jge {}  ; si >= end, salir", loop_end));
                
                // Body del loop
                for s in body {
                    self.generate_stmt_windows(s)?;
                }
                
                // Continue label (para incrementar contador)
                self.text_section.push(format!("{}:", loop_continue));
                
                // Incrementar contador
                self.text_section.push(format!("    mov rax, [rbp - {}]  ; cargar {}", var_offset + 8, var));
                self.text_section.push("    inc rax  ; incrementar".to_string());
                self.text_section.push(format!("    mov [rbp - {}], rax  ; guardar", var_offset + 8));
                
                // Saltar al inicio
                self.text_section.push(format!("    jmp {}", loop_start));
                self.text_section.push(format!("{}:", loop_end));
                
                // Pop loop context
                self.loop_stack.pop();
            }
            Stmt::Break => {
                self.add_debug_comment("break");
                if let Some(ctx) = self.loop_stack.last() {
                    let break_label = ctx.break_label.clone();
                    self.text_section.push(format!("    jmp {}  ; break", break_label));
                } else {
                    // Error: break fuera de loop (debería detectarse en análisis semántico)
                    self.text_section.push("    ; ERROR: break fuera de loop".to_string());
                }
            }
            Stmt::Continue => {
                self.add_debug_comment("continue");
                if let Some(ctx) = self.loop_stack.last() {
                    let continue_label = ctx.continue_label.clone();
                    self.text_section.push(format!("    jmp {}  ; continue", continue_label));
                } else {
                    // Error: continue fuera de loop (debería detectarse en análisis semántico)
                    self.text_section.push("    ; ERROR: continue fuera de loop".to_string());
                }
            }
            Stmt::Fn { visibility: _, name, params, body } => {
                self.add_debug_comment(&format!("fn {} ({})", name, params.len()));
                // Generate function with Windows x64 calling convention (ABI-safe)
                // Visibility no afecta la generación de código (Sprint 1.3)
                let func_label = format!("fn_{}", name);
                // No necesitamos jmp si la función se genera antes del main
                // (las funciones de usuario se generan antes del main)
                self.text_section.push(format!("{}:", func_label));
                
                // Guardar stack_offset inicial para restaurar después
                let saved_stack_offset = self.stack_offset;
                
                // Prologue ABI-safe: preservar registros no volátiles y alinear stack
                // Necesitamos shadow space si la función llama a otras funciones
                // Por ahora, siempre reservamos shadow space para seguridad
                self.generate_abi_prologue(true);
                
                // Calcular espacio necesario para variables locales
                // Parámetros adicionales (> 4) ya están en stack del caller
                // Necesitamos espacio para:
                // - Parámetros locales (primeros 4 se guardan en stack local)
                // - Variables locales de la función
                let mut local_vars_size = 0;
                
                // Guardar parámetros en variables locales
                // Parámetros en registros: RCX, RDX, R8, R9 (primeros 4)
                // Parámetros adicionales: en stack del caller [rbp + 16 + (i-4)*8]
                // Nota: Después del prologue ABI-safe, rbp apunta al frame del caller
                // Los parámetros adicionales están en [rbp + 16 + shadow_space + (i-4)*8]
                // Pero como ya hicimos push de registros, necesitamos ajustar
                // Después de prologue: rbp original está en [rbp + 0], return address en [rbp + 8]
                // Parámetros adicionales están en [rbp + 16 + (i-4)*8] (sin contar los push del prologue)
                for (i, param) in params.iter().enumerate() {
                    let offset = self.stack_offset;
                    self.stack_offset += 8;
                    local_vars_size += 8;
                    self.variables.insert(param.name.clone(), offset);
                    
                    match i {
                        0 => {
                            // RCX -> guardar en stack local
                            self.text_section.push(format!("    mov [rbp - {}], rcx  ; guardar param0: {}", offset + 8, param.name));
                        }
                        1 => {
                            // RDX -> guardar en stack local
                            self.text_section.push(format!("    mov [rbp - {}], rdx  ; guardar param1: {}", offset + 8, param.name));
                        }
                        2 => {
                            // R8 -> guardar en stack local
                            self.text_section.push(format!("    mov [rbp - {}], r8  ; guardar param2: {}", offset + 8, param.name));
                        }
                        3 => {
                            // R9 -> guardar en stack local
                            self.text_section.push(format!("    mov [rbp - {}], r9  ; guardar param3: {}", offset + 8, param.name));
                        }
                        _ => {
                            // Parámetros adicionales (> 4) están en stack del caller
                            // Después del prologue ABI-safe:
                            // - rbp apunta al frame del caller
                            // - Los push del prologue están antes de rbp
                            // - Parámetros adicionales están en [rbp + 16 + (i-4)*8]
                            //   16 = return address (8) + saved rbp (8)
                            let stack_offset = 16 + (i - 4) * 8;
                            self.text_section.push(format!("    mov rax, [rbp + {}]  ; cargar param{} desde stack del caller", stack_offset, i));
                            self.text_section.push(format!("    mov [rbp - {}], rax  ; guardar param{}: {}", offset + 8, i, param.name));
                        }
                    }
                }
                
                // Reservar espacio adicional para variables locales si es necesario
                // (ya reservamos espacio para parámetros arriba)
                // El prologue ya reservó shadow space (32 bytes) y alineación (8 bytes)
                
                // Generar cuerpo de la función
                // Nota: Si hay return statements, saltarán al epilogue
                let return_label = format!("{}_return", func_label);
                let mut has_explicit_return = false;
                
                for s in body {
                    match s {
                        Stmt::Return(_) => {
                            // Return statement: evaluar expresión y saltar al epilogue
                            has_explicit_return = true;
                            self.generate_stmt_windows(s)?;
                            // Después de return, saltar al epilogue
                            self.text_section.push(format!("    jmp {}", return_label));
                        }
                        _ => {
                            self.generate_stmt_windows(s)?;
                        }
                    }
                }
                
                // Si no hay return explícito, retornar 0 por defecto
                self.text_section.push(format!("{}:", return_label));
                if !has_explicit_return {
                    self.text_section.push("    mov rax, 0  ; return value por defecto".to_string());
                }
                // Si hay return explícito, el valor ya está en RAX
                
                // Epilogue ABI-safe: restaurar registros y limpiar stack
                self.generate_abi_epilogue(true);
                
                // Restaurar stack_offset
                self.stack_offset = saved_stack_offset;
                
                self.text_section.push(format!("{}_end:", func_label));
            }
            Stmt::Return(expr) => {
                self.add_debug_comment("return statement");
                // Return statement: evaluar expresión y poner resultado en RAX
                // El epilogue se manejará en la función (no aquí)
                if let Some(expr) = expr {
                    self.generate_expr_windows(expr)?;
                    // El resultado ya está en RAX
                } else {
                    self.text_section.push("    mov rax, 0  ; return sin valor (default 0)".to_string());
                }
                // Nota: NO llamamos leave/ret aquí porque el epilogue de la función lo hará
                // El código de la función saltará al epilogue después de return
            }
            Stmt::Struct { name, parent, fields, init, destroy, methods } => {
                // Registrar parent para super.metodo()
                self.struct_parents.insert(name.clone(), parent.clone());
                
                // Registrar campos del struct para cálculo de offsets
                // Si hay herencia, incluir campos del padre
                let mut all_field_names: Vec<String> = Vec::new();
                
                // Si hay padre, agregar sus campos primero
                if let Some(parent_name) = &parent {
                    if let Some(parent_fields) = self.struct_definitions.get(parent_name) {
                        all_field_names.extend(parent_fields.clone());
                    } else {
                        return Err(adead_common::ADeadError::RuntimeError {
                            message: format!("Struct padre '{}' no está definido antes de '{}'. Los structs padre deben definirse antes que los hijos.", parent_name, name),
                        });
                    }
                }
                
                // Agregar campos propios del struct
                let own_field_names: Vec<String> = fields.iter().map(|f| f.name.clone()).collect();
                all_field_names.extend(own_field_names.clone());
                
                self.struct_definitions.insert(name.clone(), all_field_names);
                
                // Establecer struct actual para procesar métodos
                let old_struct = self.current_struct.take();
                self.current_struct = Some(name.clone());
                
                // Registrar si tiene destructor (para RAII)
                if destroy.is_some() {
                    self.structs_with_destroy.insert(name.clone(), true);
                }
                
                // Registrar métodos de instancia (no estáticos) para vtables
                let mut instance_methods: Vec<String> = Vec::new();
                if let Some(parent_name) = &parent {
                    // Si hay padre, heredar métodos del padre
                    if let Some(parent_methods) = self.struct_methods.get(parent_name) {
                        instance_methods.extend(parent_methods.clone());
                    }
                }
                
                // Generar métodos de instancia y estáticos
                // Nota: Los métodos pueden venir del struct directamente o desde funciones globales asociadas
                for (method_name, method) in methods {
                    let method_label = format!("fn_{}_{}", name, method_name);
                    
                    // Detectar si es método estático (no tiene 'self' como primer parámetro)
                    let is_static = method.params.is_empty() || method.params[0].name != "self";
                    
                    // Registrar métodos de instancia para vtable (no estáticos)
                    if !is_static {
                        // Si el método ya existe (heredado), reemplazarlo (override)
                        if let Some(pos) = instance_methods.iter().position(|m| m == method_name) {
                            instance_methods[pos] = method_name.clone();
                        } else {
                            instance_methods.push(method_name.clone());
                        }
                    }
                    
                    self.text_section.push(format!("    jmp {}_end", method_label));
                    self.text_section.push(format!("{}:", method_label));
                    
                    // Guardar stack_offset inicial para restaurar después
                    let saved_stack_offset = self.stack_offset;
                    
                    // Prologue ABI-safe
                    self.generate_abi_prologue(true);
                    
                    if !is_static {
                        // Método de instancia: self viene en RCX (primer parámetro implícito)
                    let self_offset = self.stack_offset;
                    self.stack_offset += 8;
                    self.variables.insert("self".to_string(), self_offset);
                    self.variable_types.insert("self".to_string(), name.clone());
                    self.text_section.push(format!("    mov [rbp - {}], rcx  ; guardar self", self_offset + 8));
                    }
                    
                    // Parámetros del método vienen en RCX (si estático), RDX, R8, R9... (si instancia)
                    let param_start_reg = if is_static { 0 } else { 1 }; // Si estático, RCX es primer param; si instancia, RCX es self
                    for (i, param) in method.params.iter().enumerate() {
                        if !is_static && i == 0 && param.name == "self" {
                            // self ya fue manejado arriba, saltar
                            continue;
                        }
                        let offset = self.stack_offset;
                        self.stack_offset += 8;
                        self.variables.insert(param.name.clone(), offset);
                        
                        // Calcular índice real del parámetro (ajustado si es método de instancia)
                        let param_index = if is_static { i } else { i - 1 }; // Si instancia, self no cuenta
                        
                        match param_index {
                            0 => {
                                // RCX (si estático) o RDX (si instancia) -> guardar en stack local
                                let reg = if is_static { "rcx" } else { "rdx" };
                                self.text_section.push(format!("    mov [rbp - {}], {}  ; guardar param{}: {}", offset + 8, reg, param_index, param.name));
                            }
                            1 => {
                                // RDX (si estático) o R8 (si instancia) -> guardar en stack local
                                let reg = if is_static { "rdx" } else { "r8" };
                                self.text_section.push(format!("    mov [rbp - {}], {}  ; guardar param{}: {}", offset + 8, reg, param_index, param.name));
                            }
                            2 => {
                                // R8 (si estático) o R9 (si instancia) -> guardar en stack local
                                let reg = if is_static { "r8" } else { "r9" };
                                self.text_section.push(format!("    mov [rbp - {}], {}  ; guardar param{}: {}", offset + 8, reg, param_index, param.name));
                            }
                            3 => {
                                // R9 (si estático) o stack (si instancia) -> guardar en stack local
                                if is_static {
                                    self.text_section.push(format!("    mov [rbp - {}], r9  ; guardar param{}: {}", offset + 8, param_index, param.name));
                                } else {
                                    let stack_offset = 16 + (param_index - 3) * 8;
                                    self.text_section.push(format!("    mov rax, [rbp + {}]  ; cargar param{} desde stack del caller", stack_offset, param_index));
                                    self.text_section.push(format!("    mov [rbp - {}], rax  ; guardar param{}: {}", offset + 8, param_index, param.name));
                                }
                            }
                            _ => {
                                // Parámetros adicionales están en stack del caller
                                let stack_offset = 16 + (param_index - 4) * 8;
                                self.text_section.push(format!("    mov rax, [rbp + {}]  ; cargar param{} desde stack del caller", stack_offset, param_index));
                                self.text_section.push(format!("    mov [rbp - {}], rax  ; guardar param{}: {}", offset + 8, param_index, param.name));
                            }
                        }
                    }
                    
                    // Generar cuerpo del método
                    let return_label = format!("{}_return", method_label);
                    let mut has_explicit_return = false;
                    
                    for s in &method.body {
                        match s {
                            Stmt::Return(_) => {
                                has_explicit_return = true;
                                self.generate_stmt_windows(s)?;
                                self.text_section.push(format!("    jmp {}", return_label));
                            }
                            _ => {
                                self.generate_stmt_windows(s)?;
                            }
                        }
                    }
                    
                    // Si no hay return explícito, retornar 0 por defecto
                    self.text_section.push(format!("{}:", return_label));
                    if !has_explicit_return {
                        self.text_section.push("    mov rax, 0  ; return value por defecto".to_string());
                    }
                    
                    // Epilogue ABI-safe
                    self.generate_abi_epilogue(true);
                    
                    // Restaurar stack_offset
                    self.stack_offset = saved_stack_offset;
                    
                    self.text_section.push(format!("{}_end:", method_label));
                    
                    // Limpiar variables locales del método
                    self.variables.remove("self");
                    for param in &method.params {
                        self.variables.remove(&param.name);
                    }
                }
                
                // Guardar métodos de instancia para este struct
                self.struct_methods.insert(name.clone(), instance_methods.clone());
                
                // Generar vtable en sección .data
                if !instance_methods.is_empty() {
                    let vtable_label = format!("vtable_{}", name);
                    self.data_section.push(format!("{}:", vtable_label));
                    for method_name in &instance_methods {
                        let method_ptr = format!("fn_{}_{}", name, method_name);
                        self.data_section.push(format!("    dq {}  ; método {}", method_ptr, method_name));
                    }
                }
                
                // Registrar struct y generar código para constructor si existe
                if let Some(init_method) = init {
                    // Generar función de constructor: fn_StructName_new
                    let init_label = format!("fn_{}_new", name);
                    self.text_section.push(format!("    jmp {}_end", init_label));
                    self.text_section.push(format!("{}:", init_label));
                    self.text_section.push("    push rbp".to_string());
                    self.text_section.push("    mov rbp, rsp".to_string());
                    self.text_section.push("    sub rsp, 64  ; espacio para variables locales".to_string());
                    
                    // self viene en RCX (primer parámetro implícito)
                    let self_offset = self.stack_offset;
                    self.stack_offset += 8;
                    self.variables.insert("self".to_string(), self_offset);
                    self.variable_types.insert("self".to_string(), name.clone());
                    self.text_section.push(format!("    mov [rbp - {}], rcx  ; guardar self", self_offset + 8));
                    
                    // Si hay herencia, llamar al constructor del padre primero
                    if let Some(parent_name) = parent {
                        self.text_section.push(format!("    ; Llamar constructor del padre: {}", parent_name));
                        // self ya está en RCX, pasarlo al constructor del padre
                        self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                        self.text_section.push(format!("    call fn_{}_new  ; constructor del padre", parent_name));
                        self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
                    }
                    
                    // Los parámetros del usuario vienen en RDX, R8, R9... (RCX tiene self)
                    for (i, param) in init_method.params.iter().enumerate() {
                        let offset = self.stack_offset;
                        self.stack_offset += 8;
                        self.variables.insert(param.name.clone(), offset);
                        
                        let reg = match i {
                            0 => "rdx",   // Primer param usuario
                            1 => "r8",    // Segundo param usuario
                            2 => "r9",    // Tercer param usuario
                            _ => {
                                let stack_offset = 48 + (i - 3) * 8;  // Stack params después de shadow space
                                self.text_section.push(format!("    mov rax, [rbp + {}]", stack_offset));
                                self.text_section.push(format!("    mov [rbp - {}], rax  ; param {}", offset + 8, param.name));
                                continue;
                            }
                        };
                        self.text_section.push(format!("    mov [rbp - {}], {}  ; param {}", offset + 8, reg, param.name));
                    }
                    
                    // Generar cuerpo del constructor
                    // Los FieldAssign a self.campo se manejarán automáticamente
                    for s in &init_method.body {
                        self.generate_stmt_windows(s)?;
                    }
                    
                    self.text_section.push("    leave".to_string());
                    self.text_section.push("    ret".to_string());
                    self.text_section.push(format!("{}_end:", init_label));
                    
                    // Limpiar variables locales del constructor
                    self.variables.remove("self");
                    for param in &init_method.params {
                        self.variables.remove(&param.name);
                    }
                }
                
                // Restaurar struct anterior
                self.current_struct = old_struct;
                // Struct definitions are type information only, no code generation needed for the struct itself
            }
        }
        Ok(())
    }

    fn generate_expr_windows(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Number(n) => {
                self.text_section.push(format!("    mov rax, {}", n));
            }
            Expr::Bool(b) => {
                // Bool: 0 = false, 1 = true en RAX
                // En x86-64, convención común: 0 = false, cualquier valor != 0 = true
                let value = if *b { 1 } else { 0 };
                self.text_section.push(format!("    mov rax, {}  ; bool {}", value, b));
            }
            Expr::Float(f) => {
                // Cargar constante flotante en XMM0
                // Estrategia: almacenar float en .data y cargar desde ahí
                let label = self.add_float_data(*f);
                self.text_section.push(format!("    movsd xmm0, [rel {}]  ; cargar float {}", label, f));
            }
            Expr::String(s) => {
                // Strings como expresiones: crear estructura String dinámica usando string_from_literal()
                // Similar a cómo ArrayLiteral usa array_from_values()
                
                // Crear literal temporal en .data para pasar a string_from_literal()
                let label = self.add_string_data(s);
                let length = s.len();
                
                // Preparar parámetros para string_from_literal(puntero_a_literal, longitud)
                // RCX = puntero a literal, RDX = longitud
                self.text_section.push(format!("    lea rcx, [rel {}]  ; puntero a literal", label));
                self.text_section.push(format!("    mov rdx, {}  ; longitud", length));
                
                // Llamar a string_from_literal
                self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                self.text_section.push("    call string_from_literal".to_string());
                self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
                
                // RAX contiene el puntero al String (en heap)
                // Este puntero debe ser almacenado en una variable para uso posterior
            }
            Expr::ArrayLiteral(elements) => {
                // Array literal: [1, 2, 3]
                // Estrategia: usar array_from_values (estructura Array dinámica en heap)
                // 1. Crear array temporal en stack con los valores
                // 2. Llamar a array_from_values(count, pointer)
                // 3. Retornar puntero al Array (en heap)
                
                let count = elements.len();
                let array_size = count * 8; // 8 bytes por elemento (int64)
                let base_offset = self.stack_offset;
                
                // Reservar espacio en stack para valores temporales
                self.stack_offset += array_size as i64;
                self.text_section.push(format!("    ; Array literal: {} elementos", count));
                self.text_section.push(format!("    sub rsp, {}  ; reservar espacio temporal para valores", array_size));
                
                // Generar y almacenar cada elemento en el array temporal
                for (i, element) in elements.iter().enumerate() {
                    self.generate_expr_windows(element)?;
                    let element_offset = base_offset + (i as i64 * 8);
                    self.text_section.push(format!("    mov [rbp - {}], rax  ; valor temporal[{}]", element_offset + 8, i));
                }
                
                // Preparar parámetros para array_from_values(count, pointer)
                // RCX = count, RDX = puntero a valores temporales
                self.text_section.push(format!("    mov rcx, {}  ; count", count));
                self.text_section.push(format!("    lea rdx, [rbp - {}]  ; puntero a valores temporales", base_offset + 8));
                
                // Llamar a array_from_values
                self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                self.text_section.push("    call array_from_values".to_string());
                self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
                
                // Liberar espacio temporal del stack
                self.text_section.push(format!("    add rsp, {}  ; liberar espacio temporal", array_size));
                self.stack_offset -= array_size as i64;
                
                // RAX contiene el puntero al Array (en heap)
                // Este puntero debe ser almacenado en una variable para uso posterior
            }
            Expr::TupleLiteral(elements) => {
                // Tuple literal: (1, 2, 3)
                // Almacenar elementos consecutivamente en stack
                let count = elements.len();
                let tuple_size = count * 8;
                let base_offset = self.stack_offset;
                
                self.stack_offset += tuple_size as i64;
                self.text_section.push(format!("    ; Tuple literal: {} elementos", count));
                
                // Generar y almacenar cada elemento
                for (i, element) in elements.iter().enumerate() {
                    self.generate_expr_windows(element)?;
                    let element_offset = base_offset + (i as i64 * 8);
                    self.text_section.push(format!("    mov [rbp - {}], rax  ; tuple[{}]", element_offset + 8, i));
                }
                
                // Retornar puntero al inicio de la tupla
                self.text_section.push(format!("    lea rax, [rbp - {}]  ; puntero a tupla", base_offset + 8));
            }
            Expr::Lambda { params, body } => {
                // Lambda: lambda x, y: x + y
                // Por ahora, generar una función anónima y retornar su dirección
                let lambda_name = self.new_label("lambda");
                
                // Guardar contexto actual
                let saved_vars = self.variables.clone();
                let saved_offset = self.stack_offset;
                
                // Generar función lambda
                self.text_section.push(format!("    jmp {}_end  ; saltar definición de lambda", lambda_name));
                self.text_section.push(format!("{}:", lambda_name));
                self.text_section.push("    push rbp".to_string());
                self.text_section.push("    mov rbp, rsp".to_string());
                self.text_section.push("    sub rsp, 64".to_string());
                
                // Mapear parámetros a registros (Windows x64: RCX, RDX, R8, R9)
                let param_regs = ["rcx", "rdx", "r8", "r9"];
                self.stack_offset = 0;
                for (i, param) in params.iter().enumerate() {
                    self.stack_offset += 8;
                    self.variables.insert(param.clone(), self.stack_offset);
                    if i < 4 {
                        self.text_section.push(format!("    mov [rbp - {}], {}  ; param {}", self.stack_offset + 8, param_regs[i], param));
                    }
                }
                
                // Generar cuerpo de lambda
                self.generate_expr_windows(body)?;
                
                // Retornar
                self.text_section.push("    leave".to_string());
                self.text_section.push("    ret".to_string());
                self.text_section.push(format!("{}_end:", lambda_name));
                
                // Restaurar contexto
                self.variables = saved_vars;
                self.stack_offset = saved_offset;
                
                // Retornar dirección de la lambda
                self.text_section.push(format!("    lea rax, [rel {}]  ; dirección de lambda", lambda_name));
            }
            Expr::Borrow { expr, .. } => {
                // Borrowing: generar dirección de la expresión
                // Por ahora, solo soportamos borrowing de variables
                self.generate_expr_windows(expr)?;
                // TODO: Generar código para obtener dirección (lea)
            }
            Expr::Deref(expr) => {
                // Dereferenciar: cargar valor desde la dirección
                self.generate_expr_windows(expr)?;
                // Si el valor está en rax (dirección), cargar desde esa dirección
                self.text_section.push("    mov rax, [rax]".to_string());
            }
            Expr::Ident(name) => {
                // Verificar si es un struct definido (no una variable)
                // Esto puede pasar si se intenta usar un struct como variable
                if self.struct_definitions.contains_key(name) {
                    return Err(adead_common::ADeadError::RuntimeError {
                        message: format!("'{}' es un struct, no una variable. Usa '{}.new(...)' para crear una instancia.", name, name),
                    });
                }
                
                if let Some(&offset) = self.variables.get(name) {
                    // Load variable from stack (Windows x64)
                    // Variables are stored at negative offsets from rbp
                    // offset is the stack offset, we need to use it as negative
                    self.text_section.push(format!("    mov rax, [rbp - {}]  ; load variable {}", offset + 8, name));
                } else {
                    return Err(adead_common::ADeadError::RuntimeError {
                        message: format!("undefined variable: {} (variables must be declared with 'let')", name),
                    });
                }
            }
            Expr::BinaryOp { left, op, right } => {
                // Detectar si ambos operandos son strings (concatenación)
                let is_string_op = self.is_string_expr(left) && self.is_string_expr(right) && *op == BinOp::Add;
                
                if is_string_op {
                    // Concatenación de strings: s1 + s2
                    // Evaluar left (String 1) → RAX
                    self.generate_expr_windows(left)?;
                    self.text_section.push("    push rax  ; guardar String 1".to_string());
                    
                    // Evaluar right (String 2) → RAX
                    self.generate_expr_windows(right)?;
                    self.text_section.push("    mov rdx, rax  ; String 2 en RDX".to_string());
                    
                    // Restaurar String 1 en RCX
                    self.text_section.push("    pop rcx  ; String 1 en RCX".to_string());
                    
                    // Llamar a string_concat(String1, String2)
                    self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                    self.text_section.push("    call string_concat".to_string());
                    self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
                    
                    // RAX contiene el puntero al nuevo String (concatenado)
                } else {
                    // Detectar si alguno de los operandos es float
                    let is_float_op = self.is_float_expr(left) || self.is_float_expr(right);
                    
                    if is_float_op {
                    // Operaciones con floats usando SSE
                    // Evaluar left → XMM0, right → XMM1
                    self.generate_expr_windows(left)?;
                    // Si left es int, convertir a float
                    if !self.is_float_expr(left) {
                        // Convertir int en RAX a float64 en XMM0
                        self.text_section.push("    cvtsi2sd xmm0, rax  ; convertir int a float64".to_string());
                    } else {
                        // left ya es float en XMM0
                        self.text_section.push("    movsd xmm0, xmm0  ; mantener float en XMM0".to_string());
                    }
                    
                    // Guardar XMM0 en stack (preservar left)
                    self.text_section.push("    sub rsp, 16  ; reservar espacio para float".to_string());
                    self.text_section.push("    movsd [rsp], xmm0  ; guardar left".to_string());
                    
                    // Evaluar right
                    self.generate_expr_windows(right)?;
                    // Si right es int, convertir a float
                    if !self.is_float_expr(right) {
                        // Convertir int en RAX a float64 en XMM0
                        self.text_section.push("    cvtsi2sd xmm0, rax  ; convertir int a float64".to_string());
                    } else {
                        // right ya es float, moverlo a XMM0 si es necesario
                        self.text_section.push("    movsd xmm0, xmm0  ; mantener float en XMM0".to_string());
                    }
                    
                    // Cargar left de stack a XMM1
                    self.text_section.push("    movsd xmm1, [rsp]  ; cargar left".to_string());
                    self.text_section.push("    add rsp, 16  ; restaurar stack".to_string());
                    
                    // Realizar operación SSE
                    match op {
                        BinOp::Add => {
                            self.text_section.push("    addsd xmm1, xmm0  ; float64 addition (result in XMM1)".to_string());
                            self.text_section.push("    movsd xmm0, xmm1  ; mover resultado a XMM0".to_string());
                        }
                        BinOp::Sub => {
                            self.text_section.push("    subsd xmm1, xmm0  ; float64 subtraction (XMM1 - XMM0, result in XMM1)".to_string());
                            self.text_section.push("    movsd xmm0, xmm1  ; mover resultado a XMM0".to_string());
                        }
                        BinOp::Mul => {
                            self.text_section.push("    mulsd xmm1, xmm0  ; float64 multiplication (result in XMM1)".to_string());
                            self.text_section.push("    movsd xmm0, xmm1  ; mover resultado a XMM0".to_string());
                        }
                        BinOp::Div => {
                            self.text_section.push("    divsd xmm1, xmm0  ; float64 division (XMM1 / XMM0, result in XMM1)".to_string());
                            self.text_section.push("    movsd xmm0, xmm1  ; mover resultado a XMM0".to_string());
                        }
                        _ => {
                            return Err(adead_common::ADeadError::RuntimeError {
                                message: format!("Comparison operators (==, !=, <, etc.) with floats not yet implemented"),
                            });
                        }
                    }
                    // Resultado queda en XMM0 (convención para floats)
                } else {
                    // Operaciones con enteros (código original)
                    self.generate_expr_windows(left)?;
                    self.text_section.push("    push rax".to_string());
                    self.generate_expr_windows(right)?;
                    self.text_section.push("    pop rbx".to_string());
                    
                    match op {
                        BinOp::Add => {
                            self.text_section.push("    add rax, rbx".to_string());
                        }
                        BinOp::Sub => {
                            self.text_section.push("    sub rax, rbx".to_string());
                        }
                        BinOp::Mul => {
                            self.text_section.push("    imul rax, rbx".to_string());
                        }
                        BinOp::Div => {
                            // División entera: RAX / RBX → RAX (cociente), RDX (residuo)
                            // Necesitamos sign-extend RAX a RDX:RAX antes de dividir
                            self.text_section.push("    cqo  ; sign-extend rax to rdx:rax".to_string());
                            self.text_section.push("    idiv rbx  ; rax = rax / rbx".to_string());
                        }
                        BinOp::FloorDiv => {
                            // División entera (//): igual que Div pero asegurando resultado entero
                            self.text_section.push("    cqo  ; sign-extend rax to rdx:rax".to_string());
                            self.text_section.push("    idiv rbx  ; rax = rax // rbx (división entera)".to_string());
                        }
                        BinOp::Mod => {
                            // Módulo: RAX % RBX → RDX
                            self.text_section.push("    cqo  ; sign-extend rax to rdx:rax".to_string());
                            self.text_section.push("    idiv rbx  ; rdx = rax % rbx".to_string());
                            self.text_section.push("    mov rax, rdx  ; mover residuo a rax".to_string());
                        }
                        BinOp::Pow => {
                            // Potencia: RBX ** RAX (base ** exponente)
                            // Después de evaluar: rbx = left (base), rax = right (exponente)
                            // Implementación simple usando loop (para enteros)
                            
                            let label_not_zero = self.new_label("pow_not_zero");
                            let label_loop = self.new_label("pow_loop");
                            let label_end = self.new_label("pow_end");
                            
                            // rcx = exponente (rax), rdx = base (rbx)
                            self.text_section.push("    mov rcx, rax  ; rcx = exponente".to_string());
                            self.text_section.push("    mov rdx, rbx  ; rdx = base".to_string());
                            
                            // Caso especial: exponente 0
                            self.text_section.push("    cmp rcx, 0".to_string());
                            self.text_section.push(format!("    jne {}  ; si exp != 0", label_not_zero));
                            self.text_section.push("    mov rax, 1  ; x^0 = 1".to_string());
                            self.text_section.push(format!("    jmp {}", label_end));
                            
                            // Loop de multiplicación
                            self.text_section.push(format!("{}:", label_not_zero));
                            self.text_section.push("    mov rax, 1  ; resultado = 1".to_string());
                            self.text_section.push(format!("{}:", label_loop));
                            self.text_section.push("    cmp rcx, 0".to_string());
                            self.text_section.push(format!("    jle {}  ; si exp <= 0, terminar", label_end));
                            self.text_section.push("    imul rax, rdx  ; resultado *= base".to_string());
                            self.text_section.push("    dec rcx  ; exp--".to_string());
                            self.text_section.push(format!("    jmp {}", label_loop));
                            self.text_section.push(format!("{}:", label_end));
                        }
                        BinOp::Eq => {
                            self.text_section.push("    cmp rax, rbx".to_string());
                            self.text_section.push("    sete al".to_string());
                            self.text_section.push("    movzx rax, al".to_string());
                        }
                        BinOp::Ne => {
                            self.text_section.push("    cmp rax, rbx".to_string());
                            self.text_section.push("    setne al".to_string());
                            self.text_section.push("    movzx rax, al".to_string());
                        }
                        BinOp::Lt => {
                            self.text_section.push("    cmp rbx, rax".to_string());
                            self.text_section.push("    setl al".to_string());
                            self.text_section.push("    movzx rax, al".to_string());
                        }
                        BinOp::Le => {
                            self.text_section.push("    cmp rbx, rax".to_string());
                            self.text_section.push("    setle al".to_string());
                            self.text_section.push("    movzx rax, al".to_string());
                        }
                        BinOp::Gt => {
                            self.text_section.push("    cmp rbx, rax".to_string());
                            self.text_section.push("    setg al".to_string());
                            self.text_section.push("    movzx rax, al".to_string());
                        }
                        BinOp::Ge => {
                            self.text_section.push("    cmp rbx, rax".to_string());
                            self.text_section.push("    setge al".to_string());
                            self.text_section.push("    movzx rax, al".to_string());
                        }
                        BinOp::And => {
                            // AND lógico con short-circuit evaluation
                            // RAX = left, RBX = right
                            // Resultado: 1 si ambos son != 0, 0 si alguno es 0
                            self.text_section.push("    test rax, rax".to_string());
                            self.text_section.push("    setnz al".to_string());
                            self.text_section.push("    test rbx, rbx".to_string());
                            self.text_section.push("    setnz bl".to_string());
                            self.text_section.push("    and al, bl".to_string());
                            self.text_section.push("    movzx rax, al".to_string());
                        }
                        BinOp::Or => {
                            // OR lógico con short-circuit evaluation
                            // RAX = left, RBX = right
                            // Resultado: 1 si alguno es != 0, 0 si ambos son 0
                            self.text_section.push("    test rax, rax".to_string());
                            self.text_section.push("    setnz al".to_string());
                            self.text_section.push("    test rbx, rbx".to_string());
                            self.text_section.push("    setnz bl".to_string());
                            self.text_section.push("    or al, bl".to_string());
                            self.text_section.push("    movzx rax, al".to_string());
                        }
                    }
                }
                }
            }
            Expr::Not(inner) => {
                // Negación lógica: !expr
                // Resultado: 1 si expr == 0, 0 si expr != 0
                self.generate_expr_windows(inner)?;
                self.text_section.push("    test rax, rax".to_string());
                self.text_section.push("    setz al".to_string());
                self.text_section.push("    movzx rax, al".to_string());
            }
            Expr::Call { module, name, args } => {
                // Detectar llamada a constructor PRIMERO: ClassName.new(...)
                // DEBE ir ANTES de built-ins y otras llamadas para tener prioridad
                if let Some(class_name) = &module {
                    if name == "new" {
                        if !self.struct_definitions.contains_key(class_name) {
                            return Err(adead_common::ADeadError::RuntimeError {
                                message: format!("'{}' no es un struct definido. Asegúrate de que el struct esté definido antes de usar '{}.new(...)'.", class_name, class_name),
                            });
                    }
                            // CONSTRUCTOR: ClassName.new(args)
                        // Layout simple: [campo0 (0)] [campo1 (8)] [campo2 (16)] ...
                        // 1. Reservar espacio en stack para el struct (sin vtable para structs simples)
                            let num_fields = self.struct_definitions.get(class_name)
                                .map(|f| f.len())
                                .unwrap_or(0);
                        let struct_size = num_fields * 8; // Sin vtable para structs simples
                            let struct_offset = self.stack_offset;
                            self.stack_offset += struct_size as i64;
                            
                        self.text_section.push(format!("    ; Constructor: {}.new() ({} campos, {} bytes)", class_name, num_fields, struct_size));
                        self.text_section.push(format!("    sub rsp, {}  ; reservar espacio para struct", struct_size));
                            
                            // Calcular dirección base del struct
                            let struct_base_offset = struct_offset + 8;
                            
                            // 2. Cargar argumentos del constructor PRIMERO (antes de modificar RCX)
                            let mut arg_values = Vec::new();
                            for arg in args.iter() {
                                self.generate_expr_windows(arg)?;
                                self.text_section.push("    push rax  ; guardar arg temporalmente".to_string());
                                arg_values.push(());
                            }
                            
                            // 3. Restaurar argumentos a registros (en orden inverso)
                            for i in (0..args.len()).rev() {
                                let reg = match i {
                                    0 => "rdx",  // Primer arg del usuario
                                    1 => "r8",
                                    2 => "r9",
                                    _ => continue,  // TODO: pasar en stack
                                };
                                self.text_section.push(format!("    pop {}  ; arg{}", reg, i));
                            }
                            
                            // 4. self = dirección base del struct
                        self.text_section.push(format!("    lea rcx, [rbp - {}]  ; self = puntero al struct", struct_base_offset));
                            
                            // 5. Llamar al constructor
                        let constructor_name = format!("fn_{}_new", class_name);
                            self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                        self.text_section.push(format!("    call {}  ; constructor", constructor_name));
                            self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
                            
                            // 6. Retornar dirección del struct en RAX
                            self.text_section.push(format!("    lea rax, [rbp - {}]  ; retornar puntero al struct", struct_base_offset));
                        return Ok(());
                        } else {
                            // Llamada a método estático: StructName.metodo(args)
                            // Métodos estáticos no tienen 'self', los parámetros van directamente en RCX, RDX, R8, R9...
                            let num_args = args.len();
                            let stack_args_count = if num_args > 4 { num_args - 4 } else { 0 };
                            let total_stack_space = 32 + (stack_args_count * 8);
                            
                            self.text_section.push(format!("    ; Llamada a método estático {}.{}", class_name, name));
                            self.text_section.push(format!("    sub rsp, {}  ; shadow space", 32));
                            
                            // Cargar argumentos en registros (RCX, RDX, R8, R9 para estáticos)
                            for (i, arg) in args.iter().take(4).enumerate() {
                                self.generate_expr_windows(arg)?;
                                let reg = match i {
                                    0 => "rcx",
                                    1 => "rdx",
                                    2 => "r8",
                                    3 => "r9",
                                    _ => unreachable!(),
                                };
                                self.text_section.push(format!("    mov {}, rax  ; param{}", reg, i));
                            }
                            
                            // Argumentos adicionales en stack
                            for (i, arg) in args.iter().skip(4).enumerate() {
                                self.generate_expr_windows(arg)?;
                                self.text_section.push("    push rax  ; arg en stack".to_string());
                            }
                            
                            let function_name = format!("fn_{}_{}", class_name, name);
                            self.text_section.push(format!("    call {}  ; método estático", function_name));
                            
                            // Limpiar argumentos del stack si los hay
                            if stack_args_count > 0 {
                                self.text_section.push(format!("    add rsp, {}  ; limpiar args del stack", stack_args_count * 8));
                        }
                            
                            self.text_section.push(format!("    add rsp, {}  ; restaurar shadow space", 32));
                            return Ok(());
                    }
                }
                
                // Built-in functions de conversión de tipos
                if module.is_none() && args.len() == 1 {
                    match name.as_str() {
                        "int" => {
                            // int(x) - convertir a entero
                            self.generate_expr_windows(&args[0])?;
                            // Si es float, truncar; si es string, parsear; si es bool, 0/1
                            // Por ahora, asumimos que ya es numérico o bool
                            self.text_section.push("    ; int(x) - valor ya en rax".to_string());
                            return Ok(());
                        }
                        "float" => {
                            // float(x) - convertir a float (por ahora solo retorna el valor)
                            self.generate_expr_windows(&args[0])?;
                            self.text_section.push("    ; float(x) - conversión pendiente".to_string());
                            return Ok(());
                        }
                        "str" => {
                            // str(x) - convertir a string (retorna puntero a string)
                            self.generate_expr_windows(&args[0])?;
                            self.text_section.push("    ; str(x) - valor numérico en rax".to_string());
                            return Ok(());
                        }
                        "bool" => {
                            // bool(x) - convertir a booleano (0 = false, != 0 = true)
                            self.generate_expr_windows(&args[0])?;
                            let true_label = self.new_label("bool_true");
                            let end_label = self.new_label("bool_end");
                            self.text_section.push("    cmp rax, 0".to_string());
                            self.text_section.push(format!("    jne {}", true_label));
                            self.text_section.push("    mov rax, 0  ; false".to_string());
                            self.text_section.push(format!("    jmp {}", end_label));
                            self.text_section.push(format!("{}:", true_label));
                            self.text_section.push("    mov rax, 1  ; true".to_string());
                            self.text_section.push(format!("{}:", end_label));
                            return Ok(());
                        }
                        "abs" => {
                            // abs(x) - valor absoluto
                            self.generate_expr_windows(&args[0])?;
                            let pos_label = self.new_label("abs_pos");
                            self.text_section.push("    cmp rax, 0".to_string());
                            self.text_section.push(format!("    jge {}", pos_label));
                            self.text_section.push("    neg rax".to_string());
                            self.text_section.push(format!("{}:", pos_label));
                            return Ok(());
                        }
                        "round" => {
                            // round(x) - redondear (por ahora solo retorna el valor entero)
                            self.generate_expr_windows(&args[0])?;
                            self.text_section.push("    ; round(x) - valor ya entero".to_string());
                            return Ok(());
                        }
                        "sum" => {
                            // sum(lista) - sumar todos los elementos de un array
                            self.generate_expr_windows(&args[0])?;
                            // RAX tiene puntero al Array struct
                            self.text_section.push("    ; sum(lista)".to_string());
                            self.text_section.push("    mov rsi, rax  ; puntero al Array".to_string());
                            self.text_section.push("    mov rcx, [rsi + 8]  ; Array->length".to_string());
                            self.text_section.push("    mov rdi, [rsi + 0]  ; Array->data".to_string());
                            self.text_section.push("    xor rax, rax  ; suma = 0".to_string());
                            let loop_label = self.new_label("sum_loop");
                            let end_label = self.new_label("sum_end");
                            self.text_section.push(format!("{}:", loop_label));
                            self.text_section.push("    cmp rcx, 0".to_string());
                            self.text_section.push(format!("    je {}", end_label));
                            self.text_section.push("    add rax, [rdi]  ; suma += elemento".to_string());
                            self.text_section.push("    add rdi, 8  ; siguiente elemento".to_string());
                            self.text_section.push("    dec rcx".to_string());
                            self.text_section.push(format!("    jmp {}", loop_label));
                            self.text_section.push(format!("{}:", end_label));
                            return Ok(());
                        }
                        "type" => {
                            // type(x) - retorna tipo (por ahora solo un identificador numérico)
                            // 0 = None, 1 = int, 2 = bool, 3 = string, 4 = array
                            self.generate_expr_windows(&args[0])?;
                            self.text_section.push("    mov rax, 1  ; tipo int por defecto".to_string());
                            return Ok(());
                        }
                        "sorted" => {
                            // sorted(lista) - retorna copia ordenada (bubble sort simple)
                            self.generate_expr_windows(&args[0])?;
                            self.text_section.push("    ; sorted(lista) - ordenar array".to_string());
                            self.text_section.push("    mov rsi, rax  ; puntero al Array".to_string());
                            self.text_section.push("    mov rcx, [rsi + 8]  ; length".to_string());
                            self.text_section.push("    mov rdi, [rsi + 0]  ; data".to_string());
                            // Bubble sort inline
                            let outer_loop = self.new_label("sort_outer");
                            let inner_loop = self.new_label("sort_inner");
                            let no_swap = self.new_label("sort_no_swap");
                            let sort_end = self.new_label("sort_end");
                            self.text_section.push("    dec rcx  ; n-1 iteraciones".to_string());
                            self.text_section.push(format!("{}:", outer_loop));
                            self.text_section.push("    cmp rcx, 0".to_string());
                            self.text_section.push(format!("    jle {}", sort_end));
                            self.text_section.push("    push rcx".to_string());
                            self.text_section.push("    mov rbx, rdi  ; ptr actual".to_string());
                            self.text_section.push(format!("{}:", inner_loop));
                            self.text_section.push("    cmp rcx, 0".to_string());
                            self.text_section.push(format!("    jle {}_done", inner_loop));
                            self.text_section.push("    mov rax, [rbx]".to_string());
                            self.text_section.push("    mov rdx, [rbx + 8]".to_string());
                            self.text_section.push("    cmp rax, rdx".to_string());
                            self.text_section.push(format!("    jle {}", no_swap));
                            self.text_section.push("    mov [rbx], rdx  ; swap".to_string());
                            self.text_section.push("    mov [rbx + 8], rax".to_string());
                            self.text_section.push(format!("{}:", no_swap));
                            self.text_section.push("    add rbx, 8".to_string());
                            self.text_section.push("    dec rcx".to_string());
                            self.text_section.push(format!("    jmp {}", inner_loop));
                            self.text_section.push(format!("{}_done:", inner_loop));
                            self.text_section.push("    pop rcx".to_string());
                            self.text_section.push("    dec rcx".to_string());
                            self.text_section.push(format!("    jmp {}", outer_loop));
                            self.text_section.push(format!("{}:", sort_end));
                            self.text_section.push("    mov rax, rsi  ; retornar puntero al array".to_string());
                            return Ok(());
                        }
                        "reversed" => {
                            // reversed(lista) - invertir array in-place
                            self.generate_expr_windows(&args[0])?;
                            self.text_section.push("    ; reversed(lista)".to_string());
                            self.text_section.push("    mov rsi, rax  ; puntero al Array".to_string());
                            self.text_section.push("    mov rcx, [rsi + 8]  ; length".to_string());
                            self.text_section.push("    mov rdi, [rsi + 0]  ; data inicio".to_string());
                            self.text_section.push("    mov rbx, rcx".to_string());
                            self.text_section.push("    dec rbx".to_string());
                            self.text_section.push("    shl rbx, 3  ; * 8".to_string());
                            self.text_section.push("    add rbx, rdi  ; data fin".to_string());
                            self.text_section.push("    shr rcx, 1  ; length / 2".to_string());
                            let rev_loop = self.new_label("rev_loop");
                            let rev_end = self.new_label("rev_end");
                            self.text_section.push(format!("{}:", rev_loop));
                            self.text_section.push("    cmp rcx, 0".to_string());
                            self.text_section.push(format!("    je {}", rev_end));
                            self.text_section.push("    mov rax, [rdi]".to_string());
                            self.text_section.push("    mov rdx, [rbx]".to_string());
                            self.text_section.push("    mov [rdi], rdx".to_string());
                            self.text_section.push("    mov [rbx], rax".to_string());
                            self.text_section.push("    add rdi, 8".to_string());
                            self.text_section.push("    sub rbx, 8".to_string());
                            self.text_section.push("    dec rcx".to_string());
                            self.text_section.push(format!("    jmp {}", rev_loop));
                            self.text_section.push(format!("{}:", rev_end));
                            self.text_section.push("    mov rax, rsi".to_string());
                            return Ok(());
                        }
                        "all" => {
                            // all(lista) - true si todos los elementos son truthy
                            self.generate_expr_windows(&args[0])?;
                            self.text_section.push("    ; all(lista)".to_string());
                            self.text_section.push("    mov rsi, rax".to_string());
                            self.text_section.push("    mov rcx, [rsi + 8]  ; length".to_string());
                            self.text_section.push("    mov rdi, [rsi + 0]  ; data".to_string());
                            let all_loop = self.new_label("all_loop");
                            let all_false = self.new_label("all_false");
                            let all_end = self.new_label("all_end");
                            self.text_section.push(format!("{}:", all_loop));
                            self.text_section.push("    cmp rcx, 0".to_string());
                            self.text_section.push(format!("    je {}_true", all_end));
                            self.text_section.push("    mov rax, [rdi]".to_string());
                            self.text_section.push("    cmp rax, 0".to_string());
                            self.text_section.push(format!("    je {}", all_false));
                            self.text_section.push("    add rdi, 8".to_string());
                            self.text_section.push("    dec rcx".to_string());
                            self.text_section.push(format!("    jmp {}", all_loop));
                            self.text_section.push(format!("{}_true:", all_end));
                            self.text_section.push("    mov rax, 1".to_string());
                            self.text_section.push(format!("    jmp {}", all_end));
                            self.text_section.push(format!("{}:", all_false));
                            self.text_section.push("    mov rax, 0".to_string());
                            self.text_section.push(format!("{}:", all_end));
                            return Ok(());
                        }
                        "any" => {
                            // any(lista) - true si algún elemento es truthy
                            self.generate_expr_windows(&args[0])?;
                            self.text_section.push("    ; any(lista)".to_string());
                            self.text_section.push("    mov rsi, rax".to_string());
                            self.text_section.push("    mov rcx, [rsi + 8]  ; length".to_string());
                            self.text_section.push("    mov rdi, [rsi + 0]  ; data".to_string());
                            let any_loop = self.new_label("any_loop");
                            let any_true = self.new_label("any_true");
                            let any_end = self.new_label("any_end");
                            self.text_section.push(format!("{}:", any_loop));
                            self.text_section.push("    cmp rcx, 0".to_string());
                            self.text_section.push(format!("    je {}_false", any_end));
                            self.text_section.push("    mov rax, [rdi]".to_string());
                            self.text_section.push("    cmp rax, 0".to_string());
                            self.text_section.push(format!("    jne {}", any_true));
                            self.text_section.push("    add rdi, 8".to_string());
                            self.text_section.push("    dec rcx".to_string());
                            self.text_section.push(format!("    jmp {}", any_loop));
                            self.text_section.push(format!("{}:", any_true));
                            self.text_section.push("    mov rax, 1".to_string());
                            self.text_section.push(format!("    jmp {}", any_end));
                            self.text_section.push(format!("{}_false:", any_end));
                            self.text_section.push("    mov rax, 0".to_string());
                            self.text_section.push(format!("{}:", any_end));
                            return Ok(());
                        }
                        "enumerate" => {
                            // enumerate(lista) - retorna array de tuplas (index, value)
                            self.generate_expr_windows(&args[0])?;
                            self.text_section.push("    ; enumerate(lista)".to_string());
                            self.text_section.push("    mov rsi, rax  ; array original".to_string());
                            self.text_section.push("    mov rcx, [rsi + 8]  ; length".to_string());
                            self.text_section.push("    mov rdi, [rsi + 0]  ; data".to_string());
                            // Por ahora retornar el array original (implementación completa requiere más trabajo)
                            self.text_section.push("    mov rax, rsi".to_string());
                            return Ok(());
                        }
                        "zip" => {
                            // zip requiere 2 argumentos, se maneja abajo
                        }
                        "map" => {
                            // map(func, lista) - aplicar función a cada elemento
                            // Requiere 2 argumentos, se maneja en la sección de 2 args
                        }
                        "filter" => {
                            // filter(func, lista) - filtrar elementos
                            // Requiere 2 argumentos, se maneja en la sección de 2 args
                        }
                        "input" => {
                            // input(prompt) - leer línea de stdin
                            // Primero imprimir el prompt si es string
                            if let Expr::String(prompt) = &args[0] {
                                let label = self.add_string_data(prompt);
                                self.text_section.push("    ; input(prompt)".to_string());
                                self.text_section.push("    mov rcx, [rbp+16]  ; stdout".to_string());
                                self.text_section.push(format!("    lea rdx, [rel {}]", label));
                                self.text_section.push(format!("    mov r8, {}_len", label));
                                self.text_section.push("    lea r9, [rbp+24]".to_string());
                                self.text_section.push("    mov qword [rsp+32], 0".to_string());
                                self.text_section.push("    call WriteFile".to_string());
                            }
                            // Reservar buffer para input (256 bytes)
                            let buffer_offset = self.stack_offset;
                            self.stack_offset += 256;
                            self.text_section.push("    ; Leer de stdin".to_string());
                            self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                            self.text_section.push("    mov rcx, -10  ; STD_INPUT_HANDLE".to_string());
                            self.text_section.push("    call GetStdHandle".to_string());
                            self.text_section.push("    mov rcx, rax  ; stdin handle".to_string());
                            self.text_section.push(format!("    lea rdx, [rbp - {}]  ; buffer", buffer_offset + 8));
                            self.text_section.push("    mov r8, 255  ; max bytes".to_string());
                            self.text_section.push("    lea r9, [rbp+24]  ; bytes read".to_string());
                            self.text_section.push("    mov qword [rsp+32], 0".to_string());
                            self.text_section.push("    call ReadFile".to_string());
                            self.text_section.push("    add rsp, 32".to_string());
                            // Retornar puntero al buffer
                            self.text_section.push(format!("    lea rax, [rbp - {}]", buffer_offset + 8));
                            return Ok(());
                        }
                        _ => {}
                    }
                }
                
                // Built-in functions con 2 argumentos
                if module.is_none() && args.len() == 2 {
                    match name.as_str() {
                        "min" => {
                            // min(a, b)
                            self.generate_expr_windows(&args[0])?;
                            self.text_section.push("    push rax".to_string());
                            self.generate_expr_windows(&args[1])?;
                            self.text_section.push("    pop rbx".to_string());
                            let min_label = self.new_label("min_done");
                            self.text_section.push("    cmp rbx, rax".to_string());
                            self.text_section.push(format!("    jle {}", min_label));
                            self.text_section.push("    mov rbx, rax".to_string());
                            self.text_section.push(format!("{}:", min_label));
                            self.text_section.push("    mov rax, rbx".to_string());
                            return Ok(());
                        }
                        "max" => {
                            // max(a, b)
                            self.generate_expr_windows(&args[0])?;
                            self.text_section.push("    push rax".to_string());
                            self.generate_expr_windows(&args[1])?;
                            self.text_section.push("    pop rbx".to_string());
                            let max_label = self.new_label("max_done");
                            self.text_section.push("    cmp rbx, rax".to_string());
                            self.text_section.push(format!("    jge {}", max_label));
                            self.text_section.push("    mov rbx, rax".to_string());
                            self.text_section.push(format!("{}:", max_label));
                            self.text_section.push("    mov rax, rbx".to_string());
                            return Ok(());
                        }
                        "pow" => {
                            // pow(base, exp) - potencia
                            self.generate_expr_windows(&args[0])?;
                            self.text_section.push("    push rax  ; base".to_string());
                            self.generate_expr_windows(&args[1])?;
                            self.text_section.push("    mov rcx, rax  ; exp".to_string());
                            self.text_section.push("    pop rbx  ; base".to_string());
                            self.text_section.push("    mov rax, 1  ; resultado".to_string());
                            let loop_label = self.new_label("pow_loop");
                            let end_label = self.new_label("pow_end");
                            self.text_section.push(format!("{}:", loop_label));
                            self.text_section.push("    cmp rcx, 0".to_string());
                            self.text_section.push(format!("    je {}", end_label));
                            self.text_section.push("    imul rax, rbx".to_string());
                            self.text_section.push("    dec rcx".to_string());
                            self.text_section.push(format!("    jmp {}", loop_label));
                            self.text_section.push(format!("{}:", end_label));
                            return Ok(());
                        }
                        "zip" => {
                            // zip(a, b) - combinar dos arrays en array de tuplas
                            self.generate_expr_windows(&args[0])?;
                            self.text_section.push("    push rax  ; array a".to_string());
                            self.generate_expr_windows(&args[1])?;
                            self.text_section.push("    mov rsi, rax  ; array b".to_string());
                            self.text_section.push("    pop rdi  ; array a".to_string());
                            self.text_section.push("    ; zip(a, b) - retorna array a por ahora".to_string());
                            self.text_section.push("    mov rax, rdi".to_string());
                            return Ok(());
                        }
                        "map" => {
                            // map(func, lista) - aplicar función a cada elemento
                            self.generate_expr_windows(&args[0])?;
                            self.text_section.push("    push rax  ; función".to_string());
                            self.generate_expr_windows(&args[1])?;
                            self.text_section.push("    mov rsi, rax  ; array".to_string());
                            self.text_section.push("    pop rdi  ; función".to_string());
                            // Implementación básica: iterar y llamar función
                            self.text_section.push("    ; map(f, arr) - aplicar f a cada elemento".to_string());
                            self.text_section.push("    mov rcx, [rsi + 8]  ; length".to_string());
                            self.text_section.push("    mov rbx, [rsi + 0]  ; data".to_string());
                            let map_loop = self.new_label("map_loop");
                            let map_end = self.new_label("map_end");
                            self.text_section.push("    push rsi  ; guardar array".to_string());
                            self.text_section.push(format!("{}:", map_loop));
                            self.text_section.push("    cmp rcx, 0".to_string());
                            self.text_section.push(format!("    je {}", map_end));
                            self.text_section.push("    push rcx".to_string());
                            self.text_section.push("    push rbx".to_string());
                            self.text_section.push("    push rdi".to_string());
                            self.text_section.push("    mov rcx, [rbx]  ; elemento actual".to_string());
                            self.text_section.push("    sub rsp, 32".to_string());
                            self.text_section.push("    call rdi  ; llamar función".to_string());
                            self.text_section.push("    add rsp, 32".to_string());
                            self.text_section.push("    pop rdi".to_string());
                            self.text_section.push("    pop rbx".to_string());
                            self.text_section.push("    mov [rbx], rax  ; guardar resultado".to_string());
                            self.text_section.push("    add rbx, 8".to_string());
                            self.text_section.push("    pop rcx".to_string());
                            self.text_section.push("    dec rcx".to_string());
                            self.text_section.push(format!("    jmp {}", map_loop));
                            self.text_section.push(format!("{}:", map_end));
                            self.text_section.push("    pop rax  ; retornar array modificado".to_string());
                            return Ok(());
                        }
                        "filter" => {
                            // filter(func, lista) - filtrar elementos
                            self.generate_expr_windows(&args[0])?;
                            self.text_section.push("    push rax  ; función".to_string());
                            self.generate_expr_windows(&args[1])?;
                            self.text_section.push("    ; filter - retorna array original por ahora".to_string());
                            return Ok(());
                        }
                        _ => {}
                    }
                }

                // Detectar built-ins como len(arr) o len(s)
                if module.is_none() && name == "len" && args.len() == 1 {
                    // Detectar si el argumento es string o array
                    let is_string = self.is_string_expr(&args[0]);
                    
                    // Generar expresión (puntero al Array o String)
                            self.generate_expr_windows(&args[0])?;
                    self.text_section.push("    push rax  ; guardar puntero".to_string());
                            
                    // Preparar parámetros: RCX = puntero
                    self.text_section.push("    pop rcx  ; puntero".to_string());
                    
                    if is_string {
                        // len(s) -> string_len(s)
                        // Llamar a string_len (no necesita shadow space porque no llama a funciones externas)
                        self.text_section.push("    call string_len".to_string());
                    } else {
                        // len(arr) -> array_len(arr)
                        // Llamar a array_len (no necesita shadow space porque no llama a funciones externas)
                        self.text_section.push("    call array_len".to_string());
                    }
                            
                    // Retorna la longitud en RAX (ya está ahí)
                            return Ok(());
                        }
                        
                // Llamada a función normal (ABI-safe)
                        let num_args = args.len();
                        let stack_args_count = if num_args > 4 { num_args - 4 } else { 0 };
                        let total_stack_space = 32 + (stack_args_count * 8);
                        
                        self.text_section.push(format!("    sub rsp, {}  ; shadow space", 32));
                        
                        for arg in args.iter().skip(4).rev() {
                            self.generate_expr_windows(arg)?;
                            self.text_section.push("    push rax  ; parámetro adicional en stack".to_string());
                        }
                        
                        for (i, arg) in args.iter().take(4).enumerate() {
                            self.generate_expr_windows(arg)?;
                            let reg = match i {
                                0 => "rcx",
                                1 => "rdx",
                                2 => "r8",
                                3 => "r9",
                                _ => unreachable!(),
                            };
                            self.text_section.push(format!("    mov {}, rax  ; param{}", reg, i));
                        }
                        
                        let function_name = format!("fn_{}", name);
                        self.text_section.push(format!("    call {}", function_name));
                        self.text_section.push(format!("    add rsp, {}  ; restaurar shadow space", total_stack_space));
                    
                    // Valor de retorno está en RAX
            }
            Expr::Assign { name, value } => {
                // Verificar si es asignación a índice de array: arr[0] = value
                // El parser marca esto con name = "_array_set" y value es BinaryOp con Index
                if name == "_array_set" {
                    if let Expr::BinaryOp { left, right, .. } = value.as_ref() {
                        if let Expr::Index { array, index } = left.as_ref() {
                            // Generar expresión del array (puntero al Array)
                            self.generate_expr_windows(array)?;
                            self.text_section.push("    push rax  ; guardar puntero al Array".to_string());
                            
                            // Generar expresión del índice
                            self.generate_expr_windows(index)?;
                            self.text_section.push("    push rax  ; guardar índice".to_string());
                            
                            // Generar expresión del valor
                            self.generate_expr_windows(right)?;
                            // rax contiene el valor
                            
                            // Preparar parámetros para array_set: RCX = puntero al Array, RDX = índice, R8 = valor
                            self.text_section.push("    mov r8, rax  ; valor".to_string());
                            self.text_section.push("    pop rdx  ; índice".to_string());
                            self.text_section.push("    pop rcx  ; puntero al Array".to_string());
                            
                            // Llamar a array_set
                            self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                            self.text_section.push("    call array_set".to_string());
                            self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
                            
                            // array_set no retorna valor (void), pero dejamos el valor en rax por si se necesita
                            self.text_section.push("    mov rax, r8  ; restaurar valor en rax".to_string());
                            return Ok(());
                        }
                    }
                }
                
                // Asignación normal: variable = value
                self.generate_expr_windows(value)?;
                // Store in variable
                if let Some(&offset) = self.variables.get(name) {
                    self.text_section.push(format!("    mov [rbp - {}], rax  ; asignar a variable {}", offset + 8, name));
                } else {
                    // Variable doesn't exist, create it
                    let offset = self.stack_offset;
                    self.stack_offset += 8;
                    self.variables.insert(name.clone(), offset);
                    self.text_section.push(format!("    mov [rbp - {}], rax  ; crear variable {}", offset + 8, name));
                }
            }
            // Operadores compuestos: x += 5, x -= 3, x *= 2, x /= 4 (Sprint 1 - Python-like)
            Expr::CompoundAssign { name, op, value } => {
                self.add_debug_comment(&format!("{} {}= expr", name, match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    _ => "?",
                }));
                
                // Cargar valor actual de la variable
                if let Some(&offset) = self.variables.get(name) {
                    self.text_section.push(format!("    mov rbx, [rbp - {}]  ; cargar {} actual", offset + 8, name));
                } else {
                    return Err(adead_common::ADeadError::RuntimeError {
                        message: format!("Variable '{}' no definida para operador compuesto", name),
                    });
                }
                
                // Evaluar la expresión del valor
                self.generate_expr_windows(value)?;
                // RAX contiene el valor, RBX contiene el valor actual de la variable
                
                // Aplicar operación
                match op {
                    BinOp::Add => {
                        self.text_section.push("    add rbx, rax  ; += operación".to_string());
                    }
                    BinOp::Sub => {
                        self.text_section.push("    sub rbx, rax  ; -= operación".to_string());
                    }
                    BinOp::Mul => {
                        self.text_section.push("    imul rbx, rax  ; *= operación".to_string());
                    }
                    BinOp::Div => {
                        // División: rbx / rax
                        self.text_section.push("    push rax  ; guardar divisor".to_string());
                        self.text_section.push("    mov rax, rbx  ; dividendo en rax".to_string());
                        self.text_section.push("    cqo  ; sign-extend".to_string());
                        self.text_section.push("    pop rbx  ; divisor en rbx".to_string());
                        self.text_section.push("    idiv rbx  ; /= operación".to_string());
                        self.text_section.push("    mov rbx, rax  ; resultado en rbx".to_string());
                    }
                    _ => {
                        return Err(adead_common::ADeadError::RuntimeError {
                            message: format!("Operador compuesto no soportado: {:?}", op),
                        });
                    }
                }
                
                // Guardar resultado en la variable
                if let Some(&offset) = self.variables.get(name) {
                    self.text_section.push(format!("    mov [rbp - {}], rbx  ; guardar resultado en {}", offset + 8, name));
                    self.text_section.push("    mov rax, rbx  ; resultado en rax".to_string());
                }
            }
            // F-strings: f"Hola {nombre}" (Sprint 2.3 - Python-like)
            Expr::FString { parts } => {
                use adead_parser::FStringPart;
                // Estrategia: concatenar todas las partes en un buffer en stack
                // Por ahora, implementación simple: crear string concatenado en compile-time si es posible
                // o generar código para concatenación en runtime
                
                // Calcular tamaño total estimado y crear buffer
                let buffer_size = 256; // Buffer fijo por ahora
                let buffer_offset = self.stack_offset;
                self.stack_offset += buffer_size;
                
                self.text_section.push(format!("    ; F-string con {} partes", parts.len()));
                self.text_section.push(format!("    sub rsp, {}  ; buffer para f-string", buffer_size));
                self.text_section.push(format!("    lea rdi, [rbp - {}]  ; puntero al buffer", buffer_offset + 8));
                self.text_section.push("    xor r12, r12  ; contador de longitud total".to_string());
                
                for part in parts {
                    match part {
                        FStringPart::Literal(s) => {
                            // Copiar literal al buffer
                            let label = self.add_string_data(s);
                            self.text_section.push(format!("    ; Copiar literal: \"{}\"", s.escape_default()));
                            self.text_section.push(format!("    lea rsi, [rel {}]", label));
                            self.text_section.push(format!("    mov rcx, {}_len", label));
                            // Copiar bytes
                            self.text_section.push("    push rdi".to_string());
                            self.text_section.push("    add rdi, r12  ; posición actual en buffer".to_string());
                            let copy_loop = self.new_label("fstr_copy");
                            let copy_done = self.new_label("fstr_copy_done");
                            self.text_section.push(format!("{}:", copy_loop));
                            self.text_section.push("    cmp rcx, 0".to_string());
                            self.text_section.push(format!("    je {}", copy_done));
                            self.text_section.push("    mov al, [rsi]".to_string());
                            self.text_section.push("    mov [rdi], al".to_string());
                            self.text_section.push("    inc rsi".to_string());
                            self.text_section.push("    inc rdi".to_string());
                            self.text_section.push("    inc r12".to_string());
                            self.text_section.push("    dec rcx".to_string());
                            self.text_section.push(format!("    jmp {}", copy_loop));
                            self.text_section.push(format!("{}:", copy_done));
                            self.text_section.push("    pop rdi".to_string());
                        }
                        FStringPart::Expr(expr) => {
                            // Evaluar expresión y convertir a string
                            self.text_section.push("    ; Evaluar expresión interpolada".to_string());
                            self.text_section.push("    push rdi".to_string());
                            self.text_section.push("    push r12".to_string());
                            self.generate_expr_windows(expr)?;
                            self.text_section.push("    pop r12".to_string());
                            self.text_section.push("    pop rdi".to_string());
                            // RAX tiene el valor, convertir a string
                            // Por ahora solo soportamos números
                            self.text_section.push("    ; Convertir número a string".to_string());
                            self.text_section.push("    push rdi".to_string());
                            self.text_section.push("    add rdi, r12  ; posición en buffer".to_string());
                            self.text_section.push("    mov rbx, rdi  ; guardar inicio".to_string());
                            self.text_section.push("    mov rcx, 10".to_string());
                            // Convertir número a string (simple)
                            let num_loop = self.new_label("fstr_num");
                            let num_done = self.new_label("fstr_num_done");
                            self.text_section.push(format!("{}:", num_loop));
                            self.text_section.push("    xor rdx, rdx".to_string());
                            self.text_section.push("    div rcx".to_string());
                            self.text_section.push("    add dl, '0'".to_string());
                            self.text_section.push("    mov [rdi], dl".to_string());
                            self.text_section.push("    inc rdi".to_string());
                            self.text_section.push("    inc r12".to_string());
                            self.text_section.push("    cmp rax, 0".to_string());
                            self.text_section.push(format!("    jne {}", num_loop));
                            self.text_section.push(format!("{}:", num_done));
                            // Revertir dígitos
                            self.text_section.push("    mov rsi, rbx".to_string());
                            self.text_section.push("    mov rcx, rdi".to_string());
                            self.text_section.push("    dec rcx".to_string());
                            let rev_loop = self.new_label("fstr_rev");
                            let rev_done = self.new_label("fstr_rev_done");
                            self.text_section.push(format!("{}:", rev_loop));
                            self.text_section.push("    cmp rsi, rcx".to_string());
                            self.text_section.push(format!("    jge {}", rev_done));
                            self.text_section.push("    mov al, [rsi]".to_string());
                            self.text_section.push("    mov bl, [rcx]".to_string());
                            self.text_section.push("    mov [rsi], bl".to_string());
                            self.text_section.push("    mov [rcx], al".to_string());
                            self.text_section.push("    inc rsi".to_string());
                            self.text_section.push("    dec rcx".to_string());
                            self.text_section.push(format!("    jmp {}", rev_loop));
                            self.text_section.push(format!("{}:", rev_done));
                            self.text_section.push("    pop rdi".to_string());
                        }
                    }
                }
                
                // Resultado: dirección del buffer en rax, longitud en r12
                self.text_section.push(format!("    lea rax, [rbp - {}]  ; dirección del f-string", buffer_offset + 8));
                // Guardar longitud para uso posterior
                self.text_section.push("    mov [rax + 8], r12  ; guardar longitud".to_string());
            }
            // Option/Result constructors (O0.4) - Tagged unions en NASM
            // Opción<T>: 16 bytes = tag (8 bytes) + valor (8 bytes)
            //   Tag 0 = None, Tag 1 = Some(valor)
            // Result<T, E>: 16 bytes = tag (8 bytes) + valor (8 bytes)
            //   Tag 0 = Ok(valor), Tag 1 = Err(error)
            
            Expr::Some(expr) => {
                // Some(valor): tag = 1, valor = expr
                // Resultado en stack: [rbp - offset] = tag (1), [rbp - offset + 8] = valor
                self.generate_expr_windows(expr)?;
                // Guardar valor en rbx temporalmente
                self.text_section.push("    push rax  ; guardar valor de Some".to_string());
                // Tag = 1 para Some
                self.text_section.push("    mov rax, 1  ; tag Some = 1".to_string());
                // El resultado es la dirección del tagged union en stack
                // Necesitamos espacio para tag + valor = 16 bytes
                let offset = self.stack_offset;
                self.stack_offset += 16;  // Tag (8) + valor (8)
                self.text_section.push("    sub rsp, 16  ; espacio para Option (tag + valor)".to_string());
                // Guardar tag
                self.text_section.push(format!("    mov [rbp - {}], rax  ; tag = 1 (Some)", offset + 8));
                // Guardar valor
                self.text_section.push("    pop rax  ; recuperar valor".to_string());
                self.text_section.push(format!("    mov [rbp - {}], rax  ; valor", offset + 16));
                // Dejar dirección del tagged union en rax (dirección del tag)
                self.text_section.push(format!("    lea rax, [rbp - {}]  ; dirección del Option", offset + 8));
            }
            Expr::Ok(expr) => {
                // Ok(valor): tag = 0, valor = expr
                self.generate_expr_windows(expr)?;
                self.text_section.push("    push rax  ; guardar valor de Ok".to_string());
                // Tag = 0 para Ok
                self.text_section.push("    mov rax, 0  ; tag Ok = 0".to_string());
                let offset = self.stack_offset;
                self.stack_offset += 16;
                self.text_section.push("    sub rsp, 16  ; espacio para Result (tag + valor)".to_string());
                self.text_section.push(format!("    mov [rbp - {}], rax  ; tag = 0 (Ok)", offset + 8));
                self.text_section.push("    pop rax  ; recuperar valor".to_string());
                self.text_section.push(format!("    mov [rbp - {}], rax  ; valor", offset + 16));
                self.text_section.push(format!("    lea rax, [rbp - {}]  ; dirección del Result", offset + 8));
            }
            Expr::Err(expr) => {
                // Err(error): tag = 1, error = expr
                self.generate_expr_windows(expr)?;
                self.text_section.push("    push rax  ; guardar error de Err".to_string());
                // Tag = 1 para Err
                self.text_section.push("    mov rax, 1  ; tag Err = 1".to_string());
                let offset = self.stack_offset;
                self.stack_offset += 16;
                self.text_section.push("    sub rsp, 16  ; espacio para Result (tag + valor)".to_string());
                self.text_section.push(format!("    mov [rbp - {}], rax  ; tag = 1 (Err)", offset + 8));
                self.text_section.push("    pop rax  ; recuperar error".to_string());
                self.text_section.push(format!("    mov [rbp - {}], rax  ; error", offset + 16));
                self.text_section.push(format!("    lea rax, [rbp - {}]  ; dirección del Result", offset + 8));
            }
            Expr::None => {
                // None: tag = 0, valor = 0
                let offset = self.stack_offset;
                self.stack_offset += 16;
                self.text_section.push("    sub rsp, 16  ; espacio para Option (tag + valor)".to_string());
                // Tag = 0 para None
                self.text_section.push("    mov rax, 0  ; tag None = 0".to_string());
                self.text_section.push(format!("    mov [rbp - {}], rax  ; tag = 0 (None)", offset + 8));
                // Valor = 0
                self.text_section.push(format!("    mov [rbp - {}], rax  ; valor = 0", offset + 16));
                // Dejar dirección en rax
                self.text_section.push(format!("    lea rax, [rbp - {}]  ; dirección del Option", offset + 8));
            }
            // Structs (Fase 1.2 - O1, O3, O4)
            Expr::StructLiteral { name, fields } => {
                // Registrar los campos del struct para poder calcular offsets después
                let field_names: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();
                if !self.struct_definitions.contains_key(name) {
                    self.struct_definitions.insert(name.clone(), field_names);
                }
                
                // Generar struct literal en stack
                // Layout en memoria (stack crece hacia abajo):
                // [rbp - 8]  = campo0 (offset 0 desde base)
                // [rbp - 16] = campo1 (offset 8 desde base)
                // [rbp - 24] = campo2 (offset 16 desde base)
                // La dirección base apunta a [rbp - 8]
                let struct_size = fields.len() * 8;
                let base_offset = self.stack_offset;
                self.stack_offset += struct_size as i64;
                self.text_section.push(format!("    ; Struct literal: {} ({} campos, {} bytes)", name, fields.len(), struct_size));
                self.text_section.push(format!("    sub rsp, {}  ; reservar espacio para struct", struct_size));
                
                // Generar valores de campos en orden
                for (i, (field_name, value)) in fields.iter().enumerate() {
                    self.generate_expr_windows(value)?;
                    let field_offset = base_offset + (i as i64 * 8);
                    self.text_section.push(format!("    mov [rbp - {}], rax  ; {}.{} = {}", field_offset + 8, name, field_name, 
                        if let Expr::Number(n) = value { n.to_string() } else { "expr".to_string() }));
                }
                
                // Retornar dirección base del struct (apunta al primer campo)
                self.text_section.push(format!("    lea rax, [rbp - {}]  ; dirección del struct {}", base_offset + 8, name));
            }
            Expr::FieldAccess { object, field } => {
                // Determinar el tipo del objeto para calcular el offset correcto
                let struct_type = self.get_struct_type_from_expr(object);
                
                self.generate_expr_windows(object)?;
                
                // Calcular offset del campo
                // Layout en memoria: base apunta a campo0, campo1 está 8 bytes ABAJO en el stack
                // Pero como usamos direcciones, campo1 está en [base - 8], NO [base + 8]
                // CORRECCIÓN: En realidad, cuando hacemos lea rax, [rbp - X], rax apunta a esa dirección
                // y los campos siguientes están en direcciones MENORES (stack crece hacia abajo)
                // Por lo tanto: campo0 en [base], campo1 en [base - 8], campo2 en [base - 16]
                let field_offset = if let Some(ref type_name) = struct_type {
                    if let Some(fields) = self.struct_definitions.get(type_name) {
                        // Buscar el campo y calcular offset NEGATIVO (stack crece hacia abajo)
                        let pos = fields.iter().position(|f| f == field).unwrap_or(0);
                        -(pos as i64 * 8)
                    } else {
                        0
                    }
                } else {
                    0
                };
                
                self.text_section.push(format!("    ; accediendo campo '{}' (offset: {})", field, field_offset));
                if field_offset == 0 {
                    self.text_section.push("    mov rax, [rax]  ; cargar campo".to_string());
                } else if field_offset < 0 {
                    self.text_section.push(format!("    mov rax, [rax - {}]  ; cargar campo", -field_offset));
                } else {
                    self.text_section.push(format!("    mov rax, [rax + {}]  ; cargar campo", field_offset));
                }
            }
            Expr::FieldAssign { object, field, value } => {
                // Asignación a campo de struct: obj.field = value
                let struct_type = self.get_struct_type_from_expr(object);
                
                // PRIMERO: obtener la dirección del objeto y guardarla
                self.generate_expr_windows(object)?;
                self.text_section.push("    push rax  ; guardar dirección del objeto".to_string());
                
                // SEGUNDO: generar el valor a asignar
                self.generate_expr_windows(value)?;
                self.text_section.push("    mov rbx, rax  ; mover valor a rbx".to_string());
                
                // TERCERO: restaurar dirección del objeto
                self.text_section.push("    pop rax  ; restaurar dirección del objeto".to_string());
                
                // Calcular offset del campo (negativo porque stack crece hacia abajo)
                let field_offset = if let Some(ref type_name) = struct_type {
                    if let Some(fields) = self.struct_definitions.get(type_name) {
                        let pos = fields.iter().position(|f| f == field).unwrap_or(0);
                        -(pos as i64 * 8)
                    } else {
                        0
                    }
                } else {
                    0
                };
                
                self.text_section.push(format!("    ; asignando a campo '{}' (offset: {})", field, field_offset));
                if field_offset == 0 {
                    self.text_section.push("    mov [rax], rbx  ; asignar campo".to_string());
                } else if field_offset < 0 {
                    self.text_section.push(format!("    mov [rax - {}], rbx  ; asignar campo", -field_offset));
                } else {
                    self.text_section.push(format!("    mov [rax + {}], rbx  ; asignar campo", field_offset));
                }
            }
            Expr::Index { array, index } => {
                // Indexación: arr[0] o s[0] (para strings, solo lectura de carácter)
                // Detectar si es string o array
                let is_string = self.is_string_expr(array);
                
                if is_string {
                    // Para strings, por ahora solo soportamos lectura de carácter individual
                    // TODO: Implementar string_get_char() helper si se necesita
                    // Por ahora, retornar error o usar slicing con start=index, end=index+1
                    return Err(adead_common::ADeadError::RuntimeError {
                        message: format!("String indexing not yet implemented. Use slicing instead: s[{}:{}]", 
                            if let Expr::Number(n) = index.as_ref() { *n } else { 0 },
                            if let Expr::Number(n) = index.as_ref() { *n + 1 } else { 1 }),
                    });
                } else {
                    // Indexación de array: arr[0]
                    // Estrategia: usar array_get (estructura Array dinámica)
                    // 1. Generar expresión del array (puntero al Array en rax)
                    // 2. Generar expresión del índice
                    // 3. Llamar a array_get(array_ptr, index)
                    // 4. Retornar valor en RAX
                    
                    self.generate_expr_windows(array)?;
                    // Guardar puntero al Array en stack
                    self.text_section.push("    push rax  ; guardar puntero al Array".to_string());
                    
                    self.generate_expr_windows(index)?;
                    // rax contiene el índice
                    // Preparar parámetros: RCX = puntero al Array, RDX = índice
                    self.text_section.push("    mov rdx, rax  ; índice".to_string());
                    self.text_section.push("    pop rcx  ; puntero al Array".to_string());
                    
                    // Llamar a array_get
                    // Nota: array_get no necesita shadow space porque no llama a funciones externas
                    self.text_section.push("    call array_get".to_string());
                    
                    // RAX contiene el valor del elemento
                }
            }
            Expr::Slice { object, start, end } => {
                // Slicing: s[0:4]
                // Estrategia: usar string_slice (estructura String dinámica)
                // 1. Generar expresión del string (puntero al String en rax)
                // 2. Generar expresión del inicio
                // 3. Generar expresión del fin
                // 4. Llamar a string_slice(string_ptr, start, end)
                // 5. Retornar puntero al nuevo String en RAX
                
                self.generate_expr_windows(object)?;
                // Guardar puntero al String en stack
                self.text_section.push("    push rax  ; guardar puntero al String".to_string());
                
                self.generate_expr_windows(start)?;
                // rax contiene el índice inicio
                self.text_section.push("    push rax  ; guardar start".to_string());
                
                self.generate_expr_windows(end)?;
                // rax contiene el índice fin
                // Preparar parámetros: RCX = puntero al String, RDX = start, R8 = end
                self.text_section.push("    mov r8, rax  ; end".to_string());
                self.text_section.push("    pop rdx  ; start".to_string());
                self.text_section.push("    pop rcx  ; puntero al String".to_string());
                
                // Llamar a string_slice
                self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                self.text_section.push("    call string_slice".to_string());
                self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
                
                // RAX contiene el puntero al nuevo String (slice)
            }
            Expr::MethodCall { object, method, args } => {
                // Detectar métodos de arrays y llamar a funciones helper específicas
                match method.as_str() {
                    "append" if args.len() == 1 => {
                        // arr.append(x) -> array_append(arr, x)
                        // Generar expresión del array (puntero al Array)
                        self.generate_expr_windows(object)?;
                        self.text_section.push("    push rax  ; guardar puntero al Array".to_string());
                        
                        // Generar expresión del valor a agregar
                        self.generate_expr_windows(&args[0])?;
                        self.text_section.push("    push rax  ; guardar valor".to_string());
                        
                        // Preparar parámetros: RCX = puntero al Array, RDX = valor
                        self.text_section.push("    pop rdx  ; valor".to_string());
                        self.text_section.push("    pop rcx  ; puntero al Array".to_string());
                        
                        // Llamar a array_append
                        self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                        self.text_section.push("    call array_append".to_string());
                        self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
                        
                        // array_append no retorna valor (void), pero dejamos 0 en rax
                        self.text_section.push("    mov rax, 0  ; void return".to_string());
                    }
                    "pop" if args.is_empty() => {
                        // arr.pop() -> array_pop(arr)
                        // Generar expresión del array (puntero al Array)
                        self.generate_expr_windows(object)?;
                        self.text_section.push("    push rax  ; guardar puntero al Array".to_string());
                        
                        // Preparar parámetros: RCX = puntero al Array
                        self.text_section.push("    pop rcx  ; puntero al Array".to_string());
                        
                        // Llamar a array_pop
                        self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                        self.text_section.push("    call array_pop".to_string());
                        self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
                        
                        // array_pop retorna el valor en RAX (ya está ahí)
                    }
                    "reverse" if args.is_empty() => {
                        // arr.reverse() -> array_reverse(arr)
                        // Generar expresión del array (puntero al Array)
                        self.generate_expr_windows(object)?;
                        self.text_section.push("    push rax  ; guardar puntero al Array".to_string());
                        
                        // Preparar parámetros: RCX = puntero al Array
                        self.text_section.push("    pop rcx  ; puntero al Array".to_string());
                        
                        // Llamar a array_reverse
                        self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                        self.text_section.push("    call array_reverse".to_string());
                        self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
                        
                        // array_reverse no retorna valor (void), pero dejamos 0 en rax
                        self.text_section.push("    mov rax, 0  ; void return".to_string());
                    }
                    "insert" if args.len() == 2 => {
                        // arr.insert(i, x) -> array_insert(arr, i, x)
                        // Generar expresión del array (puntero al Array)
                        self.generate_expr_windows(object)?;
                        self.text_section.push("    push rax  ; guardar puntero al Array".to_string());
                        
                        // Generar expresión del índice
                        self.generate_expr_windows(&args[0])?;
                        self.text_section.push("    push rax  ; guardar índice".to_string());
                        
                        // Generar expresión del valor
                        self.generate_expr_windows(&args[1])?;
                        self.text_section.push("    push rax  ; guardar valor".to_string());
                        
                        // Preparar parámetros: RCX = puntero al Array, RDX = índice, R8 = valor
                        self.text_section.push("    pop r8  ; valor".to_string());
                        self.text_section.push("    pop rdx  ; índice".to_string());
                        self.text_section.push("    pop rcx  ; puntero al Array".to_string());
                        
                        // Llamar a array_insert
                        self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                        self.text_section.push("    call array_insert".to_string());
                        self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
                        
                        // array_insert no retorna valor (void), pero dejamos 0 en rax
                        self.text_section.push("    mov rax, 0  ; void return".to_string());
                    }
                    "remove" if args.len() == 1 => {
                        // arr.remove(x) -> array_remove(arr, x)
                        // Generar expresión del array (puntero al Array)
                        self.generate_expr_windows(object)?;
                        self.text_section.push("    push rax  ; guardar puntero al Array".to_string());
                        
                        // Generar expresión del valor a eliminar
                        self.generate_expr_windows(&args[0])?;
                        self.text_section.push("    push rax  ; guardar valor".to_string());
                        
                        // Preparar parámetros: RCX = puntero al Array, RDX = valor
                        self.text_section.push("    pop rdx  ; valor".to_string());
                        self.text_section.push("    pop rcx  ; puntero al Array".to_string());
                        
                        // Llamar a array_remove
                        self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                        self.text_section.push("    call array_remove".to_string());
                        self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
                        
                        // array_remove no retorna valor (void), pero dejamos 0 en rax
                        self.text_section.push("    mov rax, 0  ; void return".to_string());
                    }
                    "index" if args.len() == 1 => {
                        // arr.index(x) -> array_index(arr, x)
                        // Generar expresión del array (puntero al Array)
                        self.generate_expr_windows(object)?;
                        self.text_section.push("    push rax  ; guardar puntero al Array".to_string());
                        
                        // Generar expresión del valor a buscar
                        self.generate_expr_windows(&args[0])?;
                        self.text_section.push("    push rax  ; guardar valor".to_string());
                        
                        // Preparar parámetros: RCX = puntero al Array, RDX = valor
                        self.text_section.push("    pop rdx  ; valor".to_string());
                        self.text_section.push("    pop rcx  ; puntero al Array".to_string());
                        
                        // Llamar a array_index
                        self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                        self.text_section.push("    call array_index".to_string());
                        self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
                        
                        // array_index retorna el índice en RAX (ya está ahí)
                    }
                    "count" if args.len() == 1 => {
                        // arr.count(x) -> array_count(arr, x)
                        // Generar expresión del array (puntero al Array)
                        self.generate_expr_windows(object)?;
                        self.text_section.push("    push rax  ; guardar puntero al Array".to_string());
                        
                        // Generar expresión del valor a contar
                        self.generate_expr_windows(&args[0])?;
                        self.text_section.push("    push rax  ; guardar valor".to_string());
                        
                        // Preparar parámetros: RCX = puntero al Array, RDX = valor
                        self.text_section.push("    pop rdx  ; valor".to_string());
                        self.text_section.push("    pop rcx  ; puntero al Array".to_string());
                        
                        // Llamar a array_count
                        self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                        self.text_section.push("    call array_count".to_string());
                        self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
                        
                        // array_count retorna el conteo en RAX (ya está ahí)
                    }
                    "sort" if args.is_empty() => {
                        // arr.sort() -> array_sort(arr)
                        // Generar expresión del array (puntero al Array)
                        self.generate_expr_windows(object)?;
                        self.text_section.push("    push rax  ; guardar puntero al Array".to_string());
                        
                        // Preparar parámetros: RCX = puntero al Array
                        self.text_section.push("    pop rcx  ; puntero al Array".to_string());
                        
                        // Llamar a array_sort
                        self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                        self.text_section.push("    call array_sort".to_string());
                        self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
                        
                        // array_sort no retorna valor (void), pero dejamos 0 en rax
                        self.text_section.push("    mov rax, 0  ; void return".to_string());
                    }
                    "upper" if args.is_empty() && self.is_string_expr(object) => {
                        // s.upper() -> string_upper(s)
                        // Generar expresión del string (puntero al String)
                        self.generate_expr_windows(object)?;
                        self.text_section.push("    push rax  ; guardar puntero al String".to_string());
                        
                        // Preparar parámetros: RCX = puntero al String
                        self.text_section.push("    pop rcx  ; puntero al String".to_string());
                        
                        // Llamar a string_upper
                        self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                        self.text_section.push("    call string_upper".to_string());
                        self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
                        
                        // string_upper retorna puntero al nuevo String en RAX (ya está ahí)
                    }
                    "lower" if args.is_empty() && self.is_string_expr(object) => {
                        // s.lower() -> string_lower(s)
                        // Generar expresión del string (puntero al String)
                        self.generate_expr_windows(object)?;
                        self.text_section.push("    push rax  ; guardar puntero al String".to_string());
                        
                        // Preparar parámetros: RCX = puntero al String
                        self.text_section.push("    pop rcx  ; puntero al String".to_string());
                        
                        // Llamar a string_lower
                        self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                        self.text_section.push("    call string_lower".to_string());
                        self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
                        
                        // string_lower retorna puntero al nuevo String en RAX (ya está ahí)
                    }
                    _ => {
                        // Método de struct/clase: obj.metodo(args)
                        // Determinar tipo del objeto
                        let struct_type = self.get_struct_type_from_expr(object);
                        
                        if let Some(ref type_name) = struct_type {
                            // Es un método de struct/clase
                            // Evaluar objeto (puntero al struct en stack)
                            self.generate_expr_windows(object)?;
                            self.text_section.push("    push rax  ; guardar puntero al struct".to_string());
                            
                            // Evaluar argumentos
                            for (i, arg) in args.iter().enumerate() {
                                self.generate_expr_windows(arg)?;
                                match i {
                                    0 => {
                                        self.text_section.push("    mov rdx, rax  ; arg0".to_string());
                                    }
                                    1 => {
                                        self.text_section.push("    mov r8, rax  ; arg1".to_string());
                                    }
                                    2 => {
                                        self.text_section.push("    mov r9, rax  ; arg2".to_string());
                                    }
                                    _ => {
                                        self.text_section.push("    push rax  ; arg adicional en stack".to_string());
                                    }
                                }
                            }
                            
                            // Preparar self (RCX) desde el objeto guardado
                            self.text_section.push("    pop rcx  ; self (puntero al struct)".to_string());
                            
                            // Llamar a método: fn_StructName_method
                            let method_label = format!("fn_{}_{}", type_name, method);
                            self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                            self.text_section.push(format!("    call {}", method_label));
                            self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
                            
                            // RAX contiene el valor de retorno (si hay)
                        } else {
                            // No se pudo determinar el tipo del objeto
                            // Intentar obtener el nombre de la variable para dar un error más claro
                            let var_name = if let Expr::Ident(name) = object.as_ref() {
                                name.clone()
                            } else {
                                "objeto".to_string()
                            };
                            
                            return Err(adead_common::ADeadError::RuntimeError {
                                message: format!(
                                    "No se pudo determinar el tipo de '{}' para llamar al método '{}'. Asegúrate de que la variable esté correctamente tipada.",
                                    var_name, method
                                ),
                            });
                        }
                    }
                    _ => {
                        // Método de struct/clase: obj.metodo(args)
                        // Determinar tipo del objeto
                        let struct_type = self.get_struct_type_from_expr(object);
                        
                        if let Some(ref type_name) = struct_type {
                            // Es un método de struct/clase
                            // Evaluar objeto (puntero al struct en stack)
                            self.generate_expr_windows(object)?;
                            self.text_section.push("    push rax  ; guardar puntero al struct".to_string());
                            
                            // Evaluar argumentos
                            for (i, arg) in args.iter().enumerate() {
                                self.generate_expr_windows(arg)?;
                                match i {
                                    0 => {
                                        self.text_section.push("    mov rdx, rax  ; arg0".to_string());
                                    }
                                    1 => {
                                        self.text_section.push("    mov r8, rax  ; arg1".to_string());
                                    }
                                    2 => {
                                        self.text_section.push("    mov r9, rax  ; arg2".to_string());
                                    }
                                    _ => {
                                        self.text_section.push("    push rax  ; arg adicional en stack".to_string());
                                    }
                                }
                            }
                            
                            // Preparar self (RCX) desde el objeto guardado
                            self.text_section.push("    pop rcx  ; self (puntero al struct)".to_string());
                            
                            // Verificar si el método está en la vtable (dispatch dinámico)
                            let use_virtual_dispatch = if let Some(methods) = self.struct_methods.get(type_name) {
                                methods.contains(method)
                            } else {
                                false
                            };
                            
                            if use_virtual_dispatch {
                                // DISPATCH DINÁMICO: Usar vtable para llamada virtual
                                // 1. Cargar puntero a vtable desde [self + 0]
                                self.text_section.push("    mov rax, [rcx]  ; cargar vtable_ptr desde [self + 0]".to_string());
                                
                                // 2. Calcular offset del método en la vtable
                                if let Some(methods) = self.struct_methods.get(type_name) {
                                    if let Some(method_index) = methods.iter().position(|m| m == method) {
                                        let vtable_offset = method_index * 8; // Cada puntero es 8 bytes
                                        
                                        // 3. Cargar puntero al método desde [vtable + offset]
                                        self.text_section.push(format!("    mov rax, [rax + {}]  ; cargar puntero al método desde vtable[{}]", vtable_offset, method_index));
                                        
                                        // 4. Llamar indirectamente usando el puntero
                                        self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                                        self.text_section.push("    call rax  ; llamada virtual (dispatch dinámico)".to_string());
                                        self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
                                    } else {
                                        return Err(adead_common::ADeadError::RuntimeError {
                                            message: format!("Método '{}' no encontrado en vtable de '{}'.", method, type_name),
                                        });
                                    }
                                } else {
                                    return Err(adead_common::ADeadError::RuntimeError {
                                        message: format!("No se pudo encontrar vtable para '{}'.", type_name),
                                    });
                                }
                            } else {
                                // LLAMADA ESTÁTICA: Método no virtual, llamar directamente
                            let method_label = format!("fn_{}_{}", type_name, method);
                                self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                                self.text_section.push(format!("    call {}  ; llamada estática", method_label));
                                self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
                            }
                            
                            // RAX contiene el valor de retorno (si hay)
                        } else {
                            // No se pudo determinar el tipo del objeto
                            // Intentar obtener el nombre de la variable para dar un error más claro
                            let var_name = if let Expr::Ident(name) = object.as_ref() {
                                name.clone()
                            } else {
                                "objeto".to_string()
                            };
                            
                            return Err(adead_common::ADeadError::RuntimeError {
                                message: format!(
                                    "No se pudo determinar el tipo de '{}' para llamar al método '{}'. Asegúrate de que la variable esté correctamente tipada.",
                                    var_name, method
                                ),
                            });
                        }
                    }
                }
            }
            Expr::SuperCall { method, args } => {
                // super.metodo(args) - llamada a método del padre
                // Obtener struct padre desde current_struct
                if let Some(current_struct_name) = &self.current_struct {
                    if let Some(parent_name) = self.struct_parents.get(current_struct_name) {
                        if let Some(parent) = parent_name.clone() {
                            // Evaluar argumentos
                            for (i, arg) in args.iter().enumerate() {
                                self.generate_expr_windows(arg)?;
                                match i {
                                    0 => {
                                        self.text_section.push("    mov rdx, rax  ; arg0".to_string());
                                    }
                                    1 => {
                                        self.text_section.push("    mov r8, rax  ; arg1".to_string());
                                    }
                                    2 => {
                                        self.text_section.push("    mov r9, rax  ; arg2".to_string());
                                    }
                                    _ => {
                                        self.text_section.push("    push rax  ; arg adicional en stack".to_string());
                                    }
                                }
                            }
                            
                            // self ya está en el contexto (debe estar en RCX o en stack)
                            // Si self está en variables, cargarlo; si no, asumir que está en RCX
                            if let Some(&self_offset) = self.variables.get("self") {
                                self.text_section.push(format!("    mov rcx, [rbp - {}]  ; self desde stack", self_offset + 8));
                            }
                            // Si no está en variables, asumir que ya está en RCX (contexto del método)
                            
                            // Llamar a método del padre: fn_ParentName_method
                            let method_label = format!("fn_{}_{}", parent, method);
                            self.text_section.push(format!("    ; Llamada a super.{}(), método del padre {}", method, parent));
                            self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                            self.text_section.push(format!("    call {}", method_label));
                            self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
                            
                            // RAX contiene el valor de retorno (si hay)
                        } else {
                            return Err(adead_common::ADeadError::RuntimeError {
                                message: format!("'{}' no tiene struct padre. 'super.metodo()' solo puede usarse dentro de structs con herencia.", current_struct_name),
                            });
                        }
                    } else {
                        return Err(adead_common::ADeadError::RuntimeError {
                            message: format!("No se pudo encontrar información del struct '{}' para 'super.metodo()'.", current_struct_name),
                        });
                    }
                } else {
                    return Err(adead_common::ADeadError::RuntimeError {
                        message: "'super.metodo()' solo puede usarse dentro de métodos de structs.".to_string(),
                    });
                }
            }
            Expr::Match { expr, arms } => {
                // Generar match exhaustivo con saltos condicionales
                // Evalúa expr (debe ser Option o Result, resultado es dirección del tagged union)
                self.generate_expr_windows(expr)?;
                // rax contiene la dirección del tagged union
                // Cargar tag
                self.text_section.push("    mov rbx, [rax]  ; cargar tag del tagged union".to_string());
                
                // Generar labels para cada brazo
                let mut arm_labels = Vec::new();
                let end_label = self.new_label("match_end");
                
                // Generar código para cada brazo
                for (i, arm) in arms.iter().enumerate() {
                    let arm_label = self.new_label(&format!("match_arm_{}", i));
                    arm_labels.push(arm_label.clone());
                    
                    // Comparar tag con el patrón
                    match &arm.pattern {
                        Pattern::Some => {
                            // Tag debe ser 1 para Some
                                self.text_section.push(format!("    cmp rbx, 1  ; comparar tag con Some"));
                                self.text_section.push(format!("    je {}", arm_label));
                        }
                        Pattern::None => {
                            // Tag debe ser 0 para None
                                self.text_section.push(format!("    cmp rbx, 0  ; comparar tag con None"));
                                self.text_section.push(format!("    je {}", arm_label));
                        }
                        Pattern::Ok => {
                            // Tag debe ser 0 para Ok
                                self.text_section.push(format!("    cmp rbx, 0  ; comparar tag con Ok"));
                                self.text_section.push(format!("    je {}", arm_label));
                        }
                        Pattern::Err => {
                            // Tag debe ser 1 para Err
                                self.text_section.push(format!("    cmp rbx, 1  ; comparar tag con Err"));
                                self.text_section.push(format!("    je {}", arm_label));
                        }
                        Pattern::Wildcard => {
                            // Wildcard siempre coincide (salto incondicional al final)
                                self.text_section.push(format!("    jmp {}", arm_label));
                        }
                        _ => {
                            // Otros patrones (literales, ident) no son compatibles con Option/Result
                            // Por ahora, tratar como wildcard
                                self.text_section.push(format!("    jmp {}", arm_label));
                        }
                    }
                }
                
                // Si no coincide ningún patrón, saltar al final (o panic si no hay wildcard)
                self.text_section.push(format!("    jmp {}", end_label));
                
                // Generar código de cada brazo
                for (i, arm) in arms.iter().enumerate() {
                    self.text_section.push(format!("{}:", arm_labels[i]));
                    
                    // Si el patrón tiene binding (Some, Ok, Err), cargar el valor
                    match &arm.pattern {
                        Pattern::Some | Pattern::Ok | Pattern::Err => {
                            // Cargar valor desde [rax + 8]
                            self.text_section.push("    mov rax, [rax + 8]  ; cargar valor del tagged union".to_string());
                        }
                        _ => {
                            // No hay binding, mantener rax como está
                        }
                    }
                    
                    // Generar código del cuerpo del brazo
                    self.generate_expr_windows(&arm.body)?;
                    // Saltar al final
                    self.text_section.push(format!("    jmp {}", end_label));
                }
                
                // Label de fin
                self.text_section.push(format!("{}:", end_label));
            }
            Expr::PropagateError(expr) => {
                // Operador ?: propaga error automáticamente
                // expr? desenvuelve Result: si es Err, retorna temprano con el error
                // Si es Ok(valor), desarrolla el valor y continúa
                
                // Evaluar la expresión (debe ser Result<T, E>)
                self.generate_expr_windows(expr)?;
                // rax contiene la dirección del tagged union Result
                
                // Guardar dirección en rbx para poder acceder después
                self.text_section.push("    mov rbx, rax  ; guardar dirección del Result".to_string());
                
                // Cargar tag
                self.text_section.push("    mov rax, [rbx]  ; cargar tag (0=Ok, 1=Err)".to_string());
                
                // Crear labels
                let ok_label = self.new_label("propagate_ok");
                let error_label = self.new_label("propagate_error");
                
                // Verificar si es Ok (tag == 0)
                self.text_section.push("    cmp rax, 0  ; comparar tag con 0 (Ok)".to_string());
                self.text_section.push(format!("    je {}  ; si es Ok, desenvolver valor", ok_label));
                
                // Si llegamos aquí, es Err (tag == 1) -> propagar error
                self.text_section.push(format!("    jmp {}  ; si es Err, propagar", error_label));
                
                // Label para Ok: desenvolver valor
                self.text_section.push(format!("{}:", ok_label));
                self.text_section.push("    mov rax, [rbx + 8]  ; cargar valor de Ok desde Result".to_string());
                
                // Label para error: dejar error en rax (dirección del error)
                // El error está en [rbx + 8]
                self.text_section.push(format!("{}:", error_label));
                self.text_section.push("    mov rax, [rbx + 8]  ; cargar error de Err".to_string());
                // TODO: En una función con tipo de retorno Result, deberíamos:
                // 1. Construir un nuevo Result con el error
                // 2. Retornar temprano con "ret"
                // Por ahora, el error queda en rax para que el caller lo maneje
                self.text_section.push("    ; Nota: En función Result, debería retornar temprano".to_string());
            }
        }
        Ok(())
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Print(expr) => {
                self.text_section.push("    ; print".to_string());
                match expr {
                    Expr::String(s) => {
                        let label = self.add_string_data(s);
                            self.text_section.push(format!("    mov rax, 1  ; sys_write",));
                            self.text_section.push(format!("    mov rdi, 1  ; stdout",));
                            self.text_section.push(format!("    lea rsi, [rel {}]", label));
                            self.text_section.push(format!("    mov rdx, {}_len", label));
                            self.text_section.push(format!("    syscall",));
                    }
                    Expr::Number(n) => {
                        // Print número: convertir a string en tiempo de compilación
                        let num_str = format!("{}", n);
                        let label = self.add_string_data(&num_str);
                            self.text_section.push(format!("    mov rax, 1  ; sys_write"));
                            self.text_section.push(format!("    mov rdi, 1  ; stdout"));
                            self.text_section.push(format!("    lea rsi, [rel {}]", label));
                            self.text_section.push(format!("    mov rdx, {}_len", label));
                            self.text_section.push(format!("    syscall"));
                    }
                    Expr::Float(f) => {
                        // Print float: convertir a string en tiempo de compilación
                        let float_str = format!("{}", f);
                        let label = self.add_string_data(&float_str);
                            self.text_section.push(format!("    mov rax, 1  ; sys_write"));
                            self.text_section.push(format!("    mov rdi, 1  ; stdout"));
                            self.text_section.push(format!("    lea rsi, [rel {}]", label));
                            self.text_section.push(format!("    mov rdx, {}_len", label));
                            self.text_section.push(format!("    syscall"));
                    }
                    _ => {
                        // Evaluar expresión numérica (ej: 2 + 5) y convertir a string
                        // Para Linux, usar generate_expr (método genérico)
                        self.generate_expr(expr)?;
                        // RAX ahora contiene el resultado numérico
                        // TODO: Implementar conversión runtime completa para Linux
                        // Por ahora, solo soportamos literales y variables
                        return Err(adead_common::ADeadError::RuntimeError {
                            message: "Complex expressions in print not yet fully supported on Linux. Use Windows target or assign to variable first: let x = expr; print x".to_string(),
                        });
                    }
                }
            }
            Stmt::Let { mutable, name, value } => {
                // Si el valor es un StructLiteral, registrar para destrucción RAII si tiene destroy
                // y también registrar el tipo de la variable
                let struct_name = if let Expr::StructLiteral { name: struct_name, .. } = value {
                    if self.structs_with_destroy.contains_key(struct_name) {
                        self.variables_to_destroy.push((name.clone(), struct_name.clone()));
                    }
                    // Registrar el tipo de la variable para acceso a campos
                    self.variable_types.insert(name.clone(), struct_name.clone());
                    Some(struct_name.clone())
                } else {
                    None
                };
                
                self.generate_expr(value)?;
                
                // Si es un struct con constructor, llamarlo aquí
                // Por ahora, solo guardamos la dirección del struct
                
                // Store in stack (simplified: just track as variable)
                // Note: mutability is tracked but NASM code is the same (difference is in borrow checker)
                let offset = if let Some(&existing_offset) = self.variables.get(name) {
                    existing_offset
                } else {
                    let offset = self.stack_offset;
                    self.stack_offset += 8;
                    self.variables.insert(name.clone(), offset);
                    offset
                };
                self.text_section
                    .push(format!("    mov [rbp - {}], rax  ; variable {} ({})", 
                        offset + 8, name, if *mutable { "mutable" } else { "immutable" }));
            }
            Stmt::MultiLet { mutable, names, values } => {
                // Múltiple asignación Python-style (Linux)
                for (i, name) in names.iter().enumerate() {
                    let value = if i < values.len() { &values[i] } else { &values[values.len() - 1] };
                    self.generate_expr(value)?;
                    let offset = if let Some(&existing_offset) = self.variables.get(name) {
                        existing_offset
                    } else {
                        let offset = self.stack_offset;
                        self.stack_offset += 8;
                        self.variables.insert(name.clone(), offset);
                        offset
                    };
                    self.text_section
                        .push(format!("    mov [rbp - {}], rax  ; variable {} ({})", 
                            offset + 8, name, if *mutable { "mutable" } else { "immutable" }));
                }
            }
            Stmt::Import(_module_name) => {
                // Import statements son procesados en tiempo de compilación
                // No generan código directamente, solo información de módulos
                // TODO: Implementar resolución de módulos
                self.text_section.push(format!("    ; import {} (pendiente resolución)", _module_name));
            }
            Stmt::Expr(expr) => {
                self.generate_expr(expr)?;
            }
            Stmt::If {
                condition,
                then_body,
                else_body,
            } => {
                self.generate_expr(condition)?;
                let else_label = self.new_label("else");
                let end_label = self.new_label("endif");
                
                // Check condition: if 0, jump to else/end
                self.text_section.push("    cmp rax, 0".to_string());
                if else_body.is_some() {
                    self.text_section.push(format!("    je {}", else_label));
                } else {
                    self.text_section.push(format!("    je {}", end_label));
                }

                // Then body
                for s in then_body {
                    self.generate_stmt(s)?;
                }
                self.text_section.push(format!("    jmp {}", end_label));
                
                // Else body
                if let Some(else_body) = else_body {
                    self.text_section.push(format!("{}:", else_label));
                    for s in else_body {
                        self.generate_stmt(s)?;
                    }
                }
                self.text_section.push(format!("{}:", end_label));
            }
            Stmt::While { condition, body } => {
                let loop_start = self.new_label("while_start");
                let loop_end = self.new_label("while_end");
                let loop_continue = loop_start.clone();
                
                self.loop_stack.push(LoopContext {
                    break_label: loop_end.clone(),
                    continue_label: loop_continue,
                });
                
                    self.text_section.push(format!("{}:", loop_start));
                self.generate_expr(condition)?;
                self.text_section.push("    cmp rax, 0".to_string());
                self.text_section.push(format!("    je {}", loop_end));
                
                for s in body {
                    self.generate_stmt(s)?;
                }
                self.text_section.push(format!("    jmp {}", loop_start));
                self.text_section.push(format!("{}:", loop_end));
                
                self.loop_stack.pop();
            }
            Stmt::For { var, start, end, body } => {
                let loop_start = self.new_label("for_start");
                let loop_end = self.new_label("for_end");
                let loop_continue = self.new_label("for_continue");
                
                self.loop_stack.push(LoopContext {
                    break_label: loop_end.clone(),
                    continue_label: loop_continue.clone(),
                });
                
                // Evaluar start y guardar en variable
                self.generate_expr(start)?;
                let var_offset = self.stack_offset;
                self.stack_offset += 8;
                self.variables.insert(var.clone(), var_offset);
                self.text_section.push(format!("    mov [rbp - {}], rax  ; {} (loop counter)", var_offset + 8, var));
                
                // Evaluar end y guardar en r13
                self.generate_expr(end)?;
                self.text_section.push("    mov r13, rax  ; end value".to_string());
                
                // Loop start
                self.text_section.push(format!("{}:", loop_start));
                self.text_section.push(format!("    mov rax, [rbp - {}]", var_offset + 8));
                self.text_section.push("    cmp rax, r13".to_string());
                self.text_section.push(format!("    jge {}", loop_end));
                
                for s in body {
                    self.generate_stmt(s)?;
                }
                
                self.text_section.push(format!("{}:", loop_continue));
                self.text_section.push(format!("    mov rax, [rbp - {}]", var_offset + 8));
                self.text_section.push("    inc rax".to_string());
                self.text_section.push(format!("    mov [rbp - {}], rax", var_offset + 8));
                self.text_section.push(format!("    jmp {}", loop_start));
                self.text_section.push(format!("{}:", loop_end));
                
                self.loop_stack.pop();
            }
            Stmt::Break => {
                if let Some(ctx) = self.loop_stack.last() {
                    let label = ctx.break_label.clone();
                    self.text_section.push(format!("    jmp {}", label));
                }
            }
            Stmt::Continue => {
                if let Some(ctx) = self.loop_stack.last() {
                    let label = ctx.continue_label.clone();
                    self.text_section.push(format!("    jmp {}", label));
                }
            }
            Stmt::Fn { visibility: _, name, params, body } => {
                // Generate function
                // Visibility no afecta la generación de código (Sprint 1.3)
                let func_label = format!("fn_{}", name);
                self.text_section.push(format!("    jmp {}_end", func_label));
                self.text_section.push(format!("{}:", func_label));
                self.text_section.push("    push rbp".to_string());
                self.text_section.push("    mov rbp, rsp".to_string());
                
                // Store params in local variables
                for (i, param) in params.iter().enumerate() {
                    // System V ABI: rdi, rsi, rdx, rcx, r8, r9
                    let reg = match i {
                        0 => "rdi",
                        1 => "rsi",
                        2 => "rdx",
                        3 => "rcx",
                        4 => "r8",
                        5 => "r9",
                        _ => break, // More params would be on stack
                    };
                    let offset = self.stack_offset;
                    self.stack_offset += 8;
                    self.variables.insert(param.name.clone(), offset);
                    self.text_section
                        .push(format!("    mov [rbp - {}], {}", offset + 8, reg));
                }
                
                for s in body {
                    self.generate_stmt(s)?;
                }
                
                // Return (if no explicit return, return 0)
                self.text_section.push("    mov rax, 0".to_string());
                self.text_section.push("    pop rbp".to_string());
                self.text_section.push("    ret".to_string());
                self.text_section.push(format!("{}_end:", func_label));
            }
            Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    self.generate_expr(expr)?;
                } else {
                    self.text_section.push("    mov rax, 0".to_string());
                }
                self.text_section.push("    pop rbp".to_string());
                self.text_section.push("    ret".to_string());
            }
            Stmt::Struct { name, parent: _, fields, init, destroy: _, methods: _ } => {
                // Registrar campos del struct para cálculo de offsets
                let field_names: Vec<String> = fields.iter().map(|f| f.name.clone()).collect();
                self.struct_definitions.insert(name.clone(), field_names);
                
                // Registrar struct y generar código para constructor si existe
                if let Some(init_method) = init {
                    // Generar función de constructor: StructName_init
                    let init_label = format!("{}_init", name);
                    self.text_section.push(format!("    jmp {}_end", init_label));
                    self.text_section.push(format!("{}:", init_label));
                    self.text_section.push("    push rbp".to_string());
                    self.text_section.push("    mov rbp, rsp".to_string());
                    
                    // Guardar parámetros en stack (System V AMD64 calling convention)
                    for (i, param) in init_method.params.iter().enumerate() {
                        let offset = self.stack_offset;
                        self.stack_offset += 8;
                        self.variables.insert(param.name.clone(), offset);
                        
                        let reg = match i {
                            0 => "rdi",
                            1 => "rsi",
                            2 => "rdx",
                            3 => "rcx",
                            4 => "r8",
                            5 => "r9",
                            _ => {
                                let stack_offset = 16 + (i - 6) * 8;
                                    self.text_section.push(format!("    mov rax, [rbp + {}]", stack_offset));
                                    self.text_section.push(format!("    mov [rbp - {}], rax", offset + 8));
                                continue;
                            }
                        };
                            self.text_section.push(format!("    mov [rbp - {}], {}", offset + 8, reg));
                    }
                    
                    // Generar cuerpo del constructor
                    for s in &init_method.body {
                        self.generate_stmt(s)?;
                    }
                    
                    self.text_section.push("    leave".to_string());
                    self.text_section.push("    ret".to_string());
                    self.text_section.push(format!("{}_end:", init_label));
                }
                // Struct definitions are type information only, no code generation needed for the struct itself
            }
        }
        Ok(())
    }

    fn generate_expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Number(n) => {
                self.text_section.push(format!("    mov rax, {}", n));
            }
            Expr::Bool(b) => {
                // Bool: 0 = false, 1 = true en RAX
                let value = if *b { 1 } else { 0 };
                self.text_section.push(format!("    mov rax, {}  ; bool {}", value, b));
            }
            Expr::Float(f) => {
                // Cargar constante flotante en XMM0 (Linux)
                let label = self.add_float_data(*f);
                self.text_section.push(format!("    movsd xmm0, [rel {}]  ; cargar float {}", label, f));
            }
            Expr::String(_) => {
                // Strings handled separately in print
                return Err(adead_common::ADeadError::RuntimeError {
                    message: "cannot use string in expression yet".to_string(),
                });
            }
            Expr::ArrayLiteral(elements) => {
                // Array literal: [1, 2, 3] (Linux)
                // Similar a Windows pero con convenciones de System V
                let array_size = elements.len() * 8; // 8 bytes por elemento (int64)
                let base_offset = self.stack_offset;
                
                self.stack_offset += array_size as i64;
                self.text_section.push(format!("    ; Array literal: {} elementos ({} bytes)", elements.len(), array_size));
                self.text_section.push(format!("    sub rsp, {}  ; reservar espacio para array", array_size));
                
                for (i, element) in elements.iter().enumerate() {
                    self.generate_expr(element)?;
                    let element_offset = base_offset + (i as i64 * 8);
                    self.text_section.push(format!("    mov [rbp - {}], rax  ; array[{}]", element_offset + 8, i));
                }
                
                    self.text_section.push(format!("    lea rax, [rbp - {}]  ; dirección base del array", base_offset + 8));
            }
            Expr::Borrow { expr, .. } => {
                // Borrowing: generar dirección de la expresión
                // Por ahora, solo soportamos borrowing de variables
                self.generate_expr(expr)?;
                // TODO: Generar código para obtener dirección (lea)
            }
            Expr::Deref(expr) => {
                // Dereferenciar: cargar valor desde la dirección
                self.generate_expr(expr)?;
                // Si el valor está en rax (dirección), cargar desde esa dirección
                self.text_section.push("    mov rax, [rax]".to_string());
            }
            // Option/Result constructors (O0.4) - TODO: Implementar generación de código
            Expr::Some(expr) | Expr::Ok(expr) => {
                // Por ahora, simplemente evaluar la expresión interna
                // TODO: Implementar tagged union en NASM
                self.generate_expr(expr)?;
            }
            Expr::Err(expr) => {
                // Por ahora, simplemente evaluar la expresión interna
                // TODO: Implementar tagged union en NASM
                self.generate_expr(expr)?;
            }
            Expr::None => {
                // None/Err sin valor: poner 0 en rax como marcador
                // TODO: Implementar tag apropiado en NASM
                self.text_section.push("    mov rax, 0".to_string());
            }
            // Structs (Fase 1.2 - O1, O3, O4)
            Expr::StructLiteral { name, fields } => {
                // Registrar los campos del struct para poder calcular offsets después
                let field_names: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();
                if !self.struct_definitions.contains_key(name) {
                    self.struct_definitions.insert(name.clone(), field_names);
                }
                
                // Generar struct literal en stack
                let struct_size = fields.len() * 8;
                let offset = self.stack_offset;
                self.stack_offset += struct_size as i64;
                self.text_section.push(format!("    sub rsp, {}  ; espacio para struct {} ({} campos)", struct_size, name, fields.len()));
                
                // Generar valores de campos
                for (i, (field_name, value)) in fields.iter().enumerate() {
                    self.generate_expr(value)?;
                    let field_offset = offset + (i as i64 * 8) + 8;
                    self.text_section.push(format!("    mov [rbp - {}], rax  ; {}.{}", field_offset, name, field_name));
                }
                
                self.text_section.push(format!("    lea rax, [rbp - {}]  ; dirección del struct {}", offset + 8, name));
            }
            Expr::FieldAccess { object, field } => {
                // Determinar el tipo del objeto para calcular el offset correcto
                let struct_type = self.get_struct_type_from_expr(object);
                
                self.generate_expr(object)?;
                
                // Calcular offset del campo (negativo porque stack crece hacia abajo)
                let field_offset = if let Some(ref type_name) = struct_type {
                    if let Some(fields) = self.struct_definitions.get(type_name) {
                        fields.iter().position(|f| f == field).unwrap_or(0) * 8
                    } else {
                        0
                    }
                } else {
                    0
                };
                
                self.text_section.push(format!("    ; accediendo campo '{}' (offset: -{})", field, field_offset));
                if field_offset == 0 {
                    self.text_section.push("    mov rax, [rax]  ; cargar campo (offset 0)".to_string());
                } else {
                    self.text_section.push(format!("    mov rax, [rax - {}]  ; cargar campo", field_offset));
                }
            }
            Expr::FieldAssign { object, field, value } => {
                // Asignación a campo de struct: obj.field = value (Linux)
                let struct_type = self.get_struct_type_from_expr(object);
                
                // Primero generar el valor a asignar
                self.generate_expr(value)?;
                self.text_section.push("    push rax  ; guardar valor a asignar".to_string());
                
                // Luego obtener la dirección del objeto
                self.generate_expr(object)?;
                
                // Calcular offset del campo
                let field_offset = if let Some(ref type_name) = struct_type {
                    if let Some(fields) = self.struct_definitions.get(type_name) {
                        fields.iter().position(|f| f == field).unwrap_or(0) * 8
                    } else {
                        0
                    }
                } else {
                    0
                };
                
                self.text_section.push("    pop rbx  ; restaurar valor a asignar".to_string());
                self.text_section.push(format!("    ; asignando a campo '{}' (offset: -{})", field, field_offset));
                if field_offset == 0 {
                    self.text_section.push("    mov [rax], rbx  ; asignar campo (offset 0)".to_string());
                } else {
                    self.text_section.push(format!("    mov [rax - {}], rbx  ; asignar campo", field_offset));
                }
            }
            Expr::Index { array, index } => {
                // Indexación: arr[0] (Linux)
                self.generate_expr(array)?;
                self.text_section.push("    push rax  ; guardar dirección base del array".to_string());
                
                self.generate_expr(index)?;
                self.text_section.push("    mov rcx, rax  ; rcx = índice".to_string());
                self.text_section.push("    mov rax, 8".to_string());
                self.text_section.push("    imul rax, rcx  ; rax = índice * 8 (offset en bytes)".to_string());
                
                self.text_section.push("    pop rbx  ; restaurar dirección base".to_string());
                self.text_section.push("    add rax, rbx  ; rax = dirección base + offset".to_string());
                
                self.text_section.push("    mov rax, [rax]  ; cargar array[index]".to_string());
            }
            Expr::Slice { object: _, start: _, end: _ } => {
                // Slicing: s[0:4] (Linux)
                // Por ahora, retornar error ya que strings avanzados están implementados principalmente para Windows
                return Err(adead_common::ADeadError::RuntimeError {
                    message: "String slicing not yet implemented for Linux. Use Windows backend.".to_string(),
                });
            }
            Expr::MethodCall { object, method, args } => {
                // Llamada a método (simplificado)
                self.generate_expr(object)?;
                self.text_section.push("    push rax  ; self".to_string());
                for arg in args {
                    self.generate_expr(arg)?;
                    self.text_section.push("    push rax".to_string());
                }
                self.text_section.push(format!("    call fn_{}", method));
                let cleanup = (args.len() + 1) * 8;
                if cleanup > 0 {
                    self.text_section.push(format!("    add rsp, {}", cleanup));
                }
            }
            Expr::Match { expr, arms } => {
                // Generar match expression
                // TODO: Implementar match exhaustivo con saltos condicionales
                // Por ahora, generar el código de la expresión base
                self.generate_expr(expr)?;
                // TODO: Implementar lógica de matching real
                if let Some(first_arm) = arms.first() {
                    self.generate_expr(&first_arm.body)?;
                }
            }
            Expr::SuperCall { method, args } => {
                // super.metodo(args) - llamada a método del padre (Linux)
                // Similar a Windows pero con convenciones System V
                if let Some(current_struct_name) = &self.current_struct {
                    if let Some(parent_name) = self.struct_parents.get(current_struct_name) {
                        if let Some(parent) = parent_name.clone() {
                            // Evaluar argumentos (System V: rdi, rsi, rdx, rcx, r8, r9)
                            for (i, arg) in args.iter().enumerate() {
                                self.generate_expr(arg)?;
                                let reg = match i {
                                    0 => "rdi",
                                    1 => "rsi",
                                    2 => "rdx",
                                    3 => "rcx",
                                    4 => "r8",
                                    5 => "r9",
                                    _ => {
                                        self.text_section.push("    push rax  ; arg en stack".to_string());
                                        continue;
                                    }
                                };
                                self.text_section.push(format!("    mov {}, rax  ; arg{}", reg, i));
                            }
                            
                            // self debe estar en rdi (primer parámetro en System V)
                            if let Some(&self_offset) = self.variables.get("self") {
                                self.text_section.push(format!("    mov rdi, [rbp - {}]  ; self desde stack", self_offset + 8));
                            }
                            
                            // Llamar a método del padre
                            let method_label = format!("fn_{}_{}", parent, method);
                            self.text_section.push(format!("    ; Llamada a super.{}(), método del padre {}", method, parent));
                            self.text_section.push(format!("    call {}", method_label));
                            
                            // RAX contiene el valor de retorno (si hay)
                        } else {
                            return Err(adead_common::ADeadError::RuntimeError {
                                message: format!("'{}' no tiene struct padre. 'super.metodo()' solo puede usarse dentro de structs con herencia.", current_struct_name),
                            });
                        }
                    } else {
                        return Err(adead_common::ADeadError::RuntimeError {
                            message: format!("No se pudo encontrar información del struct '{}' para 'super.metodo()'.", current_struct_name),
                        });
                    }
                } else {
                    return Err(adead_common::ADeadError::RuntimeError {
                        message: "'super.metodo()' solo puede usarse dentro de métodos de structs.".to_string(),
                    });
                }
            }
            Expr::PropagateError(expr) => {
                // Operador ?: propaga error automáticamente (versión Linux)
                // Misma lógica que Windows pero con syscalls de Linux si es necesario
                // Evaluar la expresión (debe ser Result<T, E>)
                self.generate_expr(expr)?;
                // rax contiene la dirección del tagged union Result
                
                // Guardar dirección en rbx para poder acceder después
                self.text_section.push("    mov rbx, rax  ; guardar dirección del Result".to_string());
                
                // Cargar tag
                self.text_section.push("    mov rax, [rbx]  ; cargar tag (0=Ok, 1=Err)".to_string());
                
                // Crear labels
                let ok_label = self.new_label("propagate_ok");
                let error_label = self.new_label("propagate_error");
                
                // Verificar si es Ok (tag == 0)
                self.text_section.push("    cmp rax, 0  ; comparar tag con 0 (Ok)".to_string());
                self.text_section.push(format!("    je {}  ; si es Ok, desenvolver valor", ok_label));
                
                // Si llegamos aquí, es Err (tag == 1) -> propagar error
                self.text_section.push(format!("    jmp {}  ; si es Err, propagar", error_label));
                
                // Label para Ok: desenvolver valor
                self.text_section.push(format!("{}:", ok_label));
                self.text_section.push("    mov rax, [rbx + 8]  ; cargar valor de Ok desde Result".to_string());
                
                // Label para error: dejar error en rax (dirección del error)
                self.text_section.push(format!("{}:", error_label));
                self.text_section.push("    mov rax, [rbx + 8]  ; cargar error de Err".to_string());
                // TODO: En una función con tipo de retorno Result, deberíamos:
                // 1. Construir un nuevo Result con el error
                // 2. Retornar temprano con "ret"
                // Por ahora, el error queda en rax para que el caller lo maneje
                self.text_section.push("    ; Nota: En función Result, debería retornar temprano".to_string());
            }
            Expr::Ident(name) => {
                if let Some(&offset) = self.variables.get(name) {
                    self.text_section
                        .push(format!("    mov rax, [rbp - {}]", offset + 8));
                } else {
                    return Err(adead_common::ADeadError::RuntimeError {
                        message: format!("undefined variable: {}", name),
                    });
                }
            }
            Expr::Assign { name, value } => {
                // Generate the value first
                self.generate_expr(value)?;
                // Store in variable location
                if let Some(&offset) = self.variables.get(name) {
                    self.text_section
                        .push(format!("    mov [rbp - {}], rax", offset + 8));
                } else {
                    // Create new variable if it doesn't exist
                    let offset = self.stack_offset;
                    self.stack_offset += 8;
                    self.variables.insert(name.clone(), offset);
                    self.text_section
                        .push(format!("    mov [rbp - {}], rax", offset + 8));
                }
                // Result of assignment is the value
            }
            // Operadores compuestos: x += 5, x -= 3, x *= 2, x /= 4 (Sprint 1 - Python-like) - Linux
            Expr::CompoundAssign { name, op, value } => {
                // Cargar valor actual de la variable
                if let Some(&offset) = self.variables.get(name) {
                    self.text_section.push(format!("    mov rbx, [rbp - {}]  ; cargar {} actual", offset + 8, name));
                } else {
                    return Err(adead_common::ADeadError::RuntimeError {
                        message: format!("Variable '{}' no definida para operador compuesto", name),
                    });
                }
                
                // Evaluar la expresión del valor
                self.generate_expr(value)?;
                // RAX contiene el valor, RBX contiene el valor actual de la variable
                
                // Aplicar operación
                match op {
                    BinOp::Add => {
                        self.text_section.push("    add rbx, rax  ; += operación".to_string());
                    }
                    BinOp::Sub => {
                        self.text_section.push("    sub rbx, rax  ; -= operación".to_string());
                    }
                    BinOp::Mul => {
                        self.text_section.push("    imul rbx, rax  ; *= operación".to_string());
                    }
                    BinOp::Div => {
                        self.text_section.push("    push rax  ; guardar divisor".to_string());
                        self.text_section.push("    mov rax, rbx  ; dividendo en rax".to_string());
                        self.text_section.push("    cqo  ; sign-extend".to_string());
                        self.text_section.push("    pop rbx  ; divisor en rbx".to_string());
                        self.text_section.push("    idiv rbx  ; /= operación".to_string());
                        self.text_section.push("    mov rbx, rax  ; resultado en rbx".to_string());
                    }
                    _ => {
                        return Err(adead_common::ADeadError::RuntimeError {
                            message: format!("Operador compuesto no soportado: {:?}", op),
                        });
                    }
                }
                
                // Guardar resultado en la variable
                if let Some(&offset) = self.variables.get(name) {
                    self.text_section.push(format!("    mov [rbp - {}], rbx  ; guardar resultado en {}", offset + 8, name));
                    self.text_section.push("    mov rax, rbx  ; resultado en rax".to_string());
                }
            }
            // F-strings: f"Hola {nombre}" (Sprint 2.3 - Python-like) - Linux
            Expr::FString { parts } => {
                use adead_parser::FStringPart;
                // Implementación Linux similar a Windows
                let buffer_size = 256;
                let buffer_offset = self.stack_offset;
                self.stack_offset += buffer_size;
                
                self.text_section.push(format!("    ; F-string con {} partes (Linux)", parts.len()));
                self.text_section.push(format!("    sub rsp, {}", buffer_size));
                self.text_section.push(format!("    lea rdi, [rbp - {}]", buffer_offset + 8));
                self.text_section.push("    xor r12, r12".to_string());
                
                for part in parts {
                    match part {
                        FStringPart::Literal(s) => {
                            let label = self.add_string_data(s);
                            self.text_section.push(format!("    lea rsi, [rel {}]", label));
                            self.text_section.push(format!("    mov rcx, {}_len", label));
                            self.text_section.push("    push rdi".to_string());
                            self.text_section.push("    add rdi, r12".to_string());
                            let copy_loop = self.new_label("fstr_copy_lnx");
                            let copy_done = self.new_label("fstr_copy_done_lnx");
                            self.text_section.push(format!("{}:", copy_loop));
                            self.text_section.push("    cmp rcx, 0".to_string());
                            self.text_section.push(format!("    je {}", copy_done));
                            self.text_section.push("    mov al, [rsi]".to_string());
                            self.text_section.push("    mov [rdi], al".to_string());
                            self.text_section.push("    inc rsi".to_string());
                            self.text_section.push("    inc rdi".to_string());
                            self.text_section.push("    inc r12".to_string());
                            self.text_section.push("    dec rcx".to_string());
                            self.text_section.push(format!("    jmp {}", copy_loop));
                            self.text_section.push(format!("{}:", copy_done));
                            self.text_section.push("    pop rdi".to_string());
                        }
                        FStringPart::Expr(expr) => {
                            self.text_section.push("    push rdi".to_string());
                            self.text_section.push("    push r12".to_string());
                            self.generate_expr(expr)?;
                            self.text_section.push("    pop r12".to_string());
                            self.text_section.push("    pop rdi".to_string());
                            self.text_section.push("    push rdi".to_string());
                            self.text_section.push("    add rdi, r12".to_string());
                            self.text_section.push("    mov rbx, rdi".to_string());
                            self.text_section.push("    mov rcx, 10".to_string());
                            let num_loop = self.new_label("fstr_num_lnx");
                            let num_done = self.new_label("fstr_num_done_lnx");
                            self.text_section.push(format!("{}:", num_loop));
                            self.text_section.push("    xor rdx, rdx".to_string());
                            self.text_section.push("    div rcx".to_string());
                            self.text_section.push("    add dl, '0'".to_string());
                            self.text_section.push("    mov [rdi], dl".to_string());
                            self.text_section.push("    inc rdi".to_string());
                            self.text_section.push("    inc r12".to_string());
                            self.text_section.push("    cmp rax, 0".to_string());
                            self.text_section.push(format!("    jne {}", num_loop));
                            self.text_section.push(format!("{}:", num_done));
                            self.text_section.push("    mov rsi, rbx".to_string());
                            self.text_section.push("    mov rcx, rdi".to_string());
                            self.text_section.push("    dec rcx".to_string());
                            let rev_loop = self.new_label("fstr_rev_lnx");
                            let rev_done = self.new_label("fstr_rev_done_lnx");
                            self.text_section.push(format!("{}:", rev_loop));
                            self.text_section.push("    cmp rsi, rcx".to_string());
                            self.text_section.push(format!("    jge {}", rev_done));
                            self.text_section.push("    mov al, [rsi]".to_string());
                            self.text_section.push("    mov bl, [rcx]".to_string());
                            self.text_section.push("    mov [rsi], bl".to_string());
                            self.text_section.push("    mov [rcx], al".to_string());
                            self.text_section.push("    inc rsi".to_string());
                            self.text_section.push("    dec rcx".to_string());
                            self.text_section.push(format!("    jmp {}", rev_loop));
                            self.text_section.push(format!("{}:", rev_done));
                            self.text_section.push("    pop rdi".to_string());
                        }
                    }
                }
                
                self.text_section.push(format!("    lea rax, [rbp - {}]", buffer_offset + 8));
                self.text_section.push("    mov [rax + 8], r12".to_string());
            }
            Expr::TupleLiteral(elements) => {
                // Tuple literal: (1, 2, 3) - Linux
                let count = elements.len();
                let tuple_size = count * 8;
                let base_offset = self.stack_offset;
                
                self.stack_offset += tuple_size as i64;
                self.text_section.push(format!("    ; Tuple literal: {} elementos (Linux)", count));
                
                for (i, element) in elements.iter().enumerate() {
                    self.generate_expr(element)?;
                    let element_offset = base_offset + (i as i64 * 8);
                    self.text_section.push(format!("    mov [rbp - {}], rax  ; tuple[{}]", element_offset + 8, i));
                }
                
                self.text_section.push(format!("    lea rax, [rbp - {}]", base_offset + 8));
            }
            Expr::Lambda { params, body } => {
                // Lambda: lambda x, y: x + y - Linux
                let lambda_name = self.new_label("lambda");
                
                let saved_vars = self.variables.clone();
                let saved_offset = self.stack_offset;
                
                self.text_section.push(format!("    jmp {}_end", lambda_name));
                self.text_section.push(format!("{}:", lambda_name));
                self.text_section.push("    push rbp".to_string());
                self.text_section.push("    mov rbp, rsp".to_string());
                self.text_section.push("    sub rsp, 64".to_string());
                
                // System V ABI: RDI, RSI, RDX, RCX, R8, R9
                let param_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
                self.stack_offset = 0;
                for (i, param) in params.iter().enumerate() {
                    self.stack_offset += 8;
                    self.variables.insert(param.clone(), self.stack_offset);
                    if i < 6 {
                        self.text_section.push(format!("    mov [rbp - {}], {}  ; param {}", self.stack_offset + 8, param_regs[i], param));
                    }
                }
                
                self.generate_expr(body)?;
                
                self.text_section.push("    leave".to_string());
                self.text_section.push("    ret".to_string());
                self.text_section.push(format!("{}_end:", lambda_name));
                
                self.variables = saved_vars;
                self.stack_offset = saved_offset;
                
                self.text_section.push(format!("    lea rax, [rel {}]", lambda_name));
            }
            Expr::BinaryOp { op, left, right } => {
                // Generate right side first, push to stack
                self.generate_expr(left)?;
                self.text_section.push("    push rax".to_string());
                self.generate_expr(right)?;
                self.text_section.push("    mov rbx, rax".to_string());
                self.text_section.push("    pop rax".to_string());

                match op {
                    BinOp::Add => {
                        self.text_section.push("    add rax, rbx".to_string());
                    }
                    BinOp::Sub => {
                        self.text_section.push("    sub rax, rbx".to_string());
                    }
                    BinOp::Mul => {
                        self.text_section.push("    imul rax, rbx".to_string());
                    }
                    BinOp::Div => {
                        self.text_section.push("    cqo".to_string()); // sign extend rax to rdx:rax
                        self.text_section.push("    idiv rbx".to_string());
                    }
                    BinOp::FloorDiv => {
                        // División entera (//): igual que Div
                        self.text_section.push("    cqo".to_string());
                        self.text_section.push("    idiv rbx  ; división entera //".to_string());
                    }
                    BinOp::Mod => {
                        // Módulo: RAX = RAX % RBX (resto de división)
                        self.text_section.push("    cqo".to_string()); // sign extend rax to rdx:rax
                        self.text_section.push("    idiv rbx".to_string());
                        self.text_section.push("    mov rax, rdx  ; resto (módulo) en RAX".to_string());
                    }
                    BinOp::Pow => {
                        // Potencia: base ** exponente (Linux)
                        // Después de evaluar: rax = left (base), rbx = right (exponente)
                        
                        let label_not_zero = self.new_label("pow_not_zero");
                        let label_loop = self.new_label("pow_loop");
                        let label_end = self.new_label("pow_end");
                        
                        // rcx = exponente (rbx), rdx = base (rax)
                        self.text_section.push("    mov rcx, rbx  ; rcx = exponente".to_string());
                        self.text_section.push("    mov rdx, rax  ; rdx = base".to_string());
                        
                        self.text_section.push("    cmp rcx, 0".to_string());
                        self.text_section.push(format!("    jne {}  ; si exp != 0", label_not_zero));
                        self.text_section.push("    mov rax, 1  ; x^0 = 1".to_string());
                        self.text_section.push(format!("    jmp {}", label_end));
                        
                        self.text_section.push(format!("{}:", label_not_zero));
                        self.text_section.push("    mov rax, 1  ; resultado = 1".to_string());
                        self.text_section.push(format!("{}:", label_loop));
                        self.text_section.push("    cmp rcx, 0".to_string());
                        self.text_section.push(format!("    jle {}  ; si exp <= 0, terminar", label_end));
                        self.text_section.push("    imul rax, rdx  ; resultado *= base".to_string());
                        self.text_section.push("    dec rcx  ; exp--".to_string());
                        self.text_section.push(format!("    jmp {}", label_loop));
                        self.text_section.push(format!("{}:", label_end));
                    }
                    BinOp::Eq => {
                        self.text_section.push("    cmp rax, rbx".to_string());
                        self.text_section.push("    sete al".to_string());
                        self.text_section.push("    movzx rax, al".to_string());
                    }
                    BinOp::Ne => {
                        self.text_section.push("    cmp rax, rbx".to_string());
                        self.text_section.push("    setne al".to_string());
                        self.text_section.push("    movzx rax, al".to_string());
                    }
                    BinOp::Lt => {
                        self.text_section.push("    cmp rax, rbx".to_string());
                        self.text_section.push("    setl al".to_string());
                        self.text_section.push("    movzx rax, al".to_string());
                    }
                    BinOp::Le => {
                        self.text_section.push("    cmp rax, rbx".to_string());
                        self.text_section.push("    setle al".to_string());
                        self.text_section.push("    movzx rax, al".to_string());
                    }
                    BinOp::Gt => {
                        self.text_section.push("    cmp rax, rbx".to_string());
                        self.text_section.push("    setg al".to_string());
                        self.text_section.push("    movzx rax, al".to_string());
                    }
                    BinOp::Ge => {
                        self.text_section.push("    cmp rax, rbx".to_string());
                        self.text_section.push("    setge al".to_string());
                        self.text_section.push("    movzx rax, al".to_string());
                    }
                    BinOp::And => {
                        // AND lógico
                        self.text_section.push("    test rax, rax".to_string());
                        self.text_section.push("    setnz al".to_string());
                        self.text_section.push("    test rbx, rbx".to_string());
                        self.text_section.push("    setnz bl".to_string());
                        self.text_section.push("    and al, bl".to_string());
                        self.text_section.push("    movzx rax, al".to_string());
                    }
                    BinOp::Or => {
                        // OR lógico
                        self.text_section.push("    test rax, rax".to_string());
                        self.text_section.push("    setnz al".to_string());
                        self.text_section.push("    test rbx, rbx".to_string());
                        self.text_section.push("    setnz bl".to_string());
                        self.text_section.push("    or al, bl".to_string());
                        self.text_section.push("    movzx rax, al".to_string());
                    }
                }
            }
            Expr::Not(inner) => {
                // Negación lógica: !expr
                self.generate_expr(inner)?;
                self.text_section.push("    test rax, rax".to_string());
                self.text_section.push("    setz al".to_string());
                self.text_section.push("    movzx rax, al".to_string());
            }
            Expr::Call { module, name, args } => {
                // Generate args in reverse order (for System V: rdi, rsi, rdx, ...)
                for (i, arg) in args.iter().enumerate().rev() {
                    self.generate_expr(arg)?;
                    let reg = match i {
                        0 => "rdi",
                        1 => "rsi",
                        2 => "rdx",
                        3 => "rcx",
                        4 => "r8",
                        5 => "r9",
                        _ => {
                            // More args on stack
                            self.text_section.push("    push rax".to_string());
                            continue;
                        }
                    };
                    self.text_section.push(format!("    mov {}, rax", reg));
                }
                
                // Llamar función (con namespace si existe) (Sprint 1.3)
                let function_name = if let Some(module_name) = module {
                    format!("fn_{}_{}", module_name, name)
                } else {
                    format!("fn_{}", name)
                };
                
                    self.text_section.push(format!("    call {}", function_name));
                // Return value is in rax
            }
        }
        Ok(())
    }

    fn add_float_data(&mut self, value: f64) -> String {
        let label = format!("float_{}", self.string_counter);
        self.string_counter += 1;
        self.data_section.push(format!("{}: dq {}  ; float64 literal", label, value));
        label
    }
    
    /// Formatear float de manera inteligente:
    /// - Versión optimizada: formato limpio (ej: 5.64)
    /// - Versión completa: precisión completa cuando sea necesario
    /// Respetando siempre el cálculo correcto
    fn format_float_smart(&self, value: f64, use_precise: bool) -> String {
        if use_precise {
            // Versión completa: mostrar toda la precisión disponible
            // Útil para cálculos científicos o cuando se necesita precisión exacta
            format!("{:.15}", value).trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            // Versión optimizada: formato limpio y legible
            // Estrategia: redondear a número específico de decimales según el valor
            // Intenta detectar cuántos decimales son significativos
            
            // Redondear a 2 decimales primero (común para muchos casos)
            let rounded_2 = (value * 100.0).round() / 100.0;
            // Si el error después de redondear es muy pequeño, usar 2 decimales
            if (value - rounded_2).abs() < 0.0001 {
                let s = format!("{:.2}", rounded_2).trim_end_matches('0').trim_end_matches('.').to_string();
                if !s.is_empty() { return s; }
            }
            
            // Si no, probar con más decimales (hasta 6)
            let rounded_6 = (value * 1_000_000.0).round() / 1_000_000.0;
            let fract = rounded_6.fract().abs();
            
            if fract < 0.0001 {
                // Prácticamente entero
                format!("{:.0}", rounded_6)
            } else {
                // Formatear y eliminar ceros finales
                format!("{:.6}", rounded_6).trim_end_matches('0').trim_end_matches('.').to_string()
            }
        }
    }

    // Helper para detectar si una expresión es float
    fn is_float_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Float(_) => true,
            Expr::BinaryOp { left, op: _, right } => {
                // Si alguno de los operandos es float, la operación es float
                self.is_float_expr(left) || self.is_float_expr(right)
            }
            Expr::Ident(_name) => {
                // Por ahora, asumimos que las variables son int a menos que se especifique
                // TODO: Implementar type inference para variables
                false
            }
            _ => false,
        }
    }

    // Helper para detectar si una expresión es string
    fn is_string_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::String(_) => true,
            Expr::Ident(name) => {
                // Heurística mejorada para detectar variables string:
                // 1. Si el nombre es solo 's' (variable común para strings)
                // 2. Si el nombre empieza con 's' seguido de un número o letra (s1, s2, str1, etc.)
                // 3. Si contiene "str", "text", "msg" en el nombre
                // 4. Nombres comunes como "texto", "mensaje"
                let lower_name = name.to_lowercase();
                name == "s"  // Variable común 's' para strings (una sola letra)
                || (name.starts_with('s') && name.len() > 1 && name.chars().nth(1).map_or(false, |c| c.is_alphanumeric()))
                || lower_name.contains("str")
                || lower_name.contains("text")
                || lower_name.contains("msg")
                || lower_name == "texto"
                || lower_name == "mensaje"
                || lower_name.starts_with("s")  // Cualquier nombre que empiece con 's'
            }
            Expr::MethodCall { object, method, args: _ } => {
                // Si el objeto es String, entonces es string method
                // También métodos de strings retornan strings
                self.is_string_expr(object) || matches!(method.as_str(), "upper" | "lower" | "slice" | "substring")
            }
            Expr::BinaryOp { left, op: BinOp::Add, right } => {
                // Concatenación: si ambos operandos son strings
                // O si al menos uno es string (para permitir string + literal)
                self.is_string_expr(left) || self.is_string_expr(right)
            }
            Expr::Slice { object, .. } => {
                // Slicing siempre retorna string: s[0:4] -> string
                self.is_string_expr(object)
            }
            Expr::Call { module: _, name, args } => {
                // len(s) donde s es string
                if name == "len" && args.len() == 1 {
                    self.is_string_expr(&args[0])
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    
    // Helper para detectar si una expresión es booleana
    fn is_bool_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Bool(_) => true,
            Expr::Ident(name) => {
                // Verificar si la variable está registrada como booleana
                if let Some(type_name) = self.variable_types.get(name) {
                    return type_name == "bool";
                }
                // Heurística: nombres comunes para booleanos
                let lower = name.to_lowercase();
                lower.contains("activo") || lower.contains("active") ||
                lower.contains("enabled") || lower.contains("flag") ||
                lower.contains("is_") || lower.contains("has_") ||
                lower == "ok" || lower == "done" || lower == "valid"
            }
            Expr::BinaryOp { op, .. } => {
                // Operadores de comparación retornan bool
                matches!(op, BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge)
            }
            _ => false,
        }
    }

    /// Determinar el tipo de struct de una expresión (para calcular offsets de campos)
    fn get_struct_type_from_expr(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Ident(name) => {
                // Buscar en variable_types primero (más confiable)
                if let Some(type_name) = self.variable_types.get(name) {
                    return Some(type_name.clone());
                }
                // Fallback: buscar en struct_definitions para ver si existe un struct con ese nombre
                // Solo si el nombre de la variable coincide exactamente (case-insensitive) con el struct
                for (struct_name, _) in &self.struct_definitions {
                    if name.to_lowercase() == struct_name.to_lowercase() {
                        return Some(struct_name.clone());
                    }
                }
                None
            }
            Expr::StructLiteral { name, .. } => Some(name.clone()),
            _ => None,
        }
    }
    
    /// Evaluar una expresión constante en compile-time (solo para floats y números)
    /// Retorna Some(f64) si la expresión puede evaluarse en compile-time, None si no
    fn eval_const_expr(&self, expr: &Expr) -> Option<f64> {
        match expr {
            Expr::Float(f) => Some(*f),
            Expr::Number(n) => Some(*n as f64),
            Expr::BinaryOp { left, op, right } => {
                let left_val = self.eval_const_expr(left)?;
                let right_val = self.eval_const_expr(right)?;
                
                match op {
                    BinOp::Add => Some(left_val + right_val),
                    BinOp::Sub => Some(left_val - right_val),
                    BinOp::Mul => Some(left_val * right_val),
                    BinOp::Div => {
                        if right_val == 0.0 {
                            None
                        } else {
                            Some(left_val / right_val)
                        }
                    }
                    _ => None, // Operaciones no soportadas para evaluación constante
                }
            }
            _ => None, // Expresiones no constantes
        }
    }

    fn add_string_data(&mut self, s: &str) -> String {
        let label = format!("msg{}", self.string_counter);
        self.string_counter += 1;
        
        // Construir string NASM con caracteres especiales como bytes separados
        // Ejemplo: "Hola\nMundo" -> db "Hola", 0xA, "Mundo"
        let mut parts: Vec<String> = Vec::new();
        let mut current = String::new();
        
        for c in s.chars() {
            match c {
                '\n' => {
                    if !current.is_empty() {
                        parts.push(format!("\"{}\"", current.replace('"', "\\\"")));
                        current.clear();
                    }
                    parts.push("0xA".to_string());
                }
                '\t' => {
                    if !current.is_empty() {
                        parts.push(format!("\"{}\"", current.replace('"', "\\\"")));
                        current.clear();
                    }
                    parts.push("0x9".to_string());
                }
                '\r' => {
                    if !current.is_empty() {
                        parts.push(format!("\"{}\"", current.replace('"', "\\\"")));
                        current.clear();
                    }
                    parts.push("0xD".to_string());
                }
                '\0' => {
                    if !current.is_empty() {
                        parts.push(format!("\"{}\"", current.replace('"', "\\\"")));
                        current.clear();
                    }
                    parts.push("0x0".to_string());
                }
                _ => current.push(c),
            }
        }
        
        // Agregar último segmento si existe
        if !current.is_empty() {
            parts.push(format!("\"{}\"", current.replace('"', "\\\"")));
        }
        
        // Si no hay partes, crear string vacío
        if parts.is_empty() {
            parts.push("\"\"".to_string());
        }
        
        // Generar línea db con todas las partes
        self.data_section.push(format!(
            "{}: db {}",
            label, parts.join(", ")
        ));
        
        self.data_section.push(format!(
            "{}_len: equ $ - {}",
            label, label
        ));
        label
    }

    fn new_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    /// Generar prologue ABI-safe para funciones helper
    /// Preserva registros no volátiles: RBX, RDI, RSI, R12-R15
    /// Asegura stack alignment a 16 bytes
    fn generate_abi_prologue(&mut self, needs_shadow_space: bool) {
        // Preservar registros no volátiles (callee-saved)
        // Orden: RBP (ya se hace con push rbp), RBX, RDI, RSI, R12-R15
        self.text_section.push("    push rbp".to_string());
        self.text_section.push("    mov rbp, rsp".to_string());
        self.text_section.push("    push rbx  ; preservar registro no volátil".to_string());
        self.text_section.push("    push rdi  ; preservar registro no volátil".to_string());
        self.text_section.push("    push rsi  ; preservar registro no volátil".to_string());
        self.text_section.push("    push r12  ; preservar registro no volátil".to_string());
        self.text_section.push("    push r13  ; preservar registro no volátil".to_string());
        self.text_section.push("    push r14  ; preservar registro no volátil".to_string());
        self.text_section.push("    push r15  ; preservar registro no volátil".to_string());
        
        // Asegurar stack alignment a 16 bytes
        // Después de 7 push (rbp, rbx, rdi, rsi, r12-r15) = 56 bytes
        // 56 % 16 = 8, necesitamos 8 bytes más para alinear
        self.text_section.push("    ; Asegurar stack alignment a 16 bytes".to_string());
        self.text_section.push("    sub rsp, 8  ; alinear stack (56 bytes de push % 16 = 8)".to_string());
        
        if needs_shadow_space {
            self.text_section.push("    sub rsp, 32  ; shadow space para llamadas a funciones externas".to_string());
        }
    }

    /// Generar epilogue ABI-safe para funciones helper
    /// Restaura registros no volátiles en orden inverso
    fn generate_abi_epilogue(&mut self, needs_shadow_space: bool) {
        if needs_shadow_space {
            self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
        }
        self.text_section.push("    add rsp, 8  ; restaurar alineación de stack".to_string());
        
        // Restaurar registros no volátiles en orden inverso
        self.text_section.push("    pop r15  ; restaurar registro no volátil".to_string());
        self.text_section.push("    pop r14  ; restaurar registro no volátil".to_string());
        self.text_section.push("    pop r13  ; restaurar registro no volátil".to_string());
        self.text_section.push("    pop r12  ; restaurar registro no volátil".to_string());
        self.text_section.push("    pop rsi  ; restaurar registro no volátil".to_string());
        self.text_section.push("    pop rdi  ; restaurar registro no volátil".to_string());
        self.text_section.push("    pop rbx  ; restaurar registro no volátil".to_string());
        self.text_section.push("    leave  ; restaurar rbp y rsp".to_string());
        self.text_section.push("    ret".to_string());
    }

    /// Asegurar stack alignment antes de una llamada a función externa
    /// Debe llamarse ANTES de preparar parámetros y shadow space
    /// Calcula si el stack está alineado y ajusta si es necesario
    fn ensure_stack_alignment_before_call(&mut self, comment: &str) {
        // Nota: Si ya hicimos prologue ABI-safe, el stack debería estar alineado
        // Pero si hay pushes adicionales antes de la llamada, necesitamos verificar
        // Por ahora, confiamos en que el prologue mantiene la alineación
        // Si hay pushes adicionales, el código debe ajustar manualmente
        self.text_section.push(format!("    ; Verificar stack alignment antes de call: {}", comment));
        // En una implementación más sofisticada, podríamos calcular dinámicamente
        // Por ahora, asumimos que el prologue mantiene la alineación correcta
    }

    /// Generar sistema de panic para manejo de errores profesional
    fn generate_panic_system(&mut self) {
        self.text_section.push("".to_string());
        self.text_section.push("; ============================================".to_string());
        self.text_section.push("; RUNTIME: Sistema de Panic".to_string());
        self.text_section.push("; ============================================".to_string());
        self.text_section.push("".to_string());
        
        // panic_out_of_bounds: Error cuando se accede fuera de rango
        self.text_section.push("panic_out_of_bounds:".to_string());
        self.text_section.push("    push rbp".to_string());
        self.text_section.push("    mov rbp, rsp".to_string());
        self.text_section.push("    sub rsp, 64  ; shadow space + local vars".to_string());
        
        // Obtener stdout
        self.text_section.push("    mov ecx, -11  ; STD_OUTPUT_HANDLE".to_string());
        self.text_section.push("    call GetStdHandle".to_string());
        self.text_section.push("    mov r12, rax  ; guardar handle en r12 (preservado)".to_string());
        
        // Preparar WriteFile
        self.text_section.push("    mov rcx, r12  ; hFile (stdout)".to_string());
        self.text_section.push("    lea rdx, [rel panic_msg_out_of_bounds]  ; lpBuffer".to_string());
        self.text_section.push("    mov r8, panic_msg_out_of_bounds_len  ; nNumberOfBytesToWrite".to_string());
        self.text_section.push("    lea r9, [rbp - 8]  ; lpNumberOfBytesWritten".to_string());
        self.text_section.push("    mov qword [r9], 0  ; inicializar".to_string());
        self.text_section.push("    mov qword [rsp + 32], 0  ; lpOverlapped = NULL".to_string());
        self.text_section.push("    call WriteFile".to_string());
        
        // Exit con código de error
        self.text_section.push("    mov ecx, 1  ; exit code 1 (error)".to_string());
        self.text_section.push("    call ExitProcess".to_string());
        self.text_section.push("".to_string());
        
        // panic_null_pointer: Error cuando se desreferencia null
        self.text_section.push("panic_null_pointer:".to_string());
        self.text_section.push("    push rbp".to_string());
        self.text_section.push("    mov rbp, rsp".to_string());
        self.text_section.push("    sub rsp, 64".to_string());
        
        self.text_section.push("    mov ecx, -11".to_string());
        self.text_section.push("    call GetStdHandle".to_string());
        self.text_section.push("    mov r12, rax".to_string());
        
        self.text_section.push("    mov rcx, r12".to_string());
        self.text_section.push("    lea rdx, [rel panic_msg_null_pointer]".to_string());
        self.text_section.push("    mov r8, panic_msg_null_pointer_len".to_string());
        self.text_section.push("    lea r9, [rbp - 8]".to_string());
        self.text_section.push("    mov qword [r9], 0".to_string());
        self.text_section.push("    mov qword [rsp + 32], 0".to_string());
        self.text_section.push("    call WriteFile".to_string());
        
        self.text_section.push("    mov ecx, 1".to_string());
        self.text_section.push("    call ExitProcess".to_string());
        self.text_section.push("".to_string());
        
        // Agregar mensajes de error en data section
        self.data_section.push("".to_string());
        self.data_section.push("; Mensajes de error para panic system".to_string());
        self.data_section.push("panic_msg_out_of_bounds: db \"Error: Array index out of bounds\", 0xA, 0".to_string());
        self.data_section.push("panic_msg_out_of_bounds_len equ $ - panic_msg_out_of_bounds".to_string());
        self.data_section.push("".to_string());
        self.data_section.push("panic_msg_null_pointer: db \"Error: Null pointer dereference\", 0xA, 0".to_string());
        self.data_section.push("panic_msg_null_pointer_len equ $ - panic_msg_null_pointer".to_string());
        self.data_section.push("".to_string());
    }
    
    /// Generar funciones helper de Array en NASM (versión completa - todas las funciones)
    /// Estructura Array: [data: qword, length: qword, capacity: qword]
    /// Total: 24 bytes (3 qwords)
    fn generate_array_helpers_nasm(&mut self) {
        // Llamar a la versión selectiva con un dependency graph que incluye todo
        let mut deps = DependencyGraph::new();
        // Marcar todas las funciones de arrays como usadas
        deps.mark_used("array_new");
        deps.mark_used("array_from_values");
        deps.mark_used("array_get");
        deps.mark_used("array_set");
        deps.mark_used("array_len");
        deps.mark_used("array_append");
        deps.mark_used("array_pop");
        deps.mark_used("array_insert");
        deps.mark_used("array_remove");
        deps.mark_used("array_index");
        deps.mark_used("array_count");
        deps.mark_used("array_sort");
        deps.mark_used("array_reverse");
        deps.mark_used("array_free");
        self.generate_array_helpers_nasm_selective(&deps);
    }
    
    /// Generar funciones helper de Array en NASM (versión selectiva - solo funciones usadas)
    /// Estructura Array: [data: qword, length: qword, capacity: qword]
    /// Total: 24 bytes (3 qwords)
    fn generate_array_helpers_nasm_selective(&mut self, deps: &DependencyGraph) {
        // ============================================
        // Estructura Array en NASM (24 bytes):
        // - data: qword (puntero a memoria dinámica)
        // - length: qword (número de elementos)
        // - capacity: qword (capacidad total)
        // ============================================
        
        // array_new: Crear array vacío
        // Retorna: RAX = puntero al Array (en heap)
        if deps.should_generate("array_new") {
        self.text_section.push("array_new:".to_string());
        self.generate_abi_prologue(true);  // Necesita shadow space para VirtualAlloc
        
        // Allocar memoria para Array (24 bytes)
        self.text_section.push("    ; Allocar memoria para Array (24 bytes)".to_string());
        self.ensure_stack_alignment_before_call("VirtualAlloc");
        self.text_section.push("    mov rcx, 0  ; lpAddress (NULL = auto)".to_string());
        self.text_section.push("    mov rdx, 24  ; dwSize (24 bytes para Array struct)".to_string());
        self.text_section.push("    mov r8, 0x1000  ; flAllocationType (MEM_COMMIT)".to_string());
        self.text_section.push("    mov r9, 0x04  ; flProtect (PAGE_READWRITE)".to_string());
        self.text_section.push("    call VirtualAlloc".to_string());
        
        // Inicializar Array: length=0, capacity=4, data=NULL (se asignará después)
        self.text_section.push("    ; Inicializar Array".to_string());
        self.text_section.push("    mov qword [rax + 0], 0  ; data = NULL (se asignará después)".to_string());
        self.text_section.push("    mov qword [rax + 8], 0  ; length = 0".to_string());
        self.text_section.push("    mov qword [rax + 16], 4  ; capacity = 4".to_string());
        
        // Allocar memoria para data (capacity * 8 bytes)
        self.text_section.push("    ; Allocar memoria para data (capacity * 8 bytes = 32 bytes)".to_string());
        self.text_section.push("    mov rbx, rax  ; guardar puntero al Array en rbx (preservado)".to_string());
        self.ensure_stack_alignment_before_call("VirtualAlloc (segunda llamada)");
        self.text_section.push("    mov rcx, 0  ; lpAddress".to_string());
        self.text_section.push("    mov rdx, 32  ; dwSize (4 elementos * 8 bytes)".to_string());
        self.text_section.push("    mov r8, 0x1000  ; MEM_COMMIT".to_string());
        self.text_section.push("    mov r9, 0x04  ; PAGE_READWRITE".to_string());
        self.text_section.push("    call VirtualAlloc".to_string());
        
        // Asignar data al Array
        self.text_section.push("    mov [rbx + 0], rax  ; data = puntero a memoria".to_string());
        self.text_section.push("    mov rax, rbx  ; retornar puntero al Array".to_string());
        
        // Epilogue ABI-safe
        self.generate_abi_epilogue(true);
        self.text_section.push("".to_string());
        }
        
        // array_from_values: Crear array desde valores iniciales
        if deps.should_generate("array_from_values") {
        // Parámetros: RCX = count, RDX = puntero a valores (int64_t*)
        // Retorna: RAX = puntero al Array
        self.text_section.push("array_from_values:".to_string());
        self.generate_abi_prologue(true);  // Necesita shadow space para VirtualAlloc
        self.text_section.push("    mov r12, rcx  ; guardar count en r12 (preservado)".to_string());
        self.text_section.push("    mov r13, rdx  ; guardar puntero a valores en r13 (preservado)".to_string());
        
        // Calcular capacity: max(count * 2, 4)
        self.text_section.push("    ; Calcular capacity: max(count * 2, 4)".to_string());
        self.text_section.push("    mov rax, r12  ; count (desde r12 preservado)".to_string());
        self.text_section.push("    shl rax, 1  ; count * 2".to_string());
        self.text_section.push("    cmp rax, 4".to_string());
        self.text_section.push("    jge .capacity_ok".to_string());
        self.text_section.push("    mov rax, 4  ; mínimo 4".to_string());
        self.text_section.push(".capacity_ok:".to_string());
        self.text_section.push("    mov r14, rax  ; guardar capacity en r14 (preservado)".to_string());
        
        // Allocar memoria para Array (24 bytes)
        self.text_section.push("    ; Allocar memoria para Array".to_string());
        self.ensure_stack_alignment_before_call("VirtualAlloc (Array struct)");
        self.text_section.push("    mov rcx, 0".to_string());
        self.text_section.push("    mov rdx, 24".to_string());
        self.text_section.push("    mov r8, 0x1000".to_string());
        self.text_section.push("    mov r9, 0x04".to_string());
        self.text_section.push("    call VirtualAlloc".to_string());
        self.text_section.push("    mov r15, rax  ; guardar puntero al Array en r15 (preservado)".to_string());
        
        // Allocar memoria para data (capacity * 8 bytes)
        self.text_section.push("    ; Allocar memoria para data".to_string());
        self.text_section.push("    mov rax, r14  ; capacity".to_string());
        self.text_section.push("    shl rax, 3  ; capacity * 8 bytes".to_string());
        self.ensure_stack_alignment_before_call("VirtualAlloc (data)");
        self.text_section.push("    mov rcx, 0".to_string());
        self.text_section.push("    mov rdx, rax  ; size".to_string());
        self.text_section.push("    mov r8, 0x1000".to_string());
        self.text_section.push("    mov r9, 0x04".to_string());
        self.text_section.push("    call VirtualAlloc".to_string());
        self.text_section.push("    mov rdi, rax  ; puntero a data en rdi (preservado)".to_string());
        
        // Configurar Array struct
        self.text_section.push("    ; Configurar Array struct".to_string());
        self.text_section.push("    mov [r15 + 0], rdi  ; data = puntero".to_string());
        self.text_section.push("    mov [r15 + 8], r12  ; length = count".to_string());
        self.text_section.push("    mov [r15 + 16], r14  ; capacity".to_string());
        
        // Copiar valores usando rep movsq (OPTIMIZACIÓN: mucho más rápido que loop manual)
        // rep movsq copia RCX qwords desde [RSI] hacia [RDI]
        self.text_section.push("    ; Copiar valores con rep movsq (optimizado)".to_string());
        self.text_section.push("    mov rcx, r12  ; count (número de qwords a copiar)".to_string());
        self.text_section.push("    mov rsi, r13  ; fuente (puntero a valores)".to_string());
        self.text_section.push("    ; rdi ya tiene el destino (data buffer)".to_string());
        self.text_section.push("    test rcx, rcx  ; si count == 0, saltar".to_string());
        self.text_section.push("    jz .copy_done".to_string());
        self.text_section.push("    cld  ; clear direction flag (copiar hacia adelante)".to_string());
        self.text_section.push("    rep movsq  ; copiar RCX qwords de [RSI] a [RDI]".to_string());
        self.text_section.push(".copy_done:".to_string());
        self.text_section.push("    mov rax, r15  ; retornar puntero al Array".to_string());
        
        // Epilogue ABI-safe
        self.generate_abi_epilogue(true);
        self.text_section.push("".to_string());
        }
        
        // array_get: Obtener elemento por índice
        if deps.should_generate("array_get") {
        // Parámetros: RCX = puntero al Array, RDX = índice
        // Retorna: RAX = valor del elemento
        // Si hay error (null pointer o out of bounds), llama a panic y termina el programa
        self.text_section.push("array_get:".to_string());
        self.generate_abi_prologue(false);  // No necesita shadow space (solo lectura)
        
        // Verificar null pointer
        self.text_section.push("    ; Verificar null pointer".to_string());
        self.text_section.push("    test rcx, rcx".to_string());
        self.text_section.push("    jz panic_null_pointer".to_string());
        
        // Bounds checking (usa panic en lugar de código mágico)
        self.text_section.push("    ; Bounds checking".to_string());
        self.text_section.push("    mov r12, rcx  ; preservar puntero al Array".to_string());
        self.text_section.push("    mov r13, rdx  ; preservar índice".to_string());
        self.text_section.push("    cmp r13, [r12 + 8]  ; comparar índice con length".to_string());
        self.text_section.push("    jge panic_out_of_bounds".to_string());
        self.text_section.push("    cmp r13, 0".to_string());
        self.text_section.push("    jl panic_out_of_bounds".to_string());
        
        // Obtener elemento
        self.text_section.push("    ; Obtener elemento".to_string());
        self.text_section.push("    mov rax, [r12 + 0]  ; cargar puntero a data".to_string());
        self.text_section.push("    mov rdx, r13  ; índice".to_string());
        self.text_section.push("    shl rdx, 3  ; índice * 8 bytes".to_string());
        self.text_section.push("    add rax, rdx  ; dirección del elemento".to_string());
        self.text_section.push("    mov rax, [rax]  ; cargar valor".to_string());
        
        // Epilogue ABI-safe
        self.generate_abi_epilogue(false);
        self.text_section.push("".to_string());
        }
        
        // array_set: Establecer elemento por índice
        if deps.should_generate("array_set") {
        // Parámetros: RCX = puntero al Array, RDX = índice, R8 = valor
        // Retorna: RAX = 0 (éxito)
        // Si hay error (null pointer o out of bounds), llama a panic y termina el programa
        self.text_section.push("array_set:".to_string());
        self.generate_abi_prologue(false);  // No necesita shadow space (solo escritura)
        
        // Verificar null pointer
        self.text_section.push("    ; Verificar null pointer".to_string());
        self.text_section.push("    test rcx, rcx".to_string());
        self.text_section.push("    jz panic_null_pointer".to_string());
        
        // Bounds checking (usa panic en lugar de código mágico)
        self.text_section.push("    ; Bounds checking".to_string());
        self.text_section.push("    mov r12, rcx  ; preservar puntero al Array".to_string());
        self.text_section.push("    mov r13, rdx  ; preservar índice".to_string());
        self.text_section.push("    mov r14, r8  ; preservar valor".to_string());
        self.text_section.push("    cmp r13, [r12 + 8]  ; comparar índice con length".to_string());
        self.text_section.push("    jge panic_out_of_bounds".to_string());
        self.text_section.push("    cmp r13, 0".to_string());
        self.text_section.push("    jl panic_out_of_bounds".to_string());
        
        // Establecer elemento
        self.text_section.push("    ; Establecer elemento".to_string());
        self.text_section.push("    mov rax, [r12 + 0]  ; cargar puntero a data".to_string());
        self.text_section.push("    mov rdx, r13  ; índice".to_string());
        self.text_section.push("    shl rdx, 3  ; índice * 8 bytes".to_string());
        self.text_section.push("    add rax, rdx  ; dirección del elemento".to_string());
        self.text_section.push("    mov [rax], r14  ; guardar valor".to_string());
        
        // Retornar éxito
        self.text_section.push("    mov rax, 0  ; éxito".to_string());
        
        // Epilogue ABI-safe
        self.generate_abi_epilogue(false);
        self.text_section.push("".to_string());
        }
        
        // array_len: Obtener longitud del array
        if deps.should_generate("array_len") {
        // Parámetros: RCX = puntero al Array
        // Retorna: RAX = longitud
        // Nota: Función muy simple, no necesita prologue/epilogue completo
        // pero debemos preservar registros según ABI
        self.text_section.push("array_len:".to_string());
        self.text_section.push("    mov rax, [rcx + 8]  ; cargar length".to_string());
        self.text_section.push("    ret  ; RCX es caller-saved, no necesitamos preservarlo".to_string());
        self.text_section.push("".to_string());
        }
        
        // array_pop: Eliminar y retornar último elemento
        if deps.should_generate("array_pop") {
        // Parámetros: RCX = puntero al Array
        // Retorna: RAX = valor del último elemento, o 0x8000000000000001 si error (array vacío)
        // Nota: El caller debe verificar si RAX == 0x8000000000000001 para detectar error
        self.text_section.push("array_pop:".to_string());
        self.generate_abi_prologue(false);  // No necesita shadow space (solo lectura/escritura)
        
        // Verificar que el array no esté vacío
        self.text_section.push("    ; Verificar que el array no esté vacío".to_string());
        self.text_section.push("    mov r12, rcx  ; preservar puntero al Array".to_string());
        self.text_section.push("    mov rax, [r12 + 8]  ; length".to_string());
        self.text_section.push("    test rax, rax".to_string());
        self.text_section.push("    jz .array_pop_error".to_string());
        
        // Obtener último elemento
        self.text_section.push("    ; Obtener último elemento".to_string());
        self.text_section.push("    dec rax  ; length - 1 (índice del último)".to_string());
        self.text_section.push("    mov r13, [r12 + 0]  ; puntero a data (preservado)".to_string());
        self.text_section.push("    mov r14, rax  ; preservar índice".to_string());
        self.text_section.push("    shl r14, 3  ; índice * 8 bytes".to_string());
        self.text_section.push("    add r13, r14  ; dirección del último elemento".to_string());
        self.text_section.push("    mov rax, [r13]  ; cargar valor del último elemento".to_string());
        self.text_section.push("    mov r15, rax  ; preservar valor en r15".to_string());
        
        // Decrementar length
        self.text_section.push("    ; Decrementar length".to_string());
        self.text_section.push("    dec qword [r12 + 8]  ; length--".to_string());
        
        // Retornar valor
        self.text_section.push("    mov rax, r15  ; restaurar valor".to_string());
        
        // Epilogue ABI-safe
        self.generate_abi_epilogue(false);
        self.text_section.push("".to_string());
        
        // Error handler: retornar código de error especial en lugar de ExitProcess
        self.text_section.push(".array_pop_error:".to_string());
        self.text_section.push("    ; Error: pop de array vacío".to_string());
        self.text_section.push("    mov rax, 0x8000000000000001  ; código de error especial (array vacío)".to_string());
        self.generate_abi_epilogue(false);
        self.text_section.push("".to_string());
        }
        
        // array_append: Agregar elemento al array
        if deps.should_generate("array_append") {
        // Parámetros: RCX = puntero al Array, RDX = valor
        // Retorna: RAX = 0 (éxito) o -1 (error: fallo de memoria)
        self.text_section.push("array_append:".to_string());
        self.generate_abi_prologue(true);  // Necesita shadow space para VirtualAlloc/VirtualFree
        self.text_section.push("    mov r12, rcx  ; preservar puntero al Array".to_string());
        self.text_section.push("    mov r13, rdx  ; preservar valor".to_string());
        
        // Verificar si necesita realloc
        self.text_section.push("    ; Verificar si necesita realloc".to_string());
        self.text_section.push("    mov rax, [r12 + 8]  ; length".to_string());
        self.text_section.push("    cmp rax, [r12 + 16]  ; comparar con capacity".to_string());
        self.text_section.push("    jl .no_realloc".to_string());
        
        // Realloc: duplicar capacity
        self.text_section.push("    ; Realloc: duplicar capacity".to_string());
        self.text_section.push("    mov rax, [r12 + 16]  ; capacity actual".to_string());
        self.text_section.push("    shl rax, 1  ; capacity * 2".to_string());
        self.text_section.push("    mov [r12 + 16], rax  ; actualizar capacity".to_string());
        self.text_section.push("    mov r14, rax  ; preservar nueva capacity".to_string());
        self.text_section.push("    shl r14, 3  ; capacity * 8 bytes".to_string());
        
        // VirtualAlloc nuevo bloque
        self.text_section.push("    ; VirtualAlloc nuevo bloque".to_string());
        self.ensure_stack_alignment_before_call("VirtualAlloc");
        self.text_section.push("    mov rcx, 0".to_string());
        self.text_section.push("    mov rdx, r14  ; nuevo size".to_string());
        self.text_section.push("    mov r8, 0x1000".to_string());
        self.text_section.push("    mov r9, 0x04".to_string());
        self.text_section.push("    call VirtualAlloc".to_string());
        self.text_section.push("    mov r15, rax  ; preservar nuevo puntero".to_string());
        
        // Copiar datos antiguos (OPTIMIZADO: usar rep movsq para copia rápida)
        self.text_section.push("    ; Copiar datos antiguos (optimizado con rep movsq)".to_string());
        self.text_section.push("    mov rdi, r15  ; destino (nuevo)".to_string());
        self.text_section.push("    mov rsi, [r12 + 0]  ; fuente (antiguo)".to_string());
        self.text_section.push("    mov rcx, [r12 + 8]  ; contador (length en elementos)".to_string());
        self.text_section.push("    test rcx, rcx".to_string());
        self.text_section.push("    jz .copy_done_append".to_string());
        self.text_section.push("    cld  ; clear direction flag (forward)".to_string());
        self.text_section.push("    rep movsq  ; copiar 8 bytes a la vez (qword) - MUCHO MÁS RÁPIDO".to_string());
        self.text_section.push(".copy_done_append:".to_string());
        
        // VirtualFree bloque antiguo
        self.text_section.push("    ; VirtualFree bloque antiguo".to_string());
        self.text_section.push("    mov r14, [r12 + 0]  ; preservar puntero antiguo".to_string());
        self.ensure_stack_alignment_before_call("VirtualFree");
        self.text_section.push("    mov rcx, r14  ; lpAddress".to_string());
        self.text_section.push("    mov rdx, 0  ; dwSize (0 = liberar todo)".to_string());
        self.text_section.push("    mov r8, 0x8000  ; MEM_RELEASE".to_string());
        self.text_section.push("    call VirtualFree".to_string());
        
        // Actualizar data pointer
        self.text_section.push("    ; Actualizar data pointer".to_string());
        self.text_section.push("    mov [r12 + 0], r15  ; data = nuevo puntero".to_string());
        
        // Agregar elemento
        self.text_section.push(".no_realloc:".to_string());
        self.text_section.push("    mov rax, [r12 + 8]  ; length".to_string());
        self.text_section.push("    mov rbx, [r12 + 0]  ; data pointer".to_string());
        self.text_section.push("    shl rax, 3  ; length * 8 bytes".to_string());
        self.text_section.push("    add rbx, rax  ; dirección del nuevo elemento".to_string());
        self.text_section.push("    mov [rbx], r13  ; guardar valor".to_string());
        self.text_section.push("    inc qword [r12 + 8]  ; incrementar length".to_string());
        
        // Epilogue ABI-safe
        self.generate_abi_epilogue(true);
        self.text_section.push("".to_string());
        }
        
        // array_reverse: Invertir orden del array
        if deps.should_generate("array_reverse") {
        // Parámetros: RCX = puntero al Array
        // Retorna: RAX = 0 (éxito, siempre exitoso)
        self.text_section.push("array_reverse:".to_string());
        self.generate_abi_prologue(false);  // No necesita shadow space (solo operaciones internas)
        
        // Verificar que el array no esté vacío o tenga solo 1 elemento
        self.text_section.push("    ; Verificar si necesita reversión".to_string());
        self.text_section.push("    mov r12, rcx  ; preservar puntero al Array".to_string());
        self.text_section.push("    mov rax, [r12 + 8]  ; length".to_string());
        self.text_section.push("    cmp rax, 1".to_string());
        self.text_section.push("    jle .reverse_done  ; si length <= 1, no hacer nada".to_string());
        
        // Setup: left = 0, right = length - 1
        self.text_section.push("    ; Setup: left = 0, right = length - 1".to_string());
        self.text_section.push("    mov rdx, 0  ; left index = 0".to_string());
        self.text_section.push("    mov r8, rax  ; length".to_string());
        self.text_section.push("    dec r8  ; right index = length - 1".to_string());
        self.text_section.push("    mov r9, [r12 + 0]  ; puntero a data (preservado)".to_string());
        
        // Loop: intercambiar elementos mientras left < right
        self.text_section.push("    ; Loop: intercambiar elementos".to_string());
        self.text_section.push(".reverse_loop:".to_string());
        self.text_section.push("    cmp rdx, r8  ; comparar left con right".to_string());
        self.text_section.push("    jge .reverse_done  ; si left >= right, terminar".to_string());
        
        // Intercambiar arr[left] y arr[right]
        self.text_section.push("    ; Intercambiar arr[left] y arr[right]".to_string());
        // Cargar arr[left]
        self.text_section.push("    mov rax, rdx  ; left index".to_string());
        self.text_section.push("    shl rax, 3  ; left * 8 bytes".to_string());
        self.text_section.push("    mov r10, r9  ; puntero a data".to_string());
        self.text_section.push("    add r10, rax  ; dirección de arr[left]".to_string());
        self.text_section.push("    mov r11, [r10]  ; temp = arr[left]".to_string());
        
        // Cargar arr[right]
        self.text_section.push("    mov rax, r8  ; right index".to_string());
        self.text_section.push("    shl rax, 3  ; right * 8 bytes".to_string());
        self.text_section.push("    mov r12, r9  ; puntero a data".to_string());
        self.text_section.push("    add r12, rax  ; dirección de arr[right]".to_string());
        self.text_section.push("    mov r13, [r12]  ; arr[right]".to_string());
        
        // Intercambiar
        self.text_section.push("    mov [r10], r13  ; arr[left] = arr[right]".to_string());
        self.text_section.push("    mov [r12], r11  ; arr[right] = temp".to_string());
        
        // Incrementar left, decrementar right
        self.text_section.push("    inc rdx  ; left++".to_string());
        self.text_section.push("    dec r8  ; right--".to_string());
        self.text_section.push("    jmp .reverse_loop".to_string());
        
        self.text_section.push(".reverse_done:".to_string());
        self.text_section.push("    mov rax, 0  ; éxito".to_string());
        
        // Epilogue ABI-safe
        self.generate_abi_epilogue(false);
        self.text_section.push("".to_string());
        }
        
        // array_insert: Insertar elemento en posición específica
        if deps.should_generate("array_insert") {
        // Parámetros: RCX = puntero al Array, RDX = índice, R8 = valor
        // Retorna: RAX = 0 (éxito) o -1 (error: índice fuera de rango)
        self.text_section.push("array_insert:".to_string());
        self.generate_abi_prologue(true);  // Necesita shadow space para VirtualAlloc/VirtualFree
        self.text_section.push("    mov r12, rcx  ; preservar puntero al Array".to_string());
        self.text_section.push("    mov r13, rdx  ; preservar índice".to_string());
        self.text_section.push("    mov r14, r8  ; preservar valor".to_string());
        
        // Verificar bounds: índice debe estar entre 0 y length (inclusive)
        self.text_section.push("    ; Verificar bounds".to_string());
        self.text_section.push("    mov rax, [r12 + 8]  ; length".to_string());
        self.text_section.push("    cmp r13, rax  ; comparar índice con length".to_string());
        self.text_section.push("    jg .insert_error  ; si índice > length, error".to_string());
        
        // Verificar si necesita realloc
        self.text_section.push("    ; Verificar si necesita realloc".to_string());
        self.text_section.push("    cmp rax, [r12 + 16]  ; comparar length con capacity".to_string());
        self.text_section.push("    jl .no_realloc_insert".to_string());
        
        // Realloc: duplicar capacity
        self.text_section.push("    ; Realloc: duplicar capacity".to_string());
        self.text_section.push("    mov rax, [r12 + 16]  ; capacity actual".to_string());
        self.text_section.push("    shl rax, 1  ; capacity * 2".to_string());
        self.text_section.push("    mov [r12 + 16], rax  ; actualizar capacity".to_string());
        self.text_section.push("    mov r15, rax  ; preservar nueva capacity".to_string());
        self.text_section.push("    shl r15, 3  ; capacity * 8 bytes".to_string());
        
        // VirtualAlloc nuevo bloque
        self.text_section.push("    ; VirtualAlloc nuevo bloque".to_string());
        self.ensure_stack_alignment_before_call("VirtualAlloc");
        self.text_section.push("    mov rcx, 0".to_string());
        self.text_section.push("    mov rdx, r15  ; nuevo size".to_string());
        self.text_section.push("    mov r8, 0x1000".to_string());
        self.text_section.push("    mov r9, 0x04".to_string());
        self.text_section.push("    call VirtualAlloc".to_string());
        self.text_section.push("    mov rdi, rax  ; preservar nuevo puntero (rdi preservado)".to_string());
        
        // Copiar datos antiguos hasta el índice
        self.text_section.push("    ; Copiar datos antiguos hasta el índice".to_string());
        self.text_section.push("    mov rsi, [r12 + 0]  ; fuente (antiguo, rsi preservado)".to_string());
        self.text_section.push("    mov rcx, r13  ; contador (índice)".to_string());
        self.text_section.push("    test rcx, rcx".to_string());
        self.text_section.push("    jz .copy_done_insert1".to_string());
        self.text_section.push(".copy_loop_insert1:".to_string());
        self.text_section.push("    mov rax, [rsi]".to_string());
        self.text_section.push("    mov [rdi], rax".to_string());
        self.text_section.push("    add rsi, 8".to_string());
        self.text_section.push("    add rdi, 8".to_string());
        self.text_section.push("    dec rcx".to_string());
        self.text_section.push("    jnz .copy_loop_insert1".to_string());
        self.text_section.push(".copy_done_insert1:".to_string());
        
        // Insertar nuevo valor
        self.text_section.push("    ; Insertar nuevo valor".to_string());
        self.text_section.push("    mov [rdi], r14  ; insertar valor".to_string());
        self.text_section.push("    add rdi, 8".to_string());
        
        // Copiar datos restantes después del índice
        self.text_section.push("    ; Copiar datos restantes después del índice".to_string());
        self.text_section.push("    mov rax, [r12 + 8]  ; length".to_string());
        self.text_section.push("    sub rax, r13  ; elementos restantes".to_string());
        self.text_section.push("    mov rcx, rax  ; contador".to_string());
        self.text_section.push("    test rcx, rcx".to_string());
        self.text_section.push("    jz .copy_done_insert2".to_string());
        self.text_section.push(".copy_loop_insert2:".to_string());
        self.text_section.push("    mov rax, [rsi]".to_string());
        self.text_section.push("    mov [rdi], rax".to_string());
        self.text_section.push("    add rsi, 8".to_string());
        self.text_section.push("    add rdi, 8".to_string());
        self.text_section.push("    dec rcx".to_string());
        self.text_section.push("    jnz .copy_loop_insert2".to_string());
        self.text_section.push(".copy_done_insert2:".to_string());
        
        // VirtualFree bloque antiguo
        self.text_section.push("    ; VirtualFree bloque antiguo".to_string());
        self.text_section.push("    mov r15, [r12 + 0]  ; preservar puntero antiguo".to_string());
        self.ensure_stack_alignment_before_call("VirtualFree");
        self.text_section.push("    mov rcx, r15  ; lpAddress".to_string());
        self.text_section.push("    mov rdx, 0  ; dwSize (0 = liberar todo)".to_string());
        self.text_section.push("    mov r8, 0x8000  ; MEM_RELEASE".to_string());
        self.text_section.push("    call VirtualFree".to_string());
        
        // Actualizar data pointer y length
        self.text_section.push("    ; Actualizar data pointer y length".to_string());
        self.text_section.push("    mov [r12 + 0], rdi  ; data = nuevo puntero (ajustado)".to_string());
        self.text_section.push("    inc qword [r12 + 8]  ; incrementar length".to_string());
        self.text_section.push("    jmp .insert_done".to_string());
        
        // Sin realloc: solo mover elementos y insertar
        self.text_section.push(".no_realloc_insert:".to_string());
        self.text_section.push("    mov rax, [r12 + 8]  ; length".to_string());
        self.text_section.push("    mov rbx, [r12 + 0]  ; data pointer (rbx preservado)".to_string());
        
        // Mover elementos desde el índice hacia la derecha
        self.text_section.push("    ; Mover elementos hacia la derecha".to_string());
        self.text_section.push("    mov r15, rax  ; contador (length - índice)".to_string());
        self.text_section.push("    sub r15, r13".to_string());
        self.text_section.push("    test r15, r15".to_string());
        self.text_section.push("    jz .no_move_insert".to_string());
        self.text_section.push("    mov r10, rax  ; índice destino (empezar desde el final)".to_string());
        self.text_section.push("    dec r10".to_string());
        self.text_section.push("    mov r11, r10  ; índice fuente".to_string());
        self.text_section.push("    dec r11".to_string());
        self.text_section.push(".move_loop_insert:".to_string());
        self.text_section.push("    mov rax, r10".to_string());
        self.text_section.push("    shl rax, 3  ; índice * 8 bytes".to_string());
        self.text_section.push("    mov r8, rbx".to_string());
        self.text_section.push("    add r8, rax  ; dirección destino".to_string());
        self.text_section.push("    mov rax, r11".to_string());
        self.text_section.push("    shl rax, 3  ; índice * 8 bytes".to_string());
        self.text_section.push("    mov r9, rbx".to_string());
        self.text_section.push("    add r9, rax  ; dirección fuente".to_string());
        self.text_section.push("    mov rax, [r9]  ; cargar valor fuente".to_string());
        self.text_section.push("    mov [r8], rax  ; guardar en destino".to_string());
        self.text_section.push("    dec r10".to_string());
        self.text_section.push("    dec r11".to_string());
        self.text_section.push("    dec r15".to_string());
        self.text_section.push("    jnz .move_loop_insert".to_string());
        self.text_section.push(".no_move_insert:".to_string());
        
        // Insertar valor en la posición
        self.text_section.push("    ; Insertar valor en la posición".to_string());
        self.text_section.push("    mov rax, r13  ; índice".to_string());
        self.text_section.push("    shl rax, 3  ; índice * 8 bytes".to_string());
        self.text_section.push("    add rbx, rax  ; dirección del nuevo elemento".to_string());
        self.text_section.push("    mov [rbx], r14  ; guardar valor".to_string());
        self.text_section.push("    inc qword [r12 + 8]  ; incrementar length".to_string());
        
        self.text_section.push(".insert_done:".to_string());
        self.text_section.push("    mov rax, 0  ; éxito".to_string());
        self.generate_abi_epilogue(true);
        self.text_section.push("".to_string());
        
        self.text_section.push(".insert_error:".to_string());
        self.text_section.push("    ; Error: índice fuera de rango".to_string());
        self.text_section.push("    mov rax, -1  ; código de error: -1 (índice fuera de rango)".to_string());
        self.generate_abi_epilogue(true);
        self.text_section.push("".to_string());
        }
        
        // array_remove: Eliminar primera ocurrencia de valor
        if deps.should_generate("array_remove") {
        // Parámetros: RCX = puntero al Array, RDX = valor
        // Retorna: RAX = 0 (éxito) o -3 (error: valor no encontrado)
        // ERROR CONVENTION: Void functions usan códigos negativos (-3 = valor no encontrado)
        self.text_section.push("array_remove:".to_string());
        self.generate_abi_prologue(false);  // No necesita shadow space (solo operaciones internas)
        self.text_section.push("    mov r12, rcx  ; preservar puntero al Array".to_string());
        self.text_section.push("    mov r13, rdx  ; preservar valor".to_string());
        
        // Buscar el valor en el array
        self.text_section.push("    ; Buscar el valor en el array".to_string());
        self.text_section.push("    mov rax, [r12 + 8]  ; length".to_string());
        self.text_section.push("    test rax, rax".to_string());
        self.text_section.push("    jz .remove_error  ; si length == 0, error".to_string());
        self.text_section.push("    mov r8, [r12 + 0]  ; data pointer".to_string());
        self.text_section.push("    mov r9, 0  ; índice actual".to_string());
        self.text_section.push(".remove_search_loop:".to_string());
        self.text_section.push("    cmp r9, rax  ; comparar índice con length".to_string());
        self.text_section.push("    jge .remove_error  ; si índice >= length, no encontrado".to_string());
        self.text_section.push("    mov r10, r9".to_string());
        self.text_section.push("    shl r10, 3  ; índice * 8 bytes".to_string());
        self.text_section.push("    mov r11, r8".to_string());
        self.text_section.push("    add r11, r10  ; dirección del elemento".to_string());
        self.text_section.push("    cmp [r11], r13  ; comparar elemento con valor".to_string());
        self.text_section.push("    je .remove_found  ; si encontrado, salir".to_string());
        self.text_section.push("    inc r9  ; siguiente índice".to_string());
        self.text_section.push("    jmp .remove_search_loop".to_string());
        
        // Encontrado: mover elementos hacia la izquierda
        self.text_section.push(".remove_found:".to_string());
        self.text_section.push("    ; Mover elementos hacia la izquierda".to_string());
        self.text_section.push("    push rax  ; guardar length original (rax será usado en el loop)".to_string());
        self.text_section.push("    mov r10, r9  ; índice del elemento a eliminar".to_string());
        self.text_section.push("    inc r10  ; índice siguiente".to_string());
        self.text_section.push("    mov r11, rax  ; length".to_string());
        self.text_section.push("    dec r11  ; nuevo length".to_string());
        self.text_section.push("    cmp r10, rax  ; si índice siguiente >= length, solo decrementar".to_string());
        self.text_section.push("    jge .remove_decrement".to_string());
        self.text_section.push(".remove_move_loop:".to_string());
        self.text_section.push("    mov r12, r10".to_string());
        self.text_section.push("    shl r12, 3  ; índice * 8 bytes".to_string());
        self.text_section.push("    mov r13, r8".to_string());
        self.text_section.push("    add r13, r12  ; dirección fuente".to_string());
        self.text_section.push("    mov r14, [r13]  ; cargar valor fuente (usar r14 en lugar de rax)".to_string());
        self.text_section.push("    mov r12, r9".to_string());
        self.text_section.push("    shl r12, 3  ; índice * 8 bytes".to_string());
        self.text_section.push("    mov r13, r8".to_string());
        self.text_section.push("    add r13, r12  ; dirección destino".to_string());
        self.text_section.push("    mov [r13], r14  ; guardar valor".to_string());
        self.text_section.push("    inc r9  ; siguiente índice destino".to_string());
        self.text_section.push("    inc r10  ; siguiente índice fuente".to_string());
        self.text_section.push("    pop rax  ; restaurar length original para comparación".to_string());
        self.text_section.push("    cmp r10, rax  ; comparar con length original".to_string());
        self.text_section.push("    push rax  ; guardar length de nuevo para siguiente iteración".to_string());
        self.text_section.push("    jl .remove_move_loop".to_string());
        self.text_section.push("    pop rax  ; limpiar stack (length ya no se necesita)".to_string());
        self.text_section.push(".remove_decrement:".to_string());
        self.text_section.push("    pop rax  ; limpiar length del stack si estaba ahí".to_string());
        self.text_section.push("    dec qword [r12 + 8]  ; decrementar length".to_string());
        self.text_section.push("    jmp .remove_done".to_string());
        
        self.text_section.push(".remove_error:".to_string());
        self.text_section.push("    ; Error: valor no encontrado".to_string());
        self.text_section.push("    mov rax, -3  ; código de error: -3 (valor no encontrado)".to_string());
        self.generate_abi_epilogue(false);
        self.text_section.push("".to_string());
        
        self.text_section.push(".remove_done:".to_string());
        self.text_section.push("    mov rax, 0  ; éxito".to_string());
        self.generate_abi_epilogue(false);
        self.text_section.push("".to_string());
        }
        
        // array_index: Encontrar índice de valor
        if deps.should_generate("array_index") {
        // Parámetros: RCX = puntero al Array, RDX = valor
        // Retorna: RAX = índice (o -1 si no encontrado)
        self.text_section.push("array_index:".to_string());
        self.generate_abi_prologue(false);  // No necesita shadow space (solo lectura)
        
        // Buscar el valor en el array
        self.text_section.push("    ; Buscar el valor en el array".to_string());
        self.text_section.push("    mov r12, rcx  ; preservar puntero al Array".to_string());
        self.text_section.push("    mov r13, rdx  ; preservar valor".to_string());
        self.text_section.push("    mov rax, [r12 + 8]  ; length".to_string());
        self.text_section.push("    test rax, rax".to_string());
        self.text_section.push("    jz .index_not_found  ; si length == 0, no encontrado".to_string());
        self.text_section.push("    mov r8, [r12 + 0]  ; data pointer".to_string());
        self.text_section.push("    mov r9, 0  ; índice actual".to_string());
        self.text_section.push(".index_search_loop:".to_string());
        self.text_section.push("    cmp r9, rax  ; comparar índice con length".to_string());
        self.text_section.push("    jge .index_not_found  ; si índice >= length, no encontrado".to_string());
        self.text_section.push("    mov r10, r9".to_string());
        self.text_section.push("    shl r10, 3  ; índice * 8 bytes".to_string());
        self.text_section.push("    mov r11, r8".to_string());
        self.text_section.push("    add r11, r10  ; dirección del elemento".to_string());
        self.text_section.push("    cmp [r11], r13  ; comparar elemento con valor".to_string());
        self.text_section.push("    je .index_found  ; si encontrado, salir".to_string());
        self.text_section.push("    inc r9  ; siguiente índice".to_string());
        self.text_section.push("    jmp .index_search_loop".to_string());
        
        self.text_section.push(".index_found:".to_string());
        self.text_section.push("    mov rax, r9  ; retornar índice".to_string());
        self.generate_abi_epilogue(false);
        self.text_section.push("".to_string());
        
        self.text_section.push(".index_not_found:".to_string());
        self.text_section.push("    mov rax, -1  ; retornar -1 (no encontrado)".to_string());
        self.generate_abi_epilogue(false);
        self.text_section.push("".to_string());
        }
        
        // array_count: Contar ocurrencias de valor
        if deps.should_generate("array_count") {
        // Parámetros: RCX = puntero al Array, RDX = valor
        // Retorna: RAX = conteo
        self.text_section.push("array_count:".to_string());
        self.generate_abi_prologue(false);  // No necesita shadow space (solo lectura)
        
        // Contar ocurrencias
        self.text_section.push("    ; Contar ocurrencias".to_string());
        self.text_section.push("    mov r12, rcx  ; preservar puntero al Array".to_string());
        self.text_section.push("    mov r13, rdx  ; preservar valor".to_string());
        self.text_section.push("    mov rax, [r12 + 8]  ; length".to_string());
        self.text_section.push("    test rax, rax".to_string());
        self.text_section.push("    jz .count_zero  ; si length == 0, retornar 0".to_string());
        self.text_section.push("    mov r8, [r12 + 0]  ; data pointer".to_string());
        self.text_section.push("    mov r9, 0  ; índice actual".to_string());
        self.text_section.push("    mov r10, 0  ; contador".to_string());
        self.text_section.push(".count_loop:".to_string());
        self.text_section.push("    cmp r9, rax  ; comparar índice con length".to_string());
        self.text_section.push("    jge .count_done  ; si índice >= length, terminar".to_string());
        self.text_section.push("    mov r11, r9".to_string());
        self.text_section.push("    shl r11, 3  ; índice * 8 bytes".to_string());
        self.text_section.push("    mov r14, r8".to_string());
        self.text_section.push("    add r14, r11  ; dirección del elemento".to_string());
        self.text_section.push("    cmp [r14], r13  ; comparar elemento con valor".to_string());
        self.text_section.push("    jne .count_next  ; si no coincide, siguiente".to_string());
        self.text_section.push("    inc r10  ; incrementar contador".to_string());
        self.text_section.push(".count_next:".to_string());
        self.text_section.push("    inc r9  ; siguiente índice".to_string());
        self.text_section.push("    jmp .count_loop".to_string());
        
        self.text_section.push(".count_done:".to_string());
        self.text_section.push("    mov rax, r10  ; retornar conteo".to_string());
        self.generate_abi_epilogue(false);
        self.text_section.push("".to_string());
        
        self.text_section.push(".count_zero:".to_string());
        self.text_section.push("    mov rax, 0  ; retornar 0".to_string());
        self.generate_abi_epilogue(false);
        self.text_section.push("".to_string());
        }
        
        // array_sort: Ordenar array
        if deps.should_generate("array_sort") {
        // Parámetros: RCX = puntero al Array
        // Retorna: RAX = 0 (éxito, siempre exitoso)
        // OPTIMIZATION: Usa bubble sort (placeholder, no optimizado)
        // TODO: Implementar quicksort o mergesort para mejor rendimiento
        self.text_section.push("array_sort:".to_string());
        self.generate_abi_prologue(false);  // No necesita shadow space (solo operaciones internas)
        
        // Verificar si necesita ordenar
        self.text_section.push("    ; Verificar si necesita ordenar".to_string());
        self.text_section.push("    mov r12, rcx  ; preservar puntero al Array".to_string());
        self.text_section.push("    mov rax, [r12 + 8]  ; length".to_string());
        self.text_section.push("    cmp rax, 1".to_string());
        self.text_section.push("    jle .sort_done  ; si length <= 1, no hacer nada".to_string());
        
        // Bubble sort
        self.text_section.push("    ; Bubble sort".to_string());
        self.text_section.push("    mov r8, [r12 + 0]  ; data pointer (preservado)".to_string());
        self.text_section.push("    mov r9, 0  ; i (outer loop)".to_string());
        self.text_section.push(".sort_outer:".to_string());
        self.text_section.push("    mov r10, rax  ; length".to_string());
        self.text_section.push("    dec r10  ; length - 1".to_string());
        self.text_section.push("    cmp r9, r10  ; comparar i con length - 1".to_string());
        self.text_section.push("    jge .sort_done  ; si i >= length - 1, terminar".to_string());
        self.text_section.push("    mov r11, 0  ; j (inner loop)".to_string());
        self.text_section.push("    mov r12, rax  ; length".to_string());
        self.text_section.push("    sub r12, r9  ; length - i".to_string());
        self.text_section.push("    dec r12  ; length - i - 1".to_string());
        self.text_section.push(".sort_inner:".to_string());
        self.text_section.push("    cmp r11, r12  ; comparar j con length - i - 1".to_string());
        self.text_section.push("    jge .sort_inner_done  ; si j >= length - i - 1, siguiente i".to_string());
        self.text_section.push("    ; Comparar arr[j] y arr[j+1]".to_string());
        self.text_section.push("    mov r13, r11".to_string());
        self.text_section.push("    shl r13, 3  ; j * 8 bytes".to_string());
        self.text_section.push("    mov r14, r8".to_string());
        self.text_section.push("    add r14, r13  ; dirección de arr[j]".to_string());
        self.text_section.push("    mov r15, [r14]  ; arr[j]".to_string());
        self.text_section.push("    mov r13, r11".to_string());
        self.text_section.push("    inc r13".to_string());
        self.text_section.push("    shl r13, 3  ; (j+1) * 8 bytes".to_string());
        self.text_section.push("    mov r14, r8".to_string());
        self.text_section.push("    add r14, r13  ; dirección de arr[j+1]".to_string());
        self.text_section.push("    mov r13, [r14]  ; arr[j+1]".to_string());
        self.text_section.push("    cmp r15, r13  ; comparar arr[j] con arr[j+1]".to_string());
        self.text_section.push("    jle .sort_no_swap  ; si arr[j] <= arr[j+1], no intercambiar".to_string());
        self.text_section.push("    ; Intercambiar arr[j] y arr[j+1]".to_string());
        self.text_section.push("    mov [r14], r15  ; arr[j+1] = arr[j]".to_string());
        self.text_section.push("    mov r13, r11".to_string());
        self.text_section.push("    shl r13, 3  ; j * 8 bytes".to_string());
        self.text_section.push("    mov r14, r8".to_string());
        self.text_section.push("    add r14, r13  ; dirección de arr[j]".to_string());
        self.text_section.push("    mov r13, r11".to_string());
        self.text_section.push("    inc r13".to_string());
        self.text_section.push("    shl r13, 3  ; (j+1) * 8 bytes".to_string());
        self.text_section.push("    mov r15, r8".to_string());
        self.text_section.push("    add r15, r13  ; dirección de arr[j+1]".to_string());
        self.text_section.push("    mov r13, [r15]  ; cargar arr[j+1] nuevamente".to_string());
        self.text_section.push("    mov [r14], r13  ; arr[j] = arr[j+1]".to_string());
        self.text_section.push(".sort_no_swap:".to_string());
        self.text_section.push("    inc r11  ; j++".to_string());
        self.text_section.push("    jmp .sort_inner".to_string());
        self.text_section.push(".sort_inner_done:".to_string());
        self.text_section.push("    inc r9  ; i++".to_string());
        self.text_section.push("    jmp .sort_outer".to_string());
        
        self.text_section.push(".sort_done:".to_string());
        self.text_section.push("    mov rax, 0  ; éxito".to_string());
        self.generate_abi_epilogue(false);
        self.text_section.push("".to_string());
        }
        
        // array_free: Liberar memoria de un Array
        if deps.should_generate("array_free") {
        // Parámetros: RCX = puntero al Array
        // Retorna: RAX = 0 (éxito) o -4 (error: puntero inválido)
        // Nota: Libera tanto el Array struct como su data buffer
        //       Liberar NULL es seguro (no-op, retorna 0)
        self.text_section.push("array_free:".to_string());
        self.generate_abi_prologue(true);  // Necesita shadow space para VirtualFree
        self.text_section.push("    mov r12, rcx  ; preservar puntero al Array".to_string());
        
        // Verificar si el puntero es NULL (liberar NULL es seguro, no-op)
        self.text_section.push("    ; Verificar si el puntero es NULL".to_string());
        self.text_section.push("    test r12, r12  ; verificar si Array* es NULL".to_string());
        self.text_section.push("    jz .free_array_done  ; si es NULL, retornar éxito (no-op)".to_string());
        
        // Liberar data buffer primero
        self.text_section.push("    ; Liberar data buffer".to_string());
        self.text_section.push("    mov rcx, [r12 + 0]  ; data pointer".to_string());
        self.text_section.push("    test rcx, rcx  ; verificar si es NULL".to_string());
        self.text_section.push("    jz .free_array_struct  ; si es NULL, saltar".to_string());
        self.ensure_stack_alignment_before_call("VirtualFree (data)");
        self.text_section.push("    mov rdx, 0  ; dwSize (0 = liberar todo)".to_string());
        self.text_section.push("    mov r8, 0x8000  ; MEM_RELEASE".to_string());
        self.text_section.push("    call VirtualFree".to_string());
        
        // Liberar Array struct
        self.text_section.push(".free_array_struct:".to_string());
        self.text_section.push("    ; Liberar Array struct".to_string());
        self.ensure_stack_alignment_before_call("VirtualFree (Array struct)");
        self.text_section.push("    mov rcx, r12  ; puntero al Array struct".to_string());
        self.text_section.push("    mov rdx, 0  ; dwSize (0 = liberar todo)".to_string());
        self.text_section.push("    mov r8, 0x8000  ; MEM_RELEASE".to_string());
        self.text_section.push("    call VirtualFree".to_string());
        
        // Retornar éxito
        self.text_section.push(".free_array_done:".to_string());
        self.text_section.push("    mov rax, 0  ; éxito".to_string());
        self.generate_abi_epilogue(true);
        self.text_section.push("".to_string());
        }
    }

    /// Generar funciones helper de String en NASM (versión completa - todas las funciones)
    /// Estructura String: [data: qword, length: qword, capacity: qword, hash: qword]
    /// Total: 32 bytes (4 qwords)
    fn generate_string_helpers_nasm(&mut self) {
        // Llamar a la versión selectiva con un dependency graph que incluye todo
        let mut deps = DependencyGraph::new();
        // Marcar todas las funciones de strings como usadas
        deps.mark_used("string_new");
        deps.mark_used("string_from_literal");
        deps.mark_used("string_len");
        deps.mark_used("string_concat");
        deps.mark_used("string_slice");
        deps.mark_used("string_upper");
        deps.mark_used("string_lower");
        deps.mark_used("string_free");
        self.generate_string_helpers_nasm_selective(&deps);
    }
    
    /// Generar funciones helper de String en NASM (versión selectiva - solo funciones usadas)
    /// Estructura String: [data: qword, length: qword, capacity: qword, hash: qword]
    /// Total: 32 bytes (4 qwords)
    fn generate_string_helpers_nasm_selective(&mut self, deps: &DependencyGraph) {
        // ============================================
        // Estructura String en NASM (32 bytes):
        // - [rax + 0]  : data (qword) - puntero a memoria dinámica (char*)
        // - [rax + 8]  : length (qword) - número de caracteres
        // - [rax + 16] : capacity (qword) - capacidad total
        // - [rax + 24] : hash (qword) - hash cacheado (0 = no calculado)
        // ============================================
        
        // string_new: Crear string vacío
        // Retorna: RAX = puntero al String (en heap)
        if deps.should_generate("string_new") {
        self.text_section.push("string_new:".to_string());
        self.generate_abi_prologue(true);  // Necesita shadow space para VirtualAlloc
        
        // Allocar memoria para String struct (32 bytes)
        self.text_section.push("    ; Allocar memoria para String struct (32 bytes)".to_string());
        self.ensure_stack_alignment_before_call("VirtualAlloc (String struct)");
        self.text_section.push("    mov rcx, 0  ; lpAddress (NULL = auto)".to_string());
        self.text_section.push("    mov rdx, 32  ; dwSize (32 bytes para String struct)".to_string());
        self.text_section.push("    mov r8, 0x1000  ; flAllocationType (MEM_COMMIT)".to_string());
        self.text_section.push("    mov r9, 0x04  ; flProtect (PAGE_READWRITE)".to_string());
        self.text_section.push("    call VirtualAlloc".to_string());
        self.text_section.push("    mov r12, rax  ; preservar puntero al String".to_string());
        
        // Inicializar String: length=0, capacity=16, data=NULL (se asignará después), hash=0
        self.text_section.push("    ; Inicializar String".to_string());
        self.text_section.push("    mov qword [r12 + 0], 0  ; data = NULL (se asignará después)".to_string());
        self.text_section.push("    mov qword [r12 + 8], 0  ; length = 0".to_string());
        self.text_section.push("    mov qword [r12 + 16], 16  ; capacity = 16".to_string());
        self.text_section.push("    mov qword [r12 + 24], 0  ; hash = 0 (no calculado)".to_string());
        
        // Allocar memoria para data (capacity bytes)
        self.text_section.push("    ; Allocar memoria para data (16 bytes)".to_string());
        self.ensure_stack_alignment_before_call("VirtualAlloc (data)");
        self.text_section.push("    mov rcx, 0  ; lpAddress".to_string());
        self.text_section.push("    mov rdx, 16  ; dwSize (16 bytes)".to_string());
        self.text_section.push("    mov r8, 0x1000  ; MEM_COMMIT".to_string());
        self.text_section.push("    mov r9, 0x04  ; PAGE_READWRITE".to_string());
        self.text_section.push("    call VirtualAlloc".to_string());
        
        // Asignar data al String
        self.text_section.push("    mov [r12 + 0], rax  ; data = puntero a memoria".to_string());
        self.text_section.push("    mov byte [rax], 0  ; null terminator".to_string());
        self.text_section.push("    mov rax, r12  ; retornar puntero al String".to_string());
        
        // Epilogue ABI-safe
        self.generate_abi_epilogue(true);
        self.text_section.push("".to_string());
        }
        
        // string_from_literal: Crear string desde literal
        // Parámetros: RCX = puntero a literal (char*, null-terminated), RDX = longitud
        // Retorna: RAX = puntero al String (en heap)
        if deps.should_generate("string_from_literal") {
        self.text_section.push("string_from_literal:".to_string());
        self.generate_abi_prologue(true);  // Necesita shadow space para VirtualAlloc
        self.text_section.push("    mov r12, rcx  ; preservar puntero a literal".to_string());
        self.text_section.push("    mov r13, rdx  ; preservar longitud".to_string());
        
        // Calcular capacity: max(length + 1, 16) (length + 1 para null terminator)
        self.text_section.push("    ; Calcular capacity: max(length + 1, 16)".to_string());
        self.text_section.push("    mov rax, r13  ; longitud".to_string());
        self.text_section.push("    inc rax  ; length + 1 (para null terminator)".to_string());
        self.text_section.push("    cmp rax, 16".to_string());
        self.text_section.push("    jge .capacity_ok_string".to_string());
        self.text_section.push("    mov rax, 16  ; mínimo 16".to_string());
        self.text_section.push(".capacity_ok_string:".to_string());
        self.text_section.push("    mov r14, rax  ; preservar capacity".to_string());
        
        // Allocar memoria para String struct (32 bytes)
        self.text_section.push("    ; Allocar memoria para String struct".to_string());
        self.ensure_stack_alignment_before_call("VirtualAlloc (String struct)");
        self.text_section.push("    mov rcx, 0".to_string());
        self.text_section.push("    mov rdx, 32".to_string());
        self.text_section.push("    mov r8, 0x1000".to_string());
        self.text_section.push("    mov r9, 0x04".to_string());
        self.text_section.push("    call VirtualAlloc".to_string());
        self.text_section.push("    mov r15, rax  ; preservar puntero al String".to_string());
        
        // Allocar memoria para data (capacity bytes)
        self.text_section.push("    ; Allocar memoria para data".to_string());
        self.ensure_stack_alignment_before_call("VirtualAlloc (data)");
        self.text_section.push("    mov rcx, 0".to_string());
        self.text_section.push("    mov rdx, r14  ; capacity".to_string());
        self.text_section.push("    mov r8, 0x1000".to_string());
        self.text_section.push("    mov r9, 0x04".to_string());
        self.text_section.push("    call VirtualAlloc".to_string());
        self.text_section.push("    mov rdi, rax  ; preservar puntero a data (rdi preservado)".to_string());
        
        // Configurar String struct
        self.text_section.push("    ; Configurar String struct".to_string());
        self.text_section.push("    mov [r15 + 0], rdi  ; data = puntero".to_string());
        self.text_section.push("    mov [r15 + 8], r13  ; length".to_string());
        self.text_section.push("    mov [r15 + 16], r14  ; capacity".to_string());
        self.text_section.push("    mov qword [r15 + 24], 0  ; hash = 0".to_string());
        
        // Loop para copiar caracteres
        self.text_section.push("    ; Loop para copiar caracteres".to_string());
        self.text_section.push("    mov rcx, r13  ; longitud".to_string());
        self.text_section.push("    mov rsi, r12  ; puntero a literal fuente (rsi preservado)".to_string());
        self.text_section.push("    test rcx, rcx".to_string());
        self.text_section.push("    jz .copy_done_string".to_string());
        self.text_section.push(".copy_loop_string:".to_string());
        self.text_section.push("    mov al, [rsi]  ; cargar byte fuente".to_string());
        self.text_section.push("    mov [rdi], al  ; guardar en destino".to_string());
        self.text_section.push("    inc rsi  ; siguiente byte fuente".to_string());
        self.text_section.push("    inc rdi  ; siguiente byte destino".to_string());
        self.text_section.push("    dec rcx".to_string());
        self.text_section.push("    jnz .copy_loop_string".to_string());
        self.text_section.push(".copy_done_string:".to_string());
        self.text_section.push("    mov byte [rdi], 0  ; null terminator".to_string());
        self.text_section.push("    mov rax, r15  ; retornar puntero al String".to_string());
        
        // Epilogue ABI-safe
        self.generate_abi_epilogue(true);
        self.text_section.push("".to_string());
        }
        
        // string_len: Obtener longitud del string
        if deps.should_generate("string_len") {
        // Parámetros: RCX = puntero al String
        // Retorna: RAX = longitud
        // Nota: Función muy simple, no necesita prologue/epilogue completo
        // pero debemos preservar registros según ABI (RCX es caller-saved)
        self.text_section.push("string_len:".to_string());
        self.text_section.push("    mov rax, [rcx + 8]  ; cargar length".to_string());
        self.text_section.push("    ret  ; RCX es caller-saved, no necesitamos preservarlo".to_string());
        self.text_section.push("".to_string());
        }
        
        // string_concat: Concatenar dos strings
        if deps.should_generate("string_concat") {
        // Parámetros: RCX = puntero al String 1, RDX = puntero al String 2
        // Retorna: RAX = puntero al nuevo String (concatenado)
        self.text_section.push("string_concat:".to_string());
        self.generate_abi_prologue(true);  // Necesita shadow space para VirtualAlloc
        self.text_section.push("    mov r12, rcx  ; preservar String 1".to_string());
        self.text_section.push("    mov r13, rdx  ; preservar String 2".to_string());
        
        // Calcular nueva longitud: len1 + len2
        self.text_section.push("    ; Calcular nueva longitud: len1 + len2".to_string());
        self.text_section.push("    mov rax, [r12 + 8]  ; length1".to_string());
        self.text_section.push("    add rax, [r13 + 8]  ; length2".to_string());
        self.text_section.push("    mov r14, rax  ; preservar nueva longitud".to_string());
        
        // Calcular nueva capacity: max((len1 + len2 + 1) * 2, 16)
        self.text_section.push("    ; Calcular nueva capacity".to_string());
        self.text_section.push("    inc rax  ; +1 para null terminator".to_string());
        self.text_section.push("    shl rax, 1  ; * 2".to_string());
        self.text_section.push("    cmp rax, 16".to_string());
        self.text_section.push("    jge .capacity_ok_concat".to_string());
        self.text_section.push("    mov rax, 16  ; mínimo 16".to_string());
        self.text_section.push(".capacity_ok_concat:".to_string());
        self.text_section.push("    mov r15, rax  ; preservar capacity".to_string());
        
        // Allocar memoria para nuevo String struct (32 bytes)
        self.text_section.push("    ; Allocar memoria para nuevo String struct".to_string());
        self.ensure_stack_alignment_before_call("VirtualAlloc (String struct)");
        self.text_section.push("    mov rcx, 0".to_string());
        self.text_section.push("    mov rdx, 32".to_string());
        self.text_section.push("    mov r8, 0x1000".to_string());
        self.text_section.push("    mov r9, 0x04".to_string());
        self.text_section.push("    call VirtualAlloc".to_string());
        self.text_section.push("    mov rbx, rax  ; preservar puntero al nuevo String (rbx preservado)".to_string());
        
        // Allocar memoria para data
        self.text_section.push("    ; Allocar memoria para data".to_string());
        self.ensure_stack_alignment_before_call("VirtualAlloc (data)");
        self.text_section.push("    mov rcx, 0".to_string());
        self.text_section.push("    mov rdx, r15  ; capacity".to_string());
        self.text_section.push("    mov r8, 0x1000".to_string());
        self.text_section.push("    mov r9, 0x04".to_string());
        self.text_section.push("    call VirtualAlloc".to_string());
        self.text_section.push("    mov rdi, rax  ; preservar puntero a data (rdi preservado)".to_string());
        
        // Configurar String struct
        self.text_section.push("    ; Configurar String struct".to_string());
        self.text_section.push("    mov [rbx + 0], rdi  ; data = puntero".to_string());
        self.text_section.push("    mov [rbx + 8], r14  ; length".to_string());
        self.text_section.push("    mov [rbx + 16], r15  ; capacity".to_string());
        self.text_section.push("    mov qword [rbx + 24], 0  ; hash = 0".to_string());
        
        // Copiar String1->data
        self.text_section.push("    ; Copiar String1->data".to_string());
        self.text_section.push("    mov rsi, [r12 + 0]  ; fuente (String1->data, rsi preservado)".to_string());
        self.text_section.push("    mov rcx, [r12 + 8]  ; length1".to_string());
        self.text_section.push("    test rcx, rcx".to_string());
        self.text_section.push("    jz .copy_string2".to_string());
        self.text_section.push(".copy_loop_concat1:".to_string());
        self.text_section.push("    mov al, [rsi]".to_string());
        self.text_section.push("    mov [rdi], al".to_string());
        self.text_section.push("    inc rsi".to_string());
        self.text_section.push("    inc rdi".to_string());
        self.text_section.push("    dec rcx".to_string());
        self.text_section.push("    jnz .copy_loop_concat1".to_string());
        
        // Copiar String2->data
        self.text_section.push(".copy_string2:".to_string());
        self.text_section.push("    ; Copiar String2->data".to_string());
        self.text_section.push("    mov rsi, [r13 + 0]  ; fuente (String2->data)".to_string());
        self.text_section.push("    mov rcx, [r13 + 8]  ; length2".to_string());
        self.text_section.push("    test rcx, rcx".to_string());
        self.text_section.push("    jz .concat_done".to_string());
        self.text_section.push(".copy_loop_concat2:".to_string());
        self.text_section.push("    mov al, [rsi]".to_string());
        self.text_section.push("    mov [rdi], al".to_string());
        self.text_section.push("    inc rsi".to_string());
        self.text_section.push("    inc rdi".to_string());
        self.text_section.push("    dec rcx".to_string());
        self.text_section.push("    jnz .copy_loop_concat2".to_string());
        
        self.text_section.push(".concat_done:".to_string());
        self.text_section.push("    mov byte [rdi], 0  ; null terminator".to_string());
        self.text_section.push("    mov rax, rbx  ; retornar puntero al nuevo String".to_string());
        
        // Epilogue ABI-safe
        self.generate_abi_epilogue(true);
        self.text_section.push("".to_string());
        }
        
        // string_slice: Obtener slice de string
        if deps.should_generate("string_slice") {
        // Parámetros: RCX = puntero al String, RDX = índice inicio, R8 = índice fin (exclusivo)
        // Retorna: RAX = puntero al nuevo String (slice), o NULL (0) si error (índices inválidos)
        self.text_section.push("string_slice:".to_string());
        self.generate_abi_prologue(true);  // Necesita shadow space para VirtualAlloc
        self.text_section.push("    mov r12, rcx  ; preservar String".to_string());
        self.text_section.push("    mov r13, rdx  ; preservar start".to_string());
        self.text_section.push("    mov r14, r8  ; preservar end".to_string());
        
        // Bounds checking
        self.text_section.push("    ; Bounds checking".to_string());
        self.text_section.push("    mov rax, [r12 + 8]  ; length".to_string());
        self.text_section.push("    cmp r13, rax  ; start >= length?".to_string());
        self.text_section.push("    jge .slice_error".to_string());
        self.text_section.push("    cmp r14, rax  ; end > length?".to_string());
        self.text_section.push("    jg .slice_error".to_string());
        self.text_section.push("    cmp r13, r14  ; start >= end?".to_string());
        self.text_section.push("    jge .slice_error".to_string());
        
        // Calcular longitud: end - start
        self.text_section.push("    ; Calcular longitud: end - start".to_string());
        self.text_section.push("    mov rax, r14  ; end".to_string());
        self.text_section.push("    sub rax, r13  ; end - start".to_string());
        self.text_section.push("    mov r15, rax  ; preservar nueva longitud".to_string());
        
        // Calcular capacity: max((length + 1) * 2, 16)
        self.text_section.push("    ; Calcular capacity".to_string());
        self.text_section.push("    inc rax  ; +1 para null terminator".to_string());
        self.text_section.push("    shl rax, 1  ; * 2".to_string());
        self.text_section.push("    cmp rax, 16".to_string());
        self.text_section.push("    jge .capacity_ok_slice".to_string());
        self.text_section.push("    mov rax, 16  ; mínimo 16".to_string());
        self.text_section.push(".capacity_ok_slice:".to_string());
        self.text_section.push("    push rax  ; guardar capacity temporalmente".to_string());
        
        // Allocar memoria para nuevo String struct
        self.text_section.push("    ; Allocar memoria para nuevo String struct".to_string());
        self.ensure_stack_alignment_before_call("VirtualAlloc (String struct)");
        self.text_section.push("    mov rcx, 0".to_string());
        self.text_section.push("    mov rdx, 32".to_string());
        self.text_section.push("    mov r8, 0x1000".to_string());
        self.text_section.push("    mov r9, 0x04".to_string());
        self.text_section.push("    call VirtualAlloc".to_string());
        self.text_section.push("    mov rbx, rax  ; preservar puntero al nuevo String (rbx preservado)".to_string());
        
        // Allocar memoria para data
        self.text_section.push("    ; Allocar memoria para data".to_string());
        self.text_section.push("    pop rdx  ; capacity".to_string());
        self.text_section.push("    push rdx  ; guardar capacity de nuevo".to_string());
        self.ensure_stack_alignment_before_call("VirtualAlloc (data)");
        self.text_section.push("    mov rcx, 0".to_string());
        self.text_section.push("    mov r8, 0x1000".to_string());
        self.text_section.push("    mov r9, 0x04".to_string());
        self.text_section.push("    call VirtualAlloc".to_string());
        self.text_section.push("    mov rdi, rax  ; preservar puntero a data (rdi preservado)".to_string());
        
        // Configurar String struct
        self.text_section.push("    ; Configurar String struct".to_string());
        self.text_section.push("    mov [rbx + 0], rdi  ; data = puntero".to_string());
        self.text_section.push("    mov [rbx + 8], r15  ; length".to_string());
        self.text_section.push("    pop rdx  ; capacity".to_string());
        self.text_section.push("    mov [rbx + 16], rdx  ; capacity".to_string());
        self.text_section.push("    mov qword [rbx + 24], 0  ; hash = 0".to_string());
        
        // Calcular dirección de inicio en String->data
        self.text_section.push("    ; Calcular dirección de inicio".to_string());
        self.text_section.push("    mov rsi, [r12 + 0]  ; String->data (rsi preservado)".to_string());
        self.text_section.push("    add rsi, r13  ; String->data + start".to_string());
        self.text_section.push("    mov rcx, r15  ; longitud".to_string());
        
        // Copiar caracteres
        self.text_section.push("    ; Copiar caracteres".to_string());
        self.text_section.push("    test rcx, rcx".to_string());
        self.text_section.push("    jz .slice_copy_done".to_string());
        self.text_section.push(".copy_loop_slice:".to_string());
        self.text_section.push("    mov al, [rsi]".to_string());
        self.text_section.push("    mov [rdi], al".to_string());
        self.text_section.push("    inc rsi".to_string());
        self.text_section.push("    inc rdi".to_string());
        self.text_section.push("    dec rcx".to_string());
        self.text_section.push("    jnz .copy_loop_slice".to_string());
        
        self.text_section.push(".slice_copy_done:".to_string());
        self.text_section.push("    mov byte [rdi], 0  ; null terminator".to_string());
        self.text_section.push("    mov rax, rbx  ; retornar puntero al nuevo String".to_string());
        
        // Epilogue ABI-safe
        self.generate_abi_epilogue(true);
        self.text_section.push("".to_string());
        
        // Error handler: retornar NULL en lugar de ExitProcess
        self.text_section.push(".slice_error:".to_string());
        self.text_section.push("    ; Error: índices inválidos".to_string());
        self.text_section.push("    mov rax, 0  ; retornar NULL (error)".to_string());
        self.generate_abi_epilogue(true);
        self.text_section.push("".to_string());
        }
        
        // string_upper: Convertir a mayúsculas
        // Parámetros: RCX = puntero al String
        // Retorna: RAX = puntero al nuevo String (mayúsculas)
        if deps.should_generate("string_upper") {
        self.text_section.push("string_upper:".to_string());
        self.generate_abi_prologue(true);  // Necesita shadow space para VirtualAlloc
        self.text_section.push("    mov r12, rcx  ; preservar String".to_string());
        
        // Obtener longitud
        self.text_section.push("    ; Obtener longitud".to_string());
        self.text_section.push("    mov rdx, [r12 + 8]  ; length".to_string());
        self.text_section.push("    mov r13, rdx  ; preservar longitud".to_string());
        
        // Calcular capacity: max((length + 1) * 2, 16)
        self.text_section.push("    ; Calcular capacity".to_string());
        self.text_section.push("    mov rax, r13".to_string());
        self.text_section.push("    inc rax  ; +1 para null terminator".to_string());
        self.text_section.push("    shl rax, 1  ; * 2".to_string());
        self.text_section.push("    cmp rax, 16".to_string());
        self.text_section.push("    jge .capacity_ok_upper".to_string());
        self.text_section.push("    mov rax, 16  ; mínimo 16".to_string());
        self.text_section.push(".capacity_ok_upper:".to_string());
        self.text_section.push("    mov r14, rax  ; preservar capacity".to_string());
        
        // Allocar memoria para nuevo String struct
        self.text_section.push("    ; Allocar memoria para nuevo String struct".to_string());
        self.ensure_stack_alignment_before_call("VirtualAlloc (String struct)");
        self.text_section.push("    mov rcx, 0".to_string());
        self.text_section.push("    mov rdx, 32".to_string());
        self.text_section.push("    mov r8, 0x1000".to_string());
        self.text_section.push("    mov r9, 0x04".to_string());
        self.text_section.push("    call VirtualAlloc".to_string());
        self.text_section.push("    mov rbx, rax  ; preservar puntero al nuevo String (rbx preservado)".to_string());
        
        // Allocar memoria para data
        self.text_section.push("    ; Allocar memoria para data".to_string());
        self.ensure_stack_alignment_before_call("VirtualAlloc (data)");
        self.text_section.push("    mov rcx, 0".to_string());
        self.text_section.push("    mov rdx, r14  ; capacity".to_string());
        self.text_section.push("    mov r8, 0x1000".to_string());
        self.text_section.push("    mov r9, 0x04".to_string());
        self.text_section.push("    call VirtualAlloc".to_string());
        self.text_section.push("    mov rdi, rax  ; preservar puntero a data (rdi preservado)".to_string());
        
        // Configurar nuevo String
        self.text_section.push("    ; Configurar nuevo String".to_string());
        self.text_section.push("    mov [rbx + 0], rdi  ; data = puntero".to_string());
        self.text_section.push("    mov [rbx + 8], r13  ; length".to_string());
        self.text_section.push("    mov [rbx + 16], r14  ; capacity".to_string());
        self.text_section.push("    mov qword [rbx + 24], 0  ; hash = 0".to_string());
        
        // Copiar y convertir a mayúsculas
        self.text_section.push("    ; Copiar y convertir a mayúsculas".to_string());
        self.text_section.push("    mov rsi, [r12 + 0]  ; fuente (String->data, rsi preservado)".to_string());
        self.text_section.push("    mov rcx, r13  ; longitud".to_string());
        self.text_section.push("    test rcx, rcx".to_string());
        self.text_section.push("    jz .upper_done".to_string());
        self.text_section.push(".copy_loop_upper:".to_string());
        self.text_section.push("    mov al, [rsi]  ; cargar byte".to_string());
        self.text_section.push("    ; Convertir a mayúsculas: si 'a' <= al <= 'z', entonces al = al - 32".to_string());
        self.text_section.push("    cmp al, 'a'".to_string());
        self.text_section.push("    jl .not_lower_upper".to_string());
        self.text_section.push("    cmp al, 'z'".to_string());
        self.text_section.push("    jg .not_lower_upper".to_string());
        self.text_section.push("    sub al, 32  ; convertir a mayúscula".to_string());
        self.text_section.push(".not_lower_upper:".to_string());
        self.text_section.push("    mov [rdi], al  ; guardar byte convertido".to_string());
        self.text_section.push("    inc rsi".to_string());
        self.text_section.push("    inc rdi".to_string());
        self.text_section.push("    dec rcx".to_string());
        self.text_section.push("    jnz .copy_loop_upper".to_string());
        
        self.text_section.push(".upper_done:".to_string());
        self.text_section.push("    mov byte [rdi], 0  ; null terminator".to_string());
        self.text_section.push("    mov rax, rbx  ; retornar puntero al nuevo String".to_string());
        
        // Epilogue ABI-safe
        self.generate_abi_epilogue(true);
        self.text_section.push("".to_string());
        }
        
        // string_lower: Convertir a minúsculas
        // Parámetros: RCX = puntero al String
        // Retorna: RAX = puntero al nuevo String (minúsculas)
        if deps.should_generate("string_lower") {
        self.text_section.push("string_lower:".to_string());
        self.generate_abi_prologue(true);  // Necesita shadow space para VirtualAlloc
        self.text_section.push("    mov r12, rcx  ; preservar String".to_string());
        
        // Obtener longitud
        self.text_section.push("    ; Obtener longitud".to_string());
        self.text_section.push("    mov rdx, [r12 + 8]  ; length".to_string());
        self.text_section.push("    mov r13, rdx  ; preservar longitud".to_string());
        
        // Calcular capacity: max((length + 1) * 2, 16)
        self.text_section.push("    ; Calcular capacity".to_string());
        self.text_section.push("    mov rax, r13".to_string());
        self.text_section.push("    inc rax  ; +1 para null terminator".to_string());
        self.text_section.push("    shl rax, 1  ; * 2".to_string());
        self.text_section.push("    cmp rax, 16".to_string());
        self.text_section.push("    jge .capacity_ok_lower".to_string());
        self.text_section.push("    mov rax, 16  ; mínimo 16".to_string());
        self.text_section.push(".capacity_ok_lower:".to_string());
        self.text_section.push("    mov r14, rax  ; preservar capacity".to_string());
        
        // Allocar memoria para nuevo String struct
        self.text_section.push("    ; Allocar memoria para nuevo String struct".to_string());
        self.ensure_stack_alignment_before_call("VirtualAlloc (String struct)");
        self.text_section.push("    mov rcx, 0".to_string());
        self.text_section.push("    mov rdx, 32".to_string());
        self.text_section.push("    mov r8, 0x1000".to_string());
        self.text_section.push("    mov r9, 0x04".to_string());
        self.text_section.push("    call VirtualAlloc".to_string());
        self.text_section.push("    mov rbx, rax  ; preservar puntero al nuevo String (rbx preservado)".to_string());
        
        // Allocar memoria para data
        self.text_section.push("    ; Allocar memoria para data".to_string());
        self.ensure_stack_alignment_before_call("VirtualAlloc (data)");
        self.text_section.push("    mov rcx, 0".to_string());
        self.text_section.push("    mov rdx, r14  ; capacity".to_string());
        self.text_section.push("    mov r8, 0x1000".to_string());
        self.text_section.push("    mov r9, 0x04".to_string());
        self.text_section.push("    call VirtualAlloc".to_string());
        self.text_section.push("    mov rdi, rax  ; preservar puntero a data (rdi preservado)".to_string());
        
        // Configurar nuevo String
        self.text_section.push("    ; Configurar nuevo String".to_string());
        self.text_section.push("    mov [rbx + 0], rdi  ; data = puntero".to_string());
        self.text_section.push("    mov [rbx + 8], r13  ; length".to_string());
        self.text_section.push("    mov [rbx + 16], r14  ; capacity".to_string());
        self.text_section.push("    mov qword [rbx + 24], 0  ; hash = 0".to_string());
        
        // Copiar y convertir a minúsculas
        self.text_section.push("    ; Copiar y convertir a minúsculas".to_string());
        self.text_section.push("    mov rsi, [r12 + 0]  ; fuente (String->data, rsi preservado)".to_string());
        self.text_section.push("    mov rcx, r13  ; longitud".to_string());
        self.text_section.push("    test rcx, rcx".to_string());
        self.text_section.push("    jz .lower_done".to_string());
        self.text_section.push(".copy_loop_lower:".to_string());
        self.text_section.push("    mov al, [rsi]  ; cargar byte".to_string());
        self.text_section.push("    ; Convertir a minúsculas: si 'A' <= al <= 'Z', entonces al = al + 32".to_string());
        self.text_section.push("    cmp al, 'A'".to_string());
        self.text_section.push("    jl .not_upper_lower".to_string());
        self.text_section.push("    cmp al, 'Z'".to_string());
        self.text_section.push("    jg .not_upper_lower".to_string());
        self.text_section.push("    add al, 32  ; convertir a minúscula".to_string());
        self.text_section.push(".not_upper_lower:".to_string());
        self.text_section.push("    mov [rdi], al  ; guardar byte convertido".to_string());
        self.text_section.push("    inc rsi".to_string());
        self.text_section.push("    inc rdi".to_string());
        self.text_section.push("    dec rcx".to_string());
        self.text_section.push("    jnz .copy_loop_lower".to_string());
        
        self.text_section.push(".lower_done:".to_string());
        self.text_section.push("    mov byte [rdi], 0  ; null terminator".to_string());
        self.text_section.push("    mov rax, rbx  ; retornar puntero al nuevo String".to_string());
        
        // Epilogue ABI-safe
        self.generate_abi_epilogue(true);
        self.text_section.push("".to_string());
        }
        
        // string_free: Liberar memoria de un String
        // Parámetros: RCX = puntero al String
        // Retorna: RAX = 0 (éxito) o -4 (error: puntero inválido)
        // Nota: Libera tanto el String struct como su data buffer
        //       Liberar NULL es seguro (no-op, retorna 0)
        if deps.should_generate("string_free") {
        self.text_section.push("string_free:".to_string());
        self.generate_abi_prologue(true);  // Necesita shadow space para VirtualFree
        self.text_section.push("    mov r12, rcx  ; preservar puntero al String".to_string());
        
        // Verificar si el puntero es NULL (liberar NULL es seguro, no-op)
        self.text_section.push("    ; Verificar si el puntero es NULL".to_string());
        self.text_section.push("    test r12, r12  ; verificar si String* es NULL".to_string());
        self.text_section.push("    jz .free_string_done  ; si es NULL, retornar éxito (no-op)".to_string());
        
        // Liberar data buffer primero
        self.text_section.push("    ; Liberar data buffer".to_string());
        self.text_section.push("    mov rcx, [r12 + 0]  ; data pointer".to_string());
        self.text_section.push("    test rcx, rcx  ; verificar si es NULL".to_string());
        self.text_section.push("    jz .free_string_struct  ; si es NULL, saltar".to_string());
        self.ensure_stack_alignment_before_call("VirtualFree (data)");
        self.text_section.push("    mov rdx, 0  ; dwSize (0 = liberar todo)".to_string());
        self.text_section.push("    mov r8, 0x8000  ; MEM_RELEASE".to_string());
        self.text_section.push("    call VirtualFree".to_string());
        
        // Liberar String struct
        self.text_section.push(".free_string_struct:".to_string());
        self.text_section.push("    ; Liberar String struct".to_string());
        self.ensure_stack_alignment_before_call("VirtualFree (String struct)");
        self.text_section.push("    mov rcx, r12  ; puntero al String struct".to_string());
        self.text_section.push("    mov rdx, 0  ; dwSize (0 = liberar todo)".to_string());
        self.text_section.push("    mov r8, 0x8000  ; MEM_RELEASE".to_string());
        self.text_section.push("    call VirtualFree".to_string());
        
        // Retornar éxito
        self.text_section.push(".free_string_done:".to_string());
        self.text_section.push("    mov rax, 0  ; éxito".to_string());
        self.generate_abi_epilogue(true);
        self.text_section.push("".to_string());
        }
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adead_parser::parse;

    #[test]
    fn test_generate_simple_print() {
        let src = r#"print "Hello""#;
        let program = parse(src).unwrap();
        let mut gen = CodeGenerator::new();
        let asm = gen.generate(&program).unwrap();
        assert!(asm.contains("sys_write"));
        assert!(asm.contains("Hello"));
    }
}

