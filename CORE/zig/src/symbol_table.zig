// Tabla de símbolos para variables en Zig
// Maneja asignación de offsets en stack para variables locales
// Flujo: ADead → Zig → NASM con variables correctamente guardadas/cargadas

const std = @import("std");

/// Ubicación de una variable en el stack
pub const StackLocation = struct {
    offset: i32,  // Offset desde rbp (negativo: rbp-8, rbp-16, etc.)
    
    pub fn format(self: StackLocation, comptime fmt: []const u8, options: std.fmt.FormatOptions, writer: anytype) !void {
        _ = fmt;
        _ = options;
        try std.fmt.format(writer, "rbp-{d}", .{self.offset});
    }
};

/// Tabla de símbolos para variables
pub const SymbolTable = struct {
    variables: std.StringHashMap(StackLocation),
    next_offset: i32,
    allocator: std.mem.Allocator,
    
    pub fn init(allocator: std.mem.Allocator) SymbolTable {
        return SymbolTable{
            .variables = std.StringHashMap(StackLocation).init(allocator),
            .next_offset = 0,
            .allocator = allocator,
        };
    }
    
    pub fn deinit(self: *SymbolTable) void {
        self.variables.deinit();
    }
    
    /// Asignar espacio para una nueva variable
    /// Retorna la ubicación en stack (offset desde rbp)
    pub fn allocate(self: *SymbolTable, name: []const u8) !StackLocation {
        // Verificar si ya existe
        if (self.variables.get(name)) |existing| {
            return existing; // Variable ya existe, retornar ubicación existente
        }
        
        // Asignar nuevo offset
        // Variables se guardan en orden: primera variable en rbp-8, segunda en rbp-16, etc.
        // +16 para espacio de parámetros y alineación
        const offset = self.next_offset + 16;
        self.next_offset += 8; // 8 bytes por variable (int64_t)
        
        const location = StackLocation{ .offset = offset };
        
        // Guardar en tabla
        try self.variables.put(name, location);
        
        return location;
    }
    
    /// Obtener ubicación de una variable existente
    pub fn get(self: *SymbolTable, name: []const u8) ?StackLocation {
        return self.variables.get(name);
    }
    
    /// Obtener tamaño total del stack necesario para todas las variables
    pub fn getStackSize(self: *SymbolTable) i32 {
        // Tamaño mínimo: espacio para variables + espacio para parámetros (16 bytes) + alineación
        const var_size = self.next_offset;
        const total = var_size + 16; // +16 para parámetros y alineación
        // Redondear a múltiplo de 16 para alineación
        return ((total + 15) / 16) * 16;
    }
    
    /// Verificar si una variable existe
    pub fn contains(self: *SymbolTable, name: []const u8) bool {
        return self.variables.contains(name);
    }
    
    /// Obtener número de variables
    pub fn count(self: *SymbolTable) usize {
        return self.variables.count();
    }
};

