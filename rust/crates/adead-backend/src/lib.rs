use adead_common::Result;
use adead_parser::{BinOp, Expr, Pattern, Program, Stmt};
use std::collections::HashMap;

pub struct CodeGenerator {
    data_section: Vec<String>,
    text_section: Vec<String>,
    string_counter: usize,
    label_counter: usize,
    variables: HashMap<String, i64>, // simple: track stack offsets
    stack_offset: i64,
    structs_with_destroy: HashMap<String, bool>, // Track structs that have destroy methods (O2.1)
    variables_to_destroy: Vec<(String, String)>, // (variable_name, struct_name) - RAII tracking
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
        self.text_section.push("global main".to_string());
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

        for stmt in &program.statements {
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
        
        // Text section
        for line in &self.text_section {
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
                        let label = self.add_string_data(s);
                        // WriteFile Windows API call
                        // BOOL WriteFile(
                        //   HANDLE hFile,                    // RCX
                        //   LPCVOID lpBuffer,                // RDX
                        //   DWORD nNumberOfBytesToWrite,     // R8
                        //   LPDWORD lpNumberOfBytesWritten,  // R9
                        //   LPOVERLAPPED lpOverlapped        // [rsp+32] (shadow space)
                        // )
                        self.text_section.push("    ; Prepare WriteFile call".to_string());
                        self.text_section.push("    mov rcx, [rbp+16]  ; stdout handle".to_string());
                            self.text_section.push(format!("    lea rdx, [rel {}]  ; buffer pointer", label));
                            self.text_section.push(format!("    mov r8, {}_len  ; number of bytes to write", label));
                        self.text_section.push("    lea r9, [rbp+24]  ; lpNumberOfBytesWritten (local var)".to_string());
                        self.text_section.push("    mov qword [rsp+32], 0  ; lpOverlapped = NULL (5th param in shadow space)".to_string());
                        self.text_section.push("    call WriteFile".to_string());
                        self.text_section.push("    ; WriteFile result in RAX (BOOL), but we ignore it for now".to_string());
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
                        // Variable que contiene string: cargar la dirección del string desde la variable
                        if let Some(&offset) = self.variables.get(name) {
                            // La variable contiene la dirección del string
                                self.text_section.push(format!("    mov rdx, [rbp - {}]  ; cargar dirección del string desde variable {}", offset + 8, name));
                            // Longitud del string (simplificado: asumir longitud máxima de 100)
                            // TODO: Mejorar para almacenar longitud junto con la dirección
                            self.text_section.push("    mov r8, 100  ; longitud del string (simplificado)".to_string());
                        } else {
                            return Err(adead_common::ADeadError::RuntimeError {
                                message: format!("undefined variable: {} in print statement", name),
                            });
                        }
                        // Preparar WriteFile call
                        self.text_section.push("    ; Prepare WriteFile call for string variable".to_string());
                        self.text_section.push("    mov rcx, [rbp+16]  ; stdout handle".to_string());
                        // RDX ya está listo (dirección del string)
                        // R8 ya está listo (longitud)
                        self.text_section.push("    lea r9, [rbp+24]  ; lpNumberOfBytesWritten".to_string());
                        self.text_section.push("    mov qword [r9], 0  ; inicializar".to_string());
                        self.text_section.push("    mov qword [rsp+32], 0  ; lpOverlapped = NULL".to_string());
                        self.text_section.push("    call WriteFile".to_string());
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
                // Generate function with Windows x64 calling convention
                // Visibility no afecta la generación de código (Sprint 1.3)
                let func_label = format!("fn_{}", name);
                self.text_section.push(format!("    jmp {}_end", func_label));
                self.text_section.push(format!("{}:", func_label));
                self.text_section.push("    push rbp".to_string());
                self.text_section.push("    mov rbp, rsp".to_string());
                
                // Allocate space for locals (we'll track this with stack_offset)
                // Parameters are in RCX, RDX, R8, R9 (Windows x64 calling convention)
                // Save them to local stack variables
                for (i, param) in params.iter().enumerate() {
                    let offset = self.stack_offset;
                    self.stack_offset += 8;
                    self.variables.insert(param.name.clone(), offset);
                    
                    let reg = match i {
                        0 => "rcx",
                        1 => "rdx",
                        2 => "r8",
                        3 => "r9",
                        _ => {
                            // Additional params are on stack at [rbp + 16 + (i-4)*8]
                            // (16 = return address + saved rbp)
                            let stack_offset = 16 + (i - 4) * 8;
                                self.text_section.push(format!("    mov rax, [rbp + {}]", stack_offset));
                                self.text_section.push(format!("    mov [rbp - {}], rax", offset + 8));
                            continue;
                        }
                    };
                    self.text_section.push(format!("    mov [rbp - {}], {}", offset + 8, reg));
                }
                
                // Generate function body
                for s in body {
                    self.generate_stmt_windows(s)?;
                }
                
                self.text_section.push("    leave".to_string());
                self.text_section.push("    ret".to_string());
                self.text_section.push(format!("{}_end:", func_label));
            }
            Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    self.generate_expr_windows(expr)?;
                } else {
                    self.text_section.push("    mov rax, 0".to_string());
                }
                self.text_section.push("    leave".to_string());
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
                // Strings como expresiones: crear etiqueta en datos y retornar dirección
                let label = self.add_string_data(s);
                // Retornar dirección del string en rax
                self.text_section.push(format!("    lea rax, [rel {}]  ; dirección del string", label));
            }
            Expr::ArrayLiteral(elements) => {
                // Array literal: [1, 2, 3]
                // Estrategia: stack-allocated array
                // 1. Reservar espacio en stack para todos los elementos
                // 2. Calcular offset base
                // 3. Generar cada elemento y almacenarlo
                
                let array_size = elements.len() * 8; // 8 bytes por elemento (int64)
                let base_offset = self.stack_offset;
                
                // Reservar espacio en stack
                self.stack_offset += array_size as i64;
                self.text_section.push(format!("    ; Array literal: {} elementos ({} bytes)", elements.len(), array_size));
                self.text_section.push(format!("    sub rsp, {}  ; reservar espacio para array", array_size));
                
                // Generar y almacenar cada elemento
                for (i, element) in elements.iter().enumerate() {
                    self.generate_expr_windows(element)?;
                    // Almacenar en la posición correcta del array
                    // [rbp - base_offset - (i * 8)] donde i va de 0 a len-1
                    let element_offset = base_offset + (i as i64 * 8);
                    self.text_section.push(format!("    mov [rbp - {}], rax  ; array[{}]", element_offset + 8, i));
                }
                
                // Retornar dirección base del array (offset desde rbp)
                // Usamos LEA para calcular la dirección
                self.text_section.push(format!("    lea rax, [rbp - {}]  ; dirección base del array", base_offset + 8));
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
            Expr::Call { module, name, args } => {
                // Windows x64 calling convention
                let mut stack_args = Vec::new();
                for (i, arg) in args.iter().enumerate() {
                    if i >= 4 {
                        self.generate_expr_windows(arg)?;
                        self.text_section.push("    push rax".to_string());
                        stack_args.push(i);
                    }
                }
                
                for (i, arg) in args.iter().enumerate().take(4) {
                    self.generate_expr_windows(arg)?;
                    let reg = match i {
                        0 => "rcx",
                        1 => "rdx",
                        2 => "r8",
                        3 => "r9",
                        _ => unreachable!(),
                    };
                    if i == 0 && args.len() > 1 {
                        self.text_section.push("    mov r10, rax".to_string());
                            self.text_section.push(format!("    mov {}, r10", reg));
                    } else {
                            self.text_section.push(format!("    mov {}, rax", reg));
                    }
                }
                
                // Llamar función (con namespace si existe) (Sprint 1.3)
                let function_name = if let Some(module_name) = module {
                    format!("fn_{}_{}", module_name, name)
                } else {
                    format!("fn_{}", name)
                };
                
                self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                self.text_section.push(format!("    call {}", function_name));
                if !stack_args.is_empty() {
                    self.text_section.push(format!("    add rsp, {}", (stack_args.len() * 8) + 32));
                } else {
                    self.text_section.push("    add rsp, 32".to_string());
                }
            }
            Expr::Assign { name, value } => {
                // Evaluate value first
                self.generate_expr_windows(value)?;
                // Store in variable
                if let Some(&offset) = self.variables.get(name) {
                    self.text_section.push(format!("    mov [rbp - {}], rax", offset + 8));
                } else {
                    // Variable doesn't exist, create it
                    let offset = self.stack_offset;
                    self.stack_offset += 8;
                    self.variables.insert(name.clone(), offset);
                    self.text_section.push(format!("    mov [rbp - {}], rax", offset + 8));
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
                // Indexación: arr[0]
                // Estrategia:
                // 1. Generar expresión del array (dirección base en rax)
                // 2. Generar expresión del índice
                // 3. Calcular dirección: base + (index * 8) 
                // 4. Cargar valor desde esa dirección
                
                self.generate_expr_windows(array)?;
                // Guardar dirección base en stack
                self.text_section.push("    push rax  ; guardar dirección base del array".to_string());
                
                self.generate_expr_windows(index)?;
                // rax contiene el índice
                // Multiplicar por 8 (tamaño de int64)
                self.text_section.push("    mov rcx, rax  ; rcx = índice".to_string());
                self.text_section.push("    mov rax, 8".to_string());
                self.text_section.push("    imul rax, rcx  ; rax = índice * 8 (offset en bytes)".to_string());
                
                // Restaurar dirección base y sumar offset
                self.text_section.push("    pop rbx  ; restaurar dirección base".to_string());
                self.text_section.push("    add rax, rbx  ; rax = dirección base + offset".to_string());
                
                // Cargar valor desde la dirección calculada
                self.text_section.push("    mov rax, [rax]  ; cargar array[index]".to_string());
            }
            Expr::MethodCall { object, method, args } => {
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
        
        // Escape string for NASM (pero preservar \n como 0xA)
        let escaped = s
            .replace('\\', "\\\\")
            .replace('\n', "\\n")  // Convertir \n a \\n para NASM
            .replace('\t', "\\t")
            .replace('"', "\\\"");
        
        // Si el string no termina en \n, agregar 0xA al final
        let needs_newline = !s.ends_with('\n');
        
        if needs_newline {
            self.data_section.push(format!(
                "{}: db \"{}\", 0xA",
                label, escaped
            ));
        } else {
            // Ya tiene \n, solo usar el string escapado
            self.data_section.push(format!(
                "{}: db \"{}\"",
                label, escaped
            ));
        }
        
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

