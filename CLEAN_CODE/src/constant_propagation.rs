//! Constant Propagation - Reemplaza valores constantes conocidos

use regex::Regex;
use std::collections::HashMap;

/// Propagador de constantes
pub struct ConstantPropagator {
    // Regex para detectar asignaciones de constantes
    mov_const: Regex,
}

impl ConstantPropagator {
    pub fn new() -> Self {
        Self {
            // mov reg, constante (número)
            mov_const: Regex::new(r"mov\s+(\w+),\s+(\d+)").unwrap(),
        }
    }

    /// Propaga constantes conocidas a través del código
    pub fn propagate(&self, asm: &str) -> Result<String, String> {
        let lines: Vec<&str> = asm.lines().collect();
        let mut constants: HashMap<String, i64> = HashMap::new();
        let mut result = Vec::new();

        for line in &lines {
            let trimmed = line.trim();
            let mut modified_line = line.to_string();

            // Detectar asignaciones de constantes: mov reg, valor
            if let Some(caps) = self.mov_const.captures(trimmed) {
                if let (Some(reg), Some(val_str)) = (caps.get(1), caps.get(2)) {
                    if let Ok(val) = val_str.as_str().parse::<i64>() {
                        let reg_name = reg.as_str().to_string();
                        constants.insert(reg_name.clone(), val);
                    }
                }
            }

            // Reemplazar usos de registros con valores constantes conocidos
            for (reg, val) in &constants {
                // Reemplazar mov reg2, reg → mov reg2, val (si reg es constante)
                let pattern = format!(r"mov\s+(\w+),\s+{}\b", reg);
                if let Ok(_re) = Regex::new(&pattern) {
                    if _re.is_match(trimmed) {
                        modified_line = _re.replace(&modified_line, |caps: &regex::Captures| {
                            format!("mov {}, {}", &caps[1], val)
                        }).to_string();
                    }
                }

                // Reemplazar add reg2, reg → add reg2, val
                let pattern = format!(r"add\s+(\w+),\s+{}\b", reg);
                if let Ok(_re) = Regex::new(&pattern) {
                    if _re.is_match(trimmed) {
                        modified_line = _re.replace(&modified_line, |caps: &regex::Captures| {
                            format!("add {}, {}", &caps[1], val)
                        }).to_string();
                    }
                }
            }

            // Si el registro se modifica, eliminar de constantes
            if trimmed.contains("add ") || trimmed.contains("sub ") || 
               trimmed.contains("mul ") || trimmed.contains("div ") ||
               trimmed.contains("mov ") {
                // Detectar si algún registro constante se modifica
                for reg in constants.keys().cloned().collect::<Vec<_>>() {
                    let pattern = format!(r"\b{}\b", reg);
                    if let Ok(re) = Regex::new(&pattern) {
                        // Si aparece en el lado izquierdo de una operación, se modifica
                        if trimmed.contains(&format!("mov {} ", reg)) ||
                           trimmed.contains(&format!("add {} ", reg)) ||
                           trimmed.contains(&format!("sub {} ", reg)) {
                            constants.remove(&reg);
                        }
                    }
                }
            }

            result.push(modified_line);
        }

        Ok(result.join("\n"))
    }
}

impl Default for ConstantPropagator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_propagation() {
        let propagator = ConstantPropagator::new();
        let asm = "mov rax, 5\nmov rbx, rax\n";
        let result = propagator.propagate(asm).unwrap();
        assert!(result.contains("mov rbx, 5"));
    }
}

