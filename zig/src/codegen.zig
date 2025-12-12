// Code Generator Zig - Generación eficiente de código NASM
// Zig se encarga de generar código ASM optimizado

const std = @import("std");

/// Tipo de instrucción NASM
pub const Instruction = struct {
    opcode: []const u8,
    operands: []const []const u8,
};

/// Generador de código NASM
pub const CodeGenerator = struct {
    allocator: std.mem.Allocator,
    output: std.ArrayList(u8),
    
    pub fn init(allocator: std.mem.Allocator) CodeGenerator {
        return CodeGenerator{
            .allocator = allocator,
            .output = std.ArrayList(u8).init(allocator),
        };
    }
    
    pub fn deinit(self: *CodeGenerator) void {
        self.output.deinit();
    }
    
    /// Escribir una línea al output
    pub fn write(self: *CodeGenerator, line: []const u8) !void {
        try self.output.writer().print("{s}\n", .{line});
    }
    
    /// Generar código para mover un valor a un registro
    pub fn genMove(self: *CodeGenerator, dest: []const u8, src: []const u8) !void {
        try self.write(std.fmt.allocPrint(
            self.allocator,
            "    mov {s}, {s}",
            .{ dest, src }
        ) catch return);
    }
    
    /// Generar código para llamar a una función
    pub fn genCall(self: *CodeGenerator, func_name: []const u8) !void {
        try self.write(std.fmt.allocPrint(
            self.allocator,
            "    call {s}",
            .{func_name}
        ) catch return);
    }
    
    /// Generar header de función
    pub fn genFunctionHeader(self: *CodeGenerator, name: []const u8) !void {
        try self.write(std.fmt.allocPrint(
            self.allocator,
            "{s}:",
            .{name}
        ) catch return);
        try self.write("    push rbp");
        try self.write("    mov rbp, rsp");
    }
    
    /// Generar footer de función
    pub fn genFunctionFooter(self: *CodeGenerator) !void {
        try self.write("    pop rbp");
        try self.write("    ret");
    }
    
    /// Obtener el código generado como string
    pub fn getOutput(self: *const CodeGenerator) []const u8 {
        return self.output.items;
    }
};

/// Exportación para C/Rust FFI - Versión simplificada
/// Nota: Para implementación completa, necesitamos manejar allocators globalmente
pub export fn codegen_init() callconv(.C) usize {
    // Placeholder - implementación completa requeriría allocator global
    return 0;
}

pub export fn codegen_write_ffi(
    _: usize,
    _: [*:0]const u8,
    _: usize,
) callconv(.C) bool {
    // Placeholder - implementación completa requeriría manejo de memoria
    return true;
}

test "codegen basic" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    var cg = CodeGenerator.init(allocator);
    defer cg.deinit();
    
    try cg.write("section .text");
    try cg.write("global _start");
    try cg.genFunctionHeader("_start");
    try cg.genMove("rax", "42");
    try cg.genFunctionFooter();
    
    const output = cg.getOutput();
    try std.testing.expect(std.mem.indexOf(u8, output, "section .text") != null);
    try std.testing.expect(std.mem.indexOf(u8, output, "mov rax, 42") != null);
}

