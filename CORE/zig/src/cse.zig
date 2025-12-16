// Common Subexpression Elimination (CSE) para Zig
// Detecta expresiones comunes calculadas múltiples veces
// Guarda resultados en registros temporales y reutiliza
// Ejemplo: let x = a + b; let y = a + b → let temp = a + b; let x = temp; let y = temp

const std = @import("std");
const expr_parser = @import("expr_parser.zig");
const statement_parser = @import("statement_parser.zig");

/// Hash de una expresión para comparación
/// Usa una representación canónica de la expresión
fn exprHash(expr: *expr_parser.Expr) u64 {
    var hasher = std.hash.Wyhash.init(0);
    
    switch (expr.*) {
        .Number => |n| {
            hasher.update("Number");
            hasher.update(&std.mem.toBytes(n));
        },
        .Ident => |name| {
            hasher.update("Ident");
            hasher.update(name);
        },
        .BinaryOp => |bin| {
            hasher.update("BinaryOp");
            const op_str = switch (bin.op) {
                .Add => "Add",
                .Sub => "Sub",
                .Mul => "Mul",
                .Div => "Div",
                .Mod => "Mod",
                .Eq => "Eq",
                .Ne => "Ne",
                .Lt => "Lt",
                .Le => "Le",
                .Gt => "Gt",
                .Ge => "Ge",
            };
            hasher.update(op_str);
            // Hash recursivo de operandos
            const left_hash = exprHash(bin.left);
            const right_hash = exprHash(bin.right);
            hasher.update(&std.mem.toBytes(left_hash));
            hasher.update(&std.mem.toBytes(right_hash));
        },
        .Float => |f| {
            hasher.update("Float");
            hasher.update(&std.mem.toBytes(f));
        },
        .String => |s| {
            hasher.update("String");
            hasher.update(s);
        },
    }
    
    return hasher.final();
}

/// Compara dos expresiones para ver si son iguales
fn exprEqual(expr1: *expr_parser.Expr, expr2: *expr_parser.Expr) bool {
    // Comparación rápida por hash primero
    if (exprHash(expr1) != exprHash(expr2)) {
        return false;
    }
    
    // Comparación estructural detallada
    switch (expr1.*) {
        .Number => |n1| {
            switch (expr2.*) {
                .Number => |n2| return n1 == n2,
                else => return false,
            }
        },
        .Ident => |name1| {
            switch (expr2.*) {
                .Ident => |name2| return std.mem.eql(u8, name1, name2),
                else => return false,
            }
        },
        .BinaryOp => |bin1| {
            switch (expr2.*) {
                .BinaryOp => |bin2| {
                    if (bin1.op != bin2.op) return false;
                    return exprEqual(bin1.left, bin2.left) and exprEqual(bin1.right, bin2.right);
                },
                else => return false,
            }
        },
        .Float => |f1| {
            switch (expr2.*) {
                .Float => |f2| return f1 == f2,
                else => return false,
            }
        },
        .String => |s1| {
            switch (expr2.*) {
                .String => |s2| return std.mem.eql(u8, s1, s2),
                else => return false,
            }
        },
    }
}

/// Entrada en la tabla de expresiones comunes
const CommonExprEntry = struct {
    expr: *expr_parser.Expr,
    temp_var: []const u8,  // Nombre de variable temporal (ej: "temp_0")
    use_count: usize,      // Cuántas veces se usa
};

/// Tabla de expresiones comunes
pub const CSETable = struct {
    common_exprs: std.StringHashMap(CommonExprEntry),
    temp_counter: usize,
    allocator: std.mem.Allocator,
    
    pub fn init(allocator: std.mem.Allocator) CSETable {
        return CSETable{
            .common_exprs = std.StringHashMap(CommonExprEntry).init(allocator),
            .temp_counter = 0,
            .allocator = allocator,
        };
    }
    
    pub fn deinit(self: *CSETable) void {
        // Liberar nombres de variables temporales
        var it = self.common_exprs.iterator();
        while (it.next()) |entry| {
            self.allocator.free(entry.value_ptr.temp_var);
        }
        self.common_exprs.deinit();
    }
    
    /// Buscar si una expresión ya está en la tabla
    pub fn find(self: *CSETable, expr: *expr_parser.Expr) ?[]const u8 {
        // Buscar en la tabla comparando expresiones
        var it = self.common_exprs.iterator();
        while (it.next()) |entry| {
            // Verificar que realmente es igual
            if (exprEqual(expr, entry.value_ptr.expr)) {
                entry.value_ptr.use_count += 1;
                return entry.value_ptr.temp_var;
            }
        }
        return null;
    }
    
    /// Agregar una expresión común a la tabla
    pub fn add(self: *CSETable, expr: *expr_parser.Expr) ![]const u8 {
        // Verificar si ya existe
        if (self.find(expr)) |temp_var| {
            return temp_var;
        }
        
        // Crear nueva entrada
        const temp_name = try std.fmt.allocPrint(self.allocator, "temp_{d}", .{self.temp_counter});
        self.temp_counter += 1;
        
        // Crear clave única basada en hash convertido a string
        var hash_buf: [32]u8 = undefined;
        const hash_str = try std.fmt.bufPrint(&hash_buf, "{d}", .{exprHash(expr)});
        const hash_key = try self.allocator.dupe(u8, hash_str);
        
        try self.common_exprs.put(hash_key, CommonExprEntry{
            .expr = expr,
            .temp_var = temp_name,
            .use_count = 1,
        });
        
        return temp_name;
    }
    
    /// Obtener todas las expresiones comunes que necesitan ser calculadas
    /// Retorna un array de statements `let temp_X = expr`
    pub fn getTempAssignments(self: *CSETable) ![]statement_parser.Statement {
        var assignments = std.ArrayList(statement_parser.Statement).init(self.allocator);
        errdefer assignments.deinit();
        
        var it = self.common_exprs.iterator();
        while (it.next()) |entry| {
            // Solo agregar si se usa más de una vez
            if (entry.value_ptr.use_count > 1) {
                const stmt = statement_parser.Statement{
                    .Let = .{
                        .name = entry.value_ptr.temp_var,
                        .value = entry.value_ptr.expr,
                    },
                };
                try assignments.append(stmt);
            }
        }
        
        return try assignments.toOwnedSlice();
    }
};

/// Aplicar CSE a una expresión
/// Retorna una nueva expresión que usa variable temporal si es común, o null si no es común
pub fn applyCSEToExpr(
    allocator: std.mem.Allocator,
    expr: *expr_parser.Expr,
    cse_table: *CSETable
) !?*expr_parser.Expr {
    // Solo aplicar CSE a expresiones binarias (por ahora)
    switch (expr.*) {
        .BinaryOp => |bin| {
            // Verificar si esta expresión binaria ya está en la tabla
            if (cse_table.find(expr)) |temp_var| {
                // Crear expresión Ident que referencia la variable temporal
                const new_expr = try allocator.create(expr_parser.Expr);
                new_expr.* = expr_parser.Expr{ .Ident = temp_var };
                return new_expr;
            }
            
            // No es común todavía, agregarla a la tabla
            _ = try cse_table.add(expr);
            
            // Aplicar CSE recursivamente a los operandos
            const left_cse = try applyCSEToExpr(allocator, bin.left, cse_table);
            const right_cse = try applyCSEToExpr(allocator, bin.right, cse_table);
            
            // Si alguno de los operandos cambió, crear nueva expresión
            if (left_cse != null or right_cse != null) {
                const new_expr = try allocator.create(expr_parser.Expr);
                const new_left = left_cse orelse bin.left;
                const new_right = right_cse orelse bin.right;
                
                // Crear nuevos nodos para los operandos si cambiaron
                const new_left_node = try allocator.create(expr_parser.Expr);
                const new_right_node = try allocator.create(expr_parser.Expr);
                new_left_node.* = new_left.*;
                new_right_node.* = new_right.*;
                
                new_expr.* = expr_parser.Expr{
                    .BinaryOp = .{
                        .op = bin.op,
                        .left = new_left_node,
                        .right = new_right_node,
                    },
                };
                return new_expr;
            }
            
            return null;
        },
        else => {
            // Para expresiones simples, no aplicar CSE
            return null;
        },
    }
}

/// Aplicar CSE a un statement
/// Retorna un nuevo statement optimizado, o null si no se puede optimizar
pub fn applyCSEToStatement(
    allocator: std.mem.Allocator,
    stmt: statement_parser.Statement,
    cse_table: *CSETable
) !?statement_parser.Statement {
    switch (stmt) {
        .Let => |let_stmt| {
            // Aplicar CSE a la expresión del valor
            const value_cse = try applyCSEToExpr(allocator, let_stmt.value, cse_table);
            const value = value_cse orelse let_stmt.value;
            
            // Si cambió, crear nuevo statement
            if (value_cse != null) {
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
            // Aplicar CSE a la expresión del valor
            const value_cse = try applyCSEToExpr(allocator, assign_stmt.value, cse_table);
            const value = value_cse orelse assign_stmt.value;
            
            // Si cambió, crear nuevo statement
            if (value_cse != null) {
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
            // Aplicar CSE a la expresión a imprimir
            const expr_cse = try applyCSEToExpr(allocator, print_expr, cse_table);
            const expr = expr_cse orelse print_expr;
            
            // Si cambió, crear nuevo statement
            if (expr_cse != null) {
                return statement_parser.Statement{ .Print = expr };
            }
            
            return null;
        },
        .While => |while_stmt| {
            // Aplicar CSE a la condición
            const cond_cse = try applyCSEToExpr(allocator, while_stmt.condition, cse_table);
            const cond = cond_cse orelse while_stmt.condition;
            
            // Aplicar CSE al cuerpo
            var body_cse = std.ArrayList(statement_parser.Statement).init(allocator);
            defer body_cse.deinit();
            
            var changed = false;
            for (while_stmt.body) |body_stmt| {
                const body_stmt_cse = try applyCSEToStatement(allocator, body_stmt, cse_table);
                if (body_stmt_cse) |opt_stmt| {
                    try body_cse.append(opt_stmt);
                    changed = true;
                } else {
                    try body_cse.append(body_stmt);
                }
            }
            
            // Si cambió algo, crear nuevo statement
            if (cond_cse != null or changed) {
                const body_slice = try allocator.dupe(statement_parser.Statement, body_cse.items);
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
            // Aplicar CSE a la condición
            const cond_cse = try applyCSEToExpr(allocator, if_stmt.condition, cse_table);
            const cond = cond_cse orelse if_stmt.condition;
            
            // Aplicar CSE al cuerpo then
            var then_body_cse = std.ArrayList(statement_parser.Statement).init(allocator);
            defer then_body_cse.deinit();
            
            var then_changed = false;
            for (if_stmt.then_body) |then_stmt| {
                const then_stmt_cse = try applyCSEToStatement(allocator, then_stmt, cse_table);
                if (then_stmt_cse) |opt_stmt| {
                    try then_body_cse.append(opt_stmt);
                    then_changed = true;
                } else {
                    try then_body_cse.append(then_stmt);
                }
            }
            
            // Aplicar CSE al cuerpo else (si existe)
            var else_body_cse: ?[]statement_parser.Statement = null;
            var else_changed = false;
            if (if_stmt.else_body) |else_body| {
                var else_body_list = std.ArrayList(statement_parser.Statement).init(allocator);
                defer else_body_list.deinit();
                
                for (else_body) |else_stmt| {
                    const else_stmt_cse = try applyCSEToStatement(allocator, else_stmt, cse_table);
                    if (else_stmt_cse) |opt_stmt| {
                        try else_body_list.append(opt_stmt);
                        else_changed = true;
                    } else {
                        try else_body_list.append(else_stmt);
                    }
                }
                
                if (else_changed) {
                    else_body_cse = try allocator.dupe(statement_parser.Statement, else_body_list.items);
                }
            }
            
            // Si cambió algo, crear nuevo statement
            if (cond_cse != null or then_changed or else_changed) {
                const then_slice = if (then_changed)
                    try allocator.dupe(statement_parser.Statement, then_body_cse.items)
                else
                    if_stmt.then_body;
                
                return statement_parser.Statement{
                    .If = .{
                        .condition = cond,
                        .then_body = then_slice,
                        .else_body = else_body_cse orelse if_stmt.else_body,
                    },
                };
            }
            
            return null;
        },
        .Expr => |expr_stmt| {
            // Aplicar CSE a la expresión
            const expr_cse = try applyCSEToExpr(allocator, expr_stmt, cse_table);
            const expr = expr_cse orelse expr_stmt;
            
            // Si cambió, crear nuevo statement
            if (expr_cse != null) {
                return statement_parser.Statement{ .Expr = expr };
            }
            
            return null;
        },
    }
}

/// Aplicar CSE a un programa completo
/// Retorna un nuevo array de statements optimizado con asignaciones temporales al inicio
pub fn applyCSE(
    allocator: std.mem.Allocator,
    statements: []statement_parser.Statement
) ![]statement_parser.Statement {
    var cse_table = CSETable.init(allocator);
    defer cse_table.deinit();
    
    // Primera pasada: identificar expresiones comunes
    var optimized = std.ArrayList(statement_parser.Statement).init(allocator);
    defer optimized.deinit();
    
    for (statements) |stmt| {
        const stmt_cse = try applyCSEToStatement(allocator, stmt, &cse_table);
        if (stmt_cse) |opt_stmt| {
            try optimized.append(opt_stmt);
        } else {
            try optimized.append(stmt);
        }
    }
    
    // Segunda pasada: agregar asignaciones temporales al inicio
    const temp_assignments = try cse_table.getTempAssignments();
    defer allocator.free(temp_assignments);
    
    var result = std.ArrayList(statement_parser.Statement).init(allocator);
    defer result.deinit();
    
    // Agregar asignaciones temporales primero
    for (temp_assignments) |temp_stmt| {
        try result.append(temp_stmt);
    }
    
    // Agregar statements optimizados
    for (optimized.items) |stmt| {
        try result.append(stmt);
    }
    
    return try result.toOwnedSlice();
}

