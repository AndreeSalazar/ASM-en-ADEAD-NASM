# 📖 Ejemplo de Uso: D Language + ADead

Este documento muestra cómo usar el módulo D de metaprogramming con ADead.

## 🎯 Casos de Uso

### 1. Generación de Código ASM con Templates

El módulo D puede generar código ASM directamente usando templates:

```d
// Ejemplo: Generar instrucción MOV
mixin(GenerateMovASM!("rax", "42"));
// Genera: mov rax, 42
```

### 2. Validación en Compile-Time (CTFE)

Validar expresiones antes de ejecutar:

```d
auto expr = parseExpr("2 + 2");
if (validateExprTypes(expr)) {
    // Expresión válida, continuar
}
```

### 3. Optimización Automática

Optimizar expresiones constantes:

```d
auto expr = new BinaryOp(Number(2), Add, Number(2));
auto optimized = optimizeExpr(expr);
// Resultado: Number(4) - evaluado en compile-time
```

### 4. Integración con Rust

Desde Rust, usar las funciones D:

```rust
use adead_parser::d_ffi::parse_expr_with_d;

// Parsear y generar ASM usando D
let asm_code = parse_expr_with_d("10 + 20 * 2").unwrap();
// Genera código NASM optimizado
```

## 🔧 Pipeline Completo

```
ADead Source (.ad)
  ↓
Tree-sitter (parsing robusto)
  ↓
D Language (metaprogramming)
  ├─ CTFE: Validación en compile-time
  ├─ Templates: Generación ASM
  └─ Optimización: Evalúa constantes
  ↓
Rust (codegen final + seguridad)
  ↓
NASM → Ejecutable
```

## 💻 Ejemplo Completo

Ver `test_simple.d` para un ejemplo básico de compilación.

