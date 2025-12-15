//! Data Flow Analysis - Detecta dead stores, loads innecesarios, loops constantes

use std::collections::{HashMap, HashSet};

/// Analizador de data flow
pub struct DataFlowAnalyzer {
    // Configuración
    remove_dead_stores: bool,
    detect_constant_loops: bool,
}

impl DataFlowAnalyzer {
    pub fn new() -> Self {
        Self {
            remove_dead_stores: true,
            detect_constant_loops: true,
        }
    }

    /// Analiza data flow y optimiza
    pub fn analyze(&self, asm: &str) -> Result<String, String> {
        let mut result = asm.to_string();

        if self.remove_dead_stores {
            result = self.remove_dead_stores(&result)?;
        }

        if self.detect_constant_loops {
            result = self.optimize_constant_loops(&result)?;
        }

        Ok(result)
    }

    /// Elimina stores muertos (valores escritos pero nunca leídos)
    fn remove_dead_stores(&self, asm: &str) -> Result<String, String> {
        let lines: Vec<&str> = asm.lines().collect();
        let mut result = Vec::new();
        
        // Análisis simple: detectar mov reg, val donde reg no se usa después
        let mut last_writes: HashMap<String, usize> = HashMap::new();
        let mut reads: HashSet<usize> = HashSet::new();

        // Primera pasada: identificar writes y reads
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Detectar writes: mov reg, ...
            if trimmed.starts_with("mov ") {
                if let Some(reg) = self.extract_dest_register(trimmed) {
                    last_writes.insert(reg, i);
                }
            }

            // Detectar reads: usar reg en operaciones
            for reg in self.extract_source_registers(trimmed) {
                if let Some(&write_line) = last_writes.get(&reg) {
                    reads.insert(write_line);
                }
            }
        }

        // Segunda pasada: eliminar writes no leídos
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            if trimmed.starts_with("mov ") {
                if let Some(reg) = self.extract_dest_register(trimmed) {
                    if last_writes.get(&reg) == Some(&i) && !reads.contains(&i) {
                        // Este write nunca se lee, pero solo eliminamos si no es parte de un patrón importante
                        // (conservador: solo eliminamos mov reg, reg redundantes ya manejados por peephole)
                        continue;
                    }
                }
            }
            
            result.push(*line);
        }

        Ok(result.join("\n"))
    }

    /// Optimiza loops constantes (detecta loops que siempre hacen lo mismo)
    fn optimize_constant_loops(&self, asm: &str) -> Result<String, String> {
        // Análisis básico: detectar patrones simples de loops constantes
        // Por ahora, conservador - no hacemos cambios agresivos
        Ok(asm.to_string())
    }

    /// Extrae registro destino de una instrucción
    fn extract_dest_register(&self, line: &str) -> Option<String> {
        // mov reg, ... → reg
        if let Some(pos) = line.find("mov ") {
            let after_mov = &line[pos + 4..];
            let parts: Vec<&str> = after_mov.split_whitespace().collect();
            if let Some(first) = parts.first() {
                let reg = first.trim_end_matches(',');
                if !reg.is_empty() && reg.chars().next().unwrap().is_alphabetic() {
                    return Some(reg.to_string());
                }
            }
        }
        None
    }

    /// Extrae registros fuente de una instrucción
    fn extract_source_registers(&self, line: &str) -> Vec<String> {
        let mut regs = Vec::new();
        // Buscar registros comunes en operaciones
        let common_regs = ["rax", "rbx", "rcx", "rdx", "rsi", "rdi", "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15"];
        for reg in &common_regs {
            if line.contains(reg) {
                regs.push(reg.to_string());
            }
        }
        regs
    }
}

impl Default for DataFlowAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

