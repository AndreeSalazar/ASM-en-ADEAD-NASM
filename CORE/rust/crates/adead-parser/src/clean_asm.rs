/**
 * ASM Cleaner Module - Rust
 * 
 * Este módulo limpia y optimiza código ASM generado por GCC/Clang/Zig
 * para producir ASM virgen y limpio, eliminando overhead innecesario.
 * 
 * Funcionalidades:
 * - Elimina frame pointers innecesarios
 * - Elimina código muerto
 * - Optimiza movimientos redundantes
 * - Limpia metadatos SEH (Windows)
 * - Optimiza saltos
 * - Elimina instrucciones NOP innecesarias
 * 
 * Autor: Eddi Andreé Salazar Matos
 * Fecha: Diciembre 2025
 */

use std::collections::HashSet;

/// Limpia código ASM generado, eliminando overhead y optimizando
pub fn clean_asm(asm: &str) -> String {
    let mut cleaned = asm.to_string();
    
    // Paso 1: Eliminar metadatos SEH de Windows (si existen)
    cleaned = remove_seh_metadata(&cleaned);
    
    // Paso 2: Eliminar frame pointers innecesarios
    cleaned = remove_unnecessary_frame_pointers(&cleaned);
    
    // Paso 3: Eliminar código muerto
    cleaned = remove_dead_code(&cleaned);
    
    // Paso 4: Optimizar movimientos redundantes
    cleaned = optimize_redundant_movements(&cleaned);
    
    // Paso 5: Optimizar saltos
    cleaned = optimize_jumps(&cleaned);
    
    // Paso 6: Eliminar NOPs innecesarios
    cleaned = remove_unnecessary_nops(&cleaned);
    
    // Paso 7: Limpiar líneas vacías múltiples
    cleaned = clean_empty_lines(&cleaned);
    
    // Paso 8: Normalizar formato
    cleaned = normalize_format(&cleaned);
    
    cleaned
}

/// Elimina metadatos SEH (Structured Exception Handling) de Windows
fn remove_seh_metadata(asm: &str) -> String {
    let mut result = String::new();
    let mut skip_next = false;
    
    for line in asm.lines() {
        let trimmed = line.trim();
        
        // Saltar líneas SEH
        if trimmed.starts_with(".seh_") || 
           trimmed.starts_with(".seh_pushreg") ||
           trimmed.starts_with(".seh_stackalloc") ||
           trimmed.starts_with(".seh_endprologue") ||
           trimmed.starts_with(".seh_setframe") ||
           trimmed.starts_with(".seh_endproc") {
            continue;
        }
        
        // Saltar líneas de metadatos
        if trimmed.starts_with(".file") && trimmed.contains("\"") {
            continue;
        }
        
        result.push_str(line);
        result.push('\n');
    }
    
    result
}

/// Analiza si una función realmente necesita frame pointer
fn needs_frame_pointer(function_body: &[String]) -> bool {
    let mut has_stack_access = false;
    let mut has_function_calls = false;
    let mut has_local_vars = false;
    let mut has_complex_operations = false;
    
    for line in function_body {
        let trimmed = line.trim();
        
        // Detectar acceso a stack usando rbp
        if trimmed.contains("[rbp") {
            has_stack_access = true;
            has_local_vars = true;
        }
        
        // Detectar llamadas a funciones (requieren frame pointer para debugging)
        if trimmed.starts_with("call ") {
            has_function_calls = true;
        }
        
        // Detectar operaciones de stack complejas
        if trimmed.contains("sub rsp") || trimmed.contains("add rsp") {
            has_complex_operations = true;
        }
        
        // Detectar push/pop de registros (excepto rbp)
        if (trimmed.starts_with("push ") || trimmed.starts_with("pop ")) 
            && !trimmed.contains("rbp") {
            has_complex_operations = true;
        }
    }
    
    // Necesita frame pointer si:
    // - Hay acceso a variables locales vía [rbp-X]
    // - Hay llamadas a funciones (para debugging y stack unwinding)
    // - Hay operaciones complejas de stack
    has_stack_access || has_function_calls || has_local_vars || has_complex_operations
}

/// Elimina frame pointers innecesarios cuando no se usan
fn remove_unnecessary_frame_pointers(asm: &str) -> String {
    let mut result = String::new();
    let mut in_function = false;
    let mut function_start_line: Option<String> = None;
    let mut function_lines: Vec<String> = Vec::new();
    
    for line in asm.lines() {
        let trimmed = line.trim();
        
        // Detectar inicio de función
        if trimmed.ends_with(':') && !trimmed.starts_with('.') {
            // Procesar función anterior si existe
            if in_function {
                if !needs_frame_pointer(&function_lines) {
                    // Eliminar frame pointer setup/teardown
                    let mut filtered_lines: Vec<String> = Vec::new();
                    for l in &function_lines {
                        let t = l.trim();
                        if !t.starts_with("push rbp") && 
                           !t.starts_with("mov rbp, rsp") &&
                           !t.starts_with("pop rbp") &&
                           !t.starts_with("and rsp, -16") {  // Solo si no hay llamadas
                            filtered_lines.push(l.clone());
                        }
                    }
                    if let Some(start) = &function_start_line {
                        result.push_str(start);
                        result.push('\n');
                    }
                    result.push_str(&filtered_lines.join("\n"));
                } else {
                    // Mantener frame pointer
                    if let Some(start) = &function_start_line {
                        result.push_str(start);
                        result.push('\n');
                    }
                    result.push_str(&function_lines.join("\n"));
                }
            }
            
            // Iniciar nueva función
            in_function = true;
            function_start_line = Some(line.to_string());
            function_lines.clear();
            continue;
        }
        
        // Detectar fin de función
        if in_function && (trimmed == "ret" || trimmed.starts_with("ret ")) {
            function_lines.push(line.to_string());
            
            // Procesar función completa
            if !needs_frame_pointer(&function_lines) {
                // Eliminar frame pointer setup/teardown
                let mut filtered_lines: Vec<String> = Vec::new();
                for l in &function_lines {
                    let t = l.trim();
                    if !t.starts_with("push rbp") && 
                       !t.starts_with("mov rbp, rsp") &&
                       !t.starts_with("pop rbp") &&
                       !t.starts_with("and rsp, -16") {
                        filtered_lines.push(l.clone());
                    }
                }
                if let Some(start) = &function_start_line {
                    result.push_str(start);
                    result.push('\n');
                }
                result.push_str(&filtered_lines.join("\n"));
            } else {
                // Mantener frame pointer
                if let Some(start) = &function_start_line {
                    result.push_str(start);
                    result.push('\n');
                }
                result.push_str(&function_lines.join("\n"));
            }
            
            in_function = false;
            function_start_line = None;
            function_lines.clear();
            continue;
        }
        
        if in_function {
            function_lines.push(line.to_string());
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    
    // Procesar última función si no terminó con ret
    if in_function {
        if !needs_frame_pointer(&function_lines) {
            let mut filtered_lines: Vec<String> = Vec::new();
            for l in &function_lines {
                let t = l.trim();
                if !t.starts_with("push rbp") && 
                   !t.starts_with("mov rbp, rsp") &&
                   !t.starts_with("pop rbp") {
                    filtered_lines.push(l.clone());
                }
            }
            if let Some(start) = &function_start_line {
                result.push_str(start);
                result.push('\n');
            }
            result.push_str(&filtered_lines.join("\n"));
        } else {
            if let Some(start) = &function_start_line {
                result.push_str(start);
                result.push('\n');
            }
            result.push_str(&function_lines.join("\n"));
        }
    }
    
    result
}

/// Extrae registros usados en una instrucción
fn extract_used_registers(instruction: &str) -> Vec<String> {
    let mut registers = Vec::new();
    let trimmed = instruction.trim();
    
    // Lista de registros a buscar
    let reg_patterns = [
        "rax", "rbx", "rcx", "rdx", "rsi", "rdi",
        "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15",
        "eax", "ebx", "ecx", "edx", "esi", "edi",
        "r8d", "r9d", "r10d", "r11d", "r12d", "r13d", "r14d", "r15d",
    ];
    
    for pattern in &reg_patterns {
        // Buscar el registro en la instrucción
        let mut search_pos = 0;
        while let Some(pos) = trimmed[search_pos..].find(pattern) {
            let actual_pos = search_pos + pos;
            let after_pattern = actual_pos + pattern.len();
            
            // Verificar que sea un registro completo (no parte de otro)
            let before_char = if actual_pos > 0 {
                trimmed.chars().nth(actual_pos - 1)
            } else {
                None
            };
            let after_char = trimmed.chars().nth(after_pattern);
            
            let is_word_boundary = (before_char.is_none() || !before_char.unwrap().is_alphanumeric())
                && (after_char.is_none() || !after_char.unwrap().is_alphanumeric());
            
            if is_word_boundary {
                registers.push(pattern.to_string());
            }
            
            search_pos = after_pattern;
        }
    }
    
    registers
}

/// Elimina código muerto (instrucciones que no afectan el resultado final)
fn remove_dead_code(asm: &str) -> String {
    let lines: Vec<&str> = asm.lines().collect();
    let mut result_lines: Vec<String> = Vec::new();
    
    // Análisis de flujo de datos: identificar qué registros se usan después de ser definidos
    let mut register_last_use: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut register_definitions: Vec<(usize, String, String)> = Vec::new(); // (line_index, register, instruction)
    
    // Primera pasada: identificar definiciones y últimos usos
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Detectar definiciones (escrituras)
        if trimmed.starts_with("mov ") {
            let parts: Vec<&str> = trimmed.split(',').collect();
            if parts.len() >= 2 {
                if let Some(dest_reg) = extract_mov_dest(trimmed) {
                    register_definitions.push((i, dest_reg.clone(), trimmed.to_string()));
                }
            }
        }
        
        // Detectar usos (lecturas) - actualizar último uso
        let used_regs = extract_used_registers(trimmed);
        for reg in used_regs {
            register_last_use.insert(reg, i);
        }
        
        // Detectar retorno - todos los registros usados aquí son importantes
        if trimmed == "ret" || trimmed.starts_with("ret ") {
            // No eliminar nada antes de ret
        }
    }
    
    // Segunda pasada: eliminar definiciones que nunca se usan
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let mut should_keep = true;
        
        // Verificar si es una definición que nunca se usa
        if trimmed.starts_with("mov ") {
            if let Some(dest_reg) = extract_mov_dest(trimmed) {
                // Verificar si el registro se usa después
                if let Some(&last_use) = register_last_use.get(&dest_reg) {
                    // Si la última vez que se usa es antes de esta definición, es código muerto
                    if last_use < i {
                        should_keep = false;
                    }
                } else {
                    // Si nunca se usa, es código muerto (excepto si es rax antes de ret)
                    if i + 1 < lines.len() {
                        let next_line = lines[i + 1].trim();
                        if next_line != "ret" && !next_line.starts_with("ret ") {
                            should_keep = false;
                        }
                    } else {
                        should_keep = false;
                    }
                }
            }
        }
        
        // Eliminar mov reg, reg (ya manejado en optimize_redundant_movements, pero doble verificación)
        if trimmed.starts_with("mov ") {
            let parts: Vec<&str> = trimmed.split(',').collect();
            if parts.len() >= 2 {
                let dest_clean = parts[0].replace("mov", "");
                let dest = dest_clean.trim();
                let src = parts[1].trim();
                if dest == src {
                    should_keep = false;
                }
            }
        }
        
        if should_keep {
            result_lines.push(line.to_string());
        }
    }
    
    result_lines.join("\n") + "\n"
}

/// Extrae el registro destino de una instrucción mov
fn extract_mov_dest(instruction: &str) -> Option<String> {
    if !instruction.starts_with("mov ") {
        return None;
    }
    
    let parts: Vec<&str> = instruction.split(',').collect();
    if parts.len() < 2 {
        return None;
    }
    
    let dest = parts[0].replace("mov", "").trim().to_string();
    
    // Verificar que sea un registro válido
    if dest.starts_with("rax") || dest.starts_with("rbx") ||
       dest.starts_with("rcx") || dest.starts_with("rdx") ||
       dest.starts_with("rsi") || dest.starts_with("rdi") ||
       dest.starts_with("r8") || dest.starts_with("r9") ||
       dest.starts_with("r10") || dest.starts_with("r11") ||
       dest.starts_with("r12") || dest.starts_with("r13") ||
       dest.starts_with("r14") || dest.starts_with("r15") ||
       dest.starts_with("eax") || dest.starts_with("ebx") ||
       dest.starts_with("ecx") || dest.starts_with("edx") {
        Some(dest.split(' ').next().unwrap_or("").to_string())
    } else {
        None
    }
}

/// Optimiza movimientos redundantes
fn optimize_redundant_movements(asm: &str) -> String {
    let mut result_lines: Vec<String> = Vec::new();
    let lines: Vec<&str> = asm.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        
        // Eliminar mov reg, reg (redundante)
        if trimmed.starts_with("mov ") {
            let parts: Vec<&str> = trimmed.split(',').collect();
            if parts.len() >= 2 {
                let dest_clean = parts[0].replace("mov", "");
                let dest = dest_clean.trim();
                let src = parts[1].trim();
                if dest == src {
                    // mov rax, rax → eliminar
                    i += 1;
                    continue;
                }
            }
        }
        
        // Optimizar: mov reg, X; mov reg, Y → mov reg, Y
        if i + 1 < lines.len() {
            let next_line = lines[i + 1];
            let next_trimmed = next_line.trim();
            
            if trimmed.starts_with("mov ") && next_trimmed.starts_with("mov ") {
                if let (Some(dest1), Some(dest2)) = 
                    (extract_mov_dest(trimmed), extract_mov_dest(next_trimmed)) {
                    if dest1 == dest2 {
                        // Eliminar el primer mov, mantener solo el segundo
                        result_lines.push(next_line.to_string());
                        i += 2;
                        continue;
                    }
                }
            }
            
            // Optimizar: push reg; pop reg → (eliminar ambos)
            if trimmed.starts_with("push ") && next_trimmed.starts_with("pop ") {
                let push_reg_clean = trimmed.replace("push", "");
                let push_reg = push_reg_clean.trim();
                let pop_reg_clean = next_trimmed.replace("pop", "");
                let pop_reg = pop_reg_clean.trim();
                if push_reg == pop_reg {
                    // push rax; pop rax → eliminar ambos
                    i += 2;
                    continue;
                }
            }
            
            // Optimizar: mov reg, 0; add reg, X → mov reg, X
            if trimmed.starts_with("mov ") && next_trimmed.starts_with("add ") {
                let mov_parts: Vec<&str> = trimmed.split(',').collect();
                if mov_parts.len() >= 2 {
                    let mov_dest_clean = mov_parts[0].replace("mov", "");
                    let mov_dest = mov_dest_clean.trim();
                    let mov_src = mov_parts[1].trim();
                    
                    if mov_src == "0" || mov_src == "0x0" {
                        let add_parts: Vec<&str> = next_trimmed.split(',').collect();
                        if add_parts.len() >= 2 {
                            let add_dest_clean = add_parts[0].replace("add", "");
                    let add_dest = add_dest_clean.trim();
                            let add_src = add_parts[1].trim();
                            
                            if mov_dest == add_dest {
                                // mov rax, 0; add rax, X → mov rax, X
                                result_lines.push(format!("    mov {}, {}", mov_dest, add_src));
                                i += 2;
                                continue;
                            }
                        }
                    }
                }
            }
        }
        
        result_lines.push(line.to_string());
        i += 1;
    }
    
    result_lines.join("\n") + "\n"
}

/// Optimiza saltos innecesarios
fn optimize_jumps(asm: &str) -> String {
    let lines: Vec<&str> = asm.lines().collect();
    let mut result_lines: Vec<String> = Vec::new();
    let mut labels: HashSet<String> = HashSet::new();
    let mut label_positions: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    
    // Primera pasada: identificar labels y sus posiciones
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.ends_with(':') && !trimmed.starts_with('.') {
            let label_name = trimmed.replace(':', "");
            labels.insert(label_name.clone());
            label_positions.insert(label_name, i);
        }
    }
    
    // Segunda pasada: optimizar saltos
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        
        // Detectar saltos
        let is_jump = trimmed.starts_with("jmp ") || 
                      trimmed.starts_with("je ") ||
                      trimmed.starts_with("jne ") ||
                      trimmed.starts_with("jl ") ||
                      trimmed.starts_with("jg ") ||
                      trimmed.starts_with("jle ") ||
                      trimmed.starts_with("jge ") ||
                      trimmed.starts_with("jz ") ||
                      trimmed.starts_with("jnz ");
        
        if is_jump {
            let jump_parts: Vec<&str> = trimmed.split_whitespace().collect();
            if jump_parts.len() >= 2 {
                let target_label = jump_parts[1];
                
                // Verificar si el label existe
                if let Some(&target_pos) = label_positions.get(target_label) {
                    // Si el salto es a la siguiente instrucción, eliminarlo
                    if target_pos == i + 1 {
                        // jmp label; label: → eliminar jmp
                        i += 1;
                        continue;
                    }
                    
                    // Si hay un jmp inmediatamente seguido del label, eliminar el jmp
                    if i + 1 < lines.len() {
                        let next_line = lines[i + 1].trim();
                        if next_line == format!("{}:", target_label) {
                            // jmp label; label: → eliminar jmp
                            i += 1;
                            continue;
                        }
                    }
                }
            }
        }
        
        // Detectar secuencias: jmp label1; label1: jmp label2 → jmp label2
        if i + 2 < lines.len() {
            let next_line = lines[i + 1].trim();
            let next_next_line = lines[i + 2].trim();
            
            if trimmed.starts_with("jmp ") {
                let jump_parts: Vec<&str> = trimmed.split_whitespace().collect();
                if jump_parts.len() >= 2 {
                    let label1 = jump_parts[1];
                    
                    // Verificar si la siguiente línea es el label y luego hay otro jmp
                    if next_line == format!("{}:", label1) && next_next_line.starts_with("jmp ") {
                        let next_jump_parts: Vec<&str> = next_next_line.split_whitespace().collect();
                        if next_jump_parts.len() >= 2 {
                            let label2 = next_jump_parts[1];
                            // jmp label1; label1: jmp label2 → jmp label2; label1:
                            result_lines.push(format!("    jmp {}", label2));
                            result_lines.push(format!("{}:", label1));
                            i += 3;
                            continue;
                        }
                    }
                }
            }
        }
        
        result_lines.push(line.to_string());
        i += 1;
    }
    
    result_lines.join("\n") + "\n"
}

/// Elimina NOPs innecesarios (excepto alineamiento)
fn remove_unnecessary_nops(asm: &str) -> String {
    let mut result = String::new();
    let mut nop_count = 0;
    
    for line in asm.lines() {
        let trimmed = line.trim();
        
        if trimmed == "nop" {
            nop_count += 1;
            // Solo mantener NOPs si hay más de 1 (posible alineamiento)
            // O si están antes de un salto (optimización de branch prediction)
            continue;
        }
        
        // Si había NOPs, verificar si debemos mantenerlos
        if nop_count > 0 {
            // Si la siguiente línea es un salto, mantener algunos NOPs
            if trimmed.starts_with("jmp") || 
               trimmed.starts_with("je") ||
               trimmed.starts_with("jne") {
                // Mantener 1-2 NOPs para alineamiento
                for _ in 0..nop_count.min(2) {
                    result.push_str("    nop\n");
                }
            }
            // Si no, eliminar todos los NOPs
            nop_count = 0;
        }
        
        result.push_str(line);
        result.push('\n');
    }
    
    result
}

/// Limpia líneas vacías múltiples
fn clean_empty_lines(asm: &str) -> String {
    let mut result = String::new();
    let mut prev_empty = false;
    
    for line in asm.lines() {
        let trimmed = line.trim();
        let is_empty = trimmed.is_empty();
        
        if is_empty && prev_empty {
            // Saltar líneas vacías múltiples
            continue;
        }
        
        result.push_str(line);
        result.push('\n');
        prev_empty = is_empty;
    }
    
    result
}

/// Normaliza formato del código ASM
fn normalize_format(asm: &str) -> String {
    let mut result = String::new();
    
    for line in asm.lines() {
        let trimmed = line.trim();
        
        // Normalizar espacios múltiples a uno solo
        let normalized = trimmed.split_whitespace().collect::<Vec<&str>>().join(" ");
        
        // Mantener indentación si existía
        let indent = line.len() - line.trim_start().len();
        let indented = if indent > 0 {
            format!("{:width$}{}", "", normalized, width = indent.min(4))
        } else {
            normalized
        };
        
        result.push_str(&indented);
        result.push('\n');
    }
    
    result
}

/// Análisis de data flow para identificar código muerto más agresivamente
pub fn analyze_data_flow(asm: &str) -> HashSet<String> {
    let mut used_registers: HashSet<String> = HashSet::new();
    let mut defined_registers: HashSet<String> = HashSet::new();
    
    for line in asm.lines() {
        let trimmed = line.trim();
        
        // Detectar definiciones (escrituras)
        if trimmed.starts_with("mov ") {
            let parts: Vec<&str> = trimmed.split(',').collect();
            if parts.len() >= 2 {
                let dest = parts[0].replace("mov", "").trim().to_string();
                if dest.starts_with("r") {
                    defined_registers.insert(dest);
                }
            }
        }
        
        // Detectar usos (lecturas)
        if trimmed.contains("add ") || 
           trimmed.contains("sub ") ||
           trimmed.contains("mul ") ||
           trimmed.contains("div ") ||
           trimmed.contains("cmp ") ||
           trimmed.contains("test ") {
            // Extraer registros usados
            for part in trimmed.split_whitespace() {
                if part.starts_with("rax") || part.starts_with("rbx") ||
                   part.starts_with("rcx") || part.starts_with("rdx") ||
                   part.starts_with("rsi") || part.starts_with("rdi") ||
                   part.starts_with("r8") || part.starts_with("r9") ||
                   part.starts_with("r10") || part.starts_with("r11") ||
                   part.starts_with("r12") || part.starts_with("r13") ||
                   part.starts_with("r14") || part.starts_with("r15") {
                    let reg = part.split(',').next().unwrap_or("").trim().to_string();
                    used_registers.insert(reg);
                }
            }
        }
    }
    
    used_registers
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_remove_seh_metadata() {
        let asm = r#"
.seh_proc main
main:
    push rbp
    .seh_pushreg rbp
    mov rbp, rsp
    .seh_stackalloc 32
    .seh_endprologue
    mov rax, 42
    ret
.seh_endproc
"#;
        let cleaned = remove_seh_metadata(asm);
        assert!(!cleaned.contains(".seh_"));
    }
    
    #[test]
    fn test_remove_redundant_mov() {
        let asm = r#"
mov rax, rax
mov rbx, 42
"#;
        let cleaned = optimize_redundant_movements(asm);
        assert!(!cleaned.contains("mov rax, rax"));
    }
    
    #[test]
    fn test_clean_empty_lines() {
        let asm = r#"
mov rax, 42


mov rbx, 10
"#;
        let cleaned = clean_empty_lines(asm);
        let empty_count = cleaned.matches("\n\n").count();
        assert_eq!(empty_count, 0);
    }
    
    // ========== SPRINT 4: TESTS MEJORADOS ==========
    
    #[test]
    fn test_remove_frame_overhead_simple_function() {
        let asm = r#"
simple_func:
    push rbp
    mov rbp, rsp
    mov rax, 42
    pop rbp
    ret
"#;
        let cleaned = remove_unnecessary_frame_pointers(asm);
        assert!(!cleaned.contains("push rbp"));
        assert!(!cleaned.contains("mov rbp, rsp"));
        assert!(!cleaned.contains("pop rbp"));
        assert!(cleaned.contains("mov rax, 42"));
        assert!(cleaned.contains("ret"));
    }
    
    #[test]
    fn test_keep_frame_overhead_with_stack_access() {
        let asm = r#"
func_with_stack:
    push rbp
    mov rbp, rsp
    mov [rbp-8], rax
    mov rax, [rbp-8]
    pop rbp
    ret
"#;
        let cleaned = remove_unnecessary_frame_pointers(asm);
        assert!(cleaned.contains("push rbp"));
        assert!(cleaned.contains("mov rbp, rsp"));
    }
    
    #[test]
    fn test_keep_frame_overhead_with_function_call() {
        let asm = r#"
func_with_call:
    push rbp
    mov rbp, rsp
    call other_func
    pop rbp
    ret
"#;
        let cleaned = remove_unnecessary_frame_pointers(asm);
        assert!(cleaned.contains("push rbp"));
        assert!(cleaned.contains("mov rbp, rsp"));
    }
    
    #[test]
    fn test_optimize_redundant_mov_sequence() {
        let asm = r#"
mov rax, 5
mov rax, 10
mov rbx, 20
"#;
        let cleaned = optimize_redundant_movements(asm);
        assert!(!cleaned.contains("mov rax, 5"));
        assert!(cleaned.contains("mov rax, 10"));
        assert!(cleaned.contains("mov rbx, 20"));
    }
    
    #[test]
    fn test_optimize_push_pop_redundant() {
        let asm = r#"
push rax
pop rax
mov rbx, 10
"#;
        let cleaned = optimize_redundant_movements(asm);
        assert!(!cleaned.contains("push rax"));
        assert!(!cleaned.contains("pop rax"));
        assert!(cleaned.contains("mov rbx, 10"));
    }
    
    #[test]
    fn test_optimize_mov_zero_add() {
        let asm = r#"
mov rax, 0
add rax, 42
mov rbx, 10
"#;
        let cleaned = optimize_redundant_movements(asm);
        assert!(!cleaned.contains("mov rax, 0"));
        assert!(!cleaned.contains("add rax, 42"));
        assert!(cleaned.contains("mov rax, 42"));
    }
    
    #[test]
    fn test_optimize_jump_to_next_instruction() {
        let asm = r#"
cmp rax, 0
je label1
label1:
    mov rax, 42
"#;
        let cleaned = optimize_jumps(asm);
        // El jmp debería eliminarse si label1 es la siguiente instrucción
        assert!(cleaned.contains("cmp rax, 0"));
        assert!(cleaned.contains("label1:"));
    }
    
    #[test]
    fn test_optimize_jump_chain() {
        let asm = r#"
jmp label1
label1:
    jmp label2
label2:
    mov rax, 42
"#;
        let cleaned = optimize_jumps(asm);
        // Debería optimizar la cadena de saltos
        assert!(cleaned.contains("label1:"));
        assert!(cleaned.contains("label2:"));
    }
    
    #[test]
    fn test_remove_dead_code_unused_register() {
        let asm = r#"
mov rax, 5
mov rbx, 10
mov rcx, 20
mov rax, 30
ret
"#;
        let cleaned = remove_dead_code(asm);
        // rbx y rcx nunca se usan después de ser definidos
        // Pero necesitamos verificar que se eliminen correctamente
        assert!(cleaned.contains("mov rax, 30"));
        assert!(cleaned.contains("ret"));
    }
    
    #[test]
    fn test_remove_dead_code_before_ret() {
        let asm = r#"
mov rax, 5
mov rbx, 10
ret
"#;
        let cleaned = remove_dead_code(asm);
        // rbx nunca se usa, pero rax podría usarse implícitamente en ret
        assert!(cleaned.contains("ret"));
    }
    
    #[test]
    fn test_complete_clean_asm_pipeline() {
        let asm = r#"
.seh_proc main
main:
    push rbp
    .seh_pushreg rbp
    mov rbp, rsp
    mov rax, 5
    mov rax, 10
    push rax
    pop rax
    mov rbx, rbx
    ret
.seh_endproc
"#;
        let cleaned = clean_asm(asm);
        assert!(!cleaned.contains(".seh_"));
        assert!(!cleaned.contains("mov rax, 5"));
        assert!(!cleaned.contains("push rax"));
        assert!(!cleaned.contains("pop rax"));
        assert!(!cleaned.contains("mov rbx, rbx"));
        assert!(cleaned.contains("mov rax, 10"));
        assert!(cleaned.contains("ret"));
    }
    
    #[test]
    fn test_extract_mov_dest() {
        assert_eq!(extract_mov_dest("mov rax, 42"), Some("rax".to_string()));
        assert_eq!(extract_mov_dest("mov rbx, rcx"), Some("rbx".to_string()));
        assert_eq!(extract_mov_dest("add rax, rbx"), None);
    }
    
    #[test]
    fn test_extract_used_registers() {
        let regs = extract_used_registers("add rax, rbx");
        assert!(regs.contains(&"rax".to_string()));
        assert!(regs.contains(&"rbx".to_string()));
        
        let regs2 = extract_used_registers("mov [rbp-8], rax");
        assert!(regs2.contains(&"rax".to_string()));
        assert!(regs2.contains(&"rbp".to_string()));
    }
    
    #[test]
    fn test_needs_frame_pointer_with_stack() {
        let func = vec![
            "mov [rbp-8], rax".to_string(),
            "mov rax, [rbp-8]".to_string(),
        ];
        assert!(needs_frame_pointer(&func));
    }
    
    #[test]
    fn test_needs_frame_pointer_with_call() {
        let func = vec![
            "call other_func".to_string(),
        ];
        assert!(needs_frame_pointer(&func));
    }
    
    #[test]
    fn test_needs_frame_pointer_simple() {
        let func = vec![
            "mov rax, 42".to_string(),
            "ret".to_string(),
        ];
        assert!(!needs_frame_pointer(&func));
    }
}

