# Integración de Parser Zig para Expresiones Aritméticas

## Resumen

Se está integrando el parser de expresiones aritméticas de Zig para reemplazar el parser de Rust, aprovechando las fortalezas de Zig en parsing eficiente.

## Estado Actual

✅ **Completado:**
- Parser de expresiones Zig creado (`zig/src/expr_parser.zig`)
- Función FFI `parse_expr_ffi` exportada
- Wrapper Rust creado (`rust/crates/adead-parser/src/zig_expr_parser.rs`)
- Módulo agregado a `lib.rs`
- Parser recursivo para convertir resultado serializado de Zig a `Expr` de Rust

⏳ **En progreso:**
- Compilación de biblioteca Zig (problema de compatibilidad con Zig 0.16.0)
- Configuración de linking en Cargo.toml
- Integración en el parser principal de Rust

## Arquitectura

```
Expresión Aritmética (e.g., "2 + 5")
  ↓
Zig Parser (zig/src/expr_parser.zig)
  ↓ Serializa a formato: "BINOP:ADD:NUMBER:2:NUMBER:5"
  ↓
FFI Call (parse_expr_ffi)
  ↓
Rust Wrapper (zig_expr_parser.rs)
  ↓ Parseo recursivo del formato serializado
  ↓
Expr de Rust (adead_parser::Expr)
  ↓
Backend NASM (generación de código)
```

## Formato de Serialización Zig

Zig serializa las expresiones en formato simple:
- `NUMBER:42` → número
- `IDENT:variable` → identificador
- `BINOP:OP:LEFT:RIGHT` → operación binaria

Donde `LEFT` y `RIGHT` pueden ser expresiones anidadas del mismo formato.

## Próximos Pasos

1. ✅ Resolver compatibilidad de `build.zig` con Zig 0.16.0
2. Compilar biblioteca Zig (`zig build`)
3. Configurar linking en `Cargo.toml` del parser
4. Modificar `expr_parser()` para usar Zig cuando sea apropiado
5. Probar con `print 2 + 5`

## Ventajas de Usar Zig

- **Parsing más eficiente:** Zig maneja mejor el parsing de expresiones complejas
- **Precedencia correcta:** El parser Zig garantiza precedencia correcta de operadores
- **Código más mantenible:** Separación de responsabilidades (Zig parsing, Rust validación)

