use adead_common::Result;
use adead_parser::{BinOp, Expr, Pattern, Program, Stmt};
use std::collections::HashMap;

mod memory_pool;
mod optimizer;
mod stdlib;
mod register_optimizer;
mod dependency_graph;
mod usage_analyzer;
use memory_pool::MemoryPool;
use optimizer::CodeOptimizer;
use stdlib::StdLib;
use register_optimizer::RegisterOptimizer;
use dependency_graph::DependencyGraph;
use usage_analyzer::UsageAnalyzer;

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
        
        // Generar funciones de usuario ANTES del main
        // Separar funciones de otros statements
        let mut user_functions = Vec::new();
        let mut other_statements = Vec::new();
        
        for stmt in &program.statements {
            match stmt {
                Stmt::Fn { .. } => {
                    user_functions.push(stmt);
                }
                _ => {
                    other_statements.push(stmt);
                }
            }
        }
        
        // Generar funciones de usuario primero
        for stmt in &user_functions {
            self.generate_stmt_windows(stmt)?;
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
        for stmt in &other_statements {
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
        let mut optimizer = CodeOptimizer::new();
        let text_code = self.text_section.join("\n");
        optimizer.analyze_usage(&text_code);
        
        // Eliminar código muerto (funciones no usadas)
        let optimized_text = optimizer.remove_dead_code(&text_code);
        let optimized_lines: Vec<String> = optimized_text.lines().map(|s| s.to_string()).collect();
        
        let mut output = String::new();
        
        // En Windows, poner default rel al principio
        #[cfg(target_os = "windows")]
        {
            // default rel ya está en generate_windows, pero asegurarse aquí también
        }
        
        // Data section primero (necesario para Windows)
        if !self.data_section.is_empty() {
            output.push_str("section .data\n");
            for line in &self.data_section {
                output.push_str(line);
                output.push('\n');
            }
            output.push('\n');
        }
        
        // Text section (optimizado - dead code eliminado)
        for line in &optimized_lines {
            output.push_str(line);
            output.push('\n');
        }
        Ok(output)
    }

    fn generate_stmt_windows(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Print(expr) => {
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
                        // Verificar si la variable es string o numérica
                        if self.is_string_expr(expr) {
                            // Variable que contiene String struct: cargar puntero al String desde la variable
                            if let Some(&offset) = self.variables.get(name) {
                                // La variable contiene el puntero al String struct
                                self.text_section.push(format!("    mov rax, [rbp - {}]  ; cargar puntero al String struct desde variable {}", offset + 8, name));
                                
                                // Cargar String->data y String->length
                                self.text_section.push("    mov rdx, [rax + 0]  ; String->data".to_string());
                                self.text_section.push("    mov r8, [rax + 8]  ; String->length".to_string());
                            } else {
                                return Err(adead_common::ADeadError::RuntimeError {
                                    message: format!("undefined variable: {} in print statement", name),
                                });
                            }
                            // Preparar WriteFile call
                            self.text_section.push("    ; Prepare WriteFile call for String variable".to_string());
                            self.text_section.push("    mov rcx, [rbp+16]  ; stdout handle".to_string());
                            // RDX ya está listo (String->data)
                            // R8 ya está listo (String->length)
                            self.text_section.push("    lea r9, [rbp+24]  ; lpNumberOfBytesWritten".to_string());
                            self.text_section.push("    mov qword [r9], 0  ; inicializar".to_string());
                            self.text_section.push("    mov qword [rsp+32], 0  ; lpOverlapped = NULL".to_string());
                            self.text_section.push("    call WriteFile".to_string());
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
                // Si el valor es un StructLiteral, registrar para destrucción RAII si tiene destroy
                let struct_name = if let Expr::StructLiteral { name: struct_name, .. } = value {
                    if self.structs_with_destroy.contains_key(struct_name) {
                        self.variables_to_destroy.push((name.clone(), struct_name.clone()));
                    }
                    Some(struct_name.clone())
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
                let loop_start = self.new_label("loop_start");
                let loop_end = self.new_label("loop_end");
                
                    self.text_section.push(format!("{}:", loop_start));
                self.generate_expr_windows(condition)?;
                self.text_section.push("    cmp rax, 0".to_string());
                self.text_section.push(format!("    je {}", loop_end));
                
                for s in body {
                    self.generate_stmt_windows(s)?;
                }
                self.text_section.push(format!("    jmp {}", loop_start));
                self.text_section.push(format!("{}:", loop_end));
            }
            Stmt::Fn { visibility: _, name, params, body } => {
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
            Stmt::Struct { name, fields: _, init, destroy: _ } => {
                // Registrar struct y generar código para constructor si existe
                if let Some(init_method) = init {
                    // Generar función de constructor: StructName_init
                    let init_label = format!("{}_init", name);
                    self.text_section.push(format!("    jmp {}_end", init_label));
                    self.text_section.push(format!("{}:", init_label));
                    self.text_section.push("    push rbp".to_string());
                    self.text_section.push("    mov rbp, rsp".to_string());
                    
                    // Guardar parámetros en stack (Windows x64 calling convention)
                    for (i, param) in init_method.params.iter().enumerate() {
                        let offset = self.stack_offset;
                        self.stack_offset += 8;
                        self.variables.insert(param.name.clone(), offset);
                        
                        let reg = match i {
                            0 => "rcx",
                            1 => "rdx",
                            2 => "r8",
                            3 => "r9",
                            _ => {
                                let stack_offset = 16 + (i - 4) * 8;
                                    self.text_section.push(format!("    mov rax, [rbp + {}]", stack_offset));
                                    self.text_section.push(format!("    mov [rbp - {}], rax", offset + 8));
                                continue;
                            }
                        };
                            self.text_section.push(format!("    mov [rbp - {}], {}", offset + 8, reg));
                    }
                    
                    // Generar cuerpo del constructor
                    for s in &init_method.body {
                        self.generate_stmt_windows(s)?;
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
                            self.text_section.push("    sub rbx, rax".to_string());
                            self.text_section.push("    mov rax, rbx".to_string());
                        }
                        BinOp::Mul => {
                            self.text_section.push("    imul rax, rbx".to_string());
                        }
                        BinOp::Div => {
                            self.text_section.push("    mov rdx, 0".to_string());
                            self.text_section.push("    mov rcx, rax".to_string());
                            self.text_section.push("    mov rax, rbx".to_string());
                            self.text_section.push("    div rcx".to_string());
                        }
                        BinOp::Mod => {
                            // Módulo: RAX = RBX % RAX
                            // div rcx deja el resto en RDX
                            self.text_section.push("    mov rdx, 0".to_string());
                            self.text_section.push("    mov rcx, rax".to_string());
                            self.text_section.push("    mov rax, rbx".to_string());
                            self.text_section.push("    div rcx".to_string());
                            self.text_section.push("    mov rax, rdx  ; resto (módulo) en RAX".to_string());
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
                    }
                }
                }
            }
            Expr::Call { module, name, args } => {
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
                } else {
                    // Llamada a función normal (ABI-safe)
                    // Windows x64 calling convention:
                    // - First 4 params: RCX, RDX, R8, R9
                    // - Additional params: on stack (right-to-left)
                    // - Shadow space: 32 bytes must be reserved
                    // - Stack must be aligned to 16 bytes before call
                    
                    let num_args = args.len();
                    let stack_args_count = if num_args > 4 { num_args - 4 } else { 0 };
                    let total_stack_space = 32 + (stack_args_count * 8); // shadow space (32) + stack args (8 cada uno)
                    
                    // Reservar shadow space y espacio para parámetros adicionales
                    self.text_section.push(format!("    sub rsp, {}  ; shadow space (32) + stack args ({})", 32, stack_args_count * 8));
                    
                    // Push parámetros adicionales en stack (right-to-left)
                    // Windows x64: parámetros adicionales se pasan right-to-left
                    for arg in args.iter().skip(4).rev() {
                        self.generate_expr_windows(arg)?;
                        self.text_section.push("    push rax  ; parámetro adicional en stack".to_string());
                    }
                    
                    // Cargar primeros 4 parámetros en registros (left-to-right)
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
                    
                    // Llamar función (con namespace si existe) (Sprint 1.3)
                    let function_name = if let Some(module_name) = module {
                        format!("fn_{}_{}", module_name, name)
                    } else {
                        format!("fn_{}", name)
                    };
                    
                    self.text_section.push(format!("    call {}", function_name));
                    
                    // Restaurar stack (shadow space + parámetros adicionales)
                    self.text_section.push(format!("    add rsp, {}  ; restaurar shadow space + stack args", total_stack_space));
                    
                    // Valor de retorno está en RAX
                }
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
                // Generar struct literal en stack
                let struct_size = fields.len() * 8;
                let offset = self.stack_offset;
                self.stack_offset += struct_size as i64;
                self.text_section.push(format!("    sub rsp, {}  ; espacio para struct ({} campos)", struct_size, fields.len()));
                
                // Preparar argumentos para constructor si existe
                // Por ahora, asumimos que el constructor toma los campos como parámetros
                // TODO: Mejorar para manejar constructores con diferentes firmas
                
                // Generar valores de campos primero
                for (i, (field_name, value)) in fields.iter().enumerate() {
                    self.generate_expr_windows(value)?;
                    let field_offset = offset + (i as i64 * 8) + 8;
                    self.text_section.push(format!("    mov [rbp - {}], rax  ; campo '{}'", field_offset, field_name));
                }
                
                // Si hay constructor, llamarlo (por ahora, solo guardamos la dirección)
                // TODO: Implementar llamada real al constructor cuando se use StructLiteral en Let
                
                    self.text_section.push(format!("    lea rax, [rbp - {}]  ; dirección del struct", offset + 8));
            }
            Expr::FieldAccess { object, field } => {
                self.generate_expr_windows(object)?;
                self.text_section.push(format!("    ; accediendo campo '{}' (offset simplificado: 0)", field));
                self.text_section.push("    mov rax, [rax]  ; cargar primer campo (simplificado)".to_string());
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
                        // Método genérico: llamar a fn_{method}
                        self.generate_expr_windows(object)?;
                        self.text_section.push("    push rax  ; guardar self".to_string());
                        
                        for arg in args {
                            self.generate_expr_windows(arg)?;
                            self.text_section.push("    push rax".to_string());
                        }
                        
                        self.text_section.push(format!("    call fn_{}", method));
                        
                        let cleanup_size = (args.len() + 1) * 8;
                        if cleanup_size > 0 {
                            self.text_section.push(format!("    add rsp, {}", cleanup_size));
                        }
                    }
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
                let struct_name = if let Expr::StructLiteral { name: struct_name, .. } = value {
                    if self.structs_with_destroy.contains_key(struct_name) {
                        self.variables_to_destroy.push((name.clone(), struct_name.clone()));
                    }
                    Some(struct_name.clone())
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
                let loop_start = self.new_label("loop_start");
                let loop_end = self.new_label("loop_end");
                
                    self.text_section.push(format!("{}:", loop_start));
                self.generate_expr(condition)?;
                self.text_section.push("    cmp rax, 0".to_string());
                self.text_section.push(format!("    je {}", loop_end));
                
                for s in body {
                    self.generate_stmt(s)?;
                }
                self.text_section.push(format!("    jmp {}", loop_start));
                self.text_section.push(format!("{}:", loop_end));
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
            Stmt::Struct { name, fields: _, init, destroy: _ } => {
                // Registrar struct y generar código para constructor si existe
                if let Some(init_method) = init {
                    // Generar función de constructor: StructName_init
                    let init_label = format!("{}_init", name);
                    self.text_section.push(format!("    jmp {}_end", init_label));
                    self.text_section.push(format!("{}:", init_label));
                    self.text_section.push("    push rbp".to_string());
                    self.text_section.push("    mov rbp, rsp".to_string());
                    
                    // Guardar parámetros en stack (Windows x64 calling convention)
                    for (i, param) in init_method.params.iter().enumerate() {
                        let offset = self.stack_offset;
                        self.stack_offset += 8;
                        self.variables.insert(param.name.clone(), offset);
                        
                        let reg = match i {
                            0 => "rcx",
                            1 => "rdx",
                            2 => "r8",
                            3 => "r9",
                            _ => {
                                let stack_offset = 16 + (i - 4) * 8;
                                    self.text_section.push(format!("    mov rax, [rbp + {}]", stack_offset));
                                    self.text_section.push(format!("    mov [rbp - {}], rax", offset + 8));
                                continue;
                            }
                        };
                            self.text_section.push(format!("    mov [rbp - {}], {}", offset + 8, reg));
                    }
                    
                    // Generar cuerpo del constructor
                    for s in &init_method.body {
                        self.generate_stmt_windows(s)?;
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
            Expr::StructLiteral { name: _, fields } => {
                // Por ahora, implementación simplificada
                // TODO: Implementar layout de memoria real basado en tipos de campos
                let struct_size = fields.len() * 8;
                self.text_section.push(format!("    sub rsp, {}  ; espacio para struct", struct_size));
                // Generar valores (simplificado)
                if let Some((_, first_value)) = fields.first() {
                    self.generate_expr(first_value)?;
                }
            }
            Expr::FieldAccess { object, field: _ } => {
                // Acceso a campo (simplificado)
                self.generate_expr(object)?;
                self.text_section.push("    mov rax, [rax]  ; cargar campo (simplificado)".to_string());
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
                    BinOp::Mod => {
                        // Módulo: RAX = RAX % RBX (resto de división)
                        self.text_section.push("    cqo".to_string()); // sign extend rax to rdx:rax
                        self.text_section.push("    idiv rbx".to_string());
                        self.text_section.push("    mov rax, rdx  ; resto (módulo) en RAX".to_string());
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
                }
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
        
        // Eliminar cualquier \n al final del string para evitar doble salto de línea
        let s_clean = s.trim_end_matches('\n');
        
        // Escape string for NASM (pero preservar \n como 0xA)
        let escaped = s_clean
            .replace('\\', "\\\\")
            .replace('\n', "\\n")  // Convertir \n a \\n para NASM
            .replace('\t', "\\t")
            .replace('"', "\\\"");
        
        // Siempre agregar 0xA al final (ya limpiamos el string de \n al final)
        self.data_section.push(format!(
            "{}: db \"{}\", 0xA",
            label, escaped
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
        
        // Loop para copiar valores
        self.text_section.push("    ; Loop para copiar valores".to_string());
        self.text_section.push("    mov rcx, r12  ; count".to_string());
        self.text_section.push("    mov rsi, r13  ; puntero a valores fuente (preservado)".to_string());
        self.text_section.push("    test rcx, rcx".to_string());
        self.text_section.push("    jz .copy_done".to_string());
        self.text_section.push(".copy_loop:".to_string());
        self.text_section.push("    mov rax, [rsi]  ; cargar valor fuente".to_string());
        self.text_section.push("    mov [rdi], rax  ; guardar en destino".to_string());
        self.text_section.push("    add rsi, 8  ; siguiente elemento fuente".to_string());
        self.text_section.push("    add rdi, 8  ; siguiente elemento destino".to_string());
        self.text_section.push("    dec rcx".to_string());
        self.text_section.push("    jnz .copy_loop".to_string());
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
        
        // string_from_literal: Crear string desde literal
        // Parámetros: RCX = puntero a literal (char*, null-terminated), RDX = longitud
        // Retorna: RAX = puntero al String (en heap)
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

