// Main Zig Module - Exporta funciones para FFI con Rust
// Este módulo actúa como puente entre Rust y Zig

const std = @import("std");
const parser = @import("parser.zig");
const parser_completo = @import("parser_completo.zig");
const codegen = @import("codegen.zig");

pub usingnamespace parser;
pub usingnamespace parser_completo;
pub usingnamespace codegen;

// Exportaciones FFI para Rust
// Usar allocator global para simplificar
var gpa = std.heap.GeneralPurposeAllocator(.{}){};
var gpa_initialized = false;

fn get_allocator() std.mem.Allocator {
    if (!gpa_initialized) {
        gpa_initialized = true;
    }
    return gpa.allocator();
}

/// Parsear un struct completo usando Zig
/// Retorna un puntero a StructStmt o null si falla
pub export fn parse_struct_ffi(
    input_ptr: [*:0]const u8,
    input_len: usize,
) callconv(.C) ?*parser_completo.StructStmt {
    const allocator = get_allocator();
    const input = input_ptr[0..input_len];
    
    var parser_obj = parser_completo.Parser.init(allocator, input);
    
    const stmt = parser_obj.parseStruct() catch {
        return null;
    };
    
    // Boxear el resultado para que Rust pueda accederlo
    const boxed = allocator.create(parser_completo.StructStmt) catch {
        return null;
    };
    boxed.* = stmt;
    return boxed;
}
