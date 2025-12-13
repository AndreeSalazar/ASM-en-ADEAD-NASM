// Main Zig Module - Exporta funciones para FFI con Rust
// Este módulo actúa como puente entre Rust y Zig

const std = @import("std");
// Importar parser de expresiones
const expr_parser = @import("expr_parser.zig");

// Re-exportar todas las funciones de expr_parser (incluyendo parse_expr_ffi)
pub usingnamespace expr_parser;
