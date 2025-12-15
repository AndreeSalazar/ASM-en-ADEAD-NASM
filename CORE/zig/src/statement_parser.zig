// Parser de Statements en Zig
// Soporta while loops, if statements, let, print, etc.

const std = @import("std");
const expr_parser = @import("expr_parser.zig");

/// Tipo de statement parseado
pub const Statement = union(enum) {
    Print: *expr_parser.Expr,
    Let: struct {
        name: []const u8,
        value: *expr_parser.Expr,
    },
    Assign: struct {
        name: []const u8,
        value: *expr_parser.Expr,
    },
    While: struct {
        condition: *expr_parser.Expr,
        body: []Statement,
    },
    If: struct {
        condition: *expr_parser.Expr,
        then_body: []Statement,
        else_body: ?[]Statement,
    },
    Expr: *expr_parser.Expr,
};

/// Parser de statements
pub const StatementParser = struct {
    input: []const u8,
    pos: usize,
    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator, input: []const u8) StatementParser {
        return StatementParser{
            .input = input,
            .pos = 0,
            .allocator = allocator,
        };
    }

    pub fn skipWhitespace(self: *StatementParser) void {
        while (self.pos < self.input.len and std.ascii.isWhitespace(self.input[self.pos])) {
            self.pos += 1;
        }
    }
    
    /// Leer un identificador (helper para asignaciones)
    fn readIdent(self: *StatementParser) ?[]const u8 {
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

    fn matchKeyword(self: *StatementParser, keyword: []const u8) bool {
        const old_pos = self.pos;
        self.skipWhitespace();
        
        if (self.pos + keyword.len <= self.input.len) {
            const slice = self.input[self.pos..self.pos + keyword.len];
            if (std.mem.eql(u8, slice, keyword)) {
                // Verificar que después no haya caracteres alfanuméricos
                if (self.pos + keyword.len < self.input.len) {
                    const next = self.input[self.pos + keyword.len];
                    if (!std.ascii.isAlphanumeric(next) and next != '_') {
                        self.pos += keyword.len;
                        return true;
                    }
                } else {
                    self.pos += keyword.len;
                    return true;
                }
            }
        }
        
        self.pos = old_pos;
        return false;
    }

    /// Parsear una expresión (delegar a expr_parser)
    fn parseExpr(self: *StatementParser) !?*expr_parser.Expr {
        const start_pos = self.pos;
        self.skipWhitespace();
        
        if (self.pos >= self.input.len) {
            return null;
        }
        
        var parser = expr_parser.ExprParser.init(self.allocator, self.input[self.pos..]);
        if (parser.parse()) |expr| {
            self.pos += parser.pos;
            return expr;
        } else |_| {
            self.pos = start_pos;
            return null;
        }
    }

    /// Parsear un while statement
    fn parseWhile(self: *StatementParser) anyerror!?Statement {
        const old_pos = self.pos;
        
        if (!self.matchKeyword("while")) {
            self.pos = old_pos;
            return null;
        }
        
        self.skipWhitespace();
        
        // Parsear condición usando el parser de expresiones directamente
        // Necesitamos parsear hasta encontrar '{'
        const condition_start = self.pos;
        
        // Buscar el inicio del bloque '{'
        var brace_pos = self.pos;
        while (brace_pos < self.input.len and self.input[brace_pos] != '{') {
            brace_pos += 1;
        }
        
        if (brace_pos >= self.input.len) {
            self.pos = old_pos;
            return null;
        }
        
        // Extraer la condición (desde pos hasta brace_pos)
        const condition_str = std.mem.trim(u8, self.input[condition_start..brace_pos], " \t\n\r");
        
        if (condition_str.len == 0) {
            self.pos = old_pos;
            return null;
        }
        
        // Parsear condición con expr_parser
        var cond_parser = expr_parser.ExprParser.init(self.allocator, condition_str);
        const condition = cond_parser.parse() catch {
            self.pos = old_pos;
            return null;
        };
        
        if (condition == null) {
            self.pos = old_pos;
            return null;
        }
        
        // Avanzar posición hasta el '{'
        self.pos = brace_pos;
        
        // Consumir '{'
        if (self.pos >= self.input.len or self.input[self.pos] != '{') {
            self.pos = old_pos;
            return null;
        }
        self.pos += 1;
        
        // Parsear body hasta encontrar '}'
        self.skipWhitespace();
        const body_start = self.pos;
        var brace_count: usize = 1;
        var body_end = self.pos;
        
        while (body_end < self.input.len and brace_count > 0) {
            if (self.input[body_end] == '{') {
                brace_count += 1;
            } else if (self.input[body_end] == '}') {
                brace_count -= 1;
            }
            if (brace_count > 0) {
                body_end += 1;
            }
        }
        
        if (brace_count > 0) {
            self.pos = old_pos;
            return null;
        }
        
        const body_str = self.input[body_start..body_end];
        self.pos = body_end + 1; // Consumir '}'
        
        // Parsear statements del body
        var body_parser = StatementParser.init(self.allocator, body_str);
        var body_statements = std.ArrayList(Statement).initCapacity(self.allocator, 16) catch return null;
        defer body_statements.deinit(self.allocator);
        
        while (true) {
            const parse_result = body_parser.parse() catch {
                // Si hay error, continuar con lo que se parseó
                break;
            };
            if (parse_result) |stmt| {
                body_statements.append(self.allocator, stmt) catch {
                    // Si falla append, salir
                    break;
                };
            } else {
                // No hay más statements
                break;
            }
        }
        
        return Statement{
            .While = .{
                .condition = condition.?,
                .body = try body_statements.toOwnedSlice(self.allocator),
            },
        };
    }
    
    /// Parsear un if statement
    fn parseIf(self: *StatementParser) anyerror!?Statement {
        const old_pos = self.pos;
        
        if (!self.matchKeyword("if")) {
            self.pos = old_pos;
            return null;
        }
        
        self.skipWhitespace();
        
        // Parsear condición usando el parser de expresiones directamente
        const condition_start = self.pos;
        
        // Buscar el inicio del bloque '{'
        var brace_pos = self.pos;
        while (brace_pos < self.input.len and self.input[brace_pos] != '{') {
            brace_pos += 1;
        }
        
        if (brace_pos >= self.input.len) {
            self.pos = old_pos;
            return null;
        }
        
        // Extraer la condición
        const condition_str = std.mem.trim(u8, self.input[condition_start..brace_pos], " \t\n\r");
        
        if (condition_str.len == 0) {
            self.pos = old_pos;
            return null;
        }
        
        // Parsear condición con expr_parser
        var cond_parser = expr_parser.ExprParser.init(self.allocator, condition_str);
        const condition = cond_parser.parse() catch {
            self.pos = old_pos;
            return null;
        };
        
        if (condition == null) {
            self.pos = old_pos;
            return null;
        }
        
        // Avanzar posición hasta el '{'
        self.pos = brace_pos;
        
        // Consumir '{'
        if (self.pos >= self.input.len or self.input[self.pos] != '{') {
            self.pos = old_pos;
            return null;
        }
        self.pos += 1;
        
        // Parsear body hasta encontrar '}'
        self.skipWhitespace();
        const body_start = self.pos;
        var brace_count: usize = 1;
        var body_end = self.pos;
        
        while (body_end < self.input.len and brace_count > 0) {
            if (self.input[body_end] == '{') {
                brace_count += 1;
            } else if (self.input[body_end] == '}') {
                brace_count -= 1;
            }
            if (brace_count > 0) {
                body_end += 1;
            }
        }
        
        if (brace_count > 0) {
            self.pos = old_pos;
            return null;
        }
        
        const body_str = self.input[body_start..body_end];
        self.pos = body_end + 1; // Consumir '}'
        
        // Parsear statements del body
        var body_parser = StatementParser.init(self.allocator, body_str);
        var body_statements = std.ArrayList(Statement).initCapacity(self.allocator, 16) catch return null;
        defer body_statements.deinit(self.allocator);
        
        while (true) {
            const parse_result = body_parser.parse() catch {
                // Si hay error, continuar con lo que se parseó
                break;
            };
            if (parse_result) |stmt| {
                body_statements.append(self.allocator, stmt) catch {
                    // Si falla append, salir
                    break;
                };
            } else {
                // No hay más statements
                break;
            }
        }
        
        // Por ahora, no soportamos else (solo if sin else)
        return Statement{
            .If = .{
                .condition = condition.?,
                .then_body = try body_statements.toOwnedSlice(self.allocator),
                .else_body = null,
            },
        };
    }

    /// Parsear un statement
    pub fn parse(self: *StatementParser) !?Statement {
        self.skipWhitespace();
        
        if (self.pos >= self.input.len) {
            return null;
        }
        
        // IMPORTANTE: Orden crítico - asignación ANTES de keywords
        // porque asigna parsea identificadores que podrían ser keywords mal parseados
        
        // Intentar parsear asignación primero (ANTES de keywords)
        const assign_start = self.pos;
        if (self.readIdent()) |ident| {
            self.skipWhitespace();
            
            // Si viene un '=', es una asignación
            if (self.pos < self.input.len and self.input[self.pos] == '=') {
                const name = try self.allocator.dupe(u8, ident);
                self.pos += 1; // Skip '='
                self.skipWhitespace();
                
                // Parsear valor
                const maybe_expr = self.parseExpr() catch {
                    self.allocator.free(name);
                    self.pos = assign_start;
                    return null;
                };
                if (maybe_expr) |expr| {
                    return Statement{
                        .Assign = .{
                            .name = name,
                            .value = expr,
                        },
                    };
                } else {
                    self.allocator.free(name);
                    self.pos = assign_start;
                }
            } else {
                // No es asignación, revertir
                self.pos = assign_start;
            }
        }
        
        // Intentar parsear if primero (antes de while porque "if" aparece antes en el string)
        const maybe_if = self.parseIf() catch return null;
        if (maybe_if) |stmt| {
            return stmt;
        }
        
        // Intentar parsear while
        const maybe_while = self.parseWhile() catch return null;
        if (maybe_while) |stmt| {
            return stmt;
        }
        
        // Intentar parsear print
        if (self.matchKeyword("print")) {
            self.skipWhitespace();
            const maybe_expr = self.parseExpr() catch return null;
            if (maybe_expr) |expr| {
                return Statement{ .Print = expr };
            }
        }
        
        // Intentar parsear let
        if (self.matchKeyword("let")) {
            self.skipWhitespace();
            // Parsear nombre
            const name_start = self.pos;
            while (self.pos < self.input.len and (std.ascii.isAlphanumeric(self.input[self.pos]) or self.input[self.pos] == '_')) {
                self.pos += 1;
            }
            
            if (self.pos == name_start) {
                return null;
            }
            
            const name = try self.allocator.dupe(u8, self.input[name_start..self.pos]);
            self.skipWhitespace();
            
            // Consumir '='
            if (self.pos >= self.input.len or self.input[self.pos] != '=') {
                self.allocator.free(name);
                return null;
            }
            self.pos += 1;
            self.skipWhitespace();
            
            // Parsear valor
            const maybe_expr = self.parseExpr() catch {
                self.allocator.free(name);
                return null;
            };
            if (maybe_expr) |expr| {
                return Statement{
                    .Let = .{
                        .name = name,
                        .value = expr,
                    },
                };
            } else {
                self.allocator.free(name);
                return null;
            }
        }
        
        // Intentar parsear expresión simple
        const maybe_expr = self.parseExpr() catch return null;
        if (maybe_expr) |expr| {
            return Statement{ .Expr = expr };
        }
        
        return null;
    }
};

