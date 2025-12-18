use std::collections::HashMap;

/// Estructura para manejar metadatos de clases y vtables
pub struct ClassInfo {
    pub name: String,
    pub parent: Option<String>,
    pub fields: Vec<String>,
    pub methods: Vec<String>,
    pub vtable: Vec<String>, // Lista de métodos en orden de vtable
}

impl ClassInfo {
    pub fn new(name: String, parent: Option<String>) -> Self {
        Self {
            name,
            parent,
            fields: Vec::new(),
            methods: Vec::new(),
            vtable: Vec::new(),
        }
    }
}

/// Generador de código OOP para NASM
pub struct OOPGenerator {
    pub classes: HashMap<String, ClassInfo>,
}

impl OOPGenerator {
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
        }
    }

    /// Registrar una nueva clase
    pub fn register_class(&mut self, name: String, parent: Option<String>, fields: Vec<String>, methods: Vec<String>) {
        let mut class_info = ClassInfo::new(name.clone(), parent.clone());
        class_info.fields = fields;
        class_info.methods = methods.clone();

        // Construir VTable inicial
        // Si hay padre, copiar su VTable primero
        if let Some(parent_name) = &parent {
            if let Some(parent_info) = self.classes.get(parent_name) {
                class_info.vtable = parent_info.vtable.clone();
            }
        }

        // Agregar o sobreescribir métodos
        for method in &methods {
            if let Some(idx) = class_info.vtable.iter().position(|m| m == method) {
                // Override: ya existe en vtable (del padre), no hacemos nada (el puntero apuntará a la nueva impl)
                // Nota: En una implementación real, aquí verificaríamos firmas
            } else {
                // Nuevo método: agregar al final
                class_info.vtable.push(method.clone());
            }
        }

        self.classes.insert(name, class_info);
    }

    /// Generar sección de datos para VTables
    pub fn generate_vtables(&self) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push("; ============================================".to_string());
        lines.push("; OOP VTABLES".to_string());
        lines.push("; ============================================".to_string());

        for (name, info) in &self.classes {
            lines.push(format!("vtable_{}:", name));
            for method in &info.vtable {
                // Formato de etiqueta: ClassName_methodName
                // Si es heredado y no sobreescrito, debería apuntar a Parent_methodName
                // Por simplicidad en esta fase, asumimos que todos los métodos se generan con prefijo de la clase actual
                // si están definidos en ella, o buscamos en el padre.
                
                // TODO: Resolución correcta de nombres de métodos heredados vs sobreescritos
                // Por ahora, apuntamos a ClassName_methodName si existe en methods, o Parent_methodName si no
                
                let label = self.resolve_method_label(name, method);
                lines.push(format!("    dq {}", label));
            }
            if info.vtable.is_empty() {
                lines.push("    dq 0 ; Dummy entry for empty vtable".to_string());
            }
        }
        lines
    }

    fn resolve_method_label(&self, class_name: &str, method_name: &str) -> String {
        let info = self.classes.get(class_name).unwrap();
        if info.methods.contains(&method_name.to_string()) {
            format!("{}_{}", class_name, method_name)
        } else if let Some(parent) = &info.parent {
            self.resolve_method_label(parent, method_name)
        } else {
            // Should not happen if registered correctly
            format!("{}_{}", class_name, method_name)
        }
    }
    
    /// Obtener offset de un campo
    pub fn get_field_offset(&self, class_name: &str, field_name: &str) -> Option<i64> {
        // Layout: [vtable_ptr (8)] [field0 (8)] [field1 (8)] ...
        // Offset base = 8
        
        // Necesitamos recorrer toda la jerarquía para contar campos heredados
        let mut current_offset = 8;
        
        // Recorrer desde la raíz hasta la clase actual
        let hierarchy = self.get_class_hierarchy(class_name);
        
        for c_name in hierarchy {
            let info = self.classes.get(&c_name).unwrap();
            for f in &info.fields {
                if f == field_name && c_name == class_name {
                     return Some(current_offset);
                }
                // Si el campo está en un padre, también lo encontramos (shadowing no soportado aun)
                if f == field_name {
                    return Some(current_offset);
                }
                current_offset += 8;
            }
        }
        
        None
    }
    
    fn get_class_hierarchy(&self, class_name: &str) -> Vec<String> {
        let mut hierarchy = Vec::new();
        let mut current = Some(class_name.to_string());
        
        while let Some(name) = current {
            hierarchy.insert(0, name.clone()); // Insertar al inicio para tener orden Padre -> Hijo
            if let Some(info) = self.classes.get(&name) {
                current = info.parent.clone();
            } else {
                break;
            }
        }
        
        hierarchy
    }
    
    /// Obtener offset de un método en la vtable
    pub fn get_method_vtable_offset(&self, class_name: &str, method_name: &str) -> Option<i64> {
        if let Some(info) = self.classes.get(class_name) {
            if let Some(idx) = info.vtable.iter().position(|m| m == method_name) {
                return Some((idx as i64) * 8);
            }
        }
        None
    }
}
