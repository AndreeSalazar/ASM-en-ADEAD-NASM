// Main Zig Module - Exporta funciones para FFI con Rust
// Este módulo actúa como puente entre Rust y Zig

const std = @import("std");
// Importar parser de expresiones
const expr_parser = @import("expr_parser.zig");
// Importar generador NASM
const nasm_generator = @import("nasm_generator.zig");

// Las funciones están exportadas directamente en sus módulos:
// - nasm_generator.generate_nasm_ffi (pub export fn)
// - expr_parser.parse_expr_ffi (pub export fn)
// Al compilar como biblioteca estática, estas funciones estarán disponibles
// para linking desde Rust. No necesitamos re-exportarlas aquí para evitar
// colisiones de símbolos.

// Re-exportar para uso interno si es necesario
pub const NASMGenerator = nasm_generator.NASMGenerator;
pub const Expr = expr_parser.Expr;
pub const ExprParser = expr_parser.ExprParser;
