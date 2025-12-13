# 🔄 Flujo Completo de Compilación ADead

## 📋 Resumen Ejecutivo

**Flujo Establecido:**
```
ADead → Zig (parsea expresiones) → Rust (seguridad) → NASM → .exe
```

## 🔍 Flujo Detallado Paso a Paso

### 1️⃣ **ADead Source (.ad)**

Archivo fuente del lenguaje ADead:
```adead
print 2 + 5
```

### 2️⃣ **Zig Parser (parsea expresiones)**

**Responsabilidad:** Parsing eficiente de expresiones aritméticas

**Archivo:** `zig/src/expr_parser.zig`

**Función FFI:** `parse_expr_ffi()`

**Proceso:**
- Recibe: `"2 + 5"` (string)
- Parsea: Crea AST Zig con precedencia correcta
- Serializa: `"BINOP:ADD:NUMBER:2:NUMBER:5"`
- Retorna: Buffer serializado vía FFI

**Ventajas:**
- ⚡ Parsing más rápido que Rust
- ✅ Precedencia de operadores garantizada
- ✅ Manejo eficiente de memoria

### 3️⃣ **Rust (seguridad de memoria)**

**Responsabilidad:** Validación, seguridad y generación de código

**Archivos:**
- `rust/crates/adead-parser/src/zig_expr_parser.rs` - Wrapper FFI
- `rust/crates/adead-parser/src/lib.rs` - Parser principal
- `rust/crates/adead-backend/src/lib.rs` - Code generator

**Proceso:**
1. **Wrapper FFI:** `parse_expr_with_zig()` llama a Zig
2. **Deserialización:** Convierte `"BINOP:ADD:NUMBER:2:NUMBER:5"` → `Expr::BinaryOp`
3. **Validación:**
   - Borrow checker (seguridad de memoria)
   - Type checking
   - Validación de seguridad
4. **Code Generation:** Genera código NASM para:
   - Evaluar expresión (`2 + 5 = 7`)
   - Convertir número a string (`7` → `"7"`)
   - Llamar a `WriteFile` (Windows API)

**Ventajas:**
- 🔒 Seguridad de memoria garantizada
- ✅ Validación exhaustiva
- ✅ Generación de código optimizado

### 4️⃣ **NASM (Assembly x86_64)**

**Responsabilidad:** Compilar assembly a objeto

**Proceso:**
- Recibe: Código NASM generado por Rust
- Compila: `nasm -f win64 output.asm -o output.obj`
- Genera: Archivo objeto `.obj`

**Características:**
- ✅ Código x86_64 optimizado
- ✅ Windows ABI compliance
- ✅ Sin dependencias externas

### 5️⃣ **Linker → Ejecutable (.exe)**

**Proceso:**
- Linker: `link.exe` o `gcc` enlaza `.obj` → `.exe`
- Genera: Ejecutable nativo Windows
- Listo para ejecutar

### 6️⃣ **Ejecución**

**Resultado:**
```
7
```

## 📊 Diagrama de Flujo

```
┌─────────────────┐
│  ADead Source   │
│   print 2 + 5   │
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  ZIG PARSER                          │
│  ┌───────────────────────────────┐  │
│  │ parse_expr_ffi("2 + 5")       │  │
│  │ → AST Zig                     │  │
│  │ → Serializa:                 │  │
│  │   "BINOP:ADD:..."            │  │
│  └───────────────────────────────┘  │
└────────┬─────────────────────────────┘
         │ FFI (Foreign Function Interface)
         ▼
┌─────────────────────────────────────┐
│  RUST (Seguridad)                    │
│  ┌───────────────────────────────┐  │
│  │ parse_expr_with_zig()         │  │
│  │ → Deserializa a Expr         │  │
│  │ → Validación (borrow checker) │  │
│  │ → Type checking              │  │
│  │ → Code Generator (NASM)      │  │
│  └───────────────────────────────┘  │
└────────┬─────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  NASM ASSEMBLY                       │
│  ┌───────────────────────────────┐  │
│  │ section .data                  │  │
│  │ section .text                  │  │
│  │   mov rax, 2                   │  │
│  │   add rax, 5                   │  │
│  │   ; Convertir a string         │  │
│  │   ; WriteFile(...)             │  │
│  └───────────────────────────────┘  │
└────────┬─────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  Object File (.obj)                  │
└────────┬─────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│  Ejecutable (.exe)                  │
└────────┬─────────────────────────────┘
         │
         ▼
      ✅ 7
```

## 🔧 Componentes Técnicos

### Zig Parser (`zig/src/expr_parser.zig`)

```zig
pub export fn parse_expr_ffi(
    input_ptr: [*:0]const u8,
    input_len: usize,
    output_buffer: [*]u8,
    output_buffer_len: usize,
) i32 {
    // Parsea expresión
    // Serializa a formato simple
    // Retorna longitud o código de error
}
```

### Rust Wrapper (`rust/crates/adead-parser/src/zig_expr_parser.rs`)

```rust
#[link(name = "adead_zig")]
extern "C" {
    fn parse_expr_ffi(...) -> c_int;
}

pub fn parse_expr_with_zig(expr_str: &str) -> Option<Expr> {
    // Llama a Zig vía FFI
    // Deserializa resultado
    // Retorna Expr de Rust
}
```

### Rust Code Generator (`rust/crates/adead-backend/src/lib.rs`)

```rust
fn generate_expr_windows(&mut self, expr: &Expr) -> Result<()> {
    // Genera código NASM para evaluar expresión
    // Maneja conversión número → string
    // Genera llamadas a WriteFile
}
```

## ✅ Ventajas del Flujo

1. **Eficiencia:** Zig parsea más rápido que Rust
2. **Seguridad:** Rust garantiza seguridad de memoria
3. **Optimización:** NASM genera código assembly optimizado
4. **Separación:** Cada lenguaje hace lo que mejor sabe
5. **Mantenibilidad:** Código claro y bien estructurado

## 🎯 Flujo Establecido

**Confirmado y Funcional:**
```
ADead → Zig (parsea expresiones) → Rust (seguridad) → NASM → .exe
```

**Estado:** ✅ **COMPLETO Y OPERATIVO**

