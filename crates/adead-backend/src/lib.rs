use adead_common::Result;
use adead_parser::{BinOp, Expr, Program, Stmt};
use std::collections::HashMap;

pub struct CodeGenerator {
    data_section: Vec<String>,
    text_section: Vec<String>,
    string_counter: usize,
    label_counter: usize,
    variables: HashMap<String, i64>, // simple: track stack offsets
    stack_offset: i64,
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
                    _ => {
                        return Err(adead_common::ADeadError::RuntimeError {
                            message: "print only supports strings for now".to_string(),
                        });
                    }
                }
            }
            Stmt::Let { mutable, name, value } => {
                self.generate_expr_windows(value)?;
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
            Stmt::Fn { name, params, body } => {
                // Generate function with Windows x64 calling convention
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
        }
        Ok(())
    }

    fn generate_expr_windows(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Number(n) => {
                self.text_section.push(format!("    mov rax, {}", n));
            }
            Expr::String(_) => {
                // Strings are handled in print, not as expressions
                return Err(adead_common::ADeadError::RuntimeError {
                    message: "strings cannot be used as expressions yet".to_string(),
                });
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
                    self.text_section.push(format!("    mov rax, [rbp + {}]  ; load variable {}", 32 + offset, name));
                } else {
                    return Err(adead_common::ADeadError::RuntimeError {
                        message: format!("undefined variable: {} (variables must be declared with 'let')", name),
                    });
                }
            }
            Expr::BinaryOp { left, op, right } => {
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
            Expr::Call { name, args } => {
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
                
                self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                self.text_section.push(format!("    call fn_{}", name));
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
                    _ => {
                        // Evaluate expression and print as number
                        self.generate_expr(expr)?;
                        // Result is in rax, convert to string (simplified: just print as-is for now)
                        // For MVP, we'll handle string printing only
                        return Err(adead_common::ADeadError::RuntimeError {
                            message: "print only supports strings for now".to_string(),
                        });
                    }
                }
            }
            Stmt::Let { mutable, name, value } => {
                self.generate_expr(value)?;
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
            Stmt::Fn { name, params, body } => {
                // Generate function
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
        }
        Ok(())
    }

    fn generate_expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Number(n) => {
                self.text_section.push(format!("    mov rax, {}", n));
            }
            Expr::String(_) => {
                // Strings handled separately in print
                return Err(adead_common::ADeadError::RuntimeError {
                    message: "cannot use string in expression yet".to_string(),
                });
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
            Expr::Call { name, args } => {
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
                
                self.text_section.push(format!("    call fn_{}", name));
                // Return value is in rax
            }
        }
        Ok(())
    }

    fn add_string_data(&mut self, s: &str) -> String {
        let label = format!("msg{}", self.string_counter);
        self.string_counter += 1;
        
        // Escape string for NASM
        let escaped = s
            .replace('\\', "\\\\")
            .replace('\n', "\\n")
            .replace('\t', "\\t")
            .replace('"', "\\\"");
        
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

