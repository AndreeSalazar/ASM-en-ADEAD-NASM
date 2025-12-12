# 🛠️ Guía de Desarrollo de ADead

**Guía general para contribuir y desarrollar el compilador ADead**

---

## 🏗️ Arquitectura del Compilador

### Estructura de Crates

```
adead-cli         → Interfaz de línea de comandos
adead-parser      → Parser (lexing + parsing)
adead-typecheck   → Type checking e inference (⏳ por implementar)
adead-borrow      → Borrow checker (⏳ por implementar)
adead-backend     → Generación de código NASM
adead-common      → Tipos y errores compartidos
```

### Flujo de Compilación

```
.ad source
    ↓
adead-cli (main.rs)
    ↓
adead-parser (parse())
    ↓ AST (Program)
    ↓
adead-typecheck (⏳) → Type checking
    ↓
adead-borrow (⏳) → Borrow checking
    ↓
adead-backend (generate())
    ↓
.asm output
    ↓
nasm → .obj
    ↓
gcc/ld → .exe
```

---

## 📝 Convenciones de Código

### Rust Style Guide

- Seguir estándares de Rust (rustfmt)
- Nombres: `snake_case` para funciones/variables
- Documentación: Usar `///` para doc comments
- Tests: Un módulo `#[cfg(test)]` por archivo

### Estructura de Archivos

```rust
// 1. Imports
use ...

// 2. Types/Structs
pub struct ...

// 3. Implementaciones
impl ... {
    ...
}

// 4. Tests
#[cfg(test)]
mod tests {
    ...
}
```

---

## 🧪 Testing

### Estrategia

1. **Unit Tests**: Cada función/método
2. **Integration Tests**: Programas completos
3. **Regression Tests**: Ejemplos que funcionan

### Ejemplo de Test

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_type_inference() {
        let expr = Expr::Number(42);
        let checker = TypeChecker::new();
        let typ = checker.infer_expr_type(&expr).unwrap();
        assert_eq!(typ, Type::Int64);
    }
}
```

---

## 🐛 Debugging

### Mensajes de Error

Mensajes claros y útiles:

```rust
Err(ADeadError::TypeError {
    message: format!(
        "Tipos incompatibles en línea {}: esperaba {}, pero encontró {}",
        line, expected, found
    )
})
```

### Logging

Usar `eprintln!` para debug (no `println!`):

```rust
eprintln!("DEBUG: Variable {} tiene tipo {:?}", name, typ);
```

---

## 📚 Documentación

### Comentarios en Código

```rust
/// Infiere el tipo de una expresión
/// 
/// # Ejemplos
/// 
/// ```adead
/// let x = 42  // Infiere int64
/// ```
pub fn infer_expr_type(&self, expr: &Expr) -> Result<Type> {
    // ...
}
```

### Documentación para Usuarios

- Actualizar documentación en `Ejemplos-Reales/documentacion/`
- Agregar ejemplos cuando se añade funcionalidad
- Mantener README actualizado

---

## 🔄 Proceso de Desarrollo

### Para Agregar una Nueva Característica

1. **Planificación**
   - Revisar `ideas2.md` o `ideas3.md`
   - Verificar dependencias
   - Estimar esfuerzo

2. **Implementación**
   - Extender AST si es necesario
   - Actualizar parser
   - Implementar lógica
   - Agregar type checking si aplica

3. **Testing**
   - Unit tests
   - Integration tests
   - Probar con ejemplos reales

4. **Documentación**
   - Actualizar docs de usuario
   - Actualizar guías de implementación
   - Ejemplos de uso

---

## ⚠️ Precauciones

### Antes de Hacer Cambios Grandes

1. ✅ Verificar que tests existentes pasan
2. ✅ Planificar cambios en documentación
3. ✅ Considerar impacto en código existente
4. ✅ Probar con ejemplos reales

### Mantener Compatibilidad

- No romper sintaxis existente
- Mantener retrocompatibilidad cuando sea posible
- Documentar cambios breaking

---

## 🚀 Getting Started

### Setup del Entorno

```bash
# Clonar repo
git clone ...
cd "ASM en ADEAD"

# Build
cargo build

# Tests
cargo test

# Ejecutar ejemplos
cargo run --release -- run Ejemplos-Reales/ejemplos/hello.ad
```

### Primer Cambio

1. Escoge una tarea de `ideas2.md` o `ideas3.md`
2. Crea una rama: `git checkout -b feature/nombre`
3. Implementa
4. Tests: `cargo test`
5. Commit: `git commit -m "Add: descripción"`
6. Push y PR

---

*Guía de desarrollo - Última actualización: Diciembre 2025*

