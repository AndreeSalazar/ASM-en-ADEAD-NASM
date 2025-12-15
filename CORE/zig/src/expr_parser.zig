// Parser de Expresiones Aritméticas en Zig
// Zig parsea las expresiones matemáticas de forma eficiente
// Rust solo maneja validación y codegen

const std = @import("std");

/// Tipo de operador binario
pub const BinOp = enum {
    Add,    // +
    Sub,    // -
    Mul,    // *
    Div,    // /
    Mod,    // %
    Eq,     // ==
    Ne,     // !=
    Lt,     // <
    Le,     // <=
    Gt,     // >
    Ge,     // >=
};

/// Expresión parseada
pub const Expr = union(enum) {
    Number: i64,
    Float: f64,  // Literal flotante: 3.14, 2.5e10, etc.
    String: []const u8,
    Ident: []const u8,
    BinaryOp: struct {
        op: BinOp,
        left: *Expr,
        right: *Expr,
    },
};

/// Parser de expresiones aritméticas
pub const ExprParser = struct {
    input: []const u8,
    pos: usize,
    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator, input: []const u8) ExprParser {
        return ExprParser{
            .input = input,
            .pos = 0,
            .allocator = allocator,
        };
    }

    /// Skip espacios en blanco
    fn skipWhitespace(self: *ExprParser) void {
        while (self.pos < self.input.len and std.ascii.isWhitespace(self.input[self.pos])) {
            self.pos += 1;
        }
    }

    /// Leer un float literal (debe venir antes de readNumber)
    fn readFloat(self: *ExprParser) anyerror!?f64 {
        const save_pos = self.pos;
        if (self.pos >= self.input.len) return null;
        
        // Manejar signo negativo
        var is_negative = false;
        if (self.input[self.pos] == '-') {
            is_negative = true;
            self.pos += 1;
        }
        
        if (self.pos >= self.input.len or !std.ascii.isDigit(self.input[self.pos])) {
            if (self.pos < self.input.len and self.input[self.pos] == '.') {
                // Caso: .5 (sin parte entera)
                self.pos += 1;
                if (self.pos >= self.input.len or !std.ascii.isDigit(self.input[self.pos])) {
                    self.pos = save_pos;
                    return null;
                }
                const start = self.pos;
                while (self.pos < self.input.len and std.ascii.isDigit(self.input[self.pos])) {
                    self.pos += 1;
                }
                const dec_str = self.input[start..self.pos];
                var float_str_buf: [128]u8 = undefined;
                const float_str = std.fmt.bufPrint(&float_str_buf, "0.{s}", .{dec_str}) catch {
                    self.pos = save_pos;
                    return null;
                };
                const float_val = std.fmt.parseFloat(f64, float_str) catch {
                    self.pos = save_pos;
                    return null;
                };
                return if (is_negative) -float_val else float_val;
            }
            if (is_negative) self.pos -= 1; // Revertir
            self.pos = save_pos;
            return null;
        }
        
        // Leer parte entera
        const int_start = self.pos;
        while (self.pos < self.input.len and std.ascii.isDigit(self.input[self.pos])) {
            self.pos += 1;
        }
        
        // Verificar si viene un punto
        if (self.pos >= self.input.len or self.input[self.pos] != '.') {
            self.pos = save_pos;
            return null; // No es float
        }
        
        self.pos += 1; // Skip '.'
        
        // Leer parte decimal (opcional)
        const dec_start = self.pos;
        while (self.pos < self.input.len and std.ascii.isDigit(self.input[self.pos])) {
            self.pos += 1;
        }
        
        // Construir string del float directamente desde el input
        const int_str = self.input[int_start..(dec_start - 1)];
        const dec_str = if (dec_start < self.pos) self.input[dec_start..self.pos] else "";
        
        // Construir float_str sin usar allocator si es posible
        var float_str_buf: [128]u8 = undefined;
        const float_str = if (dec_str.len > 0) 
            try std.fmt.bufPrint(&float_str_buf, "{s}.{s}", .{ int_str, dec_str })
        else
            try std.fmt.bufPrint(&float_str_buf, "{s}.0", .{int_str});
        
        const float_val = std.fmt.parseFloat(f64, float_str) catch {
            self.pos = save_pos;
            return null;
        };
        
        return if (is_negative) -float_val else float_val;
    }

    /// Leer un número entero (solo si no es float)
    fn readNumber(self: *ExprParser) ?i64 {
        if (self.pos >= self.input.len) return null;
        
        // Manejar signo negativo
        var is_negative = false;
        if (self.input[self.pos] == '-') {
            is_negative = true;
            self.pos += 1;
        }
        
        if (self.pos >= self.input.len or !std.ascii.isDigit(self.input[self.pos])) {
            if (is_negative) self.pos -= 1; // Revertir
            return null;
        }
        
        const start = self.pos;
        while (self.pos < self.input.len and std.ascii.isDigit(self.input[self.pos])) {
            self.pos += 1;
        }
        
        // Verificar que NO viene un punto (si viene, no es int)
        if (self.pos < self.input.len and self.input[self.pos] == '.') {
            // Es float, revertir
            if (is_negative) {
                self.pos = start - 1;
            } else {
                self.pos = start;
            }
            return null;
        }
        
        const num_str = self.input[start..self.pos];
        const num = std.fmt.parseInt(i64, num_str, 10) catch return null;
        
        return if (is_negative) -num else num;
    }

    /// Leer un string literal entre comillas dobles
    fn readString(self: *ExprParser) ?[]const u8 {
        if (self.pos >= self.input.len) return null;
        if (self.input[self.pos] != '"') return null;
        
        const start = self.pos + 1; // Skip comilla inicial
        self.pos += 1;
        
        // Buscar comilla de cierre
        while (self.pos < self.input.len) {
            if (self.input[self.pos] == '"') {
                // Encontramos el final
                const str = self.input[start..self.pos];
                self.pos += 1; // Skip comilla final
                return str;
            }
            // Escapar comillas dobles escapadas \" (opcional para ahora)
            if (self.input[self.pos] == '\\' and self.pos + 1 < self.input.len and self.input[self.pos + 1] == '"') {
                self.pos += 2; // Skip \"
                continue;
            }
            self.pos += 1;
        }
        
        // No se encontró comilla de cierre
        return null;
    }
    
    /// Leer un identificador
    fn readIdent(self: *ExprParser) ?[]const u8 {
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

    /// Parsear un átomo (número, float, string, identificador, o expresión entre paréntesis)
    fn parseAtom(self: *ExprParser) anyerror!?*Expr {
        self.skipWhitespace();
        
        // IMPORTANTE: Intentar float PRIMERO (antes de número)
        if (try self.readFloat()) |float_val| {
            const expr = try self.allocator.create(Expr);
            expr.* = Expr{ .Float = float_val };
            return expr;
        }
        
        // Número entero (solo si no es float)
        if (self.readNumber()) |num| {
            const expr = try self.allocator.create(Expr);
            expr.* = Expr{ .Number = num };
            return expr;
        }
        
        // String literal (ANTES de identificador para no confundir)
        if (self.readString()) |str| {
            // Duplicar el string para ownership
            const str_owned = try self.allocator.dupe(u8, str);
            const expr = try self.allocator.create(Expr);
            expr.* = Expr{ .String = str_owned };
            return expr;
        }
        
        // Identificador
        if (self.readIdent()) |ident| {
            const expr = try self.allocator.create(Expr);
            expr.* = Expr{ .Ident = ident };
            return expr;
        }
        
        // Paréntesis: (expr)
        if (self.pos < self.input.len and self.input[self.pos] == '(') {
            self.pos += 1; // Skip '('
            self.skipWhitespace();
            
            const expr = try self.parseExpression();
            if (expr == null) return null;
            
            self.skipWhitespace();
            if (self.pos >= self.input.len or self.input[self.pos] != ')') {
                return null; // Falta ')'
            }
            self.pos += 1; // Skip ')'
            
            return expr;
        }
        
        return null;
    }

    /// Parsear una expresión completa con precedencia de operadores
    /// Precedencia:
    ///   1. Multiplicación/División (*, /)
    ///   2. Suma/Resta (+, -)
    ///   3. Comparaciones (==, !=, <, <=, >, >=)
    fn parseExpression(self: *ExprParser) anyerror!?*Expr {
        // Parsear término (multiplicación/división)
        var left = try self.parseTerm();
        if (left == null) return null;
        
        self.skipWhitespace();
        
        // Operadores de comparación (menor precedencia)
        if (self.pos < self.input.len) {
            const op_pos = self.pos;
            var op: ?BinOp = null;
            
            // ==
            if (self.pos + 1 < self.input.len and self.input[self.pos] == '=' and self.input[self.pos + 1] == '=') {
                op = BinOp.Eq;
                self.pos += 2;
            }
            // !=
            else if (self.pos + 1 < self.input.len and self.input[self.pos] == '!' and self.input[self.pos + 1] == '=') {
                op = BinOp.Ne;
                self.pos += 2;
            }
            // <=
            else if (self.pos + 1 < self.input.len and self.input[self.pos] == '<' and self.input[self.pos + 1] == '=') {
                op = BinOp.Le;
                self.pos += 2;
            }
            // >=
            else if (self.pos + 1 < self.input.len and self.input[self.pos] == '>' and self.input[self.pos + 1] == '=') {
                op = BinOp.Ge;
                self.pos += 2;
            }
            // <
            else if (self.input[self.pos] == '<') {
                op = BinOp.Lt;
                self.pos += 1;
            }
            // >
            else if (self.input[self.pos] == '>') {
                op = BinOp.Gt;
                self.pos += 1;
            }
            
            if (op) |op_val| {
                self.skipWhitespace();
                const right = try self.parseTerm();
                if (right == null) {
                    self.pos = op_pos; // Revertir
                    return left;
                }
                
                const binary = try self.allocator.create(Expr);
                binary.* = Expr{
                    .BinaryOp = .{
                        .op = op_val,
                        .left = left.?,
                        .right = right.?,
                    },
                };
                return binary;
            }
        }
        
        // Operadores de suma/resta
        while (self.pos < self.input.len) {
            self.skipWhitespace();
            
            if (self.pos >= self.input.len) break;
            
            const op_pos = self.pos;
            var op: ?BinOp = null;
            
            if (self.input[self.pos] == '+') {
                op = BinOp.Add;
                self.pos += 1;
            } else if (self.input[self.pos] == '-') {
                op = BinOp.Sub;
                self.pos += 1;
            } else {
                break; // No es operador de suma/resta
            }
            
            self.skipWhitespace();
            const right = try self.parseTerm();
            if (right == null) {
                self.pos = op_pos; // Revertir
                break;
            }
            
            const binary = try self.allocator.create(Expr);
            binary.* = Expr{
                .BinaryOp = .{
                    .op = op.?,
                    .left = left.?,
                    .right = right.?,
                },
            };
            left = binary;
        }
        
        return left;
    }

    /// Parsear un término (multiplicación/división)
    fn parseTerm(self: *ExprParser) anyerror!?*Expr {
        var left = try self.parseAtom();
        if (left == null) return null;
        
        self.skipWhitespace();
        
        // Operadores de multiplicación/división
        while (self.pos < self.input.len) {
            self.skipWhitespace();
            
            if (self.pos >= self.input.len) break;
            
            const op_pos = self.pos;
            var op: ?BinOp = null;
            
            if (self.input[self.pos] == '*') {
                op = BinOp.Mul;
                self.pos += 1;
            } else if (self.input[self.pos] == '/') {
                op = BinOp.Div;
                self.pos += 1;
            } else if (self.input[self.pos] == '%') {
                op = BinOp.Mod;
                self.pos += 1;
            } else {
                break; // No es operador de multiplicación/división/módulo
            }
            
            self.skipWhitespace();
            const right = try self.parseAtom();
            if (right == null) {
                self.pos = op_pos; // Revertir
                break;
            }
            
            const binary = try self.allocator.create(Expr);
            binary.* = Expr{
                .BinaryOp = .{
                    .op = op.?,
                    .left = left.?,
                    .right = right.?,
                },
            };
            left = binary;
        }
        
        return left;
    }

    /// Parsear expresión completa (entry point)
    pub fn parse(self: *ExprParser) anyerror!?*Expr {
        self.pos = 0; // Reset position
        return self.parseExpression();
    }
};

/// Función helper para parsear expresión desde string
pub fn parseExpressionString(allocator: std.mem.Allocator, input: []const u8) !?*Expr {
    var parser = ExprParser.init(allocator, input);
    return try parser.parse();
}

// NOTA: generate_nasm_ffi está en nasm_generator.zig
// Esta función fue movida allí para evitar duplicación

/// Exportación FFI para Rust - Parsear expresión aritmética
/// Retorna JSON-like string con la expresión parseada
/// input_ptr: puntero a string null-terminated
/// input_len: longitud del string (sin incluir null terminator)
/// output_buffer: buffer donde escribir el resultado serializado
/// output_buffer_len: tamaño del buffer
/// Retorna: longitud del resultado (sin null terminator) o código de error negativo
pub export fn parse_expr_ffi(
    input_ptr: [*:0]const u8,
    input_len: usize,
    output_buffer: [*]u8,
    output_buffer_len: usize,
) i32 {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    // Extraer input (usar input_len, no buscar null terminator)
    const input = if (input_len > 0) input_ptr[0..input_len] else "";
    
    var parser = ExprParser.init(allocator, input);
    const expr = parser.parse() catch {
        return -1; // Error parsing
    };
    
    if (expr == null) {
        return -1; // No se pudo parsear
    }
    
    // Serializar expresión directamente al buffer de salida
    // Usar un writer simple sobre el buffer proporcionado
    var pos: usize = 0;
    
    serializeExprToBuffer(expr.?, output_buffer, &pos, output_buffer_len - 1, allocator) catch {
        return -1; // Error serializando o buffer pequeño
    };
    
    output_buffer[pos] = 0; // Null terminator
    return @intCast(pos);
}

/// Serializar expresión directamente a buffer
fn serializeExprToBuffer(expr: *Expr, buffer: [*]u8, pos: *usize, max_len: usize, allocator: std.mem.Allocator) !void {
    switch (expr.*) {
        .Number => |n| {
            const str = try std.fmt.bufPrint(buffer[pos.*..max_len], "NUMBER:{}", .{n});
            pos.* += str.len;
        },
        .Float => |f| {
            const str = try std.fmt.bufPrint(buffer[pos.*..max_len], "FLOAT:{}", .{f});
            pos.* += str.len;
        },
        .String => |s| {
            const str = try std.fmt.bufPrint(buffer[pos.*..max_len], "STRING:{s}", .{s});
            pos.* += str.len;
        },
        .Ident => |ident| {
            const str = try std.fmt.bufPrint(buffer[pos.*..max_len], "IDENT:{s}", .{ident});
            pos.* += str.len;
        },
        .BinaryOp => |bin| {
            const op_str = switch (bin.op) {
                .Add => "ADD",
                .Sub => "SUB",
                .Mul => "MUL",
                .Div => "DIV",
                .Mod => "MOD",
                .Eq => "EQ",
                .Ne => "NE",
                .Lt => "LT",
                .Le => "LE",
                .Gt => "GT",
                .Ge => "GE",
            };
            const prefix = try std.fmt.bufPrint(buffer[pos.*..max_len], "BINOP:{s}:", .{op_str});
            pos.* += prefix.len;
            try serializeExprToBuffer(bin.left, buffer, pos, max_len, allocator);
            if (pos.* >= max_len) return error.BufferTooSmall;
            buffer[pos.*] = ':';
            pos.* += 1;
            try serializeExprToBuffer(bin.right, buffer, pos, max_len, allocator);
        },
    }
}

/// Serializar expresión a formato string simple (versión legacy para tests)
fn serializeExpr(expr: *Expr, writer: anytype, allocator: std.mem.Allocator) !void {
    switch (expr.*) {
        .Number => |n| {
            try writer.print("NUMBER:{}", .{n});
        },
        .String => |s| {
            try writer.print("STRING:{s}", .{s});
        },
        .Ident => |ident| {
            try writer.print("IDENT:{s}", .{ident});
        },
        .BinaryOp => |bin| {
            const op_str = switch (bin.op) {
                .Add => "ADD",
                .Sub => "SUB",
                .Mul => "MUL",
                .Div => "DIV",
                .Mod => "MOD",
                .Eq => "EQ",
                .Ne => "NE",
                .Lt => "LT",
                .Le => "LE",
                .Gt => "GT",
                .Ge => "GE",
            };
            try writer.print("BINOP:{}:", .{op_str});
            try serializeExpr(bin.left, writer, allocator);
            try writer.print(":");
            try serializeExpr(bin.right, writer, allocator);
        },
    }
}

test "parse simple number" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    const expr = try parseExpressionString(allocator, "42");
    try std.testing.expect(expr != null);
    try std.testing.expectEqual(@as(i64, 42), expr.?.*.Number);
}

test "parse addition" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    const expr = try parseExpressionString(allocator, "2 + 5");
    try std.testing.expect(expr != null);
    try std.testing.expectEqual(Expr.BinaryOp, @typeInfo(@TypeOf(expr.?.*)).Union.tag_type.?);
    
    if (expr.?.* == .BinaryOp) {
        try std.testing.expectEqual(BinOp.Add, expr.?.*.BinaryOp.op);
        try std.testing.expectEqual(@as(i64, 2), expr.?.*.BinaryOp.left.*.Number);
        try std.testing.expectEqual(@as(i64, 5), expr.?.*.BinaryOp.right.*.Number);
    }
}

test "parse multiplication" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    const expr = try parseExpressionString(allocator, "10 * 3");
    try std.testing.expect(expr != null);
    if (expr.?.* == .BinaryOp) {
        try std.testing.expectEqual(BinOp.Mul, expr.?.*.BinaryOp.op);
    }
}

test "parse precedence" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    // 2 + 3 * 4 debería ser 2 + (3 * 4)
    const expr = try parseExpressionString(allocator, "2 + 3 * 4");
    try std.testing.expect(expr != null);
    if (expr.?.* == .BinaryOp) {
        try std.testing.expectEqual(BinOp.Add, expr.?.*.BinaryOp.op);
        // El lado derecho debería ser una multiplicación
        if (expr.?.*.BinaryOp.right.* == .BinaryOp) {
            try std.testing.expectEqual(BinOp.Mul, expr.?.*.BinaryOp.right.*.BinaryOp.op);
        }
    }
}

test "parse with parentheses" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    const expr = try parseExpressionString(allocator, "(2 + 3) * 4");
    try std.testing.expect(expr != null);
    if (expr.?.* == .BinaryOp) {
        try std.testing.expectEqual(BinOp.Mul, expr.?.*.BinaryOp.op);
        // El lado izquierdo debería ser una suma
        if (expr.?.*.BinaryOp.left.* == .BinaryOp) {
            try std.testing.expectEqual(BinOp.Add, expr.?.*.BinaryOp.left.*.BinaryOp.op);
        }
    }
}

