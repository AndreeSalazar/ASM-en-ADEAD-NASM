use thiserror::Error;

/// Sistema de tipos robusto compatible con NASM x86-64
/// Cada tipo mapea correctamente a representaciones en ASM
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    // ========== ENTEROS CON SIGNO (compatible NASM) ==========
    Int8,      // 8 bits: AL, BL, CL, DL (NASM: mov al, value)
    Int16,     // 16 bits: AX, BX, CX, DX (NASM: mov ax, value)
    Int32,     // 32 bits: EAX, EBX, ECX, EDX (NASM: mov eax, value)
    Int64,     // 64 bits: RAX, RBX, RCX, RDX (NASM: mov rax, value) - estándar x86-64
    
    // ========== ENTEROS SIN SIGNO (compatible NASM) ==========
    UInt8,     // 8 bits unsigned: igual que Int8 en ASM
    UInt16,    // 16 bits unsigned: igual que Int16 en ASM
    UInt32,    // 32 bits unsigned: igual que Int32 en ASM
    UInt64,    // 64 bits unsigned: igual que Int64 en ASM
    
    // ========== PUNTO FLOTANTE (compatible NASM) ==========
    Float32,   // 32 bits: XMM0-XMM15 (NASM: movss xmm0, value)
    Float64,   // 64 bits: XMM0-XMM15 (NASM: movsd xmm0, value)
    
    // ========== OTROS PRIMITIVOS ==========
    Bool,      // 8 bits: representado como byte (0=false, 1=true)
    Char,      // 8 bits: carácter ASCII/Unicode (representado como byte)
    
    // ========== TIPOS COMPUESTOS ==========
    String,    // Puntero + longitud (8 bytes cada uno en x86-64)
    Array {
        element_type: Box<Type>,
        size: Option<usize>,  // Some(n) = tamaño fijo, None = dinámico
    },
    Tuple(Vec<Type>),
    
    // ========== TIPOS OPCIONALES Y ERRORES (preparación O0.4) ==========
    Option(Box<Type>),
    Result {
        ok: Box<Type>,
        err: Box<Type>,
    },
    
    // ========== REFERENCIAS (preparación O0.2 - Ownership) ==========
    Ref {
        inner: Box<Type>,
        mutable: bool,  // false = &T, true = &mut T
    },
    
    // ========== OTROS ==========
    Void,      // Sin valor (NASM: no hay registro, solo ret)
    Never,     // Tipo divergente (funciones que nunca retornan)
    
    // ========== INFERENCIA ==========
    Unknown,   // Para type inference durante análisis
}

#[derive(Debug, Error)]
pub enum ADeadError {
    #[error("Parse error at {line}:{col}: {message}")]
    ParseError { line: usize, col: usize, message: String },

    #[error("Type error: {message}")]
    TypeError { message: String },

    #[error("Runtime error: {message}")]
    RuntimeError { message: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Errores estándar para usar en programas ADead
/// Estos son tipos que los usuarios pueden usar con Result<T, E>
#[derive(Debug, Clone, PartialEq)]
pub enum StdError {
    FileError {
        path: String,
        message: String,
    },
    ParseError {
        message: String,
    },
    MathError {
        operation: String,
        message: String,
    },
    ValueError {
        message: String,
    },
    IOError {
        message: String,
    },
}

impl StdError {
    /// Convertir error a string para imprimir
    pub fn to_string(&self) -> String {
        match self {
            StdError::FileError { path, message } => {
                format!("FileError: {} - {}", path, message)
            }
            StdError::ParseError { message } => {
                format!("ParseError: {}", message)
            }
            StdError::MathError { operation, message } => {
                format!("MathError in {}: {}", operation, message)
            }
            StdError::ValueError { message } => {
                format!("ValueError: {}", message)
            }
            StdError::IOError { message } => {
                format!("IOError: {}", message)
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, ADeadError>;

impl Type {
    /// Obtener tamaño del tipo en bytes (para NASM/ASM)
    /// Respeta las convenciones de x86-64
    pub fn size_bytes(&self) -> usize {
        match self {
            // Primitivos con tamaño fijo
            Type::Int8 | Type::UInt8 | Type::Bool | Type::Char => 1,
            Type::Int16 | Type::UInt16 => 2,
            Type::Int32 | Type::UInt32 | Type::Float32 => 4,
            Type::Int64 | Type::UInt64 | Type::Float64 => 8,
            
            // String: puntero (8 bytes) + longitud (8 bytes) = 16 bytes en x86-64
            Type::String => 16,
            
            // Arrays: tamaño del elemento * cantidad
            Type::Array { element_type, size } => {
                let elem_size = element_type.size_bytes();
                match size {
                    Some(n) => elem_size * n,
                    None => 16, // Array dinámico: puntero (8) + capacidad (8)
                }
            }
            
            // Tuples: suma de tamaños de elementos (alineado a 8 bytes)
            Type::Tuple(types) => {
                types.iter()
                    .map(|t| t.size_bytes())
                    .sum::<usize>()
                    .max(8) // Mínimo 8 bytes por alineación
            }
            
            // Option: 1 byte tag + tamaño del tipo interno
            Type::Option(inner) => inner.size_bytes() + 1,
            
            // Result: 1 byte tag + max(ok, err) tamaño
            Type::Result { ok, err } => {
                ok.size_bytes().max(err.size_bytes()) + 1
            }
            
            // Referencias: siempre 8 bytes (puntero en x86-64)
            Type::Ref { .. } => 8,
            
            // Void/Never: 0 bytes (no hay representación)
            Type::Void | Type::Never => 0,
            Type::Unknown => 8, // Asumir 64 bits por defecto
        }
    }
    
    /// Obtener el tamaño de alineación en bytes (para NASM stack alignment)
    /// x86-64 requiere alineación de 16 bytes para la stack
    pub fn align_bytes(&self) -> usize {
        let size = self.size_bytes();
        // Alinear a múltiplo de 8 (máximo necesario en x86-64)
        if size <= 1 { 1 }
        else if size <= 2 { 2 }
        else if size <= 4 { 4 }
        else if size <= 8 { 8 }
        else { 16 } // Para tipos mayores, alinear a 16 bytes (requisito x86-64)
    }
    
    /// Obtener el registro NASM recomendado para este tipo
    /// Retorna el nombre del registro y el tamaño de operación
    pub fn nasm_register_hint(&self) -> (&'static str, &'static str) {
        match self {
            Type::Int8 | Type::UInt8 | Type::Bool | Type::Char => ("al", "byte"),
            Type::Int16 | Type::UInt16 => ("ax", "word"),
            Type::Int32 | Type::UInt32 => ("eax", "dword"),
            Type::Int64 | Type::UInt64 => ("rax", "qword"),
            Type::Float32 => ("xmm0", "dword"),
            Type::Float64 => ("xmm0", "qword"),
            Type::String => ("rax", "qword"), // Puntero (primer elemento)
            Type::Ref { .. } => ("rax", "qword"), // Punteros son 64 bits
            Type::Array { .. } => ("rax", "qword"), // Puntero al array
            Type::Tuple(_) => ("rax", "qword"), // Puntero a la tupla
            Type::Option(_) | Type::Result { .. } => ("rax", "qword"), // Tagged union
            Type::Void | Type::Never => ("", ""), // Sin registro
            Type::Unknown => ("rax", "qword"), // Asumir 64 bits
        }
    }
    
    /// Obtener la directiva NASM para declarar este tipo en .data
    pub fn nasm_declaration(&self, label: &str) -> String {
        match self {
            Type::Int8 | Type::UInt8 | Type::Bool | Type::Char => {
                format!("{}: db 0", label)
            }
            Type::Int16 | Type::UInt16 => {
                format!("{}: dw 0", label)
            }
            Type::Int32 | Type::UInt32 | Type::Float32 => {
                format!("{}: dd 0", label)
            }
            Type::Int64 | Type::UInt64 | Type::Float64 => {
                format!("{}: dq 0", label)
            }
            Type::String => {
                format!("{}: dq 0  ; string pointer\n    dq 0  ; string length", label)
            }
            Type::Array { element_type, size } => {
                let elem_decl = match element_type.as_ref() {
                    Type::Int8 | Type::UInt8 | Type::Bool | Type::Char => "db",
                    Type::Int16 | Type::UInt16 => "dw",
                    Type::Int32 | Type::UInt32 | Type::Float32 => "dd",
                    Type::Int64 | Type::UInt64 | Type::Float64 => "dq",
                    _ => "dq", // Por defecto, punteros
                };
                match size {
                    Some(n) => format!("{}: times {} {} 0", label, n, elem_decl),
                    None => format!("{}: dq 0  ; array pointer\n    dq 0  ; array capacity", label),
                }
            }
            Type::Ref { .. } => {
                format!("{}: dq 0  ; reference pointer", label)
            }
            _ => format!("{}: dq 0  ; default (8 bytes)", label),
        }
    }
    
    /// Verificar si un tipo es Copy (se puede copiar, no se mueve)
    /// En NASM, los primitivos se copian directamente
    pub fn is_copy(&self) -> bool {
        match self {
            // Todos los primitivos son Copy
            Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64 |
            Type::UInt8 | Type::UInt16 | Type::UInt32 | Type::UInt64 |
            Type::Float32 | Type::Float64 |
            Type::Bool | Type::Char => true,
            // Referencias son Copy (la referencia misma, no lo que apunta)
            Type::Ref { .. } => true,
            // Tipos compuestos generalmente no son Copy (se mueven)
            _ => false,
        }
    }
    
    /// Verificar si un tipo es Sized (tamaño conocido en compile-time)
    /// Importante para NASM: arrays de tamaño fijo vs dinámicos
    pub fn is_sized(&self) -> bool {
        match self {
            Type::Array { size: Some(_), .. } => true,
            Type::Array { size: None, .. } => false,  // Array dinámico
            Type::String => false,  // String es dinámico (heap)
            _ => true,
        }
    }
    
    /// Verificar si un tipo es numérico (enteros o flotantes)
    /// Útil para operaciones aritméticas en NASM
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64 |
            Type::UInt8 | Type::UInt16 | Type::UInt32 | Type::UInt64 |
            Type::Float32 | Type::Float64
        )
    }
    
    /// Verificar si un tipo es entero (no flotante)
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Type::Int8 | Type::Int16 | Type::Int32 | Type::Int64 |
            Type::UInt8 | Type::UInt16 | Type::UInt32 | Type::UInt64
        )
    }
    
    /// Verificar si un tipo es flotante
    pub fn is_float(&self) -> bool {
        matches!(self, Type::Float32 | Type::Float64)
    }
    
    /// Convertir a string legible para el usuario
    pub fn to_string(&self) -> String {
        match self {
            // Primitivos
            Type::Int8 => "int8".to_string(),
            Type::Int16 => "int16".to_string(),
            Type::Int32 => "int32".to_string(),
            Type::Int64 => "int64".to_string(),
            Type::UInt8 => "uint8".to_string(),
            Type::UInt16 => "uint16".to_string(),
            Type::UInt32 => "uint32".to_string(),
            Type::UInt64 => "uint64".to_string(),
            Type::Float32 => "float32".to_string(),
            Type::Float64 => "float64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Char => "char".to_string(),
            Type::String => "string".to_string(),
            Type::Void => "void".to_string(),
            Type::Never => "!".to_string(),
            Type::Unknown => "_".to_string(),
            
            // Tipos compuestos
            Type::Array { element_type, size } => {
                let size_str = match size {
                    Some(n) => format!(", {}", n),
                    None => "".to_string(),
                };
                format!("Array<{}{}>", element_type.to_string(), size_str)
            }
            Type::Tuple(types) => {
                let types_str: Vec<String> = types.iter()
                    .map(|t| t.to_string())
                    .collect();
                format!("({})", types_str.join(", "))
            }
            
            // Option/Result
            Type::Option(inner) => format!("Option<{}>", inner.to_string()),
            Type::Result { ok, err } => {
                format!("Result<{}, {}>", ok.to_string(), err.to_string())
            }
            
            // Referencias
            Type::Ref { inner, mutable } => {
                if *mutable {
                    format!("&mut {}", inner.to_string())
                } else {
                    format!("&{}", inner.to_string())
                }
            }
        }
    }
    
    /// Tipo por defecto para literales numéricos enteros (int64 - estándar x86-64)
    pub fn default_int() -> Self {
        Type::Int64
    }
    
    /// Tipo por defecto para literales flotantes (float64 - doble precisión)
    pub fn default_float() -> Self {
        Type::Float64
    }
}

