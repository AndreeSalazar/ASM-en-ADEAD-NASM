// Parser Completo de ADead en Zig
// Zig se encarga de TODO el parsing, Rust solo de seguridad y codegen

const std = @import("std");

/// Token del parser
pub const Token = union(enum) {
    Struct,
    End,
    Ident: []const u8,
    Colon,
    Comma,
    LeftParen,
    RightParen,
    Pub,
    Mut,
    Init,
    Destroy,
    Int64,
    String,
    Number: i64,
    StringLit: []const u8,
    Eof,
};

/// Campo de struct parseado
pub const StructField = struct {
    visibility: bool, // true = public
    mutable: bool,
    name: []const u8,
    ty: []const u8,
};

/// Parámetro de función/método
pub const Param = struct {
    name: []const u8,
    ty: []const u8,
};

/// Statement de método
pub const MethodStmt = struct {
    visibility: bool,
    name: []const u8, // "init" o "destroy"
    params: []Param,
    body: []const u8, // Por ahora como texto, luego parsear statements
};

/// Statement de struct completo
pub const StructStmt = struct {
    name: []const u8,
    fields: []StructField,
    init_method: ?MethodStmt,
    destroy_method: ?MethodStmt,
};

/// Parser principal de ADead
pub const Parser = struct {
    input: []const u8,
    pos: usize,
    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator, input: []const u8) Parser {
        return Parser{
            .input = input,
            .pos = 0,
            .allocator = allocator,
        };
    }

    /// Skip espacios en blanco
    fn skipWhitespace(self: *Parser) void {
        while (self.pos < self.input.len and std.ascii.isWhitespace(self.input[self.pos])) {
            self.pos += 1;
        }
    }

    /// Leer un identificador
    fn readIdent(self: *Parser) ?[]const u8 {
        if (self.pos >= self.input.len) return null;
        if (!std.ascii.isAlphabetic(self.input[self.pos]) and self.input[self.pos] != '_') {
            return null;
        }
        const start = self.pos;
        while (self.pos < self.input.len and (std.ascii.isAlphanumeric(self.input[self.pos]) or self.input[self.pos] == '_')) {
            self.pos += 1;
        }
        return self.input[start..self.pos];
    }

    /// Parsear un struct completo
    pub fn parseStruct(self: *Parser) !StructStmt {
        // Esperar "struct"
        self.skipWhitespace();
        if (!std.mem.eql(u8, self.input[self.pos..][0..6], "struct")) {
            return error.ExpectedStruct;
        }
        self.pos += 6;
        self.skipWhitespace();

        // Parsear nombre
        const name = self.readIdent() orelse return error.ExpectedIdent;
        self.skipWhitespace();

        // Parsear campos y métodos
        var fields = std.ArrayList(StructField).initCapacity(self.allocator, 8) catch return error.OutOfMemory;
        defer fields.deinit();
        var init_method: ?MethodStmt = null;
        var destroy_method: ?MethodStmt = null;

        while (true) {
            self.skipWhitespace();
            
            // Verificar si es "end"
            if (self.pos < self.input.len - 2 and std.mem.eql(u8, self.input[self.pos..][0..3], "end")) {
                self.pos += 3;
                break;
            }

            // Intentar parsear método primero (más específico)
            if (self.tryParseMethod()) |method| {
                if (std.mem.eql(u8, method.name, "init")) {
                    init_method = method;
                } else if (std.mem.eql(u8, method.name, "destroy")) {
                    destroy_method = method;
                }
                continue;
            }

            // Si no es método, parsear campo
            const field = try self.parseField();
            try fields.append(field);
        }

        const fields_slice = try fields.toOwnedSlice();
        return StructStmt{
            .name = name,
            .fields = fields_slice,
            .init_method = init_method,
            .destroy_method = destroy_method,
        };
    }

    /// Intentar parsear un método (retorna null si no es un método)
    fn tryParseMethod(self: *Parser) ?MethodStmt {
        const saved_pos = self.pos;
        
        self.skipWhitespace();
        
        // Opcional "pub"
        var is_public = false;
        if (self.pos < self.input.len - 2 and std.mem.eql(u8, self.input[self.pos..][0..3], "pub")) {
            self.pos += 3;
            self.skipWhitespace();
            is_public = true;
        }

        // Debe ser "init" o "destroy"
        var method_name: []const u8 = undefined;
        if (self.pos < self.input.len - 3 and std.mem.eql(u8, self.input[self.pos..][0..4], "init")) {
            self.pos += 4;
            method_name = "init";
        } else if (self.pos < self.input.len - 5 and std.mem.eql(u8, self.input[self.pos..][0..7], "destroy")) {
            self.pos += 7;
            method_name = "destroy";
        } else {
            self.pos = saved_pos;
            return null;
        }

        self.skipWhitespace();

        // Parsear parámetros
        if (self.pos >= self.input.len or self.input[self.pos] != '(') {
            self.pos = saved_pos;
            return null;
        }
        self.pos += 1; // Skip '('
        self.skipWhitespace();

        var params = std.ArrayList(Param).initCapacity(self.allocator, 4) catch return null;
        defer params.deinit();

        if (self.pos < self.input.len and self.input[self.pos] != ')') {
            // Hay parámetros
            while (true) {
                self.skipWhitespace();
                const param_name = self.readIdent() orelse {
                    self.pos = saved_pos;
                    return null;
                };
                self.skipWhitespace();

                if (self.pos >= self.input.len or self.input[self.pos] != ':') {
                    self.pos = saved_pos;
                    return null;
                }
                self.pos += 1; // Skip ':'
                self.skipWhitespace();

                const param_ty = self.readIdent() orelse {
                    self.pos = saved_pos;
                    return null;
                };

                params.append(Param{
                    .name = param_name,
                    .ty = param_ty,
                }) catch {
                    return null;
                };

                self.skipWhitespace();
                if (self.pos >= self.input.len or self.input[self.pos] != ',') {
                    break;
                }
                self.pos += 1; // Skip ','
            }
        }

        if (self.pos >= self.input.len or self.input[self.pos] != ')') {
            self.pos = saved_pos;
            return null;
        }
        self.pos += 1; // Skip ')'
        self.skipWhitespace();

        // Por ahora, capturar el cuerpo hasta "end" como texto
        // TODO: Parsear statements del cuerpo
        const body_start = self.pos;
        var found_end = false;
        while (self.pos < self.input.len) {
            if (self.pos < self.input.len - 2 and std.mem.eql(u8, self.input[self.pos..][0..3], "end")) {
                self.pos += 3;
                found_end = true;
                break;
            }
            self.pos += 1;
        }

        const body = if (found_end) self.input[body_start..self.pos - 3] else "";

        const params_slice = params.toOwnedSlice() catch return null;
        return MethodStmt{
            .visibility = is_public,
            .name = method_name,
            .params = params_slice,
            .body = body,
        };
    }

    /// Parsear un campo de struct
    fn parseField(self: *Parser) !StructField {
        self.skipWhitespace();

        // Opcional "pub"
        var is_public = false;
        if (self.pos < self.input.len - 2 and std.mem.eql(u8, self.input[self.pos..][0..3], "pub")) {
            self.pos += 3;
            self.skipWhitespace();
            is_public = true;
        }

        // Opcional "mut"
        var is_mutable = false;
        if (self.pos < self.input.len - 2 and std.mem.eql(u8, self.input[self.pos..][0..3], "mut")) {
            self.pos += 3;
            self.skipWhitespace();
            is_mutable = true;
        }

        // Nombre del campo
        const name = self.readIdent() orelse return error.ExpectedIdent;
        self.skipWhitespace();

        // Debe haber ':'
        if (self.pos >= self.input.len or self.input[self.pos] != ':') {
            return error.ExpectedColon;
        }
        self.pos += 1;
        self.skipWhitespace();

        // Tipo
        const ty = self.readIdent() orelse return error.ExpectedIdent;

        return StructField{
            .visibility = is_public,
            .mutable = is_mutable,
            .name = name,
            .ty = ty,
        };
    }
};

test "parse struct with fields" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const input = "struct Banco\n    saldo: int64\n    pub nombre: string\nend";
    var parser = Parser.init(allocator, input);
    const stmt = try parser.parseStruct();
    defer allocator.free(stmt.fields);

    try std.testing.expectEqualStrings("Banco", stmt.name);
    try std.testing.expectEqual(2, stmt.fields.len);
    try std.testing.expectEqualStrings("saldo", stmt.fields[0].name);
    try std.testing.expect(!stmt.fields[0].visibility);
    try std.testing.expectEqualStrings("nombre", stmt.fields[1].name);
    try std.testing.expect(stmt.fields[1].visibility);
}

test "parse struct with method" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const input = "struct Banco\n    pub init(nombre: string)\n    end\nend";
    var parser = Parser.init(allocator, input);
    const stmt = try parser.parseStruct();
    defer allocator.free(stmt.fields);
    if (stmt.init_method) |init| {
        defer allocator.free(init.params);
        try std.testing.expect(init.visibility);
        try std.testing.expectEqualStrings("init", init.name);
        try std.testing.expectEqual(1, init.params.len);
        try std.testing.expectEqualStrings("nombre", init.params[0].name);
        try std.testing.expectEqualStrings("string", init.params[0].ty);
    } else {
        try std.testing.expect(false);
    }
}

