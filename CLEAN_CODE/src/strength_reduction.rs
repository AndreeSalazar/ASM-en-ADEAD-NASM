//! Strength Reduction - Reemplaza operaciones costosas por equivalentes más rápidas

use regex::Regex;

/// Reductor de fuerza (strength reduction)
pub struct StrengthReducer {
    mul_power2: Regex,
    div_power2: Regex,
}

impl StrengthReducer {
    pub fn new() -> Self {
        Self {
            // mul reg, 2^n → shl reg, n
            mul_power2: Regex::new(r"mul\s+(\w+),\s+(2|4|8|16|32|64|128|256|512|1024)").unwrap(),
            // div reg, 2^n → shr reg, n
            div_power2: Regex::new(r"div\s+(\w+),\s+(2|4|8|16|32|64|128|256|512|1024)").unwrap(),
        }
    }

    /// Aplica strength reduction
    pub fn reduce(&self, asm: &str) -> Result<String, String> {
        let mut result = asm.to_string();

        // Reemplazar mul por shl (multiplicación por potencia de 2)
        result = self.mul_power2.replace_all(&result, |caps: &regex::Captures| {
            let reg = &caps[1];
            let val: u64 = caps[2].parse().unwrap();
            let shift = val.trailing_zeros();
            format!("shl {}, {}", reg, shift)
        }).to_string();

        // Reemplazar div por shr (división por potencia de 2)
        result = self.div_power2.replace_all(&result, |caps: &regex::Captures| {
            let reg = &caps[1];
            let val: u64 = caps[2].parse().unwrap();
            let shift = val.trailing_zeros();
            format!("shr {}, {}", reg, shift)
        }).to_string();

        Ok(result)
    }
}

impl Default for StrengthReducer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mul_to_shl() {
        let reducer = StrengthReducer::new();
        let asm = "mul rax, 8\n";
        let result = reducer.reduce(asm).unwrap();
        assert!(result.contains("shl rax, 3"));
    }

    #[test]
    fn test_div_to_shr() {
        let reducer = StrengthReducer::new();
        let asm = "div rax, 4\n";
        let result = reducer.reduce(asm).unwrap();
        assert!(result.contains("shr rax, 2"));
    }
}

