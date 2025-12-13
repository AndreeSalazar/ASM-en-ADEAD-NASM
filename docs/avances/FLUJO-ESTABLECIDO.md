# ✅ Flujo Establecido: ADead → Zig → Rust → NASM → .exe

## 🎯 Flujo Confirmado

**Flujo Principal:**
```
ADead → Zig (parsea expresiones) → Rust (seguridad) → NASM → .exe
```

## 📋 Componentes del Flujo

### 1. ADead Source
- Archivo fuente `.ad`
- Ejemplo: `print 2 + 5`

### 2. Zig Parser
- **Archivo:** `zig/src/expr_parser.zig`
- **Función:** `parse_expr_ffi()`
- **Responsabilidad:** Parsear expresiones aritméticas eficientemente
- **Output:** String serializado: `"BINOP:ADD:NUMBER:2:NUMBER:5"`

### 3. Rust (Seguridad)
- **Archivos:**
  - `rust/crates/adead-parser/src/zig_expr_parser.rs` - Wrapper FFI
  - `rust/crates/adead-parser/src/lib.rs` - Parser principal
  - `rust/crates/adead-backend/src/lib.rs` - Code generator
- **Responsabilidades:**
  - Deserializar resultado de Zig
  - Validación de memoria (borrow checker)
  - Type checking
  - Generación de código NASM

### 4. NASM
- Compila assembly x86_64
- Genera `.obj`

### 5. Ejecutable
- Linker genera `.exe`
- Ejecución produce output

## ✅ Estado

**COMPLETO Y OPERATIVO**

- ✅ Zig parser compilado y funcional
- ✅ FFI establecido correctamente
- ✅ Rust wrapper implementado
- ✅ Code generator funcionando
- ✅ Flujo completo documentado

## 📚 Documentación

- [FLUJO-COMPLETO.md](../FLUJO-COMPLETO.md) - Documentación detallada
- [README.md](../../README.md) - Resumen del proyecto

