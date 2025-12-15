// Main Zig Module - Exporta funciones para FFI con Rust
// Este módulo actúa como puente entre Rust y Zig

const std = @import("std");
// Importar parser de expresiones
const expr_parser = @import("expr_parser.zig");
// Importar generador NASM
const nasm_generator = @import("nasm_generator.zig");

// Re-exportar generate_nasm_ffi desde nasm_generator
pub const generate_nasm_ffi = nasm_generator.generate_nasm_ffi;
// Re-exportar NASMGenerator para uso directo si es necesario
pub const NASMGenerator = nasm_generator.NASMGenerator;
// Re-exportar funciones de expr_parser manualmente
pub const parse_expr_ffi = expr_parser.parse_expr_ffi;
pub const Expr = expr_parser.Expr;
pub const ExprParser = expr_parser.ExprParser;
