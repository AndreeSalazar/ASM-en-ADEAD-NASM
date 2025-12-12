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

        let mut output = String::new();
        if !self.data_section.is_empty() {
            output.push_str("section .data\n");
            for line in &self.data_section {
                output.push_str(line);
                output.push('\n');
            }
            output.push('\n');
        }
        for line in &self.text_section {
            output.push_str(line);
            output.push('\n');
        }
        Ok(output)
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
            Stmt::Let { name, value } => {
                self.generate_expr(value)?;
                // Store in stack (simplified: just track as variable)
                // Check if variable already exists (for reassignment)
                let offset = if let Some(&existing_offset) = self.variables.get(name) {
                    existing_offset
                } else {
                    let offset = self.stack_offset;
                    self.stack_offset += 8;
                    self.variables.insert(name.clone(), offset);
                    offset
                };
                self.text_section
                    .push(format!("    mov [rbp - {}], rax", offset + 8));
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
                    self.variables.insert(param.clone(), offset);
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

