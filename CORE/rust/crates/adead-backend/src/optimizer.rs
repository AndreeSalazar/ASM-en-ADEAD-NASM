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
    pub fn remove_dead_code(&self, code: &str) -> String {
        let mut result = Vec::new();
        let mut in_unused_function = false;
        let mut function_name = String::new();
        let mut brace_count = 0;

        for line in code.lines() {
            // Detectar inicio de función
            if line.trim().starts_with("fn_") && line.trim().ends_with(":") {
                function_name = line.trim().trim_end_matches(":").to_string();
                in_unused_function = !self.used_functions.contains(&function_name);
                brace_count = 0;
                
                if !in_unused_function {
                    result.push(line.to_string());
                }
                continue;
            }

            if in_unused_function {
                // Contar niveles de anidación (simplificado)
                if line.contains("{") {
                    brace_count += 1;
                }
                if line.contains("}") {
                    brace_count -= 1;
                    if brace_count < 0 {
                        in_unused_function = false;
                    }
                }
                continue;
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
}

