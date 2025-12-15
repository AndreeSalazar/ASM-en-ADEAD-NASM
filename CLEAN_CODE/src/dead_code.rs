//! Eliminación de Dead Code - Labels no referenciados, código inalcanzable

use std::collections::HashSet;

/// Eliminador de Dead Code
pub struct DeadCodeEliminator {
    // Configuración
    remove_unused_labels: bool,
    remove_unreachable_code: bool,
}

impl DeadCodeEliminator {
    pub fn new() -> Self {
        Self {
            remove_unused_labels: true,
            remove_unreachable_code: true,
        }
    }

    /// Elimina dead code del ASM
    pub fn eliminate(&self, asm: &str) -> Result<String, String> {
        let mut cleaned = asm.to_string();

        if self.remove_unused_labels {
            cleaned = self.remove_unused_labels(&cleaned)?;
        }

        if self.remove_unreachable_code {
            cleaned = self.remove_unreachable_code(&cleaned)?;
        }

        Ok(cleaned)
    }

    /// Elimina labels que no son referenciados
    fn remove_unused_labels(&self, asm: &str) -> Result<String, String> {
        let lines: Vec<&str> = asm.lines().collect();
        
        // Paso 1: Encontrar todos los labels definidos
        let mut labels_defined: HashSet<String> = HashSet::new();
        for line in &lines {
            if let Some(label) = self.extract_label_definition(line) {
                labels_defined.insert(label);
            }
        }

        // Paso 2: Encontrar todos los labels referenciados
        let mut labels_referenced: HashSet<String> = HashSet::new();
        for line in &lines {
            let refs = self.extract_label_references(line);
            labels_referenced.extend(refs);
        }

        // Paso 3: Labels no usados = definidos pero no referenciados
        let unused_labels: HashSet<String> = labels_defined
            .difference(&labels_referenced)
            .cloned()
            .collect();

        // Paso 4: Eliminar líneas con labels no usados
        let mut result = Vec::new();
        for line in &lines {
            if let Some(label) = self.extract_label_definition(line) {
                if unused_labels.contains(&label) {
                    // Saltar esta línea (label no usado)
                    continue;
                }
            }
            result.push(*line);
        }

        Ok(result.join("\n"))
    }

    /// Elimina código inalcanzable (después de ret/jmp incondicional)
    fn remove_unreachable_code(&self, asm: &str) -> Result<String, String> {
        let lines: Vec<&str> = asm.lines().collect();
        let mut result = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();
            result.push(lines[i]);

            // Si encontramos ret o jmp incondicional, el código siguiente es inalcanzable
            // hasta encontrar un label
            if line.starts_with("ret") || 
               (line.starts_with("jmp") && !line.contains(";")) {
                i += 1;
                
                // Saltar código hasta encontrar un label
                while i < lines.len() {
                    let next_line = lines[i].trim();
                    if next_line.ends_with(':') {
                        // Encontramos un label, el código después es alcanzable
                        break;
                    }
                    // Saltar esta línea (inalcanzable)
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        Ok(result.join("\n"))
    }

    /// Extrae label de definición (ej: "label1:" → "label1")
    fn extract_label_definition(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();
        if trimmed.ends_with(':') && !trimmed.starts_with(';') {
            let label = trimmed.trim_end_matches(':').trim().to_string();
            if !label.is_empty() {
                return Some(label);
            }
        }
        None
    }

    /// Extrae todas las referencias a labels en una línea
    fn extract_label_references(&self, line: &str) -> Vec<String> {
        let mut refs = Vec::new();
        let trimmed = line.trim();

        // Buscar jmp label, call label, etc.
        // Patrón simple: palabra clave seguida de label
        let keywords = ["jmp", "call", "je", "jne", "jl", "jg", "jle", "jge"];
        
        for keyword in &keywords {
            if let Some(pos) = trimmed.find(keyword) {
                let after_keyword = &trimmed[pos + keyword.len()..];
                let parts: Vec<&str> = after_keyword.split_whitespace().collect();
                if let Some(first_part) = parts.first() {
                    // Remover coma si existe
                    let label = first_part.trim_end_matches(',');
                    if !label.is_empty() && !label.chars().next().unwrap().is_numeric() {
                        refs.push(label.to_string());
                    }
                }
            }
        }

        refs
    }
}

impl Default for DeadCodeEliminator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_unused_labels() {
        let eliminator = DeadCodeEliminator::new();
        let asm = "main:\nmov rax, 5\nunused_label:\nnop\nret\n";
        let result = eliminator.eliminate(asm).unwrap();
        assert!(!result.contains("unused_label:"));
        assert!(result.contains("main:"));
    }

    #[test]
    fn test_extract_label_definition() {
        let eliminator = DeadCodeEliminator::new();
        assert_eq!(eliminator.extract_label_definition("main:"), Some("main".to_string()));
        assert_eq!(eliminator.extract_label_definition("  label1:  "), Some("label1".to_string()));
        assert_eq!(eliminator.extract_label_definition("; comment"), None);
    }
}

