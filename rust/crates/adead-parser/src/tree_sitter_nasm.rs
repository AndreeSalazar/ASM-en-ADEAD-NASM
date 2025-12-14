// Tree-sitter → NASM directo (sin pasar por Rust AST)
// Permite generar NASM directamente desde Tree-sitter AST
// Flujo: ADead Source → Tree-sitter → NASM

use tree_sitter::{Tree, Node};
use std::fmt::Write;

pub struct TreeSitterNASMGenerator {
    data_section: String,
    text_section: String,
    label_count: usize,
    msg_count: usize,
    variable_offset: i32,
    variables: std::collections::HashMap<String, i32>, // nombre -> offset en stack
    stack_offset: i32,
    in_while_loop: bool, // Flag para saber si estamos dentro de un while loop
}

impl TreeSitterNASMGenerator {
    pub fn new() -> Self {
        Self {
            data_section: String::new(),
            text_section: String::new(),
            label_count: 0,
            msg_count: 0,
            variable_offset: -8, // Empezar desde [rbp-8]
            variables: std::collections::HashMap::new(),
            stack_offset: 32, // Espacio después de shadow space
            in_while_loop: false, // Inicialmente no estamos en un while
        }
    }

    pub fn generate(&mut self, tree: &Tree, source: &str) -> String {
        let root = tree.root_node();
        
        // DEBUG: Verificar estructura del árbol COMPLETO
        // Escribir a un archivo para asegurar que se vea
        use std::fs::OpenOptions;
        use std::io::Write;
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("tree_debug.txt") {
            writeln!(file, "\n=== DEBUG: Estructura completa del AST ===").ok();
            writeln!(file, "Root node kind: {}, children: {}", root.kind(), root.child_count()).ok();
            self.debug_print_tree_to_file(&root, source, 0, &mut file);
        }
        
        // Generar encabezado NASM
        let mut nasm = String::new();
        nasm.push_str("default rel\n");
        nasm.push_str("section .data\n");
        
        // Procesar el árbol
        self.process_node(root, source);
        
        // Agregar sección de datos generada
        nasm.push_str(&self.data_section);
        nasm.push_str("\nsection .text\n");
        nasm.push_str("extern GetStdHandle\n");
        nasm.push_str("extern WriteFile\n");
        nasm.push_str("extern ExitProcess\n");
        nasm.push_str("global main\n");
        nasm.push_str("main:\n");
        nasm.push_str("    push rbp\n");
        nasm.push_str("    mov rbp, rsp\n");
        nasm.push_str("    and rsp, -16\n");
        
        // Ajustar stack para variables (calcular tamaño necesario)
        let stack_size = if self.stack_offset > 32 {
            // Redondear a múltiplo de 16
            ((self.stack_offset + 15) / 16) * 16
        } else {
            64
        };
        nasm.push_str(&format!("    sub rsp, {}\n", stack_size));
        nasm.push_str("    mov ecx, -11\n");
        nasm.push_str("    call GetStdHandle\n");
        nasm.push_str("    mov [rbp+16], rax\n");
        
        // Agregar código generado
        nasm.push_str(&self.text_section);
        
        // Finalizar
        nasm.push_str("    mov ecx, 0\n");
        nasm.push_str("    call ExitProcess\n");
        nasm.push_str("    leave\n");
        nasm.push_str("    ret\n");
        
        nasm
    }

    fn process_node(&mut self, node: Node, source: &str) {
        let node_kind = node.kind();
        eprintln!("DEBUG process_node: kind={}, start={}, end={}, text='{}'", 
                  node_kind, 
                  node.start_byte(), 
                  node.end_byte(),
                  &source[node.start_byte()..node.end_byte()].replace("\n", "\\n"));
        
        // IMPORTANTE: Detectar cuando Tree-sitter parseea 'while' como ERROR
        // Estructura esperada: ERROR -> binary_expression -> [primary_expression('i'), primary_expression -> struct_literal('max { ... }')]
        // Necesitamos extraer: condition = binary_expression simplificado, body = contenido del struct_literal
        if node_kind == "ERROR" {
            let node_text = &source[node.start_byte()..node.end_byte()];
            // Verificar si contiene 'while' - el texto puede ser 'while i <= max {' o solo 'i <= max {'
            // Buscar en el source_file completo para detectar si hay 'while' antes de este ERROR
            let text_before = &source[..node.start_byte()];
            let has_while_keyword = text_before.trim_end().ends_with("while") || 
                                   node_text.trim_start().starts_with("while") ||
                                   (text_before.contains("while") && node_text.contains('{'));
            
            if has_while_keyword && node_text.contains('{') {
                // Buscar binary_expression dentro del ERROR
                let mut binary_expr: Option<Node> = None;
                let mut struct_lit: Option<Node> = None;
                
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        if child.kind() == "binary_expression" {
                            binary_expr = Some(child);
                            // Dentro del binary_expression, buscar struct_literal en el right side
                            for j in 0..child.child_count() {
                                if let Some(grandchild) = child.child(j) {
                                    if grandchild.kind() == "struct_literal" {
                                        struct_lit = Some(grandchild);
                                        break;
                                    }
                                    // También buscar dentro de primary_expression
                                    if grandchild.kind() == "primary_expression" {
                                        for k in 0..grandchild.child_count() {
                                            if let Some(ggchild) = grandchild.child(k) {
                                                if ggchild.kind() == "struct_literal" {
                                                    struct_lit = Some(ggchild);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
               // Si encontramos binary_expression con struct_literal, es un while mal parseado
               // Procesarlo manualmente usando generate_while para mantener consistencia
               if binary_expr.is_some() && struct_lit.is_some() {
                   eprintln!("DEBUG: While loop detectado como ERROR node, procesando con generate_while...");
                   // Crear un nodo sintético para pasar a generate_while
                   // Pero mejor: procesar directamente usando generate_while pasando el nodo ERROR completo
                   // generate_while buscará la condición y el body dentro del nodo
                   self.generate_while(node, source);
                   // IMPORTANTE: Retornar sin procesar hijos para evitar duplicación
                   return;
               }
            }
            // Si no es un while, procesar recursivamente hijos
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    self.process_node(child, source);
                }
            }
            return;
        }
        
        match node_kind {
            "source_file" => {
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.process_node(child, source);
                    }
                }
            }
            "print_statement" => {
                self.generate_print(node, source);
            }
            "let_statement" => {
                self.generate_let(node, source);
            }
            "while_statement" => {
                eprintln!("DEBUG: Found while_statement in process_node!");
                self.in_while_loop = true; // Estamos en un while loop
                self.generate_while(node, source);
                self.in_while_loop = false; // Ya terminamos el while
            }
            "if_statement" => {
                self.generate_if(node, source);
            }
            "assign_statement" => {
                self.generate_assignment(node, source);
            }
            "assignment" => {
                // Alias por compatibilidad
                self.generate_assignment(node, source);
            }
            _ => {
                // Procesar hijos recursivamente para nodos no reconocidos
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.process_node(child, source);
                    }
                }
            }
        }
    }

    fn generate_print(&mut self, node: Node, source: &str) {
        // Si el nodo es directamente un identifier o number, generar print para ese valor
        if node.kind() == "identifier" {
            // Es un identifier directo (por ejemplo, cuando se llama desde el procesamiento de tokens)
            let var_name = &source[node.start_byte()..node.end_byte()];
            if let Some(&offset) = self.variables.get(var_name) {
                // Cargar variable y generar print COMPLETO (con conversión a string)
                writeln!(self.text_section, "    mov rax, [rbp - {}]  ; cargar variable {} para print", offset, var_name).unwrap();
                
                // Generar print completo con conversión int_to_str
                let buffer_offset = self.stack_offset;
                self.stack_offset += 32;
                let conv_label = format!("int_to_str_{}", self.label_count);
                self.label_count += 1;
                writeln!(self.text_section, "    mov rbx, rax  ; guardar número").unwrap();
                writeln!(self.text_section, "    lea rdx, [rbp - {}]  ; dirección del buffer", buffer_offset).unwrap();
                // int_to_str_3 restaura rdx automáticamente al final, NO hacer push/pop aquí
                writeln!(self.text_section, "    mov rax, rbx  ; número en rax").unwrap();
                writeln!(self.text_section, "    call {}", conv_label).unwrap();
                // PASO 2: int_to_str_3 YA restauró rdx desde r10 antes de ret, así que rdx ya tiene el buffer
                writeln!(self.text_section, "    mov r8, rax  ; longitud").unwrap();
                // NO necesitamos mov rdx, r10 aquí porque int_to_str_3 ya restauró rdx antes de retornar
                writeln!(self.text_section, "    mov rcx, [rbp+16]  ; stdout handle").unwrap();
                writeln!(self.text_section, "    lea r9, [rbp+24]  ; lpNumberOfBytesWritten").unwrap();
                writeln!(self.text_section, "    sub rsp, 32  ; reservar shadow space para WriteFile").unwrap();
                writeln!(self.text_section, "    mov qword [rsp+32], 0  ; lpOverlapped (NULL)").unwrap();
                writeln!(self.text_section, "    call WriteFile").unwrap();
                writeln!(self.text_section, "    add rsp, 32  ; restaurar shadow space").unwrap();
                writeln!(self.text_section, "    test rax, rax  ; verificar retorno de WriteFile").unwrap();
                writeln!(self.text_section, "    jz {}_writefile_error  ; si rax == 0, hubo error", conv_label).unwrap();
                writeln!(self.text_section, "    jmp {}_end", conv_label).unwrap();
                writeln!(self.text_section, "{}_writefile_error:", conv_label).unwrap();
                writeln!(self.text_section, "    ; WriteFile falló, pero continuamos").unwrap();
                writeln!(self.text_section, "    jmp {}_end", conv_label).unwrap();
                self.generate_int_to_str_function(&conv_label);
                writeln!(self.text_section, "{}_end:", conv_label).unwrap();
                return; // Ya generamos el print completo, no necesitamos continuar
            } else {
                writeln!(self.text_section, "    mov rax, 0  ; variable {} no encontrada", var_name).unwrap();
            }
        } else if node.kind() == "number" {
            // Es un number directo
            let num_str = &source[node.start_byte()..node.end_byte()];
            if let Ok(num) = num_str.parse::<i64>() {
                writeln!(self.text_section, "    mov rax, {}  ; número literal para print", num).unwrap();
            } else {
                writeln!(self.text_section, "    mov rax, 0  ; número inválido").unwrap();
            }
        } else {
            // Buscar expresión dentro del print_statement
            for i in 0..node.child_count() {
                if let Some(child) = node.child(i) {
                    match child.kind() {
                        "expression" | "binary_expression" | "primary_expression" | "number" | "identifier" => {
                            // Generar código para evaluar la expresión (resultado en RAX)
                            self.generate_expression_code(&child, source);
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Generar código para convertir número a string e imprimir
        // (RAX ya contiene el valor a imprimir)
        let buffer_offset = self.stack_offset;
        self.stack_offset += 32; // Buffer para conversión
        
        let conv_label = format!("int_to_str_{}", self.label_count);
        self.label_count += 1;
        
        // Guardar número y preparar conversión
        // int_to_str_3 preserva rdx en el stack, así que NO hacer push/pop aquí
        writeln!(self.text_section, "    mov rbx, rax  ; guardar número").unwrap();
        writeln!(self.text_section, "    lea rdx, [rbp - {}]  ; dirección del buffer", buffer_offset).unwrap();
        // int_to_str_3 espera: rax = número, rdx = buffer
        // int_to_str_3 preserva rdx en el stack al inicio y lo restaura al final
        writeln!(self.text_section, "    mov rax, rbx  ; número en rax").unwrap();
        writeln!(self.text_section, "    call {}", conv_label).unwrap();
        
        // int_to_str_3 retorna con rax=longitud, rdx ya restaurado desde stack
        writeln!(self.text_section, "    mov r8, rax  ; longitud").unwrap();
        // rdx ya tiene el buffer correcto (restaurado por int_to_str_3)
        
        // WriteFile call - Windows x64 calling convention
        writeln!(self.text_section, "    mov rcx, [rbp+16]  ; stdout handle").unwrap();
        writeln!(self.text_section, "    lea r9, [rbp+24]  ; lpNumberOfBytesWritten").unwrap();
        writeln!(self.text_section, "    sub rsp, 32  ; reservar shadow space (32 bytes)").unwrap();
        writeln!(self.text_section, "    mov qword [rsp+32], 0  ; lpOverlapped (NULL)").unwrap();
        writeln!(self.text_section, "    call WriteFile").unwrap();
        writeln!(self.text_section, "    add rsp, 32  ; restaurar shadow space").unwrap();
        writeln!(self.text_section, "    test rax, rax  ; verificar retorno").unwrap();
        writeln!(self.text_section, "    jz {}_writefile_error", conv_label).unwrap();
        writeln!(self.text_section, "    jmp {}_end", conv_label).unwrap();
        writeln!(self.text_section, "{}_writefile_error:", conv_label).unwrap();
        writeln!(self.text_section, "    ; WriteFile falló, continuamos").unwrap();
        
        // Saltar la función helper int_to_str
        writeln!(self.text_section, "    jmp {}_end", conv_label).unwrap();
        
        // Generar función helper int_to_str
        writeln!(self.text_section, "{}:", conv_label).unwrap();
        writeln!(self.text_section, "    push rbp").unwrap();
        writeln!(self.text_section, "    mov rbp, rsp").unwrap();
        writeln!(self.text_section, "    push rbx").unwrap();
        writeln!(self.text_section, "    push rcx").unwrap();
        writeln!(self.text_section, "    push r8").unwrap();
        writeln!(self.text_section, "    mov r8, rdx  ; guardar buffer en r8").unwrap();
        writeln!(self.text_section, "    mov rcx, r8  ; dirección buffer").unwrap();
        writeln!(self.text_section, "    mov rbx, rax  ; número").unwrap();
        writeln!(self.text_section, "    cmp rbx, 0").unwrap();
        writeln!(self.text_section, "    jge {}_pos", conv_label).unwrap();
        writeln!(self.text_section, "    mov byte [rcx], '-'").unwrap();
        writeln!(self.text_section, "    inc rcx").unwrap();
        writeln!(self.text_section, "    neg rbx").unwrap();
        writeln!(self.text_section, "{}_pos:", conv_label).unwrap();
        writeln!(self.text_section, "    mov rax, rbx").unwrap();
        writeln!(self.text_section, "    mov rbx, 10").unwrap();
        writeln!(self.text_section, "    push rcx").unwrap();
        writeln!(self.text_section, "    mov rsi, rcx").unwrap();
        writeln!(self.text_section, "    cmp rax, 0").unwrap();
        writeln!(self.text_section, "    jne {}_notz", conv_label).unwrap();
        writeln!(self.text_section, "    mov byte [rsi], '0'").unwrap();
        writeln!(self.text_section, "    inc rsi").unwrap();
        writeln!(self.text_section, "    jmp {}_endd", conv_label).unwrap();
        writeln!(self.text_section, "{}_notz:", conv_label).unwrap();
        writeln!(self.text_section, "{}_loop:", conv_label).unwrap();
        writeln!(self.text_section, "    mov rdx, 0").unwrap();
        writeln!(self.text_section, "    div rbx").unwrap();
        writeln!(self.text_section, "    add dl, '0'").unwrap();
        writeln!(self.text_section, "    mov [rsi], dl").unwrap();
        writeln!(self.text_section, "    inc rsi").unwrap();
        writeln!(self.text_section, "    cmp rax, 0").unwrap();
        writeln!(self.text_section, "    jne {}_loop", conv_label).unwrap();
        writeln!(self.text_section, "{}_endd:", conv_label).unwrap();
        writeln!(self.text_section, "    mov byte [rsi], 0xA").unwrap();
        writeln!(self.text_section, "    inc rsi").unwrap();
        writeln!(self.text_section, "    pop rcx").unwrap();
        writeln!(self.text_section, "    mov rax, rcx").unwrap();
        writeln!(self.text_section, "    mov rbx, rsi").unwrap();
        writeln!(self.text_section, "    dec rbx").unwrap();
        writeln!(self.text_section, "    cmp byte [rcx], '-'").unwrap();
        writeln!(self.text_section, "    jne {}_ns", conv_label).unwrap();
        writeln!(self.text_section, "    inc rax").unwrap();
        writeln!(self.text_section, "{}_ns:", conv_label).unwrap();
        writeln!(self.text_section, "{}_rev:", conv_label).unwrap();
        writeln!(self.text_section, "    cmp rax, rbx").unwrap();
        writeln!(self.text_section, "    jge {}_revd", conv_label).unwrap();
        writeln!(self.text_section, "    mov dl, [rax]").unwrap();
        writeln!(self.text_section, "    mov dh, [rbx]").unwrap();
        writeln!(self.text_section, "    mov [rax], dh").unwrap();
        writeln!(self.text_section, "    mov [rbx], dl").unwrap();
        writeln!(self.text_section, "    inc rax").unwrap();
        writeln!(self.text_section, "    dec rbx").unwrap();
        writeln!(self.text_section, "    jmp {}_rev", conv_label).unwrap();
        writeln!(self.text_section, "{}_revd:", conv_label).unwrap();
        writeln!(self.text_section, "    mov rax, rsi").unwrap();
        writeln!(self.text_section, "    pop r8").unwrap();
        writeln!(self.text_section, "    sub rax, r8").unwrap();
        writeln!(self.text_section, "    pop rcx").unwrap();
        writeln!(self.text_section, "    pop rbx").unwrap();
        writeln!(self.text_section, "    leave").unwrap();
        writeln!(self.text_section, "    mov rdx, r8").unwrap();
        writeln!(self.text_section, "    ret").unwrap();
        writeln!(self.text_section, "{}_end:", conv_label).unwrap();
    }
    
    // Función auxiliar para manejar strings en print statements
    fn generate_print_string(&mut self, node: Node, source: &str) {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                match child.kind() {
                    "string" => {
                        let text = &source[child.start_byte()..child.end_byte()];
                        let msg_name = format!("msg_{}", self.msg_count);
                        self.msg_count += 1;
                        
                        if text.starts_with('"') && text.ends_with('"') {
                            let content = &text[1..text.len()-1];
                            writeln!(self.data_section, "{}: db \"{}\", 0xA", msg_name, content).unwrap();
                        } else {
                            writeln!(self.data_section, "{}: db \"{}\", 0xA", msg_name, text).unwrap();
                        }
                        
                        writeln!(self.data_section, "{}_len: equ $ - {}", msg_name, msg_name).unwrap();
                        
                        writeln!(self.text_section, "    mov rcx, [rbp+16]").unwrap();
                        writeln!(self.text_section, "    lea rdx, [rel {}]", msg_name).unwrap();
                        writeln!(self.text_section, "    mov r8, {}_len", msg_name).unwrap();
                        writeln!(self.text_section, "    lea r9, [rbp+24]").unwrap();
                        writeln!(self.text_section, "    sub rsp, 32  ; reservar shadow space").unwrap();
                        writeln!(self.text_section, "    mov qword [rsp+32], 0").unwrap();
                        writeln!(self.text_section, "    call WriteFile").unwrap();
                        writeln!(self.text_section, "    add rsp, 32  ; restaurar shadow space").unwrap();
                        return;
                    }
                    _ => {}
                }
            }
        }
    }

    fn generate_let(&mut self, node: Node, source: &str) {
        // Estructura Tree-sitter para let_statement:
        // let_statement
        //   - 'let' (keyword)
        //   - identifier (nombre de variable)
        //   - '=' (operador)
        //   - expression (valor)
        
        let mut var_name: Option<String> = None;
        let mut value_expr: Option<Node> = None;
        let mut found_equals = false;

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                match child.kind() {
                    "identifier" => {
                        if var_name.is_none() {
                            var_name = Some(source[child.start_byte()..child.end_byte()].to_string());
                        }
                    }
                    "expression" => {
                        if found_equals {
                            value_expr = Some(child);
                        }
                    }
                    "binary_expression" | "primary_expression" | "number" | "string" => {
                        if found_equals {
                            value_expr = Some(child);
                        }
                    }
                    _ => {
                        // Verificar si es el operador '='
                        let text = &source[child.start_byte()..child.end_byte()];
                        if text == "=" {
                            found_equals = true;
                        }
                        // También buscar recursivamente en hijos
                        for j in 0..child.child_count() {
                            if let Some(grandchild) = child.child(j) {
                                match grandchild.kind() {
                                    "identifier" if var_name.is_none() => {
                                        var_name = Some(source[grandchild.start_byte()..grandchild.end_byte()].to_string());
                                    }
                                    "expression" | "binary_expression" | "primary_expression" | "number" | "string" if found_equals && value_expr.is_none() => {
                                        value_expr = Some(grandchild);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(name) = var_name {
            let offset = self.stack_offset;
            self.stack_offset += 8; // 8 bytes por variable
            self.variables.insert(name.clone(), offset);
            
            // Generar código para evaluar el valor
            if let Some(expr) = value_expr {
                // Intentar buscar número directamente en el texto si no encontramos en el AST
                let expr_text = &source[expr.start_byte()..expr.end_byte()].trim();
                if let Ok(num) = expr_text.parse::<i64>() {
                    writeln!(self.text_section, "    mov rax, {}  ; valor encontrado directamente del texto", num).unwrap();
                } else if let Ok(num) = expr_text.replace("_", "").parse::<i64>() {
                    writeln!(self.text_section, "    mov rax, {}  ; valor encontrado (sin guiones)", num).unwrap();
                } else {
                    // Intentar generar código desde el AST
                    self.generate_expression_code(&expr, source);
                }
            } else {
                // Si no encontramos expresión, intentar buscar número en el texto completo del nodo
                let node_text = &source[node.start_byte()..node.end_byte()];
                // Buscar número después del '='
                if let Some(equals_pos) = node_text.find('=') {
                    let value_part = node_text[equals_pos+1..].trim();
                    if let Ok(num) = value_part.parse::<i64>() {
                        writeln!(self.text_section, "    mov rax, {}  ; valor encontrado en texto del nodo", num).unwrap();
                    } else if let Ok(num) = value_part.replace("_", "").parse::<i64>() {
                        writeln!(self.text_section, "    mov rax, {}  ; valor encontrado en texto (sin guiones)", num).unwrap();
                    } else {
                        writeln!(self.text_section, "    mov rax, 0  ; valor por defecto (no se encontró expresión ni en texto: '{}')", value_part).unwrap();
                    }
                } else {
                    writeln!(self.text_section, "    mov rax, 0  ; valor por defecto (no se encontró '=' en texto)").unwrap();
                }
            }
            
            // Guardar valor en stack
            writeln!(self.text_section, "    mov [rbp - {}], rax  ; variable {}", offset, name).unwrap();
        } else {
            writeln!(self.text_section, "    ; ERROR: No se encontró nombre de variable en let statement").unwrap();
        }
    }
    
    fn find_identifier_in_node(&self, node: &Node, source: &str) -> Option<(String, usize)> {
        // Buscar identificador recursivamente en el nodo
        if node.kind() == "identifier" {
            let name = source[node.start_byte()..node.end_byte()].to_string();
            return Some((name, 0));
        }
        
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if let Some((name, depth)) = self.find_identifier_in_node(&child, source) {
                    return Some((name, depth + 1));
                }
            }
        }
        
        None
    }
    
    fn find_number_in_node(&self, node: &Node, source: &str) -> Option<i64> {
        // Buscar número recursivamente en el nodo
        if node.kind() == "number" {
            let num_str = &source[node.start_byte()..node.end_byte()].trim();
            if let Ok(num) = num_str.parse::<i64>() {
                return Some(num);
            } else if let Ok(num) = num_str.replace("_", "").parse::<i64>() {
                return Some(num);
            }
        }
        
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if let Some(num) = self.find_number_in_node(&child, source) {
                    return Some(num);
                }
            }
        }
        
        None
    }
    
    fn generate_expression_code(&mut self, expr_node: &Node, source: &str) {
        // Primero, intentar encontrar identificador recursivamente
        // Si es primary_expression con struct_literal, extraer el identificador primero
        if expr_node.kind() == "primary_expression" {
            // Buscar identifier dentro del primary_expression
            for i in 0..expr_node.child_count() {
                if let Some(child) = expr_node.child(i) {
                    if child.kind() == "identifier" {
                        let var_name = &source[child.start_byte()..child.end_byte()];
                        if let Some(&offset) = self.variables.get(var_name) {
                            writeln!(self.text_section, "    mov rax, [rbp - {}]  ; cargar variable {} (primary_expression)", offset, var_name).unwrap();
                            return;
                        }
                    } else if child.kind() == "struct_literal" {
                        // Buscar identifier dentro del struct_literal
                        for j in 0..child.child_count() {
                            if let Some(grandchild) = child.child(j) {
                                if grandchild.kind() == "identifier" {
                                    let var_name = &source[grandchild.start_byte()..grandchild.end_byte()];
                                    if let Some(&offset) = self.variables.get(var_name) {
                                        writeln!(self.text_section, "    mov rax, [rbp - {}]  ; cargar variable {} (struct_literal)", offset, var_name).unwrap();
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Segundo, intentar encontrar identificador recursivamente (método general)
        if let Some((var_name, _)) = self.find_identifier_in_node(expr_node, source) {
            if let Some(&offset) = self.variables.get(var_name.as_str()) {
                writeln!(self.text_section, "    mov rax, [rbp - {}]  ; cargar variable {} (recursivo)", offset, var_name).unwrap();
                return;
            }
        }
        
        // Tercero, intentar encontrar número recursivamente
        if let Some(num) = self.find_number_in_node(expr_node, source) {
            writeln!(self.text_section, "    mov rax, {}  ; número literal (recursivo)", num).unwrap();
            return;
        }
        
        // Cuarto, intentar parsear directamente del texto como fallback
        let expr_text = source[expr_node.start_byte()..expr_node.end_byte()].trim();
        if let Some(&offset) = self.variables.get(expr_text) {
            writeln!(self.text_section, "    mov rax, [rbp - {}]  ; cargar variable {} (del texto)", offset, expr_text).unwrap();
            return;
        } else if let Ok(num) = expr_text.parse::<i64>() {
            writeln!(self.text_section, "    mov rax, {}  ; número literal (parseado del texto)", num).unwrap();
            return;
        } else if let Ok(num) = expr_text.replace("_", "").parse::<i64>() {
            writeln!(self.text_section, "    mov rax, {}  ; número literal (sin guiones, parseado del texto)", num).unwrap();
            return;
        }
        
        match expr_node.kind() {
            "number" => {
                let num_str = &source[expr_node.start_byte()..expr_node.end_byte()];
                if let Ok(num) = num_str.parse::<i64>() {
                    writeln!(self.text_section, "    mov rax, {}", num).unwrap();
                }
            }
            "identifier" => {
                let name = &source[expr_node.start_byte()..expr_node.end_byte()];
                if let Some(&offset) = self.variables.get(name) {
                    writeln!(self.text_section, "    mov rax, [rbp - {}]  ; cargar variable {}", offset, name).unwrap();
                } else {
                    writeln!(self.text_section, "    mov rax, 0  ; variable {} no encontrada", name).unwrap();
                }
            }
            "binary_expression" => {
                // Tree-sitter parsea binary_expression como:
                // binary_expression
                //   - expression (left)
                //   - operator (como "<=", "+", etc.)
                //   - expression (right)
                // PERO en el caso de 'i <= max {', el right es un primary_expression -> struct_literal
                // Necesitamos extraer solo 'max' del struct_literal para la comparación
                
                // Recopilar todos los hijos
                let mut children: Vec<Node> = Vec::new();
                for i in 0..expr_node.child_count() {
                    if let Some(child) = expr_node.child(i) {
                        children.push(child);
                    }
                }
                
                // Intentar encontrar operador y operandos
                let mut left: Option<Node> = None;
                let mut right: Option<Node> = None;
                let mut operator: Option<&str> = None;
                
                // Estrategia 1: Si tenemos 3 hijos, asumir left, operator, right
                if children.len() >= 3 {
                    left = Some(children[0]);
                    // Buscar operador en el medio
                    for i in 1..children.len()-1 {
                        let op_text = &source[children[i].start_byte()..children[i].end_byte()];
                        if op_text == "<=" || op_text == ">=" || op_text == "<" || op_text == ">" || 
                           op_text == "==" || op_text == "!=" || op_text == "+" || op_text == "-" || 
                           op_text == "*" || op_text == "/" || op_text == "%" {
                            operator = Some(op_text);
                            // El right podría ser un primary_expression que contiene struct_literal
                            // Si es así, necesitamos extraer solo el identifier del struct_literal
                            let right_node = &children[i+1];
                            if right_node.kind() == "primary_expression" {
                                // Buscar dentro del primary_expression
                                for j in 0..right_node.child_count() {
                                    if let Some(grandchild) = right_node.child(j) {
                                        if grandchild.kind() == "identifier" {
                                            // Este es el identifier que necesitamos (ej: 'max')
                                            right = Some(grandchild);
                                            break;
                                        } else if grandchild.kind() == "struct_literal" {
                                            // El struct_literal tiene un identifier como primer hijo
                                            for k in 0..grandchild.child_count() {
                                                if let Some(ggchild) = grandchild.child(k) {
                                                    if ggchild.kind() == "identifier" {
                                                        right = Some(ggchild);
                                                        break;
                                                    }
                                                }
                                            }
                                            break;
                                        }
                                    }
                                }
                            } else {
                                right = Some(children[i+1]);
                            }
                            break;
                        }
                    }
                }
                
                // Estrategia 2: Buscar en todos los hijos si no encontramos
                if operator.is_none() {
                    for i in 0..expr_node.child_count() {
                        if let Some(child) = expr_node.child(i) {
                            match child.kind() {
                                "expression" | "binary_expression" | "identifier" | "number" | "primary_expression" => {
                                    if left.is_none() {
                                        left = Some(child);
                                    } else if right.is_none() {
                                        // Si right contiene struct_literal, extraer solo el identifier
                                        if child.kind() == "primary_expression" {
                                            for j in 0..child.child_count() {
                                                if let Some(grandchild) = child.child(j) {
                                                    if grandchild.kind() == "identifier" {
                                                        right = Some(grandchild);
                                                        break;
                                                    } else if grandchild.kind() == "struct_literal" {
                                                        for k in 0..grandchild.child_count() {
                                                            if let Some(ggchild) = grandchild.child(k) {
                                                                if ggchild.kind() == "identifier" {
                                                                    right = Some(ggchild);
                                                                    break;
                                                                }
                                                            }
                                                        }
                                                        break;
                                                    }
                                                }
                                            }
                                        } else {
                                            right = Some(child);
                                        }
                                    }
                                }
                                _ => {
                                    // Puede ser el operador (como "<=")
                                    let text = &source[child.start_byte()..child.end_byte()];
                                    if text == "<=" || text == ">=" || text == "<" || text == ">" || 
                                       text == "==" || text == "!=" || text == "+" || text == "-" || 
                                       text == "*" || text == "/" || text == "%" {
                                        operator = Some(text);
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Generar código para left - usar búsqueda recursiva
                if let Some(left_node) = left {
                    // Intentar buscar identificador/número recursivamente primero
                    if let Some((var_name, _)) = self.find_identifier_in_node(&left_node, source) {
                        if let Some(&offset) = self.variables.get(var_name.as_str()) {
                            writeln!(self.text_section, "    mov rax, [rbp - {}]  ; cargar variable {} (left recursivo)", offset, var_name).unwrap();
                        } else {
                            self.generate_expression_code(&left_node, source);
                        }
                    } else if let Some(num) = self.find_number_in_node(&left_node, source) {
                        writeln!(self.text_section, "    mov rax, {}  ; número literal {} (left recursivo)", num, num).unwrap();
                    } else {
                        self.generate_expression_code(&left_node, source);
                    }
                } else {
                    // Si no encontramos left, intentar parsear del texto
                    let text = &source[expr_node.start_byte()..expr_node.end_byte()];
                    if let Some(op) = operator {
                        if let Some(equals_pos) = text.find(op) {
                            let left_part = text[..equals_pos].trim();
                            if let Some(&offset) = self.variables.get(left_part) {
                                writeln!(self.text_section, "    mov rax, [rbp - {}]  ; cargar variable {} (left texto)", offset, left_part).unwrap();
                            } else if let Ok(num) = left_part.parse::<i64>() {
                                writeln!(self.text_section, "    mov rax, {}  ; número literal (left texto)", num).unwrap();
                            } else {
                                writeln!(self.text_section, "    mov rax, 0  ; left no encontrado: '{}'", left_part).unwrap();
                            }
                        } else {
                            writeln!(self.text_section, "    mov rax, 0  ; left no encontrado").unwrap();
                        }
                    } else {
                        writeln!(self.text_section, "    mov rax, 0  ; left no encontrado (sin operador)").unwrap();
                    }
                }
                
                // Guardar left en stack temporalmente
                writeln!(self.text_section, "    push rax  ; guardar left").unwrap();
                
                // Generar código para right - usar búsqueda recursiva
                if let Some(right_node) = right {
                    // Intentar buscar identificador/número recursivamente primero
                    if let Some((var_name, _)) = self.find_identifier_in_node(&right_node, source) {
                        if let Some(&offset) = self.variables.get(var_name.as_str()) {
                            writeln!(self.text_section, "    mov rax, [rbp - {}]  ; cargar variable {} (right recursivo)", offset, var_name).unwrap();
                        } else {
                            self.generate_expression_code(&right_node, source);
                        }
                    } else if let Some(num) = self.find_number_in_node(&right_node, source) {
                        writeln!(self.text_section, "    mov rax, {}  ; número literal {} (right recursivo)", num, num).unwrap();
                    } else {
                        self.generate_expression_code(&right_node, source);
                    }
                } else {
                    // Si no encontramos right, intentar parsear del texto
                    let text = &source[expr_node.start_byte()..expr_node.end_byte()];
                    if let Some(op) = operator {
                        if let Some(equals_pos) = text.find(op) {
                            let right_part = text[equals_pos+op.len()..].trim();
                            if let Some(&offset) = self.variables.get(right_part) {
                                writeln!(self.text_section, "    mov rax, [rbp - {}]  ; cargar variable {} (right texto)", offset, right_part).unwrap();
                            } else if let Ok(num) = right_part.parse::<i64>() {
                                writeln!(self.text_section, "    mov rax, {}  ; número literal (right texto)", num).unwrap();
                            } else {
                                writeln!(self.text_section, "    mov rax, 0  ; right no encontrado: '{}'", right_part).unwrap();
                            }
                        } else {
                            writeln!(self.text_section, "    mov rax, 0  ; right no encontrado").unwrap();
                        }
                    } else {
                        writeln!(self.text_section, "    mov rax, 0  ; right no encontrado (sin operador)").unwrap();
                    }
                }
                
                // Restaurar left
                writeln!(self.text_section, "    pop rbx  ; restaurar left").unwrap();
                
                // Generar código para operación
                if let Some(op) = operator {
                    match op {
                        "<=" => {
                            writeln!(self.text_section, "    cmp rbx, rax  ; rbx <= rax").unwrap();
                            writeln!(self.text_section, "    setle al  ; set al=1 si rbx <= rax").unwrap();
                            writeln!(self.text_section, "    movzx rax, al  ; extender a 64 bits").unwrap();
                        }
                        ">=" => {
                            writeln!(self.text_section, "    cmp rbx, rax  ; rbx >= rax").unwrap();
                            writeln!(self.text_section, "    setge al").unwrap();
                            writeln!(self.text_section, "    movzx rax, al").unwrap();
                        }
                        "<" => {
                            writeln!(self.text_section, "    cmp rbx, rax  ; rbx < rax").unwrap();
                            writeln!(self.text_section, "    setl al").unwrap();
                            writeln!(self.text_section, "    movzx rax, al").unwrap();
                        }
                        ">" => {
                            writeln!(self.text_section, "    cmp rbx, rax  ; rbx > rax").unwrap();
                            writeln!(self.text_section, "    setg al").unwrap();
                            writeln!(self.text_section, "    movzx rax, al").unwrap();
                        }
                        "==" => {
                            writeln!(self.text_section, "    cmp rbx, rax  ; rbx == rax").unwrap();
                            writeln!(self.text_section, "    sete al").unwrap();
                            writeln!(self.text_section, "    movzx rax, al").unwrap();
                        }
                        "+" => {
                            writeln!(self.text_section, "    add rax, rbx  ; rax = rax + rbx").unwrap();
                        }
                        "-" => {
                            writeln!(self.text_section, "    sub rbx, rax  ; rbx = rbx - rax").unwrap();
                            writeln!(self.text_section, "    mov rax, rbx").unwrap();
                        }
                        "*" => {
                            writeln!(self.text_section, "    imul rax, rbx  ; rax = rax * rbx").unwrap();
                        }
                        "/" => {
                            writeln!(self.text_section, "    mov rcx, rax").unwrap();
                            writeln!(self.text_section, "    mov rax, rbx").unwrap();
                            writeln!(self.text_section, "    cqo  ; extender rax a rdx:rax").unwrap();
                            writeln!(self.text_section, "    idiv rcx  ; rax = rax / rcx").unwrap();
                        }
                        "%" => {
                            writeln!(self.text_section, "    mov rcx, rax").unwrap();
                            writeln!(self.text_section, "    mov rax, rbx").unwrap();
                            writeln!(self.text_section, "    cqo").unwrap();
                            writeln!(self.text_section, "    idiv rcx  ; rdx = rax % rcx").unwrap();
                            writeln!(self.text_section, "    mov rax, rdx  ; resultado en rdx").unwrap();
                        }
                        _ => {
                            writeln!(self.text_section, "    mov rax, 1  ; operador {} no soportado, asumir true", op).unwrap();
                        }
                    }
                }
            }
            _ => {
                // Por defecto, 0
                writeln!(self.text_section, "    mov rax, 0").unwrap();
            }
        }
    }

    fn generate_while(&mut self, node: Node, source: &str) {
        eprintln!("DEBUG: generate_while llamado, node kind: {}, child_count: {}", node.kind(), node.child_count());
        self.in_while_loop = true; // Estamos en un while loop
        let loop_start = format!("loop_start_{}", self.label_count);
        let loop_end = format!("loop_end_{}", self.label_count);
        self.label_count += 1;

        // Buscar condición y cuerpo
        let mut condition: Option<Node> = None;
        let mut body: Option<Node> = None;

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                eprintln!("DEBUG: while child[{}]: kind={}", i, child.kind());
                match child.kind() {
                    "expression" | "binary_expression" | "comparison_expression" => {
                        condition = Some(child);
                        eprintln!("DEBUG: Condición encontrada: kind={}", child.kind());
                    }
                    "block" => {
                        body = Some(child);
                        eprintln!("DEBUG: Block encontrado");
                    }
                    "struct_literal" => {
                        // Si encontramos struct_literal, podría ser el bloque mal parseado
                        // Intentar procesarlo como block
                        body = Some(child);
                        eprintln!("DEBUG: struct_literal encontrado (bloque mal parseado)");
                    }
                    _ => {
                        eprintln!("DEBUG: Hijo ignorado: kind={}", child.kind());
                    }
                }
            }
        }

        if let Some(cond_node) = condition {
            // Label de inicio del loop
            writeln!(self.text_section, "{}:", loop_start).unwrap();
            
            // Generar código para evaluar condición
            // Esto genera: cmp, jg, y salta a loop_end si la condición es falsa
            let old_in_while = self.in_while_loop;
            self.in_while_loop = true; // Asegurar que esté en true
            // NO pasar loop_start aquí - el jmp se generará después del body
            self.generate_condition_code(&cond_node, source, &loop_end, None);
            self.in_while_loop = old_in_while; // Restaurar estado
            
            // PROCESAR BODY DEL LOOP - esto es crítico
            if let Some(body_node) = body {
                // Procesar todos los statements del body
                self.process_loop_body_recursive(body_node, source);
            }
            
            // Jmp al inicio del loop (después de procesar el body)
            writeln!(self.text_section, "    jmp {}", loop_start).unwrap();
            
            // Label de fin del loop
            writeln!(self.text_section, "{}:", loop_end).unwrap();
            }
        self.in_while_loop = false; // Terminamos el while
    }
    
    fn debug_print_tree_to_file(&self, node: &Node, source: &str, depth: usize, file: &mut std::fs::File) {
        use std::io::Write;
        let indent = "  ".repeat(depth);
        let text = &source[node.start_byte()..node.end_byte()];
        let text_preview = if text.len() > 50 {
            format!("{}...", &text[..50])
        } else {
            text.to_string()
        };
        writeln!(file, "{}{} ({}..{}) '{}'", indent, node.kind(), node.start_byte(), node.end_byte(), text_preview.replace("\n", "\\n")).ok();
        
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                self.debug_print_tree_to_file(&child, source, depth + 1, file);
            }
        }
    }
    
    fn generate_int_to_str_function(&mut self, conv_label: &str) {
        writeln!(self.text_section, "{}:", conv_label).unwrap();
        writeln!(self.text_section, "    push rbp").unwrap();
        writeln!(self.text_section, "    mov rbp, rsp").unwrap();
        writeln!(self.text_section, "    push rbx").unwrap();
        writeln!(self.text_section, "    push rcx").unwrap();
        writeln!(self.text_section, "    push r8").unwrap();
        writeln!(self.text_section, "    push rdx  ; PASO 6: CRÍTICO - preservar buffer original en STACK (más seguro que r10 volátil)").unwrap();
        writeln!(self.text_section, "    mov r8, rdx  ; guardar buffer en r8 para uso interno").unwrap();
        writeln!(self.text_section, "    mov rcx, r8  ; dirección buffer").unwrap();
        writeln!(self.text_section, "    mov rbx, rax  ; número").unwrap();
        writeln!(self.text_section, "    cmp rbx, 0").unwrap();
        writeln!(self.text_section, "    jge {}_pos", conv_label).unwrap();
        writeln!(self.text_section, "    mov byte [rcx], '-'").unwrap();
        writeln!(self.text_section, "    inc rcx").unwrap();
        writeln!(self.text_section, "    neg rbx").unwrap();
        writeln!(self.text_section, "{}_pos:", conv_label).unwrap();
        writeln!(self.text_section, "    mov rax, rbx").unwrap();
        writeln!(self.text_section, "    mov rbx, 10").unwrap();
        writeln!(self.text_section, "    push rcx").unwrap();
        writeln!(self.text_section, "    mov rsi, rcx").unwrap();
        writeln!(self.text_section, "    cmp rax, 0").unwrap();
        writeln!(self.text_section, "    jne {}_notz", conv_label).unwrap();
        writeln!(self.text_section, "    mov byte [rsi], '0'").unwrap();
        writeln!(self.text_section, "    inc rsi").unwrap();
        writeln!(self.text_section, "    jmp {}_endd", conv_label).unwrap();
        writeln!(self.text_section, "{}_notz:", conv_label).unwrap();
        writeln!(self.text_section, "{}_loop:", conv_label).unwrap();
        writeln!(self.text_section, "    mov rdx, 0").unwrap();
        writeln!(self.text_section, "    div rbx").unwrap();
        writeln!(self.text_section, "    add dl, '0'").unwrap();
        writeln!(self.text_section, "    mov [rsi], dl").unwrap();
        writeln!(self.text_section, "    inc rsi").unwrap();
        writeln!(self.text_section, "    cmp rax, 0").unwrap();
        writeln!(self.text_section, "    jne {}_loop", conv_label).unwrap();
        writeln!(self.text_section, "{}_endd:", conv_label).unwrap();
        writeln!(self.text_section, "    mov byte [rsi], 0xA").unwrap();
        writeln!(self.text_section, "    inc rsi").unwrap();
        writeln!(self.text_section, "    pop rcx").unwrap();
        writeln!(self.text_section, "    mov rax, rcx").unwrap();
        writeln!(self.text_section, "    mov rbx, rsi").unwrap();
        writeln!(self.text_section, "    dec rbx").unwrap();
        writeln!(self.text_section, "    cmp byte [rcx], '-'").unwrap();
        writeln!(self.text_section, "    jne {}_ns", conv_label).unwrap();
        writeln!(self.text_section, "    inc rax").unwrap();
        writeln!(self.text_section, "{}_ns:", conv_label).unwrap();
        writeln!(self.text_section, "{}_rev:", conv_label).unwrap();
        writeln!(self.text_section, "    cmp rax, rbx").unwrap();
        writeln!(self.text_section, "    jge {}_revd", conv_label).unwrap();
        writeln!(self.text_section, "    mov dl, [rax]").unwrap();
        writeln!(self.text_section, "    mov dh, [rbx]").unwrap();
        writeln!(self.text_section, "    mov [rax], dh").unwrap();
        writeln!(self.text_section, "    mov [rbx], dl").unwrap();
        writeln!(self.text_section, "    inc rax").unwrap();
        writeln!(self.text_section, "    dec rbx").unwrap();
        writeln!(self.text_section, "    jmp {}_rev", conv_label).unwrap();
        writeln!(self.text_section, "{}_revd:", conv_label).unwrap();
        writeln!(self.text_section, "    mov rax, rsi  ; final del string").unwrap();
        writeln!(self.text_section, "    pop r8  ; PASO 4: restaurar dirección inicio buffer (guardado como rcx en push rcx)").unwrap();
        writeln!(self.text_section, "    sub rax, r8  ; longitud = final - inicio").unwrap();
        writeln!(self.text_section, "    pop rcx  ; restaurar rcx original del caller").unwrap();
        writeln!(self.text_section, "    pop rbx  ; restaurar rbx original del caller").unwrap();
        // PASO 6: El stack ahora está: [r8 original, rdx buffer]
        // Restaurar r8 antes de leave
        writeln!(self.text_section, "    pop r8  ; PASO 6: restaurar r8 original del caller (antes de leave)").unwrap();
        writeln!(self.text_section, "    leave  ; restaura rbp y rsp (equivalente a mov rsp, rbp; pop rbp)").unwrap();
        // PASO 6: Después de leave, rsp apunta al return address (dejado por call)
        // El buffer está en el stack del caller, restaurarlo ahora
        writeln!(self.text_section, "    pop rdx  ; PASO 6: restaurar buffer desde stack (más seguro que r10 volátil)").unwrap();
        writeln!(self.text_section, "    ret  ; retorna con rax=longitud, rdx=buffer").unwrap();
    }
    
    fn find_and_process_statements_in_node(&mut self, node: Node, source: &str) {
        // Buscar recursivamente print_statement y assign_statement en CUALQUIER parte del árbol
        match node.kind() {
            "print_statement" => {
                self.generate_print(node, source);
                return;
            }
            "assign_statement" | "assignment" => {
                self.generate_assignment(node, source);
                return;
            }
            _ => {
                // Procesar todos los hijos recursivamente
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.find_and_process_statements_in_node(child, source);
                    }
                }
            }
        }
    }
    
    fn process_loop_body_recursive(&mut self, node: Node, source: &str) {
        // Procesar el body del loop recursivamente, buscando print_statement y assign_statement
        // sin importar la estructura del AST (puede ser ERROR, struct_literal, block, etc.)
        
        // PRIMERO: Intentar procesar como statement directo
        match node.kind() {
            "print_statement" => {
                self.generate_print(node, source);
                return;
            }
            "assign_statement" | "assignment" => {
                self.generate_assignment(node, source);
                return;
            }
            _ => {}
        }
        
        // SEGUNDO: Si es un bloque (block, ERROR, struct_literal), procesar hijos
        match node.kind() {
            "block" | "ERROR" | "struct_literal" => {
                // Para bloques, procesar TODOS los hijos recursivamente
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        // Si el hijo es un statement directo, procesarlo
                        match child.kind() {
                            "print_statement" => {
                                self.generate_print(child, source);
                                continue;
                            }
                            "assign_statement" | "assignment" => {
                                self.generate_assignment(child, source);
                                continue;
                            }
                            _ => {
                                // Procesar recursivamente
                                self.process_loop_body_recursive(child, source);
                            }
                        }
                    }
                }
            }
            _ => {
                // Para otros nodos, procesar recursivamente todos los hijos
                // Esto es importante para encontrar print_statement/assign_statement anidados
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.process_loop_body_recursive(child, source);
                    }
                }
                
                // También intentar procesar tokens manualmente si es necesario
                // (para casos donde Tree-sitter parsea mal)
                let node_text = &source[node.start_byte()..node.end_byte()];
                if node_text.contains("print") && node_text.contains("i") {
                    // Intentar encontrar print i en el texto
                    let mut tokens: Vec<(String, Node)> = Vec::new();
                    for i in 0..node.child_count() {
                        if let Some(child) = node.child(i) {
                            let text = &source[child.start_byte()..child.end_byte()];
                            tokens.push((text.to_string(), child));
                        }
                    }
                    
                    // Buscar patrón "print" seguido de identificador
                    let mut j = 0;
                    while j < tokens.len() {
                        let (text, token_node) = &tokens[j];
                        if text.trim() == "print" && j + 1 < tokens.len() {
                            let (next_text, next_node) = &tokens[j + 1];
                            if next_node.kind() == "identifier" && next_text == "i" {
                                // Generar print para 'i'
                                if let Some(&offset) = self.variables.get("i") {
                                    writeln!(self.text_section, "    mov rax, [rbp - {}]  ; cargar variable i para print", offset).unwrap();
                                    // Generar print completo
                                    let buffer_offset = self.stack_offset;
                                    self.stack_offset += 32;
                                    let conv_label = format!("int_to_str_{}", self.label_count);
                                    self.label_count += 1;
                                    writeln!(self.text_section, "    mov rbx, rax  ; guardar número").unwrap();
                                    writeln!(self.text_section, "    lea rdx, [rbp - {}]  ; dirección del buffer", buffer_offset).unwrap();
                                    writeln!(self.text_section, "    push rbx").unwrap();
                                    writeln!(self.text_section, "    push rdx").unwrap();
                                    writeln!(self.text_section, "    mov rax, rbx").unwrap();
                                    writeln!(self.text_section, "    call {}", conv_label).unwrap();
                                    writeln!(self.text_section, "    mov r8, rax  ; longitud").unwrap();
                                    writeln!(self.text_section, "    mov rcx, [rbp+16]  ; stdout handle").unwrap();
                                    writeln!(self.text_section, "    lea r9, [rbp+24]  ; lpNumberOfBytesWritten").unwrap();
                                    writeln!(self.text_section, "    mov qword [rsp+32], 0  ; lpOverlapped").unwrap();
                                    writeln!(self.text_section, "    call WriteFile").unwrap();
                                    writeln!(self.text_section, "    jmp {}_end", conv_label).unwrap();
                                    // Generar función helper (simplificada)
                                    writeln!(self.text_section, "{}:", conv_label).unwrap();
                                    writeln!(self.text_section, "    push rbp").unwrap();
                                    writeln!(self.text_section, "    mov rbp, rsp").unwrap();
                                    writeln!(self.text_section, "    push rbx").unwrap();
                                    writeln!(self.text_section, "    push rcx").unwrap();
                                    writeln!(self.text_section, "    push r8").unwrap();
                                    writeln!(self.text_section, "    mov r8, rdx").unwrap();
                                    writeln!(self.text_section, "    mov rcx, r8").unwrap();
                                    writeln!(self.text_section, "    mov rbx, rax").unwrap();
                                    writeln!(self.text_section, "    mov rax, rbx").unwrap();
                                    writeln!(self.text_section, "    mov rbx, 10").unwrap();
                                    writeln!(self.text_section, "    push rcx").unwrap();
                                    writeln!(self.text_section, "    mov rsi, rcx").unwrap();
                                    writeln!(self.text_section, "    cmp rax, 0").unwrap();
                                    writeln!(self.text_section, "    jne {}_notz", conv_label).unwrap();
                                    writeln!(self.text_section, "    mov byte [rsi], '0'").unwrap();
                                    writeln!(self.text_section, "    inc rsi").unwrap();
                                    writeln!(self.text_section, "    jmp {}_endd", conv_label).unwrap();
                                    writeln!(self.text_section, "{}_notz:", conv_label).unwrap();
                                    writeln!(self.text_section, "{}_loop:", conv_label).unwrap();
                                    writeln!(self.text_section, "    mov rdx, 0").unwrap();
                                    writeln!(self.text_section, "    div rbx").unwrap();
                                    writeln!(self.text_section, "    add dl, '0'").unwrap();
                                    writeln!(self.text_section, "    mov [rsi], dl").unwrap();
                                    writeln!(self.text_section, "    inc rsi").unwrap();
                                    writeln!(self.text_section, "    cmp rax, 0").unwrap();
                                    writeln!(self.text_section, "    jne {}_loop", conv_label).unwrap();
                                    writeln!(self.text_section, "{}_endd:", conv_label).unwrap();
                                    writeln!(self.text_section, "    mov byte [rsi], 0xA").unwrap();
                                    writeln!(self.text_section, "    inc rsi").unwrap();
                                    writeln!(self.text_section, "    pop rcx").unwrap();
                                    writeln!(self.text_section, "    mov rax, rcx").unwrap();
                                    writeln!(self.text_section, "    mov rbx, rsi").unwrap();
                                    writeln!(self.text_section, "    dec rbx").unwrap();
                                    writeln!(self.text_section, "{}_rev:", conv_label).unwrap();
                                    writeln!(self.text_section, "    cmp rax, rbx").unwrap();
                                    writeln!(self.text_section, "    jge {}_revd", conv_label).unwrap();
                                    writeln!(self.text_section, "    mov dl, [rax]").unwrap();
                                    writeln!(self.text_section, "    mov dh, [rbx]").unwrap();
                                    writeln!(self.text_section, "    mov [rax], dh").unwrap();
                                    writeln!(self.text_section, "    mov [rbx], dl").unwrap();
                                    writeln!(self.text_section, "    inc rax").unwrap();
                                    writeln!(self.text_section, "    dec rbx").unwrap();
                                    writeln!(self.text_section, "    jmp {}_rev", conv_label).unwrap();
                                    writeln!(self.text_section, "{}_revd:", conv_label).unwrap();
                                    writeln!(self.text_section, "    mov rax, rsi").unwrap();
                                    writeln!(self.text_section, "    pop r8").unwrap();
                                    writeln!(self.text_section, "    sub rax, r8").unwrap();
                                    writeln!(self.text_section, "    pop rcx").unwrap();
                                    writeln!(self.text_section, "    pop rbx").unwrap();
                                    writeln!(self.text_section, "    leave").unwrap();
                                    writeln!(self.text_section, "    mov rdx, r8").unwrap();
                                    writeln!(self.text_section, "    ret").unwrap();
                                    writeln!(self.text_section, "{}_end:", conv_label).unwrap();
                                }
                                j += 2;
                                continue;
                            }
                        }
                        
                        // Detectar "i = i + 1"
                        if text == "i" && j + 4 < tokens.len() {
                            let (eq_text, _) = &tokens[j + 1];
                            let (next_i_text, _) = &tokens[j + 2];
                            let (plus_text, _) = &tokens[j + 3];
                            let (num_text, _) = &tokens[j + 4];
                            if eq_text == "=" && next_i_text == "i" && plus_text == "+" && num_text == "1" {
                                if let Some(&offset) = self.variables.get("i") {
                                    // GENERAR PRINT I ANTES del incremento (dentro del loop)
                                    writeln!(self.text_section, "    mov rax, [rbp - {}]  ; cargar variable i para print", offset).unwrap();
                                    let buffer_offset = self.stack_offset;
                                    self.stack_offset += 32;
                                    let conv_label = format!("int_to_str_{}", self.label_count);
                                    self.label_count += 1;
                                    writeln!(self.text_section, "    mov rbx, rax  ; guardar número").unwrap();
                                    writeln!(self.text_section, "    lea rdx, [rbp - {}]  ; dirección del buffer", buffer_offset).unwrap();
                                    writeln!(self.text_section, "    push rbx").unwrap();
                                    writeln!(self.text_section, "    push rdx").unwrap();
                                    writeln!(self.text_section, "    mov rax, rbx").unwrap();
                                    writeln!(self.text_section, "    call {}", conv_label).unwrap();
                                    writeln!(self.text_section, "    mov r8, rax  ; longitud").unwrap();
                                    writeln!(self.text_section, "    mov rcx, [rbp+16]  ; stdout handle").unwrap();
                                    writeln!(self.text_section, "    lea r9, [rbp+24]  ; lpNumberOfBytesWritten").unwrap();
                                    writeln!(self.text_section, "    mov qword [rsp+32], 0  ; lpOverlapped").unwrap();
                                    writeln!(self.text_section, "    call WriteFile").unwrap();
                                    writeln!(self.text_section, "    jmp {}_end", conv_label).unwrap();
                                    self.generate_int_to_str_function(&conv_label);
                                    writeln!(self.text_section, "{}_end:", conv_label).unwrap();
                                    
                                    // GENERAR INCREMENTO
                                    writeln!(self.text_section, "    mov rax, [rbp - {}]  ; cargar variable i", offset).unwrap();
                                    writeln!(self.text_section, "    add rax, 1  ; incrementar").unwrap();
                                    writeln!(self.text_section, "    mov [rbp - {}], rax  ; guardar variable i", offset).unwrap();
                                }
                                j += 5;
                                continue;
                            }
                        }
                        j += 1;
                    }
                }
            }
        }
    }
    
    fn generate_condition_code(&mut self, cond_node: &Node, source: &str, loop_end: &str, loop_start: Option<&str>) {
        // Si la condición es un binary_expression con operador de comparación,
        // generar código de comparación directo en lugar de evaluar a booleano
        if cond_node.kind() == "binary_expression" {
            // Intentar extraer left, operator, right
            let mut left: Option<Node> = None;
            let mut right: Option<Node> = None;
            let mut operator: Option<&str> = None;
            
            // Recopilar hijos
            let mut children: Vec<Node> = Vec::new();
            for i in 0..cond_node.child_count() {
                if let Some(child) = cond_node.child(i) {
                    children.push(child);
                }
            }
            
            // Buscar operador de comparación
            for i in 0..children.len() {
                let child = &children[i];
                let text = &source[child.start_byte()..child.end_byte()];
                if text == "<=" || text == ">=" || text == "<" || text == ">" || 
                   text == "==" || text == "!=" {
                    operator = Some(text);
                    if i > 0 {
                        left = Some(children[i-1]);
                    }
                    if i < children.len() - 1 {
                        let right_node = &children[i+1];
                        // Si right es primary_expression, extraer el identifier
                        if right_node.kind() == "primary_expression" {
                            for j in 0..right_node.child_count() {
                                if let Some(grandchild) = right_node.child(j) {
                                    if grandchild.kind() == "identifier" {
                                        right = Some(grandchild);
                                        break;
                                    } else if grandchild.kind() == "struct_literal" {
                                        // Buscar identifier dentro del struct_literal
                                        for k in 0..grandchild.child_count() {
                                            if let Some(ggchild) = grandchild.child(k) {
                                                if ggchild.kind() == "identifier" {
                                                    right = Some(ggchild);
                                                    break;
                                                }
                                            }
                                        }
                                        break;
                                    }
                                }
                            }
                        } else {
                            right = Some(children[i+1]);
                        }
                    }
                    break;
                }
            }
            
            // Si encontramos operador de comparación, generar comparación directa
            if let (Some(op), Some(left_node), Some(right_node)) = (operator, left, right) {
                // Evaluar left en rax (cargar variable i)
                self.generate_expression_code(&left_node, source);
                // Guardar left (i) en stack
                writeln!(self.text_section, "    push rax  ; guardar left (i)").unwrap();
                
                // Evaluar right en rax (cargar variable max)
                self.generate_expression_code(&right_node, source);
                // Mover right (max) a rbx
                writeln!(self.text_section, "    mov rbx, rax  ; right (max) en rbx").unwrap();
                // Restaurar left (i) a rax
                writeln!(self.text_section, "    pop rax  ; restaurar left (i)").unwrap();
                
                // Comparar rax (i) con rbx (max) según el operador
                // NO hacer push/pop de rbx - rbx se puede usar libremente para la comparación
                writeln!(self.text_section, "    cmp rax, rbx  ; comparar i con max").unwrap();
                
                match op {
                    "<=" => {
                        // Para i <= max: si i > max, salir del loop
                        // cmp rax, rbx hace: rax - rbx
                        // jg salta si (SF != OF), es decir, si rax > rbx (signed)
                        // Esto significa: si i > max, salir del loop
                        writeln!(self.text_section, "    jg {}  ; si i > max, salir del loop", loop_end).unwrap();
                    }
                    ">=" => {
                        // Para i >= max: si i < max, salir del loop
                        writeln!(self.text_section, "    jl {}  ; si i < max, salir del loop", loop_end).unwrap();
                    }
                    "<" => {
                        // Para i < max: si i >= max, salir del loop
                        writeln!(self.text_section, "    jge {}  ; si i >= max, salir del loop", loop_end).unwrap();
                    }
                    ">" => {
                        // Para i > max: si i <= max, salir del loop
                        writeln!(self.text_section, "    jle {}  ; si i <= max, salir del loop", loop_end).unwrap();
                    }
                    "==" => {
                        // Para i == max: si i != max, salir del loop
                        writeln!(self.text_section, "    jne {}  ; si i != max, salir del loop", loop_end).unwrap();
                    }
                    "!=" => {
                        // Para i != max: si i == max, salir del loop
                        writeln!(self.text_section, "    je {}  ; si i == max, salir del loop", loop_end).unwrap();
                    }
                    _ => {
                        // Fallback: evaluar como expresión booleana
                        writeln!(self.text_section, "    cmp rax, 0").unwrap();
                        writeln!(self.text_section, "    je {}", loop_end).unwrap();
                    }
                }
                
                // NO generar print aquí - el body del loop se procesa después
                // El print debe estar en el body del loop, no en la condición
                return; // IMPORTANTE: return aquí para no ejecutar el fallback
            }
        }
        
        // Fallback: evaluar condición como expresión booleana
        self.generate_expression_code(cond_node, source);
        writeln!(self.text_section, "    cmp rax, 0  ; comparar condición con 0").unwrap();
        writeln!(self.text_section, "    je {}  ; si condición es falsa (0), salir", loop_end).unwrap();
    }

    fn generate_if(&mut self, node: Node, source: &str) {
        let if_end = format!("if_end_{}", self.label_count);
        self.label_count += 1;

        // Buscar condición y cuerpo
        let mut condition: Option<Node> = None;
        let mut then_body: Option<Node> = None;

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                match child.kind() {
                    "expression" | "binary_expression" | "comparison_expression" => {
                        condition = Some(child);
                    }
                    "block" => {
                        then_body = Some(child);
                    }
                    _ => {}
                }
            }
        }

        if let Some(cond_node) = condition {
            // Generar código para evaluar condición
            self.generate_condition_code(&cond_node, source, &if_end, None);
            
            if let Some(body_node) = then_body {
                // Procesar cuerpo del if
                for i in 0..body_node.child_count() {
                    if let Some(child) = body_node.child(i) {
                        self.process_node(child, source);
                    }
                }
            }
            
            writeln!(self.text_section, "{}:", if_end).unwrap();
        }
    }

    // Funciones auxiliares removidas - usando búsqueda directa en texto en su lugar
    
    fn generate_assignment(&mut self, node: Node, source: &str) {
        // Estructura: identifier = expression
        let mut var_name: Option<String> = None;
        let mut value_expr: Option<Node> = None;
        let mut found_equals = false;

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                match child.kind() {
                    "identifier" => {
                        if var_name.is_none() {
                            var_name = Some(source[child.start_byte()..child.end_byte()].to_string());
                        }
                    }
                    "expression" | "binary_expression" | "primary_expression" | "number" => {
                        if found_equals {
                            value_expr = Some(child);
                        }
                    }
                    _ => {
                        // Verificar si es el operador '='
                        let text = &source[child.start_byte()..child.end_byte()];
                        if text == "=" {
                            found_equals = true;
                        }
                    }
                }
            }
        }

        if let Some(name) = var_name {
            if let Some(&offset) = self.variables.get(&name) {
                // Generar código para evaluar el valor
                if let Some(expr) = value_expr {
                    self.generate_expression_code(&expr, source);
                } else {
                    // Intentar buscar en el texto completo del nodo
                    let node_text = &source[node.start_byte()..node.end_byte()];
                    if let Some(equals_pos) = node_text.find('=') {
                        let value_part = node_text[equals_pos+1..].trim();
                        if let Some(&var_offset) = self.variables.get(value_part) {
                            writeln!(self.text_section, "    mov rax, [rbp - {}]  ; cargar variable {}", var_offset, value_part).unwrap();
                        } else if let Ok(num) = value_part.parse::<i64>() {
                            writeln!(self.text_section, "    mov rax, {}  ; número literal", num).unwrap();
                        } else {
                            writeln!(self.text_section, "    mov rax, 0  ; valor no encontrado").unwrap();
                        }
                    } else {
                        writeln!(self.text_section, "    mov rax, 0  ; valor no encontrado").unwrap();
                    }
                }
                
                // Guardar valor en variable
                writeln!(self.text_section, "    mov [rbp - {}], rax  ; asignar a variable {}", offset, name).unwrap();
            } else {
                writeln!(self.text_section, "    ; WARNING: Variable {} no declarada", name).unwrap();
            }
        }
    }
}

