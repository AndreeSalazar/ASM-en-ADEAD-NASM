// Optimizador de código NASM generado
// Elimina código muerto, optimiza registros, etc.

use std::collections::HashSet;

pub struct CodeOptimizer {
    used_labels: HashSet<String>,
    used_functions: HashSet<String>,
}

impl CodeOptimizer {
    pub fn new() -> Self {
        Self {
            used_labels: HashSet::new(),
            used_functions: HashSet::new(),
        }
    }

    /// Analizar código y encontrar funciones/labels usados
    pub fn analyze_usage(&mut self, code: &str) {
        // Buscar llamadas a funciones: call fn_*
        for line in code.lines() {
            if let Some(call_pos) = line.find("call ") {
                let rest = &line[call_pos + 5..];
                if let Some(end) = rest.find(|c: char| c.is_whitespace() || c == ';') {
                    let func_name = rest[..end].trim();
                    if func_name.starts_with("fn_") {
                        self.used_functions.insert(func_name.to_string());
                    }
                }
            }
            
            // Buscar saltos a labels: jmp label, je label, etc.
            let jump_ops = ["jmp ", "je ", "jne ", "jl ", "jle ", "jg ", "jge "];
            for op in &jump_ops {
                if let Some(jmp_pos) = line.find(op) {
                    let rest = &line[jmp_pos + op.len()..];
                    if let Some(end) = rest.find(|c: char| c.is_whitespace() || c == ';') {
                        let label = rest[..end].trim();
                        if !label.is_empty() {
                            self.used_labels.insert(label.to_string());
                        }
                    }
                }
            }
        }
    }

    /// Eliminar código muerto (funciones no usadas)
    /// NOTA: Solo elimina funciones stdlib_ no usadas, NUNCA funciones de usuario (fn_)
    pub fn remove_dead_code(&self, code: &str) -> String {
        let mut result = Vec::new();
        let mut in_unused_stdlib = false;
        let mut function_name = String::new();

        for line in code.lines() {
            // Solo eliminar funciones stdlib_ no usadas (NUNCA funciones fn_ de usuario)
            if line.trim().starts_with("stdlib_") && line.trim().ends_with(":") {
                function_name = line.trim().trim_end_matches(":").to_string();
                // Solo eliminar si es stdlib Y no está usada
                in_unused_stdlib = !self.used_functions.contains(&function_name);
                
                if !in_unused_stdlib {
                    result.push(line.to_string());
                }
                continue;
            }
            
            // Detectar fin de función stdlib (next label o ret simple)
            if in_unused_stdlib {
                // Detectar inicio de otra función (termina la stdlib)
                if (line.trim().starts_with("stdlib_") || 
                    line.trim().starts_with("fn_") ||
                    line.trim().starts_with("main:") ||
                    line.trim().starts_with("; DEBUG") ||
                    line.trim().starts_with("; ADead")) && 
                   (line.trim().ends_with(":") || !line.trim().is_empty()) {
                    in_unused_stdlib = false;
                } else {
                    continue; // Saltar líneas de stdlib no usada
                }
            }

            result.push(line.to_string());
        }

        result.join("\n")
    }

    /// Optimizar uso de registros (reducir push/pop innecesarios)
    pub fn optimize_registers(&self, code: &str) -> String {
        // Por ahora, retornar código sin cambios
        // En el futuro, podríamos analizar qué registros realmente se usan
        code.to_string()
    }

    /// Limpiar y formatear el ASM generado para mejor legibilidad
    pub fn clean_asm(&self, code: &str) -> String {
        let mut result = Vec::new();
        let mut prev_was_empty = false;
        let mut in_data_section = false;
        
        for line in code.lines() {
            let trimmed = line.trim();
            
            // Detectar secciones
            if trimmed.starts_with("section .data") {
                in_data_section = true;
                if !result.is_empty() && !prev_was_empty {
                    result.push(String::new());
                }
                result.push(line.to_string());
                prev_was_empty = false;
                continue;
            }
            if trimmed.starts_with("section .text") {
                in_data_section = false;
                if !result.is_empty() && !prev_was_empty {
                    result.push(String::new());
                }
                result.push(line.to_string());
                prev_was_empty = false;
                continue;
            }
            
            // Eliminar comentarios de debug redundantes
            if trimmed.starts_with("; ADead:") && trimmed.contains("...") {
                continue;
            }
            
            // Eliminar líneas vacías consecutivas
            if trimmed.is_empty() {
                if !prev_was_empty {
                    result.push(String::new());
                    prev_was_empty = true;
                }
                continue;
            }
            
            // Añadir línea vacía antes de labels de función
            if (trimmed.starts_with("fn_") || trimmed == "main:") && trimmed.ends_with(":") {
                if !result.is_empty() && !prev_was_empty {
                    result.push(String::new());
                }
            }
            
            result.push(line.to_string());
            prev_was_empty = false;
        }
        
        result.join("\n")
    }
}

