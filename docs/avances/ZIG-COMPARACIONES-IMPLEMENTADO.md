# ✅ Parsing de Comparaciones Movido a Zig

## 🎯 Objetivo

Mover el parsing de operadores de comparación (`<`, `<=`, `>`, `>=`, `==`, `!=`) de Rust a Zig, siguiendo el patrón establecido:
- **Zig:** Parsea expresiones y comparaciones eficientemente
- **Rust:** Valida, asegura seguridad de memoria y genera código NASM

## ✅ Estado Actual

### Zig (`zig/src/expr_parser.zig`)
**✅ COMPLETO** - Ya soporta todos los operadores de comparación:
- `==` (Eq)
- `!=` (Ne)
- `<` (Lt)
- `<=` (Le)
- `>` (Gt)
- `>=` (Ge)

**Implementación:** Los operadores están en `BinOp` enum y se parsean correctamente en `parseExpression()` con la precedencia adecuada.

### Rust Wrapper (`rust/crates/adead-parser/src/zig_expr_parser.rs`)
**✅ COMPLETO** - Ya deserializa comparaciones:
- Soporta todos los operadores: `EQ`, `NE`, `LT`, `LE`, `GT`, `GE`
- Conversión correcta a `BinOp` de Rust

### Parser Rust (`rust/crates/adead-parser/src/lib.rs`)
**🔄 EN PROGRESO** - Modificación de `while_stmt`:

**Cambios Aplicados:**
- `while_stmt` ahora captura la condición como string
- Intenta usar Zig primero para parsear la condición
- Fallback a parser Rust si Zig falla

**Código:**
```rust
let while_stmt = just("while")
    .padded()
    .ignore_then(
        none_of("{")
            .repeated()
            .at_least(1)
            .collect::<String>()
            .then_ignore(just("{").padded())
            .try_map({
                let expr_clone = expr.clone();
                move |condition_str, span| {
                    // Intentar parsear con Zig primero
                    let trimmed_cond = condition_str.trim();
                    if let Some(zig_expr) = zig_expr_parser::parse_expr_with_zig(trimmed_cond) {
                        Ok(zig_expr)
                    } else {
                        // Fallback a parser Rust
                        match expr_clone.clone().parse(trimmed_cond) {
                            Ok(parsed_expr) => Ok(parsed_expr),
                            Err(e) => Err(Simple::custom(span, format!("Parse error: {:?}", e)))
                        }
                    }
                }
            })
    )
    .then(stmt.clone().repeated().then_ignore(just("}").padded()))
    .map(|(condition, body)| Stmt::While { condition, body });
```

## ⚠️ Problema Actual

**Errores de compilación:** Problemas de lifetime con `expr.clone()` dentro del closure. 

**Errores:**
- `E0373`: closure may outlive the current function
- `E0505`: cannot move out of `expr` because it is borrowed

**Solución pendiente:** Ajustar el manejo de lifetime del parser `expr` dentro del closure.

## 🎯 Flujo Final (Cuando esté completo)

```
while suma < 1000000000 {
    ...
}
```

1. **Parser Rust** captura: `"suma < 1000000000"`
2. **Zig parsea:** `parse_expr_with_zig("suma < 1000000000")`
   - Serializa: `"BINOP:LT:IDENT:suma:NUMBER:1000000000"`
3. **Rust recibe:** Deserializa a `Expr::BinaryOp { op: Lt, left: Ident("suma"), right: Number(1000000000) }`
4. **Rust valida:** Borrow checker, type checking
5. **Rust genera NASM:** Código para comparar y saltar condicionalmente

## ✅ Componentes Listos

1. ✅ Zig parser - Soporta comparaciones
2. ✅ Zig FFI - Exporta correctamente
3. ✅ Rust wrapper - Deserializa comparaciones
4. 🔄 Parser Rust - Modificación en progreso (problemas de lifetime)

## 📝 Próximos Pasos

1. Resolver problemas de lifetime en `while_stmt`
2. Probar con `1_billón.ad`
3. Verificar que funciona correctamente
4. Documentar el flujo completo

---

**Estado:** 🔄 **EN PROGRESO** - Zig listo, Rust wrapper listo, falta ajustar parser Rust

