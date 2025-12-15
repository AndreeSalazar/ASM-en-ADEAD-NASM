//! Limpieza principal de ASM

use crate::peephole::PeepholeOptimizer;
use crate::dead_code::DeadCodeEliminator;
use crate::constant_propagation::ConstantPropagator;
use crate::strength_reduction::StrengthReducer;
use crate::data_flow::DataFlowAnalyzer;
use crate::objconv_integration::ObjconvOptimizer;

/// Nivel de optimización
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OptimizationLevel {
    /// Nivel 1: Básico - Regex simple, peephole local, dead code básico
    Basic = 1,
    /// Nivel 2: Avanzado - Peephole ampliado, constant propagation, strength reduction
    Advanced = 2,
    /// Nivel 3: Extremo - Todo lo anterior + data flow analysis + objconv (si disponible)
    Extreme = 3,
}

/// Resultado de la limpieza
#[derive(Debug)]
pub struct CleanResult {
    pub original_lines: usize,
    pub cleaned_lines: usize,
    pub removed_lines: usize,
    pub reduction_percent: f64,
}

/// Limpiador principal de ASM
pub struct AsmCleaner {
    level: OptimizationLevel,
    peephole: PeepholeOptimizer,
    dead_code: DeadCodeEliminator,
    constant_prop: ConstantPropagator,
    strength_reducer: StrengthReducer,
    data_flow: DataFlowAnalyzer,
    objconv: ObjconvOptimizer,
}

impl AsmCleaner {
    /// Crea un nuevo limpiador de ASM con nivel básico
    pub fn new() -> Self {
        Self::with_level(OptimizationLevel::Basic)
    }

    /// Crea un nuevo limpiador con nivel de optimización específico
    pub fn with_level(level: OptimizationLevel) -> Self {
        Self {
            level,
            peephole: PeepholeOptimizer::new(),
            dead_code: DeadCodeEliminator::new(),
            constant_prop: ConstantPropagator::new(),
            strength_reducer: StrengthReducer::new(),
            data_flow: DataFlowAnalyzer::new(),
            objconv: ObjconvOptimizer::new(),
        }
    }

    /// Limpia ASM sucio y retorna ASM virgen puro según el nivel configurado
    pub fn clean(&self, asm: &str) -> Result<String, String> {
        let original_lines = asm.lines().count();
        let mut cleaned = asm.to_string();

        // Paso 1: Limpieza básica (siempre)
        cleaned = self.clean_basic(&cleaned);

        // Paso 2: Optimizaciones peephole (siempre)
        cleaned = self.peephole.optimize(&cleaned)?;

        // Paso 3: Eliminación de dead code (siempre)
        cleaned = self.dead_code.eliminate(&cleaned)?;

        // Nivel Avanzado y Extremo
        if self.level >= OptimizationLevel::Advanced {
            // Paso 4: Constant propagation
            cleaned = self.constant_prop.propagate(&cleaned)?;

            // Paso 5: Strength reduction
            cleaned = self.strength_reducer.reduce(&cleaned)?;

            // Paso 6: Peephole ampliado (ventanas más grandes)
            cleaned = self.peephole.optimize_extended(&cleaned)?;
        }

        // Nivel Extremo
        if self.level >= OptimizationLevel::Extreme {
            // Paso 7: Data flow analysis
            cleaned = self.data_flow.analyze(&cleaned)?;

            // Paso 8: Objconv (si está disponible, opcional)
            if let Ok(objconv_result) = self.objconv.optimize(&cleaned) {
                cleaned = objconv_result;
            }
        }

        // Paso final: Limpieza final (siempre)
        cleaned = self.clean_final(&cleaned);

        let cleaned_lines = cleaned.lines().count();
        let _removed_lines = original_lines.saturating_sub(cleaned_lines);
        let _reduction_percent = if original_lines > 0 {
            (_removed_lines as f64 / original_lines as f64) * 100.0
        } else {
            0.0
        };

        Ok(cleaned)
    }

    /// Limpieza básica: espacios, comentarios vacíos
    fn clean_basic(&self, asm: &str) -> String {
        let mut cleaned = asm.to_string();

        // Eliminar comentarios vacíos (solo `;` o `; `)
        cleaned = cleaned.replace(";\n", "\n");
        cleaned = cleaned.replace("; \n", "\n");

        // Normalizar espacios múltiples
        while cleaned.contains("  ") {
            cleaned = cleaned.replace("  ", " ");
        }

        // Normalizar tabs a espacios (opcional, para consistencia)
        cleaned = cleaned.replace("\t", "    ");

        cleaned
    }

    /// Limpieza final: espacios múltiples, líneas vacías excesivas
    fn clean_final(&self, asm: &str) -> String {
        let cleaned = asm.to_string();

        // Eliminar líneas vacías múltiples (máximo 1 línea vacía)
        let lines: Vec<&str> = cleaned.lines().collect();
        let mut result = Vec::new();
        let mut last_was_empty = false;

        for line in lines {
            let is_empty = line.trim().is_empty();
            if !is_empty || !last_was_empty {
                result.push(line);
            }
            last_was_empty = is_empty;
        }

        result.join("\n")
    }
}

impl Default for AsmCleaner {
    fn default() -> Self {
        Self::new()
    }
}

