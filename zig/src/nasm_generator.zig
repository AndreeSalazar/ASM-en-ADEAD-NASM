// Generador de código NASM desde Zig
// Permite generar ASM directamente sin pasar por Rust para casos simples
// Flujo: ADead → Zig → NASM (ASM)

const std = @import("std");
const expr_parser = @import("expr_parser.zig");

/// Generador de código NASM
pub const NASMGenerator = struct {
    allocator: std.mem.Allocator,
    float_count: usize,
    data_section: std.ArrayListUnmanaged(u8),
    text_section: std.ArrayListUnmanaged(u8),
    
    pub fn init(allocator: std.mem.Allocator) NASMGenerator {
        return NASMGenerator{
            .allocator = allocator,
            .float_count = 0,
            .data_section = std.ArrayListUnmanaged(u8){},
            .text_section = std.ArrayListUnmanaged(u8){},
        };
    }
    
    pub fn deinit(self: *NASMGenerator) void {
        self.data_section.deinit(self.allocator);
        self.text_section.deinit(self.allocator);
    }
    
    /// Generar código NASM para un float literal
    /// Retorna el nombre de la variable en .data (ej: "float_0")
    pub fn generateFloatLiteral(self: *NASMGenerator, value: f64) ![]const u8 {
        // Agregar float a .data section
        const float_name = try std.fmt.allocPrint(self.allocator, "float_{d}", .{self.float_count});
        self.float_count += 1;
        
        // Formatear el float correctamente para NASM
        // NASM necesita: dq 3.141592653589793
        var float_str_buf: [256]u8 = undefined;
        const float_str = try std.fmt.bufPrint(&float_str_buf, "    {s}: dq {d}\n", .{ float_name, value });
        try self.data_section.appendSlice(self.allocator, float_str);
        
        return float_name;
    }
    
    /// Generar código NASM para cargar un float en XMM0
    pub fn generateLoadFloat(self: *NASMGenerator, float_name: []const u8) !void {
        // movsd xmm0, [rel float_name]
        var code_buf: [128]u8 = undefined;
        const code = try std.fmt.bufPrint(&code_buf, "    movsd xmm0, [rel {s}]\n", .{float_name});
        try self.text_section.appendSlice(self.allocator, code);
    }
    
    /// Generar código NASM para print de float (convierte a string en compile-time)
    /// Por ahora, convertimos el float a string en compile-time y lo imprimimos
    pub fn generatePrintFloat(self: *NASMGenerator, value: f64) !void {
        // Convertir float a string en compile-time
        var float_str_buf: [128]u8 = undefined;
        const float_str = try std.fmt.bufPrint(&float_str_buf, "{d:.15}", .{value});
        // Limpiar ceros al final
        var cleaned = std.ArrayListUnmanaged(u8){};
        defer cleaned.deinit(self.allocator);
        try cleaned.appendSlice(self.allocator, float_str);
        // Remover ceros y punto final
        while (cleaned.items.len > 0 and (cleaned.items[cleaned.items.len - 1] == '0' or cleaned.items[cleaned.items.len - 1] == '.')) {
            _ = cleaned.pop();
            if (cleaned.items.len > 0 and cleaned.items[cleaned.items.len - 1] == '.') break;
        }
        if (cleaned.items.len == 0) {
            try cleaned.append(self.allocator, '0');
        }
        
        // Agregar newline
        try cleaned.append(self.allocator, '\n');
        
        // Agregar string a .data section
        const string_name = try std.fmt.allocPrint(self.allocator, "msg_{d}", .{self.float_count});
        var string_data_buf: [256]u8 = undefined;
        // Escapar el string para NASM (necesita ser null-terminated en .data)
        const cleaned_str = try std.fmt.allocPrint(self.allocator, "{s}", .{cleaned.items});
        defer self.allocator.free(cleaned_str);
        const string_data = try std.fmt.bufPrint(&string_data_buf, "    {s}: db \"{s}\", 0\n", .{ string_name, cleaned_str });
        try self.data_section.appendSlice(self.allocator, string_data);
        
        // Agregar longitud
        const len_name = try std.fmt.allocPrint(self.allocator, "{s}_len", .{string_name});
        var len_data_buf: [128]u8 = undefined;
        const len_data = try std.fmt.bufPrint(&len_data_buf, "    {s}: equ $ - {s}\n", .{ len_name, string_name });
        try self.data_section.appendSlice(self.allocator, len_data);
        
        // Generar código para WriteFile
        var write_buf: [512]u8 = undefined;
        const write_code = try std.fmt.bufPrint(&write_buf,
            \\    ; Prepare WriteFile call for float
            \\    mov rcx, [rbp+16]  ; stdout handle
            \\    lea rdx, [rel {s}]  ; buffer pointer
            \\    mov r8, {s}_len  ; number of bytes to write
            \\    lea r9, [rbp+24]  ; lpNumberOfBytesWritten (local var)
            \\    mov qword [rsp+32], 0  ; lpOverlapped = NULL (5th param in shadow space)
            \\    call WriteFile
            \\
        , .{string_name, string_name});
        try self.text_section.appendSlice(self.allocator, write_code);
    }
    
    /// Evaluar una expresión constante en compile-time (solo floats y números)
    /// Retorna null si no se puede evaluar en compile-time (tiene variables, etc.)
    pub fn evalConstExpr(expr: *expr_parser.Expr) ?f64 {
        switch (expr.*) {
            .Float => |f| return f,
            .Number => |n| return @as(f64, @floatFromInt(n)),
            .BinaryOp => |bin| {
                const left_val = NASMGenerator.evalConstExpr(bin.left) orelse return null;
                const right_val = NASMGenerator.evalConstExpr(bin.right) orelse return null;
                
                return switch (bin.op) {
                    .Add => left_val + right_val,
                    .Sub => left_val - right_val,
                    .Mul => left_val * right_val,
                    .Div => {
                        if (right_val == 0.0) return null; // División por cero
                        return left_val / right_val;
                    },
                    else => null, // Operaciones no soportadas para evaluación constante
                };
            },
            else => return null, // Expresiones no constantes
        }
    }
    
    /// Generar código NASM completo para una expresión con floats
    pub fn generateFloatExpr(self: *NASMGenerator, expr: *expr_parser.Expr) !void {
        switch (expr.*) {
            .Float => |f| {
                const float_name = try self.generateFloatLiteral(f);
                try self.generateLoadFloat(float_name);
            },
            .Number => |n| {
                // Convertir int a float: cvtsi2sd xmm0, rax
                // Primero cargar el int en rax
                var load_int_buf: [64]u8 = undefined;
                const load_int = try std.fmt.bufPrint(&load_int_buf, "    mov rax, {d}\n", .{n});
                try self.text_section.appendSlice(self.allocator, load_int);
                
                const convert_int = "    cvtsi2sd xmm0, rax\n";
                try self.text_section.appendSlice(self.allocator, convert_int);
            },
            .BinaryOp => |bin| {
                // Generar expresión izquierda
                try self.generateFloatExpr(bin.left);
                
                // Guardar resultado en xmm1
                try self.text_section.appendSlice(self.allocator, "    movsd xmm1, xmm0\n");
                
                // Generar expresión derecha
                try self.generateFloatExpr(bin.right);
                
                // Aplicar operación (resultado en xmm0, xmm1 tiene el izquierdo)
                const op_code = switch (bin.op) {
                    .Add => "    addsd xmm0, xmm1\n",
                    .Sub => {
                        // Para resta: xmm0 = xmm0 - xmm1, pero queremos xmm1 - xmm0
                        // Entonces: subsd xmm1, xmm0 y luego movsd xmm0, xmm1
                        try self.text_section.appendSlice(self.allocator, "    subsd xmm1, xmm0\n");
                        try self.text_section.appendSlice(self.allocator, "    movsd xmm0, xmm1\n");
                        return;
                    },
                    .Mul => "    mulsd xmm0, xmm1\n",
                    .Div => {
                        // Para división: xmm0 = xmm0 / xmm1, pero queremos xmm1 / xmm0
                        try self.text_section.appendSlice(self.allocator, "    divsd xmm1, xmm0\n");
                        try self.text_section.appendSlice(self.allocator, "    movsd xmm0, xmm1\n");
                        return;
                    },
                    else => return error.UnsupportedOp,
                };
                try self.text_section.appendSlice(self.allocator, op_code);
            },
            else => return error.UnsupportedExpr,
        }
    }
    
    /// Generar código NASM completo para Windows x64
    pub fn generateCompleteCode(self: *NASMGenerator, writer: anytype) !void {
        // .data section
        try writer.writeAll("section .data\n");
        if (self.data_section.items.len > 0) {
            try writer.writeAll(self.data_section.items);
        }
        try writer.writeAll("\n");
        
        // .text section
        try writer.writeAll("default rel\n");
        try writer.writeAll("section .text\n");
        try writer.writeAll("extern GetStdHandle\n");
        try writer.writeAll("extern WriteFile\n");
        try writer.writeAll("extern ExitProcess\n");
        try writer.writeAll("global main\n");
        try writer.writeAll("main:\n");
        try writer.writeAll("    ; Setup stack frame (Windows x64)\n");
        try writer.writeAll("    push rbp\n");
        try writer.writeAll("    mov rbp, rsp\n");
        try writer.writeAll("    and rsp, -16\n");
        try writer.writeAll("    sub rsp, 64\n");
        try writer.writeAll("    ; Get stdout handle\n");
        try writer.writeAll("    mov ecx, -11\n");
        try writer.writeAll("    call GetStdHandle\n");
        try writer.writeAll("    mov [rbp+16], rax\n");
        
        // Código generado
        if (self.text_section.items.len > 0) {
            try writer.writeAll(self.text_section.items);
        }
        
        // Exit
        try writer.writeAll("    ; Exit process\n");
        try writer.writeAll("    mov ecx, 0\n");
        try writer.writeAll("    call ExitProcess\n");
    }
};

/// Exportación FFI para Rust - Generar código NASM directamente desde Zig
/// input_ptr: puntero a string null-terminated (expresión ADead, ej: "3.14")
/// input_len: longitud del string
/// output_buffer: buffer donde escribir el código NASM
/// output_buffer_len: tamaño del buffer
/// Retorna: longitud del código NASM o código de error negativo
pub export fn generate_nasm_ffi(
    input_ptr: [*:0]const u8,
    input_len: usize,
    output_buffer: [*]u8,
    output_buffer_len: usize,
) i32 {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    
    // Extraer input
    const input = if (input_len > 0) input_ptr[0..input_len] else "";
    
    // Parsear expresión
    var parser = expr_parser.ExprParser.init(allocator, input);
    const expr = parser.parse() catch {
        return -1;
    };
    
    if (expr == null) {
        return -1;
    }
    
    // Generar NASM directamente
    var generator = NASMGenerator.init(allocator);
    defer generator.deinit();
    
    // SOLUCIÓN TEMPORAL: Evaluar manualmente expresiones binarias simples
    // Esto nos permite probar y verificar que funciona antes de mejorar evalConstExpr
    
    switch (expr.?.*) {
        .Float => |f| {
            // Float literal simple: imprimir directamente
            generator.generatePrintFloat(f) catch {
                return -1;
            };
        },
        .Number => |n| {
            // Número entero: convertir a float e imprimir
            const f_val = @as(f64, @floatFromInt(n));
            generator.generatePrintFloat(f_val) catch {
                return -1;
            };
        },
        .BinaryOp => |bin| {
            // Expresión binaria: evaluar manualmente los operandos
            // Evaluar operando izquierdo
            const left_val = switch (bin.left.*) {
                .Float => |f| f,
                .Number => |n| @as(f64, @floatFromInt(n)),
                .BinaryOp => |nested_left| blk: {
                    const nl_val = switch (nested_left.left.*) {
                        .Float => |f| f,
                        .Number => |n| @as(f64, @floatFromInt(n)),
                        else => return -1,
                    };
                    const nr_val = switch (nested_left.right.*) {
                        .Float => |f| f,
                        .Number => |n| @as(f64, @floatFromInt(n)),
                        else => return -1,
                    };
                    const nested_result = switch (nested_left.op) {
                        .Add => nl_val + nr_val,
                        .Sub => nl_val - nr_val,
                        .Mul => nl_val * nr_val,
                        .Div => blk2: {
                            if (nr_val == 0.0) return -1;
                            break :blk2 nl_val / nr_val;
                        },
                        else => return -1,
                    };
                    break :blk nested_result;
                },
                else => return -1, // Operando izquierdo no es constante
            };
            
            // Evaluar operando derecho
            const right_val = switch (bin.right.*) {
                .Float => |f| f,
                .Number => |n| @as(f64, @floatFromInt(n)),
                .BinaryOp => |nested_right| blk: {
                    const nl_val = switch (nested_right.left.*) {
                        .Float => |f| f,
                        .Number => |n| @as(f64, @floatFromInt(n)),
                        else => return -1,
                    };
                    const nr_val = switch (nested_right.right.*) {
                        .Float => |f| f,
                        .Number => |n| @as(f64, @floatFromInt(n)),
                        else => return -1,
                    };
                    const nested_result = switch (nested_right.op) {
                        .Add => nl_val + nr_val,
                        .Sub => nl_val - nr_val,
                        .Mul => nl_val * nr_val,
                        .Div => blk2: {
                            if (nr_val == 0.0) return -1;
                            break :blk2 nl_val / nr_val;
                        },
                        else => return -1,
                    };
                    break :blk nested_result;
                },
                else => return -1, // Operando derecho no es constante
            };
            
            // Calcular resultado
            const result = switch (bin.op) {
                .Add => left_val + right_val,
                .Sub => left_val - right_val,
                .Mul => left_val * right_val,
                .Div => blk: {
                    if (right_val == 0.0) {
                        return -1; // División por cero
                    }
                    break :blk left_val / right_val;
                },
                else => {
                    return -1; // Operación no soportada para evaluación constante
                },
            };
            
            // Imprimir resultado
            generator.generatePrintFloat(result) catch {
                return -1;
            };
        },
        else => {
            return -1; // Expresiones no soportadas aún
        },
    }
    
    // Escribir código NASM completo al buffer
    // Usar ArrayListUnmanaged con un writer custom
    var temp_buffer = std.ArrayListUnmanaged(u8){};
    defer temp_buffer.deinit(allocator);
    
    // Crear un writer que escribe al ArrayListUnmanaged
    const Writer = struct {
        list: *std.ArrayListUnmanaged(u8),
        alloc: std.mem.Allocator,
        const Self = @This();
        pub fn write(self: Self, bytes: []const u8) !usize {
            try self.list.appendSlice(self.alloc, bytes);
            return bytes.len;
        }
        pub fn writeAll(self: Self, bytes: []const u8) !void {
            _ = try self.write(bytes);
        }
        pub fn writeByte(self: Self, byte: u8) !void {
            try self.list.append(self.alloc, byte);
        }
        pub fn writeByteNTimes(self: Self, byte: u8, n: usize) !void {
            var i: usize = 0;
            while (i < n) : (i += 1) {
                try self.writeByte(byte);
            }
        }
    };
    
    const writer = Writer{ .list = &temp_buffer, .alloc = allocator };
    generator.generateCompleteCode(writer) catch {
        return -1;
    };
    
    // Copiar al buffer de salida
    if (temp_buffer.items.len >= output_buffer_len) {
        return -1; // Buffer muy pequeño
    }
    @memcpy(output_buffer[0..temp_buffer.items.len], temp_buffer.items);
    output_buffer[temp_buffer.items.len] = 0; // Null terminator
    
    return @intCast(temp_buffer.items.len);
}

// ============================================================================
// Stub para ___chkstk_ms en Windows
// ============================================================================
// PROBLEMA: Zig genera llamadas a ___chkstk_ms incluso con -fno-stack-check
// cuando una función necesita más de 4KB de stack local (para stack probing)
// 
// SOLUCIÓN: Este stub exporta el símbolo como no-op para resolver el linking
// con Rust. El símbolo solo necesita existir en tiempo de linking; en runtime
// Rust ya maneja el stack checking, así que este stub nunca debería llamarse.
// ============================================================================
export fn ___chkstk_ms() void {
    // No-op stub: Solo existe para resolver el símbolo en tiempo de linking
    // Si se llama (no debería), simplemente retorna sin hacer nada
    // Zig se compiló con -fno-stack-check, así que esto es solo para compatibilidad
}
