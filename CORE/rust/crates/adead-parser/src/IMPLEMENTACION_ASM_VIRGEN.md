# 🔥 Implementación: ASM Virgen 100% - Guía Técnica

## 🎯 Objetivo Final

Generar ASM virgen y puro SIEMPRE, aprovechando los 5 componentes al máximo.

## 📋 Checklist de Implementación

### ✅ FASE 1: D Language CTFE (CRÍTICO)

#### Tarea 1.1: Implementar evaluación de constantes

**Archivo:** `CORE/d/src/adead_ctfe.d`

```d
/**
 * Evalúa expresión constante en compile-time
 * Ejemplo: "5 + 3" → "8"
 */
pure string evaluateConstExpr(string expr) {
    import std.algorithm : split;
    import std.string : strip;
    
    // Buscar operadores
    if (expr.indexOf("+") != -1) {
        auto parts = expr.split("+");
        if (parts.length == 2) {
            long a = to!long(strip(parts[0]));
            long b = to!long(strip(parts[1]));
            return to!string(a + b);
        }
    }
    
    if (expr.indexOf("*") != -1) {
        auto parts = expr.split("*");
        if (parts.length == 2) {
            long a = to!long(strip(parts[0]));
            long b = to!long(strip(parts[1]));
            return to!string(a * b);
        }
    }
    
    return expr; // No optimizable
}
```

#### Tarea 1.2: Integrar con Rust

**Archivo:** `CORE/rust/crates/adead-parser/src/d_ctfe.rs` (nuevo)

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[link(name = "adead_d")]
extern "C" {
    fn evaluate_const_expr(expr: *const c_char) -> *const c_char;
    fn free_d_string(ptr: *const c_char);
}

pub fn optimize_constants(source: &str) -> Result<String, String> {
    let c_source = CString::new(source)
        .map_err(|e| format!("Failed to create CString: {}", e))?;
    
    unsafe {
        let optimized_ptr = evaluate_const_expr(c_source.as_ptr());
        if optimized_ptr.is_null() {
            return Err("D CTFE falló".to_string());
        }
        
        let optimized = CStr::from_ptr(optimized_ptr)
            .to_string_lossy()
            .into_owned();
        
        free_d_string(optimized_ptr);
        Ok(optimized)
    }
}
```

**Integrar en:** `optimized_pipeline.rs`

```rust
fn optimize_with_d_ctfe(source: &str) -> Result<String, String> {
    use crate::d_ctfe;
    
    // Optimizar constantes
    d_ctfe::optimize_constants(source)
        .map_err(|e| format!("D CTFE error: {}", e))
}
```

---

### ✅ FASE 2: Zig Variables (CRÍTICO)

#### Tarea 2.1: Implementar SymbolTable

**Archivo:** `CORE/zig/src/symbol_table.zig` (nuevo)

```zig
const std = @import("std");

pub const SymbolTable = struct {
    variables: std.StringHashMap(StackLocation),
    next_offset: i32,
    allocator: std.mem.Allocator,
    
    pub fn init(allocator: std.mem.Allocator) SymbolTable {
        return SymbolTable{
            .variables = std.StringHashMap(StackLocation).init(allocator),
            .next_offset = 0,
            .allocator = allocator,
        };
    }
    
    pub fn deinit(self: *SymbolTable) void {
        self.variables.deinit();
    }
    
    pub fn allocate(self: *SymbolTable, name: []const u8) !StackLocation {
        const offset = self.next_offset;
        self.next_offset += 8; // 8 bytes por int64_t
        
        try self.variables.put(name, StackLocation{ .offset = offset });
        return StackLocation{ .offset = offset };
    }
    
    pub fn get(self: *SymbolTable, name: []const u8) ?StackLocation {
        return self.variables.get(name);
    }
    
    pub fn getStackSize(self: *SymbolTable) i32 {
        return self.next_offset;
    }
};

pub const StackLocation = struct {
    offset: i32,  // Offset desde rbp (negativo: rbp-8, rbp-16, etc.)
};
```

#### Tarea 2.2: Integrar en NASMGenerator

**Archivo:** `CORE/zig/src/nasm_generator.zig`

```zig
pub const NASMGenerator = struct {
    symbol_table: SymbolTable,
    // ... resto de campos
    
    pub fn generateVarAssignment(self: *NASMGenerator, name: []const u8, value: Expr) !void {
        // Generar código para calcular value
        try self.generateExpr(value);
        
        // Guardar en stack
        const location = try self.symbol_table.allocate(name);
        var code_buf: [128]u8 = undefined;
        const code = try std.fmt.bufPrint(&code_buf, "    mov [rbp-{d}], rax\n", .{location.offset + 16}); // +16 para espacio de parámetros
        try self.text_section.appendSlice(self.allocator, code);
    }
    
    pub fn generateVarLoad(self: *NASMGenerator, name: []const u8) !void {
        const location = self.symbol_table.get(name) orelse return error.VariableNotFound;
        var code_buf: [128]u8 = undefined;
        const code = try std.fmt.bufPrint(&code_buf, "    mov rax, [rbp-{d}]\n", .{location.offset + 16});
        try self.text_section.appendSlice(self.allocator, code);
    }
    
    pub fn generateStackSetup(self: *NASMGenerator) !void {
        const stack_size = self.symbol_table.getStackSize();
        if (stack_size > 0) {
            var code_buf: [256]u8 = undefined;
            const code = try std.fmt.bufPrint(&code_buf,
                \\    push rbp
                \\    mov rbp, rsp
                \\    and rsp, -16
                \\    sub rsp, {d}
                \\
            , .{stack_size + 16}); // +16 para alineación
            try self.text_section.appendSlice(self.allocator, code);
        }
    }
};
```

---

### ✅ FASE 3: Zig Optimizaciones

#### Tarea 3.1: Strength Reduction

**Archivo:** `CORE/zig/src/optimizer.zig` (nuevo)

```zig
pub fn optimizeOperation(op: BinOp, left: i64, right: i64) ?OptimizedOp {
    switch (op) {
        .Mul => {
            // x * 2 → shl x, 1
            if (right == 2) return OptimizedOp{ .ShiftLeft = 1 };
            if (right == 4) return OptimizedOp{ .ShiftLeft = 2 };
            if (right == 8) return OptimizedOp{ .ShiftLeft = 3 };
            if (right == 16) return OptimizedOp{ .ShiftLeft = 4 };
        },
        .Add => {
            // x + 0 → x
            if (right == 0) return OptimizedOp{ .Identity = {} };
        },
        .Mul => {
            // x * 1 → x
            if (right == 1) return OptimizedOp{ .Identity = {} };
            // x * 0 → 0
            if (right == 0) return OptimizedOp{ .Constant = 0 };
        },
        else => {},
    }
    return null;
}
```

---

## 🎯 Orden de Implementación Recomendado

1. **Sprint 1 (Semana 1-2):** D CTFE - Impacto máximo inmediato
2. **Sprint 2 (Semana 3-5):** Zig Variables - Funcionalidad completa
3. **Sprint 3 (Semana 6):** Zig Optimizaciones - Performance
4. **Sprint 4 (Semana 7):** Rust Limpieza - Pulido final

**Total: 7 semanas para ASM Virgen 100%**

---

## 📊 Métricas de Éxito

| Métrica | Actual | Objetivo | Estado |
|---------|--------|----------|--------|
| CTFE aplicado | 0% | 100% | ⏳ Pendiente |
| Variables funcionales | 0% | 100% | ⏳ Pendiente |
| Optimizaciones | 0% | 80% | ⏳ Pendiente |
| ASM limpio | ✅ 100% | ✅ 100% | ✅ Logrado |

---

**¡Con esta implementación, ADead generará ASM virgen 100% SIEMPRE!** 🚀

