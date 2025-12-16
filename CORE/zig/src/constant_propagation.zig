// Constant Propagation Avanzada para Zig
// Propaga constantes a través de variables en el AST
// Ejemplo: let x = 5; let y = x + 3 → let y = 8

const std = @import("std");
const expr_parser = @import("expr_parser.zig");
const statement_parser = @import("statement_parser.zig");

/// Tabla de constantes conocidas (nombre de variable → valor constante)
pub const ConstantTable = struct {
    constants: std.StringHashMap(i64),
    allocator: std.mem.Allocator,
    
    pub fn init(allocator: std.mem.Allocator) ConstantTable {
        return ConstantTable{
            .constants = std.StringHashMap(i64).init(allocator),
            .allocator = allocator,
        };
    }
    
    pub fn deinit(self: *ConstantTable) void {
        self.constants.deinit();
    }
    
    /// Agregar una constante conocida
    pub fn add(self: *ConstantTable, name: []const u8, value: i64) !void {
        try self.constants.put(name, value);
    }
    
    /// Obtener el valor de una constante, o null si no es constante
    pub fn get(self: *ConstantTable, name: []const u8) ?i64 {
        return self.constants.get(name);
    }
    
    /// Eliminar una constante (cuando se reasigna)
    pub fn remove(self: *ConstantTable, name: []const u8) void {
        _ = self.constants.remove(name);
    }
    
    /// Verificar si una variable es constante
    pub fn isConstant(self: *ConstantTable, name: []const u8) bool {
        return self.constants.contains(name);
    }
};

/// Propagar constantes en una expresión
/// Retorna una nueva expresión optimizada, o null si no se puede optimizar
pub fn propagateInExpr(
    allocator: std.mem.Allocator,
    expr: *expr_parser.Expr,
    constants: *ConstantTable
) !?*expr_parser.Expr {
    switch (expr.*) {
        .Number => {
            // Ya es constante, retornar copia
            const new_expr = try allocator.create(expr_parser.Expr);
            new_expr.* = expr.*;
            return new_expr;
        },
        .Ident => |name| {
            // Si es una constante conocida, reemplazar por su valor
            if (constants.get(name)) |value| {
                const new_expr = try allocator.create(expr_parser.Expr);
                new_expr.* = expr_parser.Expr{ .Number = value };
                return new_expr;
            }
            // No es constante, retornar copia
            const new_expr = try allocator.create(expr_parser.Expr);
            new_expr.* = expr.*;
            return new_expr;
        },
        .BinaryOp => |bin| {
            // Propagar en ambos operandos primero
            const left_opt = try propagateInExpr(allocator, bin.left, constants);
            const right_opt = try propagateInExpr(allocator, bin.right, constants);
            
            const left = left_opt orelse bin.left;
            const right = right_opt orelse bin.right;
            
            // Si ambos operandos son constantes, evaluar en compile-time
            const left_is_const = switch (left.*) {
                .Number => true,
                else => false,
            };
            const right_is_const = switch (right.*) {
                .Number => true,
                else => false,
            };
            
            if (left_is_const and right_is_const) {
                // Evaluar expresión en compile-time
                const left_val = left.*.Number;
                const right_val = right.*.Number;
                
                const result = switch (bin.op) {
                    .Add => left_val + right_val,
                    .Sub => left_val - right_val,
                    .Mul => left_val * right_val,
                    .Div => if (right_val != 0) @divTrunc(left_val, right_val) else 0,
                    .Mod => if (right_val != 0) @mod(left_val, right_val) else 0,
                    // Comparaciones: retornar 1 si verdadero, 0 si falso
                    .Eq => if (left_val == right_val) 1 else 0,
                    .Ne => if (left_val != right_val) 1 else 0,
                    .Lt => if (left_val < right_val) 1 else 0,
                    .Le => if (left_val <= right_val) 1 else 0,
                    .Gt => if (left_val > right_val) 1 else 0,
                    .Ge => if (left_val >= right_val) 1 else 0,
                };
                
                const new_expr = try allocator.create(expr_parser.Expr);
                new_expr.* = expr_parser.Expr{ .Number = result };
                return new_expr;
            }
            
            // Si al menos uno cambió, crear nueva expresión binaria
            if (left_opt != null or right_opt != null) {
                const new_expr = try allocator.create(expr_parser.Expr);
                const new_left = try allocator.create(expr_parser.Expr);
                const new_right = try allocator.create(expr_parser.Expr);
                new_left.* = left.*;
                new_right.* = right.*;
                new_expr.* = expr_parser.Expr{
                    .BinaryOp = .{
                        .op = bin.op,
                        .left = new_left,
                        .right = new_right,
                    },
                };
                return new_expr;
            }
            
            // No cambió nada
            return null;
        },
        .Float => {
            // Floats no se propagan por ahora
            const new_expr = try allocator.create(expr_parser.Expr);
            new_expr.* = expr.*;
            return new_expr;
        },
        .String => {
            // Strings no se propagan por ahora
            const new_expr = try allocator.create(expr_parser.Expr);
            new_expr.* = expr.*;
            return new_expr;
        },
    }
}

/// Propagar constantes en un statement
/// Retorna un nuevo statement optimizado, o null si no se puede optimizar
pub fn propagateInStatement(
    allocator: std.mem.Allocator,
    stmt: statement_parser.Statement,
    constants: *ConstantTable
) !?statement_parser.Statement {
    switch (stmt) {
        .Let => |let_stmt| {
            // Propagar constantes en el valor
            const value_opt = try propagateInExpr(allocator, let_stmt.value, constants);
            const value = value_opt orelse let_stmt.value;
            
            // Si el valor es constante, agregarlo a la tabla
            switch (value.*) {
                .Number => |num| {
                    try constants.add(let_stmt.name, num);
                },
                else => {
                    // No es constante, eliminar de la tabla si estaba
                    constants.remove(let_stmt.name);
                },
            }
            
            // Si el valor cambió, crear nuevo statement
            if (value_opt != null) {
                return statement_parser.Statement{
                    .Let = .{
                        .name = let_stmt.name,
                        .value = value,
                    },
                };
            }
            
            return null;
        },
        .Assign => |assign_stmt| {
            // Propagar constantes en el valor
            const value_opt = try propagateInExpr(allocator, assign_stmt.value, constants);
            const value = value_opt orelse assign_stmt.value;
            
            // Reasignación elimina la constante (puede cambiar)
            constants.remove(assign_stmt.name);
            
            // Si el valor cambió, crear nuevo statement
            if (value_opt != null) {
                return statement_parser.Statement{
                    .Assign = .{
                        .name = assign_stmt.name,
                        .value = value,
                    },
                };
            }
            
            return null;
        },
        .Print => |print_expr| {
            // Propagar constantes en la expresión a imprimir
            const expr_opt = try propagateInExpr(allocator, print_expr, constants);
            const expr = expr_opt orelse print_expr;
            
            // Si cambió, crear nuevo statement
            if (expr_opt != null) {
                return statement_parser.Statement{ .Print = expr };
            }
            
            return null;
        },
        .While => |while_stmt| {
            // Propagar constantes en la condición
            const cond_opt = try propagateInExpr(allocator, while_stmt.condition, constants);
            const cond = cond_opt orelse while_stmt.condition;
            
            // Propagar en el cuerpo (pero no propagar constantes que cambian dentro del loop)
            // Por ahora, propagamos pero no guardamos constantes modificadas en el loop
            var body_opt = std.ArrayList(statement_parser.Statement).init(allocator);
            defer body_opt.deinit();
            
            var changed = false;
            for (while_stmt.body) |body_stmt| {
                const body_stmt_opt = try propagateInStatement(allocator, body_stmt, constants);
                if (body_stmt_opt) |opt_stmt| {
                    try body_opt.append(opt_stmt);
                    changed = true;
                } else {
                    try body_opt.append(body_stmt);
                }
            }
            
            // Si cambió algo, crear nuevo statement
            if (cond_opt != null or changed) {
                const body_slice = try allocator.dupe(statement_parser.Statement, body_opt.items);
                return statement_parser.Statement{
                    .While = .{
                        .condition = cond,
                        .body = body_slice,
                    },
                };
            }
            
            return null;
        },
        .If => |if_stmt| {
            // Propagar constantes en la condición
            const cond_opt = try propagateInExpr(allocator, if_stmt.condition, constants);
            const cond = cond_opt orelse if_stmt.condition;
            
            // Propagar en el cuerpo then
            var then_body_opt = std.ArrayList(statement_parser.Statement).init(allocator);
            defer then_body_opt.deinit();
            
            var then_changed = false;
            for (if_stmt.then_body) |then_stmt| {
                const then_stmt_opt = try propagateInStatement(allocator, then_stmt, constants);
                if (then_stmt_opt) |opt_stmt| {
                    try then_body_opt.append(opt_stmt);
                    then_changed = true;
                } else {
                    try then_body_opt.append(then_stmt);
                }
            }
            
            // Propagar en el cuerpo else (si existe)
            var else_body_opt: ?[]statement_parser.Statement = null;
            var else_changed = false;
            if (if_stmt.else_body) |else_body| {
                var else_body_list = std.ArrayList(statement_parser.Statement).init(allocator);
                defer else_body_list.deinit();
                
                for (else_body) |else_stmt| {
                    const else_stmt_opt = try propagateInStatement(allocator, else_stmt, constants);
                    if (else_stmt_opt) |opt_stmt| {
                        try else_body_list.append(opt_stmt);
                        else_changed = true;
                    } else {
                        try else_body_list.append(else_stmt);
                    }
                }
                
                if (else_changed) {
                    else_body_opt = try allocator.dupe(statement_parser.Statement, else_body_list.items);
                }
            }
            
            // Si cambió algo, crear nuevo statement
            if (cond_opt != null or then_changed or else_changed) {
                const then_slice = if (then_changed)
                    try allocator.dupe(statement_parser.Statement, then_body_opt.items)
                else
                    if_stmt.then_body;
                
                return statement_parser.Statement{
                    .If = .{
                        .condition = cond,
                        .then_body = then_slice,
                        .else_body = else_body_opt orelse if_stmt.else_body,
                    },
                };
            }
            
            return null;
        },
        .Expr => |expr_stmt| {
            // Propagar constantes en la expresión
            const expr_opt = try propagateInExpr(allocator, expr_stmt, constants);
            const expr = expr_opt orelse expr_stmt;
            
            // Si cambió, crear nuevo statement
            if (expr_opt != null) {
                return statement_parser.Statement{ .Expr = expr };
            }
            
            return null;
        },
    }
}

/// Propagar constantes en un programa completo (array de statements)
/// Retorna un nuevo array optimizado
pub fn propagateConstants(
    allocator: std.mem.Allocator,
    statements: []statement_parser.Statement
) ![]statement_parser.Statement {
    var constants = ConstantTable.init(allocator);
    defer constants.deinit();
    
    var optimized = std.ArrayList(statement_parser.Statement).init(allocator);
    defer optimized.deinit();
    
    for (statements) |stmt| {
        const stmt_opt = try propagateInStatement(allocator, stmt, &constants);
        if (stmt_opt) |opt_stmt| {
            try optimized.append(opt_stmt);
        } else {
            try optimized.append(stmt);
        }
    }
    
    return try allocator.dupe(statement_parser.Statement, optimized.items);
}

