// Optimizador de uso de registros
// Reduce push/pop innecesarios analizando qué registros realmente se usan

pub struct RegisterOptimizer {
    used_registers: std::collections::HashSet<String>,
}

impl RegisterOptimizer {
    pub fn new() -> Self {
        Self {
            used_registers: std::collections::HashSet::new(),
        }
    }

    /// Analizar qué registros se usan en una función
    pub fn analyze_function(&mut self, code: &str) {
        // Buscar uso de registros: mov rXX, ... o ... rXX
        let register_patterns = [
            "rbx", "rdi", "rsi", "r12", "r13", "r14", "r15",
            "ebx", "edi", "esi", "r12d", "r13d", "r14d", "r15d",
        ];
        
        for pattern in &register_patterns {
            if code.contains(pattern) {
                self.used_registers.insert(pattern.to_string());
            }
        }
    }

    /// Generar prologue optimizado (solo preserva registros usados)
    pub fn generate_optimized_prologue(&self, needs_shadow_space: bool) -> Vec<String> {
        let mut code = Vec::new();
        
        code.push("    push rbp".to_string());
        code.push("    mov rbp, rsp".to_string());
        
        // Solo preservar registros que realmente se usan
        let registers_to_save = [
            ("rbx", "rbx"),
            ("rdi", "rdi"),
            ("rsi", "rsi"),
            ("r12", "r12"),
            ("r13", "r13"),
            ("r14", "r14"),
            ("r15", "r15"),
        ];
        
        let mut saved_count = 0;
        for (reg, _) in &registers_to_save {
            if self.used_registers.contains(*reg) || 
               self.used_registers.contains(&format!("e{}", &reg[1..])) {
                code.push(format!("    push {}  ; preservar registro no volátil", reg));
                saved_count += 1;
            }
        }
        
        // Calcular alineación necesaria
        // rbp (1) + registros guardados = (1 + saved_count) * 8 bytes
        let total_bytes = (1 + saved_count) * 8;
        let alignment_needed = total_bytes % 16;
        
        if alignment_needed != 0 {
            let padding = 16 - alignment_needed;
            code.push(format!("    sub rsp, {}  ; alinear stack ({} bytes % 16 = {} bytes padding)", 
                padding, total_bytes, padding));
        }
        
        if needs_shadow_space {
            code.push("    sub rsp, 32  ; shadow space".to_string());
        }
        
        code
    }

    /// Generar epilogue optimizado
    pub fn generate_optimized_epilogue(&self, needs_shadow_space: bool) -> Vec<String> {
        let mut code = Vec::new();
        
        if needs_shadow_space {
            code.push("    add rsp, 32  ; restaurar shadow space".to_string());
        }
        
        // Calcular padding (debe coincidir con prologue)
        let registers_to_restore = [
            ("r15", "r15"),
            ("r14", "r14"),
            ("r13", "r13"),
            ("r12", "r12"),
            ("rsi", "rsi"),
            ("rdi", "rdi"),
            ("rbx", "rbx"),
        ];
        
        let mut restored_count = 0;
        for (reg, _) in &registers_to_restore {
            if self.used_registers.contains(*reg) || 
               self.used_registers.contains(&format!("e{}", &reg[1..])) {
                code.push(format!("    pop {}  ; restaurar registro no volátil", reg));
                restored_count += 1;
            }
        }
        
        // Restaurar padding si existe
        let total_bytes = (1 + restored_count) * 8;
        let alignment_needed = total_bytes % 16;
        if alignment_needed != 0 {
            let padding = 16 - alignment_needed;
            code.push(format!("    add rsp, {}  ; restaurar alineación", padding));
        }
        
        code.push("    leave  ; restaurar rbp y rsp".to_string());
        code.push("    ret".to_string());
        
        code
    }
}

