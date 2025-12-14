// Main Zig Module - Exporta funciones para FFI con Rust
// Este módulo actúa como puente entre Rust y Zig

const std = @import("std");
// Importar parser de expresiones
const expr_parser = @import("expr_parser.zig");
// Importar generador NASM
const nasm_generator = @import("nasm_generator.zig");

// Re-exportar todas las funciones de expr_parser (incluyendo parse_expr_ffi)
pub usingnamespace expr_parser;
// Re-exportar generate_nasm_ffi desde nasm_generator
pub const generate_nasm_ffi = nasm_generator.generate_nasm_ffi;
// Re-exportar NASMGenerator para uso directo si es necesario
pub const NASMGenerator = nasm_generator.NASMGenerator;
