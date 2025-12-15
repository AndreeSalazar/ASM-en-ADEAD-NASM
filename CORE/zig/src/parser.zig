// Parser Zig - Parte específica para parsing eficiente
// Zig se encarga de parsing de bajo nivel y optimizaciones

const std = @import("std");

/// Tipo de parámetro parseado
pub const Param = struct {
    name: []const u8,
    tipo: []const u8,
};

/// Resultado del parsing de parámetros
pub const ParamList = struct {
    params: []Param,
    allocator: std.mem.Allocator,
    
    pub fn deinit(self: *ParamList) void {
        for (self.params) |param| {
            self.allocator.free(param.name);
            self.allocator.free(param.tipo);
        }
        self.allocator.free(self.params);
    }
};

/// Parsea parámetros desde un string
/// Formato esperado: "nombre: tipo, otro: tipo" o ""
/// Zig se encarga del parsing rápido y eficiente
pub fn parseParams(allocator: std.mem.Allocator, input: []const u8) !ParamList {
    var params = std.ArrayList(Param).init(allocator);
    errdefer params.deinit();
    
    // Si está vacío, retornar lista vacía
    if (input.len == 0) {
        return ParamList{
            .params = try params.toOwnedSlice(),
            .allocator = allocator,
        };
    }
    
    // Dividir por comas
    var iter = std.mem.tokenizeSequence(u8, input, ",");
    
    while (iter.next()) |param_str| {
        // Buscar el separador ':'
        if (std.mem.indexOfScalar(u8, param_str, ':')) |colon_pos| {
            const name = param_str[0..colon_pos];
            const tipo = param_str[colon_pos + 1..];
            
            // Trim espacios
            const name_trimmed = std.mem.trim(u8, name, " \t\n\r");
            const tipo_trimmed = std.mem.trim(u8, tipo, " \t\n\r");
            
            // Duplicar strings para ownership
            const name_owned = try allocator.dupe(u8, name_trimmed);
            errdefer allocator.free(name_owned);
            
            const tipo_owned = try allocator.dupe(u8, tipo_trimmed);
            errdefer allocator.free(tipo_owned);
            
            try params.append(Param{
                .name = name_owned,
                .tipo = tipo_owned,
            });
        }
    }
    
    return ParamList{
        .params = try params.toOwnedSlice(),
        .allocator = allocator,
    };
}

/// Exportación para C/Rust FFI - Versión mejorada
/// Esta función parsea y devuelve los parámetros en buffers proporcionados
pub export fn parse_params_simple(
    input_ptr: [*:0]const u8,
    output_buffer: [*]u8,
    output_buffer_len: usize,
) callconv(.C) i32 {
    // Usar allocator temporal
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    const input = std.mem.span(input_ptr);
    
    var param_list = parseParams(allocator, input) catch {
        return -1;
    };
    defer param_list.deinit();
    
    // Escribir resultado en formato simple: "name1:type1,name2:type2"
    var stream = std.io.fixedBufferStream(output_buffer[0..output_buffer_len]);
    var writer = stream.writer();
    var first = true;
    
    for (param_list.params) |param| {
        if (!first) {
            writer.writeAll(",") catch return -1;
        }
        first = false;
        writer.writeAll(param.name) catch return -1;
        writer.writeAll(":") catch return -1;
        writer.writeAll(param.tipo) catch return -1;
    }
    
    return @intCast(param_list.params.len);
}

test "parse empty params" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    const result = try parseParams(allocator, "");
    defer result.deinit();
    
    try std.testing.expectEqual(@as(usize, 0), result.params.len);
}

test "parse single param" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    const result = try parseParams(allocator, "nombre: string");
    defer result.deinit();
    
    try std.testing.expectEqual(@as(usize, 1), result.params.len);
    try std.testing.expectEqualStrings("nombre", result.params[0].name);
    try std.testing.expectEqualStrings("string", result.params[0].tipo);
}

test "parse multiple params" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    const result = try parseParams(allocator, "nombre: string, edad: int64");
    defer result.deinit();
    
    try std.testing.expectEqual(@as(usize, 2), result.params.len);
    try std.testing.expectEqualStrings("nombre", result.params[0].name);
    try std.testing.expectEqualStrings("string", result.params[0].tipo);
    try std.testing.expectEqualStrings("edad", result.params[1].name);
    try std.testing.expectEqualStrings("int64", result.params[1].tipo);
}
