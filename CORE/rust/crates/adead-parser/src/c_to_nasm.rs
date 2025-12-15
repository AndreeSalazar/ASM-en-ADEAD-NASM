//! Generador ASM limpio desde código C
//! Flujo: ADead → C → Rust (genera ASM limpio) → ASM → Ejecutar
//! 
//! Este módulo toma código C generado y lo convierte a ASM NASM limpio y optimizado

use crate::{Program, Stmt, Expr, BinOp};

/// Generador de ASM NASM desde código C (intermediario)
/// Estrategia: Parsear el código C generado y convertirlo directamente a NASM limpio
pub struct CToNASMGenerator {
    nasm_output: String,
    stack_offset: i32,
    label_count: u32,
    data_section: String,
    text_section: String,
    bss_section: String,
}

impl CToNASMGenerator {
    pub fn new() -> Self {
        Self {
            nasm_output: String::new(),
            stack_offset: 0,
            label_count: 0,
            data_section: String::new(),
            text_section: String::new(),
            bss_section: String::new(),
        }
    }

    /// Generar ASM NASM limpio desde código C
    /// Estrategia: Trabajar con el AST de ADead (que ya fue convertido a C)
    /// y generar ASM limpio directamente sin depender de GCC
    pub fn generate_clean_nasm_from_c_ast(&mut self, program: &Program) -> String {
        use std::fmt::Write;
        
        // Generar header NASM limpio
        writeln!(self.nasm_output, "bits 64").unwrap();
        writeln!(self.nasm_output, "default rel").unwrap();
        writeln!(self.nasm_output).unwrap();
        
        // Sección .data para strings y constantes
        self.nasm_output.push_str("section .data\n");
        
        // Sección .bss para variables
        self.bss_section.push_str("section .bss\n");
        
        // Sección .text para código
        self.text_section.push_str("section .text\n");
        writeln!(self.text_section, "global main").unwrap();
        writeln!(self.text_section, "extern printf, puts, putchar").unwrap();
        writeln!(self.text_section, "extern ExitProcess").unwrap();
        writeln!(self.text_section).unwrap();
        
        // Procesar el programa y generar NASM limpio
        self.process_program(program);
        
        // Combinar todas las secciones
        self.nasm_output.push_str(&self.data_section);
        self.nasm_output.push_str("\n");
        self.nasm_output.push_str(&self.bss_section);
        self.nasm_output.push_str("\n");
        self.nasm_output.push_str(&self.text_section);
        
        self.nasm_output.clone()
    }
    
    fn process_program(&mut self, program: &Program) {
        use std::fmt::Write;
        use std::collections::HashMap;
        
        // Primera pasada: Mapear todas las variables
        let mut var_map: HashMap<String, i32> = HashMap::new();
        let mut var_offset = 128; // Comenzar desde rbp - 128
        
        for stmt in &program.statements {
            if let Stmt::Let { name, .. } = stmt {
                var_map.insert(name.clone(), var_offset);
                var_offset += 8; // int64_t = 8 bytes
            }
        }
        
        // Generar función main
        writeln!(self.text_section, "main:").unwrap();
        writeln!(self.text_section, "    push rbp").unwrap();
        writeln!(self.text_section, "    mov rbp, rsp").unwrap();
        writeln!(self.text_section, "    sub rsp, {}  ; Reservar espacio para variables", var_offset).unwrap();
        writeln!(self.text_section).unwrap();
        
        // Segunda pasada: Generar código dentro de main
        for stmt in &program.statements {
            match stmt {
                Stmt::Let { name, value, .. } => {
                    // Inicializar variable
                    self.generate_expression_to_rax_with_map(value, &var_map);
                    if let Some(offset) = var_map.get(name) {
                        writeln!(self.text_section, "    mov [rbp - {}], rax  ; {} = ...", offset, name).unwrap();
                    }
                }
                Stmt::Print(expr) => {
                    self.generate_print_clean_with_map(expr, &var_map);
                }
                Stmt::While { condition, body } => {
                    self.generate_while_clean_with_map(condition, body, &var_map);
                }
                Stmt::If { condition, then_body, else_body } => {
                    self.generate_if_clean_with_map(condition, then_body, else_body.as_ref(), &var_map);
                }
                Stmt::Expr(Expr::Assign { name, value }) => {
                    // Asignación: variable = valor
                    if let Some(offset) = var_map.get(name) {
                        self.generate_expression_to_rax_with_map(value, &var_map);
                        writeln!(self.text_section, "    mov [rbp - {}], rax  ; {} = ...", offset, name).unwrap();
                    }
                }
                _ => {}
            }
        }
        
        writeln!(self.text_section).unwrap();
        writeln!(self.text_section, "    xor eax, eax  ; return 0").unwrap();
        writeln!(self.text_section, "    leave").unwrap();
        writeln!(self.text_section, "    ret").unwrap();
    }
    
    fn generate_expression_to_rax_with_map(&mut self, expr: &Expr, var_map: &std::collections::HashMap<String, i32>) {
        use std::fmt::Write;
        match expr {
            Expr::Number(n) => {
                writeln!(self.text_section, "    mov rax, {}", n).unwrap();
            }
            Expr::Ident(name) => {
                if let Some(offset) = var_map.get(name) {
                    writeln!(self.text_section, "    mov rax, [rbp - {}]  ; {}", offset, name).unwrap();
                } else {
                    writeln!(self.text_section, "    mov rax, 0  ; variable {} no encontrada", name).unwrap();
                }
            }
            Expr::BinaryOp { op, left, right } => {
                self.generate_expression_to_rax_with_map(left, var_map);
                writeln!(self.text_section, "    push rax").unwrap();
                self.generate_expression_to_rax_with_map(right, var_map);
                writeln!(self.text_section, "    pop rbx").unwrap();
                
                match op {
                    BinOp::Add => writeln!(self.text_section, "    add rbx, rax").unwrap(),
                    BinOp::Sub => writeln!(self.text_section, "    sub rbx, rax").unwrap(),
                    BinOp::Mul => {
                        writeln!(self.text_section, "    imul rbx, rax").unwrap();
                        writeln!(self.text_section, "    mov rax, rbx").unwrap();
                        return;
                    }
                    BinOp::Div => {
                        writeln!(self.text_section, "    mov rax, rbx").unwrap();
                        writeln!(self.text_section, "    cqo").unwrap();
                        writeln!(self.text_section, "    idiv rax").unwrap();
                        return;
                    }
                    BinOp::Mod => {
                        writeln!(self.text_section, "    mov rax, rbx").unwrap();
                        writeln!(self.text_section, "    cqo").unwrap();
                        writeln!(self.text_section, "    idiv rax").unwrap();
                        writeln!(self.text_section, "    mov rax, rdx").unwrap();
                        return;
                    }
                    BinOp::Eq => {
                        writeln!(self.text_section, "    cmp rbx, rax").unwrap();
                        writeln!(self.text_section, "    sete al").unwrap();
                        writeln!(self.text_section, "    movzx rax, al").unwrap();
                        return;
                    }
                    BinOp::Le => {
                        writeln!(self.text_section, "    cmp rbx, rax").unwrap();
                        writeln!(self.text_section, "    setle al").unwrap();
                        writeln!(self.text_section, "    movzx rax, al").unwrap();
                        return;
                    }
                    _ => {
                        writeln!(self.text_section, "    add rbx, rax").unwrap();
                    }
                }
                writeln!(self.text_section, "    mov rax, rbx").unwrap();
            }
            _ => {
                writeln!(self.text_section, "    mov rax, 0  ; expresión no soportada").unwrap();
            }
        }
    }
    
    fn generate_print_clean_with_map(&mut self, expr: &Expr, var_map: &std::collections::HashMap<String, i32>) {
        use std::fmt::Write;
        match expr {
            Expr::String(s) => {
                let label = format!("str_{}", self.label_count);
                self.label_count += 1;
                writeln!(self.data_section, "{} db '{}', 10, 0", label, s.replace("'", "''")).unwrap();
                writeln!(self.text_section, "    lea rcx, [{}]", label).unwrap();
                writeln!(self.text_section, "    call puts").unwrap();
            }
            Expr::Number(_) | Expr::Ident(_) => {
                // Print número o variable - usar printf
                self.generate_expression_to_rax_with_map(expr, var_map);
                // Usar printf("%ld\n", rax)
                let format_label = format!("fmt_ld_{}", self.label_count);
                self.label_count += 1;
                writeln!(self.data_section, "{} db '%ld', 10, 0", format_label).unwrap();
                writeln!(self.text_section, "    lea rcx, [{}]", format_label).unwrap();
                writeln!(self.text_section, "    mov rdx, rax").unwrap();
                writeln!(self.text_section, "    sub rsp, 32  ; Shadow space").unwrap();
                writeln!(self.text_section, "    call printf").unwrap();
                writeln!(self.text_section, "    add rsp, 32").unwrap();
            }
            _ => {
                self.generate_expression_to_rax_with_map(expr, var_map);
                let format_label = format!("fmt_ld_{}", self.label_count);
                self.label_count += 1;
                writeln!(self.data_section, "{} db '%ld', 10, 0", format_label).unwrap();
                writeln!(self.text_section, "    lea rcx, [{}]", format_label).unwrap();
                writeln!(self.text_section, "    mov rdx, rax").unwrap();
                writeln!(self.text_section, "    sub rsp, 32").unwrap();
                writeln!(self.text_section, "    call printf").unwrap();
                writeln!(self.text_section, "    add rsp, 32").unwrap();
            }
        }
    }
    
    fn generate_while_clean_with_map(&mut self, condition: &Expr, body: &[Stmt], var_map: &std::collections::HashMap<String, i32>) {
        use std::fmt::Write;
        let loop_start = format!("loop_start_{}", self.label_count);
        let loop_end = format!("loop_end_{}", self.label_count);
        self.label_count += 1;
        
        writeln!(self.text_section, "{}:", loop_start).unwrap();
        
        // Generar condición
        self.generate_expression_to_rax_with_map(condition, var_map);
        writeln!(self.text_section, "    cmp rax, 0").unwrap();
        writeln!(self.text_section, "    je {}", loop_end).unwrap();
        
        // Generar body
        for stmt in body {
            match stmt {
                Stmt::Print(expr) => self.generate_print_clean_with_map(expr, var_map),
                Stmt::If { condition, then_body, else_body } => {
                    self.generate_if_clean_with_map(condition, then_body, else_body.as_ref(), var_map);
                }
                Stmt::Expr(Expr::Assign { name, value }) => {
                    if let Some(offset) = var_map.get(name) {
                        self.generate_expression_to_rax_with_map(value, var_map);
                        writeln!(self.text_section, "    mov [rbp - {}], rax  ; {} = ...", offset, name).unwrap();
                    }
                }
                _ => {}
            }
        }
        
        writeln!(self.text_section, "    jmp {}", loop_start).unwrap();
        writeln!(self.text_section, "{}:", loop_end).unwrap();
    }
    
    fn generate_if_clean_with_map(&mut self, condition: &Expr, then_body: &[Stmt], else_body: Option<&Vec<Stmt>>, var_map: &std::collections::HashMap<String, i32>) {
        use std::fmt::Write;
        let if_end = format!("if_end_{}", self.label_count);
        let if_else = format!("if_else_{}", self.label_count);
        self.label_count += 1;
        
        // Generar condición
        self.generate_expression_to_rax_with_map(condition, var_map);
        writeln!(self.text_section, "    cmp rax, 0").unwrap();
        
        if else_body.is_some() {
            writeln!(self.text_section, "    je {}", if_else).unwrap();
        } else {
            writeln!(self.text_section, "    je {}", if_end).unwrap();
        }
        
        // Generar then_body
        for stmt in then_body {
            match stmt {
                Stmt::Print(expr) => self.generate_print_clean_with_map(expr, var_map),
                Stmt::While { condition, body } => {
                    self.generate_while_clean_with_map(condition, body, var_map);
                }
                Stmt::If { condition, then_body, else_body } => {
                    self.generate_if_clean_with_map(condition, then_body, else_body.as_ref(), var_map);
                }
                _ => {}
            }
        }
        
        if let Some(else_body_stmts) = else_body {
            writeln!(self.text_section, "    jmp {}", if_end).unwrap();
            writeln!(self.text_section, "{}:", if_else).unwrap();
            for stmt in else_body_stmts {
                match stmt {
                    Stmt::Print(expr) => self.generate_print_clean_with_map(expr, var_map),
                    _ => {}
                }
            }
        }
        
        writeln!(self.text_section, "{}:", if_end).unwrap();
    }
}

/// Función pública para generar ASM limpio desde código C (vía AST de ADead)
pub fn generate_clean_nasm_from_c(program: &Program) -> String {
    let mut generator = CToNASMGenerator::new();
    generator.generate_clean_nasm_from_c_ast(program)
}

