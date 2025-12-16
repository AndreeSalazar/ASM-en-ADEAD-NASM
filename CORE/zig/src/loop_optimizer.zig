// Loop Invariant Code Motion (LICM) para Zig
// Mueve código fuera de loops cuando no depende de variables del loop
// Ejemplo: while i < 10 { let x = 5 + 3; ... } → let x = 8; while i < 10 { ... }

const std = @import("std");
const expr_parser = @import("expr_parser.zig");
const statement_parser = @import("statement_parser.zig");

/// Analizar qué variables se modifican dentro de un bloque de código
/// Retorna un HashSet con los nombres de variables modificadas
fn analyzeModifiedVars(
    allocator: std.mem.Allocator,
    statements: []statement_parser.Statement
) !std.StringHashMap(void) {
    var modified = std.StringHashMap(void).init(allocator);
    errdefer modified.deinit();
    
    for (statements) |stmt| {
        switch (stmt) {
            .Let => |let_stmt| {
                // Variable declarada
                try modified.put(let_stmt.name, {});
            },
            .Assign => |assign_stmt| {
                // Variable reasignada
                try modified.put(assign_stmt.name, {});
            },
            .While => |while_stmt| {
                // Analizar recursivamente el cuerpo del loop
                const body_modified = try analyzeModifiedVars(allocator, while_stmt.body);
                defer body_modified.deinit();
                
                // Agregar todas las variables modificadas en el cuerpo
                var it = body_modified.iterator();
                while (it.next()) |entry| {
                    try modified.put(entry.key_ptr.*, {});
                }
            },
            .If => |if_stmt| {
                // Analizar recursivamente ambos cuerpos
                const then_modified = try analyzeModifiedVars(allocator, if_stmt.then_body);
                defer then_modified.deinit();
                
                var it = then_modified.iterator();
                while (it.next()) |entry| {
                    try modified.put(entry.key_ptr.*, {});
                }
                
                if (if_stmt.else_body) |else_body| {
                    const else_modified = try analyzeModifiedVars(allocator, else_body);
                    defer else_modified.deinit();
                    
                    var it2 = else_modified.iterator();
                    while (it2.next()) |entry| {
                        try modified.put(entry.key_ptr.*, {});
                    }
                }
            },
            else => {
                // Print, Expr no modifican variables
            },
        }
    }
    
    return modified;
}

/// Analizar qué variables se leen en una expresión
/// Retorna un HashSet con los nombres de variables leídas
fn analyzeReadVars(
    allocator: std.mem.Allocator,
    expr: *expr_parser.Expr
) !std.StringHashMap(void) {
    var read = std.StringHashMap(void).init(allocator);
    errdefer read.deinit();
    
    switch (expr.*) {
        .Ident => |name| {
            // Variable leída
            try read.put(name, {});
        },
        .BinaryOp => |bin| {
            // Analizar recursivamente ambos operandos
            const left_read = try analyzeReadVars(allocator, bin.left);
            defer left_read.deinit();
            
            var it = left_read.iterator();
            while (it.next()) |entry| {
                try read.put(entry.key_ptr.*, {});
            }
            
            const right_read = try analyzeReadVars(allocator, bin.right);
            defer right_read.deinit();
            
            var it2 = right_read.iterator();
            while (it2.next()) |entry| {
                try read.put(entry.key_ptr.*, {});
            }
        },
        else => {
            // Number, Float, String no leen variables
        },
    }
    
    return read;
}

/// Verificar si un statement es invariante respecto a un conjunto de variables modificadas
fn isInvariant(
    allocator: std.mem.Allocator,
    stmt: statement_parser.Statement,
    modified_vars: *std.StringHashMap(void)
) !bool {
    switch (stmt) {
        .Let => |let_stmt| {
            // Verificar si la expresión depende de variables modificadas
            const read_vars = try analyzeReadVars(allocator, let_stmt.value);
            defer read_vars.deinit();
            
            // Si lee alguna variable modificada, no es invariante
            var it = read_vars.iterator();
            while (it.next()) |entry| {
                if (modified_vars.contains(entry.key_ptr.*)) {
                    return false;
                }
            }
            
            // Si la variable misma está modificada, no es invariante
            if (modified_vars.contains(let_stmt.name)) {
                return false;
            }
            
            return true;
        },
        .Print => |print_expr| {
            // Verificar si la expresión depende de variables modificadas
            const read_vars = try analyzeReadVars(allocator, print_expr);
            defer read_vars.deinit();
            
            // Si lee alguna variable modificada, no es invariante
            var it = read_vars.iterator();
            while (it.next()) |entry| {
                if (modified_vars.contains(entry.key_ptr.*)) {
                    return false;
                }
            }
            
            return true;
        },
        .Assign => {
            // Las asignaciones modifican variables, no son invariantes
            return false;
        },
        .While => {
            // Los loops anidados no se mueven (por ahora)
            return false;
        },
        .If => {
            // Los ifs pueden tener efectos secundarios, no se mueven (por ahora)
            return false;
        },
        .Expr => |expr_stmt| {
            // Verificar si la expresión depende de variables modificadas
            const read_vars = try analyzeReadVars(allocator, expr_stmt);
            defer read_vars.deinit();
            
            // Si lee alguna variable modificada, no es invariante
            var it = read_vars.iterator();
            while (it.next()) |entry| {
                if (modified_vars.contains(entry.key_ptr.*)) {
                    return false;
                }
            }
            
            return true;
        },
    }
}

/// Aplicar Loop Invariant Code Motion a un while loop
/// Retorna: (statements antes del loop, loop optimizado)
pub fn optimizeWhileLoop(
    allocator: std.mem.Allocator,
    while_stmt: statement_parser.Statement.While
) !struct { []statement_parser.Statement, statement_parser.Statement } {
    // Analizar qué variables se modifican dentro del loop
    const modified_vars = try analyzeModifiedVars(allocator, while_stmt.body);
    defer modified_vars.deinit();
    
    // Separar código invariante del código dependiente
    var invariant_statements = std.ArrayList(statement_parser.Statement).init(allocator);
    defer invariant_statements.deinit();
    
    var loop_body = std.ArrayList(statement_parser.Statement).init(allocator);
    defer loop_body.deinit();
    
    for (while_stmt.body) |body_stmt| {
        if (try isInvariant(allocator, body_stmt, &modified_vars)) {
            // Es invariante, mover fuera del loop
            try invariant_statements.append(body_stmt);
        } else {
            // Depende del loop, mantener dentro
            try loop_body.append(body_stmt);
        }
    }
    
    // Crear loop optimizado
    const optimized_loop = statement_parser.Statement{
        .While = .{
            .condition = while_stmt.condition,
            .body = try allocator.dupe(statement_parser.Statement, loop_body.items),
        },
    };
    
    // Retornar statements invariantes y loop optimizado
    return .{
        try allocator.dupe(statement_parser.Statement, invariant_statements.items),
        optimized_loop,
    };
}

/// Aplicar Loop Invariant Code Motion a un programa completo
/// Retorna un nuevo array de statements optimizado
pub fn applyLICM(
    allocator: std.mem.Allocator,
    statements: []statement_parser.Statement
) ![]statement_parser.Statement {
    var result = std.ArrayList(statement_parser.Statement).init(allocator);
    defer result.deinit();
    
    for (statements) |stmt| {
        switch (stmt) {
            .While => |while_stmt| {
                // Optimizar el loop
                const optimized = try optimizeWhileLoop(allocator, while_stmt);
                defer allocator.free(optimized[0]);
                
                // Agregar statements invariantes antes del loop
                for (optimized[0]) |invariant_stmt| {
                    try result.append(invariant_stmt);
                }
                
                // Agregar loop optimizado
                try result.append(optimized[1]);
            },
            .If => |if_stmt| {
                // Optimizar loops dentro del if
                var optimized_then = std.ArrayList(statement_parser.Statement).init(allocator);
                defer optimized_then.deinit();
                
                var then_invariants = std.ArrayList(statement_parser.Statement).init(allocator);
                defer then_invariants.deinit();
                
                for (if_stmt.then_body) |then_stmt| {
                    switch (then_stmt) {
                        .While => |then_while| {
                            const optimized = try optimizeWhileLoop(allocator, then_while);
                            defer allocator.free(optimized[0]);
                            
                            for (optimized[0]) |invariant_stmt| {
                                try then_invariants.append(invariant_stmt);
                            }
                            try optimized_then.append(optimized[1]);
                        },
                        else => {
                            try optimized_then.append(then_stmt);
                        },
                    }
                }
                
                // Manejar else body si existe
                var optimized_else: ?[]statement_parser.Statement = null;
                if (if_stmt.else_body) |else_body| {
                    var else_list = std.ArrayList(statement_parser.Statement).init(allocator);
                    defer else_list.deinit();
                    
                    for (else_body) |else_stmt| {
                        switch (else_stmt) {
                            .While => |else_while| {
                                const optimized = try optimizeWhileLoop(allocator, else_while);
                                defer allocator.free(optimized[0]);
                                
                                // Agregar invariantes antes del if (compartidos)
                                for (optimized[0]) |invariant_stmt| {
                                    try then_invariants.append(invariant_stmt);
                                }
                                try else_list.append(optimized[1]);
                            },
                            else => {
                                try else_list.append(else_stmt);
                            },
                        }
                    }
                    
                    optimized_else = try allocator.dupe(statement_parser.Statement, else_list.items);
                }
                
                // Agregar invariantes antes del if
                for (then_invariants.items) |invariant_stmt| {
                    try result.append(invariant_stmt);
                }
                
                // Agregar if optimizado
                try result.append(statement_parser.Statement{
                    .If = .{
                        .condition = if_stmt.condition,
                        .then_body = try allocator.dupe(statement_parser.Statement, optimized_then.items),
                        .else_body = optimized_else,
                    },
                });
            },
            else => {
                // Otros statements se mantienen igual
                try result.append(stmt);
            },
        }
    }
    
    return try result.toOwnedSlice();
}

