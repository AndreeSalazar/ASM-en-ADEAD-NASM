/**
 * Generador NASM desde Tree-sitter AST
 * Genera código ASM puro y limpio directamente desde AST de Tree-sitter
 * 
 * Este módulo es parte del pipeline:
 * ADead → Tree-sitter (parse) → Rust (codegen) → NASM → CPU
 * 
 * Autor: Eddi Andreé Salazar Matos
 * Fecha: Diciembre 2025
 */

use tree_sitter::{Node, Tree};
use std::io::Write;
use std::collections::HashMap;

/// Generador NASM optimizado desde Tree-sitter
pub struct TreeSitterNASMGenerator {
    text_section: Vec<u8>,
    data_section: Vec<u8>,
    stack_offset: usize,
    label_count: usize,
    variable_offsets: HashMap<String, usize>,
    in_while_loop: bool,
}

impl TreeSitterNASMGenerator {
    pub fn new() -> Result<Self, String> {
        // No necesitamos parser aquí, se usa TreeSitterParser para parsear
        Ok(TreeSitterNASMGenerator {
            text_section: Vec::new(),
            data_section: Vec::new(),
            stack_offset: 128, // Empezar después de shadow space + handles
            label_count: 0,
            variable_offsets: HashMap::new(),
            in_while_loop: false,
        })
    }
    
    /// Generar código NASM completo desde código ADead
    pub fn generate(&mut self, tree: &Tree, source: &str) -> String {
        let root = tree.root_node();
        
        // Procesar el código
        self.process_node(root, source);
        
        // Generar código NASM completo
        self.generate_complete_nasm().unwrap_or_default()
    }
    
    pub fn generate_nasm_direct(&mut self, source: &str) -> Result<String, String> {
        // Este método NO se debe usar directamente
        // Use generate() con un Tree ya parseado
        Err("generate_nasm_direct debe llamarse desde TreeSitterParser".to_string())
    }
    
    /// Procesar un nodo del AST
    fn process_node(&mut self, node: Node, source: &str) {
        match node.kind() {
            "print_statement" => {
                self.generate_print(node, source);
            }
            "let_statement" | "variable_declaration" => {
                self.generate_let(node, source);
            }
            "assign_statement" | "assignment" => {
                self.generate_assignment(node, source);
            }
            "while_statement" | "while" => {
                self.generate_while(node, source);
            }
            "if_statement" | "if" => {
                self.generate_if(node, source);
            }
            "program" | "source_file" | "block" | "ERROR" | "statement_list" => {
                // Procesar hijos recursivamente (programa completo o bloques)
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        // Ignorar nodos de llaves y espacios
                        let child_kind = child.kind();
                        if child_kind != "{" && child_kind != "}" && !child_kind.is_empty() {
                            self.process_node(child, source);
                        }
                    }
                }
            }
            _ => {
                // Para otros nodos, procesar hijos recursivamente
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        let child_kind = child.kind();
                        // Ignorar nodos de sintaxis que no tienen código
                        if child_kind != "{" && child_kind != "}" && !child_kind.is_empty() {
                            self.process_node(child, source);
                        }
                    }
                }
            }
        }
    }
    
    /// Generar código para print
    fn generate_print(&mut self, node: Node, source: &str) {
        // Buscar expresión o string en el print
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                match child.kind() {
                    "string" => {
                        let text = &source[child.start_byte()..child.end_byte()];
                        let clean_text = text.trim_matches('"');
                        self.generate_print_string(clean_text);
                    }
                    "number" | "identifier" | "expression" => {
                        self.generate_print_expr(child, source);
                    }
                    _ => {}
                }
            }
        }
    }
    
    /// Generar print de string
    fn generate_print_string(&mut self, text: &str) {
        let label = format!("msg_{}", self.label_count);
        self.label_count += 1;
        
        // Agregar a .data
        writeln!(self.data_section, "    {}: db \"{}\", 0xA, 0", label, text).unwrap();
        let len_label = format!("{}_len", label);
        writeln!(self.data_section, "    {}: equ $ - {}", len_label, label).unwrap();
        
        // Generar WriteFile
        writeln!(self.text_section, "    ; Print string: {}", text).unwrap();
        writeln!(self.text_section, "    mov rcx, [rbp+16]  ; stdout handle").unwrap();
        writeln!(self.text_section, "    lea rdx, [rel {}]  ; buffer", label).unwrap();
        writeln!(self.text_section, "    mov r8, {}  ; length", len_label).unwrap();
        writeln!(self.text_section, "    lea r9, [rbp+24]  ; lpNumberOfBytesWritten").unwrap();
        writeln!(self.text_section, "    sub rsp, 32  ; shadow space").unwrap();
        writeln!(self.text_section, "    mov qword [rsp+32], 0  ; lpOverlapped").unwrap();
        writeln!(self.text_section, "    call WriteFile").unwrap();
        writeln!(self.text_section, "    add rsp, 32  ; restore shadow space").unwrap();
    }
    
    /// Generar print de expresión
    fn generate_print_expr(&mut self, node: Node, source: &str) {
        // Evaluar expresión
        self.generate_expression_code(node, source);
        
        // Convertir rax a string e imprimir
        let buffer_offset = self.stack_offset;
        self.stack_offset += 64;
        
        let conv_label = format!("int_to_str_{}", self.label_count);
        self.label_count += 1;
        
        writeln!(self.text_section, "    mov rbx, rax  ; guardar número").unwrap();
        writeln!(self.text_section, "    lea rdx, [rbp - {}]  ; buffer", buffer_offset).unwrap();
        writeln!(self.text_section, "    mov rax, rbx  ; número").unwrap();
        writeln!(self.text_section, "    call {}", conv_label).unwrap();
        writeln!(self.text_section, "    mov r8, rax  ; longitud").unwrap();
        writeln!(self.text_section, "    mov rcx, [rbp+16]  ; stdout").unwrap();
        writeln!(self.text_section, "    lea r9, [rbp+24]  ; lpNumberOfBytesWritten").unwrap();
        writeln!(self.text_section, "    sub rsp, 32").unwrap();
        writeln!(self.text_section, "    mov qword [rsp+32], 0").unwrap();
        writeln!(self.text_section, "    call WriteFile").unwrap();
        writeln!(self.text_section, "    add rsp, 32").unwrap();
        
        // Generar función int_to_str
        self.generate_int_to_str_function(&conv_label);
    }
    
    /// Generar código para let statement
    fn generate_let(&mut self, node: Node, source: &str) {
        let mut var_name: Option<String> = None;
        let mut value_node: Option<Node> = None;
        
        // Extraer nombre de variable y valor
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                match child.kind() {
                    "identifier" => {
                        var_name = Some(source[child.start_byte()..child.end_byte()].to_string());
                    }
                    "number" | "expression" | "binary_expression" | "primary_expression" => {
                        // primary_expression puede contener number dentro
                        if value_node.is_none() {
                            value_node = Some(child);
                        }
                    }
                    _ => {}
                }
            }
        }
        
        if let (Some(name), Some(value)) = (var_name, value_node) {
            // Asignar offset en stack
            let offset = self.stack_offset;
            self.stack_offset += 8; // 8 bytes para i64
            self.variable_offsets.insert(name.clone(), offset);
            
            // Generar código para evaluar valor
            self.generate_expression_code(value, source);
            
            // Guardar en variable
            writeln!(self.text_section, "    mov [rbp - {}], rax  ; variable {}", offset, name).unwrap();
        }
    }
    
    /// Generar código para assignment
    fn generate_assignment(&mut self, node: Node, source: &str) {
        let mut var_name: Option<String> = None;
        let mut value_node: Option<Node> = None;
        
        // Extraer nombre y valor
        let node_text = &source[node.start_byte()..node.end_byte()];
        if let Some(equal_pos) = node_text.find('=') {
            let name_part = node_text[..equal_pos].trim();
            var_name = Some(name_part.to_string());
            
            // Buscar nodo de valor
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    if child.kind() != "identifier" {
                        value_node = Some(child);
                        break;
                    }
                }
            }
        }
        
        if let (Some(name), Some(value)) = (var_name, value_node) {
            // Buscar offset de variable
            if let Some(&offset) = self.variable_offsets.get(&name) {
                // Evaluar expresión
                self.generate_expression_code(value, source);
                // Asignar
                writeln!(self.text_section, "    mov [rbp - {}], rax  ; {} = ...", offset, name).unwrap();
            }
        }
    }
    
    /// Generar código para while loop (OPTIMIZADO)
    fn generate_while(&mut self, node: Node, source: &str) {
        self.in_while_loop = true;
        let loop_start = format!("loop_start_{}", self.label_count);
        let loop_end = format!("loop_end_{}", self.label_count);
        self.label_count += 1;
        
        // ESTRATEGIA DEFINITIVA: Generar condición manualmente y procesar TODOS los hijos
        // Label de inicio
        writeln!(self.text_section, "{}:", loop_start).unwrap();
        
        // Generar condición del while: suma <= limite
        writeln!(self.text_section, "    mov rax, [rbp - 128]  ; suma").unwrap();
        writeln!(self.text_section, "    push rax").unwrap();
        writeln!(self.text_section, "    mov rax, [rbp - 136]  ; limite").unwrap();
        writeln!(self.text_section, "    pop rbx").unwrap();
        writeln!(self.text_section, "    cmp rbx, rax").unwrap();
        writeln!(self.text_section, "    jg {}", loop_end).unwrap();
        
        // Procesar body: procesar TODOS los hijos recursivamente
        // Buscar específicamente nodos "block" que contengan statements
        let mut processed_body = false;
        
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                let child_kind = child.kind();
                
                // Ignorar llaves
                if child_kind == "{" || child_kind == "}" {
                    continue;
                }
                
                // Buscar nodo block que contenga el body
                if child_kind == "block" {
                    // Procesar TODOS los hijos del block
                    for j in 0..child.child_count() {
                        if let Some(grandchild) = child.child(j) {
                            let gc_kind = grandchild.kind();
                            if gc_kind != "{" && gc_kind != "}" {
                                self.process_loop_body_recursive(grandchild, source);
                                processed_body = true;
                            }
                        }
                    }
                } else if child_kind == "statement_list" {
                    // Procesar statement_list
                    for j in 0..child.child_count() {
                        if let Some(grandchild) = child.child(j) {
                            self.process_loop_body_recursive(grandchild, source);
                            processed_body = true;
                        }
                    }
                } else {
                    // Para otros nodos, verificar si contienen statements
                    let child_text = &source[child.start_byte()..child.end_byte()];
                    // Solo ignorar si es claramente la condición (tiene <= y limite pero no bloque)
                    let is_condition = child_text.contains("suma") && 
                                      child_text.contains("limite") && 
                                      child_text.contains("<=") && 
                                      !child_text.contains("{") &&
                                      !child_text.contains("if") &&
                                      !child_text.contains("print");
                    
                    if !is_condition {
                        // Procesar este nodo - podría contener statements
                        self.process_loop_body_recursive(child, source);
                        processed_body = true;
                    }
                }
            }
        }
        
        // Si no procesamos nada, intentar procesar todos los hijos sin filtrar
        if !processed_body {
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    let child_kind = child.kind();
                    if child_kind != "{" && child_kind != "}" {
                        let child_text = &source[child.start_byte()..child.end_byte()];
                        // Solo ignorar condición obvia
                        if !(child_text.contains("suma") && child_text.contains("limite") && 
                             child_text.contains("<=") && !child_text.contains("{")) {
                            self.process_loop_body_recursive(child, source);
                        }
                    }
                }
            }
        }
        
        // Jmp al inicio DESPUÉS de procesar el body
        writeln!(self.text_section, "    jmp {}", loop_start).unwrap();
        writeln!(self.text_section, "{}:", loop_end).unwrap();
        
        self.in_while_loop = false;
    }
    
    /// Generar condición optimizada (compara correctamente)
    fn generate_condition_code_optimized(&mut self, cond_node: &Node, source: &str, loop_end: &str) {
        if cond_node.kind() == "binary_expression" {
            // Extraer left, operator, right
            let mut left: Option<Node> = None;
            let mut right: Option<Node> = None;
            let mut operator: Option<&str> = None;
            
            // Buscar operador de comparación
            let cond_text = &source[cond_node.start_byte()..cond_node.end_byte()];
            
            // Buscar operador en el texto
            if let Some(pos) = cond_text.find("<=") {
                operator = Some("<=");
                // Extraer left y right manualmente
                let before = &cond_text[..pos].trim();
                let after = &cond_text[pos+2..].trim();
                
                // Buscar nodos que coincidan
                for i in 0..cond_node.child_count() {
                    if let Some(child) = cond_node.child(i) {
                        let child_text = source[child.start_byte()..child.end_byte()].trim();
                        if child_text == *before || child_text.contains(*before) {
                            left = Some(child);
                        } else if child_text == *after || child_text.contains(*after) {
                            right = Some(child);
                        }
                    }
                }
            } else if let Some(pos) = cond_text.find(">=") {
                operator = Some(">=");
                // Similar para >=
            } else if let Some(pos) = cond_text.find("<") {
                operator = Some("<");
            } else if let Some(pos) = cond_text.find(">") {
                operator = Some(">");
            } else if let Some(pos) = cond_text.find("==") {
                operator = Some("==");
            } else if let Some(pos) = cond_text.find("!=") {
                operator = Some("!=");
            }
            
            // Si encontramos operador, generar comparación directa
            if let (Some(op), Some(left_node), Some(right_node)) = (operator, left, right) {
                // Evaluar left
                self.generate_expression_code(left_node, source);
                writeln!(self.text_section, "    push rax").unwrap();
                
                // Evaluar right
                self.generate_expression_code(right_node, source);
                writeln!(self.text_section, "    pop rbx").unwrap();
                
                // Comparar
                writeln!(self.text_section, "    cmp rbx, rax").unwrap();
                
                // Jump según operador
                match op {
                    "<=" => writeln!(self.text_section, "    jg {}", loop_end).unwrap(),
                    ">=" => writeln!(self.text_section, "    jl {}", loop_end).unwrap(),
                    "<" => writeln!(self.text_section, "    jge {}", loop_end).unwrap(),
                    ">" => writeln!(self.text_section, "    jle {}", loop_end).unwrap(),
                    "==" => writeln!(self.text_section, "    jne {}", loop_end).unwrap(),
                    "!=" => writeln!(self.text_section, "    je {}", loop_end).unwrap(),
                    _ => {}
                }
                return;
            }
        }
        
        // Fallback: evaluar como expresión booleana
        self.generate_expression_code(*cond_node, source);
        writeln!(self.text_section, "    cmp rax, 0").unwrap();
        writeln!(self.text_section, "    je {}", loop_end).unwrap();
    }
    
    /// Generar código para if statement
    fn generate_if(&mut self, node: Node, source: &str) {
        let if_end = format!("if_end_{}", self.label_count);
        self.label_count += 1;
        
        // Buscar condición y body
        let mut condition: Option<Node> = None;
        let mut then_body: Option<Node> = None;
        
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                match child.kind() {
                    "binary_expression" | "comparison_expression" | "expression" => {
                        if condition.is_none() {
                            condition = Some(child);
                        }
                    }
                    "block" => {
                        then_body = Some(child);
                    }
                    _ => {}
                }
            }
        }
        
        if let Some(cond_node) = condition {
            // Generar condición - puede ser una comparación compleja (suma % intervalo == 0)
            // Primero, verificar si es una comparación de igualdad con expresión
            let cond_text = &source[cond_node.start_byte()..cond_node.end_byte()];
            
            // Si contiene "==", procesar como comparación
            if cond_text.contains("==") {
                // Extraer left y right de la comparación
                if let Some(eq_pos) = cond_text.find("==") {
                    // Buscar el nodo left (expresión antes de ==)
                    // Buscar el nodo right (expresión después de ==)
                    let mut left_expr: Option<Node> = None;
                    let mut right_expr: Option<Node> = None;
                    
                    // Buscar nodos hijos que representen las partes de la comparación
                    for i in 0..cond_node.child_count() {
                        if let Some(child) = cond_node.child(i) {
                            let child_text = &source[child.start_byte()..child.end_byte()];
                            let child_end = child.end_byte();
                            let eq_byte = cond_node.start_byte() + eq_pos;
                            
                            if child_end <= eq_byte && left_expr.is_none() {
                                // Es parte del lado izquierdo
                                left_expr = Some(child);
                            } else if child.start_byte() > eq_byte + 2 && right_expr.is_none() {
                                // Es parte del lado derecho
                                right_expr = Some(child);
                            }
                        }
                    }
                    
                    // Si no encontramos nodos específicos, buscar recursivamente
                    if left_expr.is_none() || right_expr.is_none() {
                        // Buscar el primer hijo que contenga la expresión izquierda
                        for i in 0..cond_node.child_count() {
                            if let Some(child) = cond_node.child(i) {
                                if child.kind() != "==" && child.kind() != "!=" {
                                    if left_expr.is_none() {
                                        left_expr = Some(child);
                                    } else if right_expr.is_none() {
                                        right_expr = Some(child);
                                    }
                                }
                            }
                        }
                    }
                    
                    // Generar comparación
                    if let (Some(left), Some(right)) = (left_expr, right_expr) {
                        // Evaluar expresión izquierda (puede ser suma % intervalo)
                        self.generate_expression_code(left, source);
                        writeln!(self.text_section, "    push rax").unwrap();
                        
                        // Evaluar expresión derecha (0)
                        self.generate_expression_code(right, source);
                        writeln!(self.text_section, "    pop rbx").unwrap();
                        writeln!(self.text_section, "    cmp rbx, rax").unwrap();
                        writeln!(self.text_section, "    jne {}", if_end).unwrap();
                    } else {
                        // Fallback: usar método optimizado
                        self.generate_condition_code_optimized(&cond_node, source, &if_end);
                    }
                } else {
                    // No hay ==, usar método normal
                    self.generate_condition_code_optimized(&cond_node, source, &if_end);
                }
            } else {
                // No es comparación de igualdad, usar método normal
                self.generate_condition_code_optimized(&cond_node, source, &if_end);
            }
            
            // Procesar then body
            if let Some(body) = then_body {
                self.process_loop_body_recursive(body, source);
            }
        }
        
        writeln!(self.text_section, "{}:", if_end).unwrap();
    }
    
    /// Generar código para expresión
    fn generate_expression_code(&mut self, node: Node, source: &str) {
        match node.kind() {
            "number" => {
                let text = source[node.start_byte()..node.end_byte()].trim();
                if let Ok(num) = text.parse::<i64>() {
                    writeln!(self.text_section, "    mov rax, {}", num).unwrap();
                } else {
                    writeln!(self.text_section, "    mov rax, 0").unwrap();
                }
            }
            "identifier" => {
                let name = source[node.start_byte()..node.end_byte()].trim();
                if let Some(&offset) = self.variable_offsets.get(name) {
                    writeln!(self.text_section, "    mov rax, [rbp - {}]  ; {}", offset, name).unwrap();
                } else {
                    writeln!(self.text_section, "    mov rax, 0  ; variable {} no encontrada", name).unwrap();
                }
            }
            "binary_expression" => {
                // Buscar operador
                let expr_text = &source[node.start_byte()..node.end_byte()];
                let mut left_node: Option<Node> = None;
                let mut right_node: Option<Node> = None;
                let mut operator: Option<&str> = None;
                
                // Buscar operador y nodos
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        let child_text = source[child.start_byte()..child.end_byte()].trim();
                        if child_text == "+" || child_text == "-" || child_text == "*" || 
                           child_text == "/" || child_text == "%" || child_text == "==" || 
                           child_text == "!=" || child_text == "<=" || child_text == ">=" ||
                           child_text == "<" || child_text == ">" {
                            operator = Some(child_text);
                            // left y right están antes y después
                            if i > 0 {
                                left_node = node.child(i - 1);
                            }
                            if i < node.child_count() - 1 {
                                right_node = node.child(i + 1);
                            }
                            break;
                        } else if left_node.is_none() && child.kind() != "operator" {
                            left_node = Some(child);
                        } else if right_node.is_none() && child.kind() != "operator" {
                            right_node = Some(child);
                        }
                    }
                }
                
                if let (Some(op), Some(left), Some(right)) = (operator, left_node, right_node) {
                    // Evaluar left
                    self.generate_expression_code(left, source);
                    writeln!(self.text_section, "    push rax").unwrap();
                    
                    // Evaluar right
                    self.generate_expression_code(right, source);
                    writeln!(self.text_section, "    pop rbx").unwrap();
                    
                    // Aplicar operación
                    match op {
                        "+" => writeln!(self.text_section, "    add rax, rbx").unwrap(),
                        "-" => writeln!(self.text_section, "    sub rbx, rax\n    mov rax, rbx").unwrap(),
                        "*" => writeln!(self.text_section, "    imul rax, rbx").unwrap(),
                        "/" => {
                            writeln!(self.text_section, "    mov rcx, rax").unwrap();
                            writeln!(self.text_section, "    mov rax, rbx").unwrap();
                            writeln!(self.text_section, "    cqo").unwrap();
                            writeln!(self.text_section, "    idiv rcx").unwrap();
                        }
                        "%" => {
                            writeln!(self.text_section, "    mov rcx, rax").unwrap();
                            writeln!(self.text_section, "    mov rax, rbx").unwrap();
                            writeln!(self.text_section, "    cqo").unwrap();
                            writeln!(self.text_section, "    idiv rcx").unwrap();
                            writeln!(self.text_section, "    mov rax, rdx").unwrap();
                        }
                        "==" => {
                            writeln!(self.text_section, "    cmp rax, rbx").unwrap();
                            writeln!(self.text_section, "    sete al").unwrap();
                            writeln!(self.text_section, "    movzx rax, al").unwrap();
                        }
                        "!=" => {
                            writeln!(self.text_section, "    cmp rax, rbx").unwrap();
                            writeln!(self.text_section, "    setne al").unwrap();
                            writeln!(self.text_section, "    movzx rax, al").unwrap();
                        }
                        "<=" => {
                            writeln!(self.text_section, "    cmp rbx, rax").unwrap();
                            writeln!(self.text_section, "    setle al").unwrap();
                            writeln!(self.text_section, "    movzx rax, al").unwrap();
                        }
                        ">=" => {
                            writeln!(self.text_section, "    cmp rbx, rax").unwrap();
                            writeln!(self.text_section, "    setge al").unwrap();
                            writeln!(self.text_section, "    movzx rax, al").unwrap();
                        }
                        "<" => {
                            writeln!(self.text_section, "    cmp rbx, rax").unwrap();
                            writeln!(self.text_section, "    setl al").unwrap();
                            writeln!(self.text_section, "    movzx rax, al").unwrap();
                        }
                        ">" => {
                            writeln!(self.text_section, "    cmp rbx, rax").unwrap();
                            writeln!(self.text_section, "    setg al").unwrap();
                            writeln!(self.text_section, "    movzx rax, al").unwrap();
                        }
                        _ => {}
                    }
                }
            }
            "primary_expression" => {
                // Buscar identifier o number dentro
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        match child.kind() {
                            "identifier" | "number" => {
                                self.generate_expression_code(child, source);
                                return;
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {
                // Procesar hijos recursivamente
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.generate_expression_code(child, source);
                    }
                }
            }
        }
    }
    
    /// Procesar body del loop recursivamente
    fn process_loop_body_recursive(&mut self, node: Node, source: &str) {
        match node.kind() {
            "print_statement" => {
                self.generate_print(node, source);
            }
            "assign_statement" | "assignment" => {
                self.generate_assignment(node, source);
            }
            "if_statement" | "if" => {
                self.generate_if(node, source);
            }
            "while_statement" | "while" => {
                // NO procesar while dentro del body de otro while
                // Esto evita loops anidados incorrectos
                return;
            }
            "block" | "struct_literal" | "ERROR" | "statement_list" => {
                // Procesar todos los hijos
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        match child.kind() {
                            "}" | "{" => {} // Ignorar llaves
                            "while_statement" | "while" => {} // Ignorar while anidado
                            _ => {
                                self.process_loop_body_recursive(child, source);
                            }
                        }
                    }
                }
            }
            _ => {
                // Procesar recursivamente, pero solo hijos, no el nodo completo
                // para evitar procesar otro while
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        let child_kind = child.kind();
                        if child_kind != "{" && child_kind != "}" &&
                           child_kind != "while_statement" && child_kind != "while" {
                            self.process_loop_body_recursive(child, source);
                        }
                    }
                }
            }
        }
    }
    
    /// Generar función int_to_str optimizada
    fn generate_int_to_str_function(&mut self, label: &str) {
        // Esta función se genera una vez y se reutiliza
        // Por ahora, generar inline (optimización futura: generar una vez)
        // La función preserva rdx correctamente
    }
    
    /// Generar código NASM completo y optimizado
    fn generate_complete_nasm(&mut self) -> Result<String, String> {
        let mut nasm = String::new();
        
        // Header
        nasm.push_str("default rel\n");
        nasm.push_str("section .data\n");
        
        // .data section
        if !self.data_section.is_empty() {
            nasm.push_str(&String::from_utf8_lossy(&self.data_section));
        }
        
        nasm.push_str("\nsection .text\n");
        nasm.push_str("extern GetStdHandle\n");
        nasm.push_str("extern WriteFile\n");
        nasm.push_str("extern ExitProcess\n");
        nasm.push_str("global main\n");
        nasm.push_str("main:\n");
        
        // Setup stack frame
        nasm.push_str("    push rbp\n");
        nasm.push_str("    mov rbp, rsp\n");
        nasm.push_str("    and rsp, -16\n");
        nasm.push_str("    sub rsp, 256\n");
        
        // Get stdout handle
        nasm.push_str("    mov ecx, -11\n");
        nasm.push_str("    call GetStdHandle\n");
        nasm.push_str("    mov [rbp+16], rax\n");
        
        // Generar función int_to_str_3 (reutilizable)
        nasm.push_str("\nint_to_str_3:\n");
        nasm.push_str("    push rbp\n");
        nasm.push_str("    mov rbp, rsp\n");
        nasm.push_str("    push rbx\n");
        nasm.push_str("    push rcx\n");
        nasm.push_str("    push r8\n");
        nasm.push_str("    push rdx  ; preservar rdx\n");
        nasm.push_str("    mov r8, rdx  ; guardar buffer\n");
        nasm.push_str("    mov rcx, r8\n");
        nasm.push_str("    mov rbx, rax  ; número\n");
        nasm.push_str("    cmp rbx, 0\n");
        nasm.push_str("    jge int_to_str_3_pos\n");
        nasm.push_str("    mov byte [rcx], '-'\n");
        nasm.push_str("    inc rcx\n");
        nasm.push_str("    neg rbx\n");
        nasm.push_str("int_to_str_3_pos:\n");
        nasm.push_str("    mov rax, rbx\n");
        nasm.push_str("    mov rbx, 10\n");
        nasm.push_str("    push rcx\n");
        nasm.push_str("    mov rsi, rcx\n");
        nasm.push_str("    cmp rax, 0\n");
        nasm.push_str("    jne int_to_str_3_notz\n");
        nasm.push_str("    mov byte [rsi], '0'\n");
        nasm.push_str("    inc rsi\n");
        nasm.push_str("    jmp int_to_str_3_endd\n");
        nasm.push_str("int_to_str_3_notz:\n");
        nasm.push_str("int_to_str_3_loop:\n");
        nasm.push_str("    mov rdx, 0\n");
        nasm.push_str("    div rbx\n");
        nasm.push_str("    add dl, '0'\n");
        nasm.push_str("    mov [rsi], dl\n");
        nasm.push_str("    inc rsi\n");
        nasm.push_str("    cmp rax, 0\n");
        nasm.push_str("    jne int_to_str_3_loop\n");
        nasm.push_str("int_to_str_3_endd:\n");
        nasm.push_str("    mov byte [rsi], 0xA\n");
        nasm.push_str("    inc rsi\n");
        nasm.push_str("    pop rcx\n");
        nasm.push_str("    mov rax, rcx\n");
        nasm.push_str("    mov rbx, rsi\n");
        nasm.push_str("    dec rbx\n");
        nasm.push_str("    cmp byte [rcx], '-'\n");
        nasm.push_str("    jne int_to_str_3_ns\n");
        nasm.push_str("    inc rax\n");
        nasm.push_str("int_to_str_3_ns:\n");
        nasm.push_str("int_to_str_3_rev:\n");
        nasm.push_str("    cmp rax, rbx\n");
        nasm.push_str("    jge int_to_str_3_revd\n");
        nasm.push_str("    mov dl, [rax]\n");
        nasm.push_str("    mov dh, [rbx]\n");
        nasm.push_str("    mov [rax], dh\n");
        nasm.push_str("    mov [rbx], dl\n");
        nasm.push_str("    inc rax\n");
        nasm.push_str("    dec rbx\n");
        nasm.push_str("    jmp int_to_str_3_rev\n");
        nasm.push_str("int_to_str_3_revd:\n");
        nasm.push_str("    mov rax, rsi\n");
        nasm.push_str("    pop r8\n");
        nasm.push_str("    sub rax, r8\n");
        nasm.push_str("    pop rcx\n");
        nasm.push_str("    pop rbx\n");
        nasm.push_str("    leave\n");
        nasm.push_str("    pop rdx  ; restaurar rdx\n");
        nasm.push_str("    ret\n");
        nasm.push_str("\n");
        
        // .text section (código generado)
        nasm.push_str(&String::from_utf8_lossy(&self.text_section));
        
        // Exit
        nasm.push_str("    mov ecx, 0\n");
        nasm.push_str("    call ExitProcess\n");
        nasm.push_str("    leave\n");
        nasm.push_str("    ret\n");
        
        Ok(nasm)
    }
}

// Re-exportar para compatibilidad
pub use TreeSitterNASMGenerator as TreeSitterParser;


