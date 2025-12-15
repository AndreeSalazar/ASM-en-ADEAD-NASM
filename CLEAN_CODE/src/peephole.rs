//! Optimizaciones Peephole - Patrones locales (3-5 líneas)

use regex::Regex;

/// Optimizador Peephole para ASM
pub struct PeepholeOptimizer {
    // Regex compilados para mejor performance
    redundant_mov: Regex,
    jmp_to_next: Regex,
    mov_zero_add: Regex,
    push_pop_pair: Regex,
    nop_sequence: Regex,
}

impl PeepholeOptimizer {
    pub fn new() -> Self {
        Self {
            // mov reg, reg (redundante) - sin backreference, se maneja manualmente
            redundant_mov: Regex::new(r"(?m)^\s*mov\s+(\w+),\s+(\w+)\s*$").unwrap(),
            
            // jmp label seguido inmediatamente por label: - sin backreference
            jmp_to_next: Regex::new(r"(?m)^\s*jmp\s+(\w+)\s*$").unwrap(),
            
            // mov reg, 0 seguido de add reg, val → mov reg, val
            mov_zero_add: Regex::new(r"(?m)^\s*mov\s+(\w+),\s+0\s*$\n\s*add\s+(\w+),\s+(\d+)\s*$").unwrap(),
            
            // push reg seguido de pop reg (sin código entre medio) - sin backreference
            push_pop_pair: Regex::new(r"(?m)^\s*push\s+(\w+)\s*$\n\s*pop\s+(\w+)\s*$").unwrap(),
            
            // Secuencias de nop
            nop_sequence: Regex::new(r"(?m)^\s*nop\s*$\n(?:\s*nop\s*$\n)+").unwrap(),
        }
    }

    /// Aplica todas las optimizaciones peephole básicas
    pub fn optimize(&self, asm: &str) -> Result<String, String> {
        let mut optimized = asm.to_string();

        // Aplicar optimizaciones en orden de seguridad
        optimized = self.remove_redundant_mov(&optimized);
        optimized = self.remove_jmp_to_next(&optimized);
        optimized = self.simplify_mov_zero_add(&optimized);
        optimized = self.remove_push_pop_pair(&optimized);
        optimized = self.remove_nop_sequences(&optimized);

        Ok(optimized)
    }

    /// Aplica optimizaciones peephole ampliadas (ventanas más grandes, patrones GCC/Clang)
    pub fn optimize_extended(&self, asm: &str) -> Result<String, String> {
        let mut optimized = self.optimize(asm)?;

        // Optimizaciones adicionales para nivel avanzado/extremo
        optimized = self.remove_frame_setup_if_unused(&optimized)?;
        optimized = self.optimize_lea_patterns(&optimized)?;
        optimized = self.remove_redundant_stack_ops(&optimized)?;

        Ok(optimized)
    }

    /// Elimina frame setup (push rbp / mov rbp, rsp) si no se usa
    fn remove_frame_setup_if_unused(&self, asm: &str) -> Result<String, String> {
        let lines: Vec<&str> = asm.lines().collect();
        let mut result = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Detectar patrón: push rbp seguido de mov rbp, rsp
            if line == "push rbp" && i + 1 < lines.len() {
                let next_line = lines[i + 1].trim();
                if next_line == "mov rbp, rsp" {
                    // Buscar si rbp se usa después (conservador: solo eliminamos si no se usa)
                    let mut rbp_used = false;
                    for j in (i + 2)..lines.len().min(i + 20) {
                        let check_line = lines[j].trim();
                        if check_line.contains("rbp") && !check_line.contains("pop rbp") {
                            rbp_used = true;
                            break;
                        }
                        // Si encontramos pop rbp, el frame se cierra
                        if check_line == "pop rbp" {
                            break;
                        }
                    }

                    if !rbp_used {
                        // Saltar push rbp y mov rbp, rsp
                        i += 2;
                        continue;
                    }
                }
            }

            result.push(lines[i]);
            i += 1;
        }

        Ok(result.join("\n"))
    }

    /// Optimiza patrones LEA complejos
    fn optimize_lea_patterns(&self, asm: &str) -> Result<String, String> {
        // Por ahora, conservador - no hacemos cambios agresivos en LEA
        Ok(asm.to_string())
    }

    /// Elimina operaciones de stack redundantes
    fn remove_redundant_stack_ops(&self, asm: &str) -> Result<String, String> {
        // Ya manejado por remove_push_pop_pair, pero podemos agregar más patrones aquí
        Ok(asm.to_string())
    }

    /// Elimina movimientos redundantes: mov reg, reg
    fn remove_redundant_mov(&self, asm: &str) -> String {
        let lines: Vec<&str> = asm.lines().collect();
        let mut result = Vec::new();

        for line in &lines {
            let trimmed = line.trim();
            // Detectar mov reg1, reg2 donde reg1 == reg2
            if let Some(caps) = self.redundant_mov.captures(trimmed) {
                if let (Some(reg1), Some(reg2)) = (caps.get(1), caps.get(2)) {
                    if reg1.as_str() == reg2.as_str() {
                        // Es redundante, saltar esta línea
                        continue;
                    }
                }
            }
            result.push(*line);
        }

        result.join("\n")
    }

    /// Elimina jmp a label que está inmediatamente después
    fn remove_jmp_to_next(&self, asm: &str) -> String {
        // Buscar patrones jmp label seguido de label:
        let lines: Vec<&str> = asm.lines().collect();
        let mut new_lines = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();
            
            // Verificar si es jmp label
            if let Some(jmp_match) = line.strip_prefix("jmp ") {
                let label = jmp_match.trim();
                
                // Verificar si la siguiente línea es el label
                if i + 1 < lines.len() {
                    let next_line = lines[i + 1].trim();
                    if next_line == format!("{}:", label) {
                        // Saltar el jmp, mantener el label
                        i += 1;
                        continue;
                    }
                }
            }
            
            new_lines.push(lines[i]);
            i += 1;
        }

        new_lines.join("\n")
    }

    /// Simplifica mov reg, 0 seguido de add reg, val → mov reg, val
    fn simplify_mov_zero_add(&self, asm: &str) -> String {
        let lines: Vec<&str> = asm.lines().collect();
        let mut new_lines: Vec<String> = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();
            
            // Verificar si es mov reg, 0 seguido de add reg, val
            if i + 1 < lines.len() {
                let next_line = lines[i + 1].trim();
                let combined = format!("{}\n{}", line, next_line);
                
                if let Some(mov_match) = self.mov_zero_add.captures(&combined) {
                    if let (Some(reg1), Some(reg2), Some(val)) = (mov_match.get(1), mov_match.get(2), mov_match.get(3)) {
                        // Verificar que ambos registros sean iguales
                        if reg1.as_str() == reg2.as_str() {
                            // Reemplazar con mov reg, val
                            new_lines.push(format!("    mov {}, {}", reg1.as_str(), val.as_str()));
                            i += 2; // Saltar ambas líneas
                            continue;
                        }
                    }
                }
            }
            
            new_lines.push(lines[i].to_string());
            i += 1;
        }

        new_lines.join("\n")
    }

    /// Elimina push reg seguido de pop reg (sin código entre medio)
    fn remove_push_pop_pair(&self, asm: &str) -> String {
        let lines: Vec<&str> = asm.lines().collect();
        let mut result = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();
            
            // Verificar si es push reg seguido de pop reg
            if let Some(push_match) = line.strip_prefix("push ") {
                let reg_push = push_match.trim();
                
                if i + 1 < lines.len() {
                    let next_line = lines[i + 1].trim();
                    if let Some(pop_match) = next_line.strip_prefix("pop ") {
                        let reg_pop = pop_match.trim();
                        
                        // Si los registros son iguales, eliminar ambas líneas
                        if reg_push == reg_pop {
                            i += 2;
                            continue;
                        }
                    }
                }
            }
            
            result.push(lines[i]);
            i += 1;
        }

        result.join("\n")
    }

    /// Elimina secuencias de nop (múltiples nop seguidos)
    fn remove_nop_sequences(&self, asm: &str) -> String {
        // Reemplazar múltiples nop seguidos con uno solo (o ninguno)
        self.nop_sequence.replace_all(asm, "").to_string()
    }
}

impl Default for PeepholeOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_redundant_mov() {
        let optimizer = PeepholeOptimizer::new();
        let asm = "mov rax, rax\nmov rbx, rbx\nmov rax, 5\n";
        let result = optimizer.optimize(asm).unwrap();
        assert!(!result.contains("mov rax, rax"));
        assert!(!result.contains("mov rbx, rbx"));
        assert!(result.contains("mov rax, 5"));
    }

    #[test]
    fn test_remove_jmp_to_next() {
        let optimizer = PeepholeOptimizer::new();
        let asm = "jmp label1\nlabel1:\nmov rax, 5\n";
        let result = optimizer.optimize(asm).unwrap();
        assert!(!result.contains("jmp label1"));
        assert!(result.contains("label1:"));
        assert!(result.contains("mov rax, 5"));
    }
}

