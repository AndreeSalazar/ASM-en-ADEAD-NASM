// Register Allocation Inteligente para Zig
// Asigna variables frecuentemente usadas a registros callee-saved (r12-r15)
// Reduce accesos al stack mejorando performance
// Ejemplo: let x = 5; let y = 10; ... (usar r12, r13 en vez de stack)

const std = @import("std");
const expr_parser = @import("expr_parser.zig");
const statement_parser = @import("statement_parser.zig");

/// Registros callee-saved disponibles en x86-64
/// r12, r13, r14, r15 son registros de propósito general que se preservan entre llamadas
pub const CalleeSavedReg = enum {
    r12,
    r13,
    r14,
    r15,
    
    pub fn name(self: CalleeSavedReg) []const u8 {
        return switch (self) {
            .r12 => "r12",
            .r13 => "r13",
            .r14 => "r14",
            .r15 => "r15",
        };
    }
};

/// Información de uso de una variable
const VariableUsage = struct {
    read_count: usize,
    write_count: usize,
    name: []const u8,
    
    /// Calcular score de prioridad (mayor = más importante para asignar a registro)
    pub fn priority(self: VariableUsage) usize {
        // Variables leídas múltiples veces tienen mayor prioridad
        return self.read_count * 2 + self.write_count;
    }
};

/// Asignación de variable a registro o stack
pub const VariableLocation = union(enum) {
    Register: CalleeSavedReg,
    Stack: i32, // Offset desde rbp
    
    pub fn format(self: VariableLocation, comptime fmt: []const u8, options: std.fmt.FormatOptions, writer: anytype) !void {
        _ = fmt;
        _ = options;
        switch (self) {
            .Register => |reg| try writer.print("{s}", .{reg.name()}),
            .Stack => |offset| try writer.print("[rbp-{d}]", .{offset}),
        }
    }
};

/// Asignador de registros inteligente
pub const RegisterAllocator = struct {
    /// Mapa de variable → ubicación (registro o stack)
    allocations: std.StringHashMap(VariableLocation),
    /// Registros disponibles
    available_regs: std.ArrayList(CalleeSavedReg),
    /// Próximo offset de stack
    next_stack_offset: i32,
    allocator: std.mem.Allocator,
    
    pub fn init(allocator: std.mem.Allocator) RegisterAllocator {
        var available_regs = std.ArrayList(CalleeSavedReg).init(allocator);
        available_regs.append(.r12) catch {};
        available_regs.append(.r13) catch {};
        available_regs.append(.r14) catch {};
        available_regs.append(.r15) catch {};
        
        return RegisterAllocator{
            .allocations = std.StringHashMap(VariableLocation).init(allocator),
            .available_regs = available_regs,
            .next_stack_offset = 8, // Empezar en rbp-8
            .allocator = allocator,
        };
    }
    
    pub fn deinit(self: *RegisterAllocator) void {
        self.allocations.deinit();
        self.available_regs.deinit();
    }
    
    /// Analizar uso de variables en un programa
    fn analyzeVariableUsage(
        allocator: std.mem.Allocator,
        statements: []statement_parser.Statement
    ) !std.StringHashMap(VariableUsage) {
        var usage = std.StringHashMap(VariableUsage).init(allocator);
        errdefer usage.deinit();
        
        for (statements) |stmt| {
            switch (stmt) {
                .Let => |let_stmt| {
                    // Variable escrita
                    const entry = usage.getPtr(let_stmt.name) orelse blk: {
                        try usage.put(let_stmt.name, VariableUsage{
                            .read_count = 0,
                            .write_count = 0,
                            .name = let_stmt.name,
                        });
                        break :blk usage.getPtr(let_stmt.name).?;
                    };
                    entry.write_count += 1;
                    
                    // Analizar variables leídas en la expresión
                    try analyzeExprReads(allocator, let_stmt.value, &usage);
                },
                .Assign => |assign_stmt| {
                    // Variable escrita
                    const entry = usage.getPtr(assign_stmt.name) orelse blk: {
                        try usage.put(assign_stmt.name, VariableUsage{
                            .read_count = 0,
                            .write_count = 0,
                            .name = assign_stmt.name,
                        });
                        break :blk usage.getPtr(assign_stmt.name).?;
                    };
                    entry.write_count += 1;
                    
                    // Analizar variables leídas en la expresión
                    try analyzeExprReads(allocator, assign_stmt.value, &usage);
                },
                .Print => |print_expr| {
                    // Analizar variables leídas en la expresión
                    try analyzeExprReads(allocator, print_expr, &usage);
                },
                .While => |while_stmt| {
                    // Analizar recursivamente el cuerpo del loop
                    const body_usage = try analyzeVariableUsage(allocator, while_stmt.body);
                    defer body_usage.deinit();
                    
                    // Multiplicar uso por número estimado de iteraciones (conservador: 5)
                    var it = body_usage.iterator();
                    while (it.next()) |entry| {
                        const main_entry = usage.getPtr(entry.key_ptr.*) orelse blk: {
                            try usage.put(entry.key_ptr.*, VariableUsage{
                                .read_count = 0,
                                .write_count = 0,
                                .name = entry.key_ptr.*,
                            });
                            break :blk usage.getPtr(entry.key_ptr.*).?;
                        };
                        main_entry.read_count += entry.value_ptr.read_count * 5;
                        main_entry.write_count += entry.value_ptr.write_count * 5;
                    }
                    
                    // Analizar condición del loop
                    try analyzeExprReads(allocator, while_stmt.condition, &usage);
                },
                .If => |if_stmt| {
                    // Analizar recursivamente ambos cuerpos
                    const then_usage = try analyzeVariableUsage(allocator, if_stmt.then_body);
                    defer then_usage.deinit();
                    
                    var it = then_usage.iterator();
                    while (it.next()) |entry| {
                        const main_entry = usage.getPtr(entry.key_ptr.*) orelse blk: {
                            try usage.put(entry.key_ptr.*, VariableUsage{
                                .read_count = 0,
                                .write_count = 0,
                                .name = entry.key_ptr.*,
                            });
                            break :blk usage.getPtr(entry.key_ptr.*).?;
                        };
                        main_entry.read_count += entry.value_ptr.read_count;
                        main_entry.write_count += entry.value_ptr.write_count;
                    }
                    
                    if (if_stmt.else_body) |else_body| {
                        const else_usage = try analyzeVariableUsage(allocator, else_body);
                        defer else_usage.deinit();
                        
                        var it2 = else_usage.iterator();
                        while (it2.next()) |entry| {
                            const main_entry = usage.getPtr(entry.key_ptr.*) orelse blk: {
                                try usage.put(entry.key_ptr.*, VariableUsage{
                                    .read_count = 0,
                                    .write_count = 0,
                                    .name = entry.key_ptr.*,
                                });
                                break :blk usage.getPtr(entry.key_ptr.*).?;
                            };
                            main_entry.read_count += entry.value_ptr.read_count;
                            main_entry.write_count += entry.value_ptr.write_count;
                        }
                    }
                    
                    // Analizar condición del if
                    try analyzeExprReads(allocator, if_stmt.condition, &usage);
                },
                .Expr => |expr_stmt| {
                    // Analizar variables leídas en la expresión
                    try analyzeExprReads(allocator, expr_stmt, &usage);
                },
            }
        }
        
        return usage;
    }
    
    /// Analizar variables leídas en una expresión
    fn analyzeExprReads(
        allocator: std.mem.Allocator,
        expr: *expr_parser.Expr,
        usage: *std.StringHashMap(VariableUsage)
    ) !void {
        switch (expr.*) {
            .Ident => |name| {
                // Variable leída
                const entry = usage.getPtr(name) orelse blk: {
                    try usage.put(name, VariableUsage{
                        .read_count = 0,
                        .write_count = 0,
                        .name = name,
                    });
                    break :blk usage.getPtr(name).?;
                };
                entry.read_count += 1;
            },
            .BinaryOp => |bin| {
                // Analizar recursivamente ambos operandos
                try analyzeExprReads(allocator, bin.left, usage);
                try analyzeExprReads(allocator, bin.right, usage);
            },
            else => {
                // Number, Float, String no leen variables
            },
        }
    }
    
    /// Asignar ubicaciones a variables basado en su uso
    pub fn allocateRegisters(
        self: *RegisterAllocator,
        statements: []statement_parser.Statement
    ) !void {
        // Analizar uso de variables
        const usage = try analyzeVariableUsage(self.allocator, statements);
        defer usage.deinit();
        
        // Convertir a array y ordenar por prioridad
        var usage_list = std.ArrayList(VariableUsage).init(self.allocator);
        defer usage_list.deinit();
        
        var it = usage.iterator();
        while (it.next()) |entry| {
            try usage_list.append(entry.value_ptr.*);
        }
        
        // Ordenar por prioridad (mayor primero)
        const SortContext = struct {
            pub fn lessThan(context: void, a: VariableUsage, b: VariableUsage) bool {
                _ = context;
                return a.priority() > b.priority();
            }
        };
        std.mem.sort(VariableUsage, usage_list.items, {}, SortContext.lessThan);
        
        // Asignar registros a las variables más usadas
        var reg_index: usize = 0;
        for (usage_list.items) |var_usage| {
            if (reg_index < self.available_regs.items.len) {
                // Asignar a registro
                const reg = self.available_regs.items[reg_index];
                try self.allocations.put(var_usage.name, VariableLocation{ .Register = reg });
                reg_index += 1;
            } else {
                // Asignar a stack
                const offset = self.next_stack_offset;
                try self.allocations.put(var_usage.name, VariableLocation{ .Stack = offset });
                self.next_stack_offset += 8; // 8 bytes por variable (int64_t)
            }
        }
    }
    
    /// Obtener la ubicación de una variable
    pub fn getLocation(self: *RegisterAllocator, name: []const u8) ?VariableLocation {
        return self.allocations.get(name);
    }
    
    /// Obtener el tamaño total del stack necesario
    pub fn getStackSize(self: *RegisterAllocator) i32 {
        const raw_size = self.next_stack_offset - 8; // Excluir el primer offset de 8
        if (raw_size == 0) return 0;
        // Asegurar que el tamaño del stack sea múltiplo de 16 para alineación
        return ((raw_size + 15) / 16) * 16;
    }
};

/// Aplicar register allocation a un programa
/// Retorna un RegisterAllocator con las asignaciones
pub fn applyRegisterAllocation(
    allocator: std.mem.Allocator,
    statements: []statement_parser.Statement
) !RegisterAllocator {
    var reg_alloc = RegisterAllocator.init(allocator);
    errdefer reg_alloc.deinit();
    
    try reg_alloc.allocateRegisters(statements);
    
    return reg_alloc;
}

