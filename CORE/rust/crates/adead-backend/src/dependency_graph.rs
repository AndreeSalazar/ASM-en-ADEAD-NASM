// Dependency Graph para Dead Code Elimination
// Rastrea qué funciones del runtime se usan y sus dependencias

use std::collections::{HashMap, HashSet};

pub struct DependencyGraph {
    used_functions: HashSet<String>,
    dependencies: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        let mut deps = HashMap::new();
        
        // ============================================
        // DEPENDENCIAS DE ARRAYS
        // ============================================
        
        // array_new: función base, no tiene dependencias
        deps.insert("array_new".to_string(), vec![]);
        
        // array_from_values: necesita array_new
        deps.insert("array_from_values".to_string(), vec!["array_new".to_string()]);
        
        // array_get: necesita panic_out_of_bounds y panic_null_pointer
        deps.insert("array_get".to_string(), vec!["panic_out_of_bounds".to_string(), "panic_null_pointer".to_string()]);
        
        // array_set: necesita panic_out_of_bounds y panic_null_pointer
        deps.insert("array_set".to_string(), vec!["panic_out_of_bounds".to_string(), "panic_null_pointer".to_string()]);
        
        // array_len: no tiene dependencias
        deps.insert("array_len".to_string(), vec![]);
        
        // array_append: necesita array_new (para realloc) y panic_null_pointer
        deps.insert("array_append".to_string(), vec!["panic_null_pointer".to_string()]);
        
        // array_pop: necesita panic_out_of_bounds y panic_null_pointer
        deps.insert("array_pop".to_string(), vec!["panic_out_of_bounds".to_string(), "panic_null_pointer".to_string()]);
        
        // array_insert: necesita array_append (para realloc) y panic_null_pointer
        deps.insert("array_insert".to_string(), vec!["array_append".to_string(), "panic_null_pointer".to_string()]);
        
        // array_remove: necesita panic_out_of_bounds y panic_null_pointer
        deps.insert("array_remove".to_string(), vec!["panic_out_of_bounds".to_string(), "panic_null_pointer".to_string()]);
        
        // array_index: necesita panic_null_pointer
        deps.insert("array_index".to_string(), vec!["panic_null_pointer".to_string()]);
        
        // array_count: necesita panic_null_pointer
        deps.insert("array_count".to_string(), vec!["panic_null_pointer".to_string()]);
        
        // array_sort: necesita panic_null_pointer
        deps.insert("array_sort".to_string(), vec!["panic_null_pointer".to_string()]);
        
        // array_reverse: necesita panic_null_pointer
        deps.insert("array_reverse".to_string(), vec!["panic_null_pointer".to_string()]);
        
        // array_free: no tiene dependencias
        deps.insert("array_free".to_string(), vec![]);
        
        // ============================================
        // DEPENDENCIAS DE STRINGS
        // ============================================
        
        // string_new: no tiene dependencias
        deps.insert("string_new".to_string(), vec![]);
        
        // string_from_literal: no tiene dependencias
        deps.insert("string_from_literal".to_string(), vec![]);
        
        // string_len: necesita panic_null_pointer
        deps.insert("string_len".to_string(), vec!["panic_null_pointer".to_string()]);
        
        // string_concat: necesita string_from_literal y panic_null_pointer
        deps.insert("string_concat".to_string(), vec!["string_from_literal".to_string(), "panic_null_pointer".to_string()]);
        
        // string_slice: necesita panic_null_pointer y panic_out_of_bounds
        deps.insert("string_slice".to_string(), vec!["panic_null_pointer".to_string(), "panic_out_of_bounds".to_string()]);
        
        // string_upper: necesita panic_null_pointer
        deps.insert("string_upper".to_string(), vec!["panic_null_pointer".to_string()]);
        
        // string_lower: necesita panic_null_pointer
        deps.insert("string_lower".to_string(), vec!["panic_null_pointer".to_string()]);
        
        // string_free: no tiene dependencias
        deps.insert("string_free".to_string(), vec![]);
        
        // ============================================
        // DEPENDENCIAS DEL SISTEMA DE PANIC
        // ============================================
        
        // panic_out_of_bounds: necesita WriteFile y ExitProcess (siempre se incluye si se usa panic)
        deps.insert("panic_out_of_bounds".to_string(), vec![]);
        
        // panic_null_pointer: necesita WriteFile y ExitProcess (siempre se incluye si se usa panic)
        deps.insert("panic_null_pointer".to_string(), vec![]);
        
        // ============================================
        // DEPENDENCIAS DE FUNCIONES ESPECIALES
        // ============================================
        
        // int_to_str_runtime: función inline para convertir int a string
        deps.insert("int_to_str_runtime".to_string(), vec![]);
        
        Self {
            used_functions: HashSet::new(),
            dependencies: deps,
        }
    }
    
    /// Marcar una función como usada y marcar recursivamente sus dependencias
    pub fn mark_used(&mut self, func: &str) {
        if self.used_functions.contains(func) {
            return; // Ya marcada, evitar recursión infinita
        }
        
        self.used_functions.insert(func.to_string());
        
        // Marcar dependencias recursivamente
        // Clonar las dependencias para evitar problemas de borrowing
        if let Some(deps) = self.dependencies.get(func) {
            let deps_clone = deps.clone();
            for dep in &deps_clone {
                self.mark_used(dep);
            }
        }
    }
    
    /// Verificar si una función debe generarse
    pub fn should_generate(&self, func: &str) -> bool {
        self.used_functions.contains(func)
    }
    
    /// Obtener todas las funciones marcadas como usadas
    pub fn get_used_functions(&self) -> &HashSet<String> {
        &self.used_functions
    }
    
    /// Verificar si se usa alguna función de arrays
    pub fn uses_arrays(&self) -> bool {
        self.used_functions.iter().any(|f| f.starts_with("array_"))
    }
    
    /// Verificar si se usa alguna función de strings
    pub fn uses_strings(&self) -> bool {
        self.used_functions.iter().any(|f| f.starts_with("string_"))
    }
    
    /// Verificar si se usa el sistema de panic
    pub fn uses_panic(&self) -> bool {
        self.used_functions.contains("panic_out_of_bounds") || 
        self.used_functions.contains("panic_null_pointer")
    }
    
    /// Verificar si se usa alguna función específica (helper para evitar borrowing issues)
    pub fn uses_function(&self, func: &str) -> bool {
        self.used_functions.contains(func)
    }
}
