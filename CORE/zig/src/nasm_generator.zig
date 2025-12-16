// Generador de código NASM desde Zig
// Permite generar ASM directamente sin pasar por Rust para casos simples
// Flujo: ADead → Zig → NASM (ASM)

const std = @import("std");
const expr_parser = @import("expr_parser.zig");
const statement_parser = @import("statement_parser.zig");
const symbol_table = @import("symbol_table.zig");
const optimizer = @import("optimizer.zig");
const constant_propagation = @import("constant_propagation.zig");
const cse = @import("cse.zig");
const loop_optimizer = @import("loop_optimizer.zig");
const register_allocator = @import("register_allocator.zig");

/// Generador de código NASM
pub const NASMGenerator = struct {
    allocator: std.mem.Allocator,
    float_count: usize,
    label_count: usize,
    symbol_table: symbol_table.SymbolTable,
    constant_table: constant_propagation.ConstantTable,
    register_alloc: ?register_allocator.RegisterAllocator,
    data_section: std.ArrayListUnmanaged(u8),
    text_section: std.ArrayListUnmanaged(u8),
    
    pub fn init(allocator: std.mem.Allocator) NASMGenerator {
        return NASMGenerator{
            .allocator = allocator,
            .float_count = 0,
            .label_count = 0,
            .symbol_table = symbol_table.SymbolTable.init(allocator),
            .constant_table = constant_propagation.ConstantTable.init(allocator),
            .register_alloc = null,
            .data_section = std.ArrayListUnmanaged(u8){},
            .text_section = std.ArrayListUnmanaged(u8){},
        };
    }
    
    pub fn deinit(self: *NASMGenerator) void {
        self.symbol_table.deinit();
        self.constant_table.deinit();
        if (self.register_alloc) |*reg_alloc| {
            reg_alloc.deinit();
        }
        self.data_section.deinit(self.allocator);
        self.text_section.deinit(self.allocator);
    }
    
    /// Generar un label único
    fn newLabel(self: *NASMGenerator) ![]const u8 {
        const label = try std.fmt.allocPrint(self.allocator, "label_{d}", .{self.label_count});
        self.label_count += 1;
        return label;
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
    
    /// Generar código para una expresión que devuelve un valor en RAX
    fn generateExpr(self: *NASMGenerator, expr: *expr_parser.Expr) !void {
        switch (expr.*) {
            .Number => |n| {
                // CRÍTICO: Verificar tamaño ANTES de agregar código
                const len_before = self.text_section.items.len;
                
                // Generar código
                var buf: [64]u8 = undefined;
                const code = try std.fmt.bufPrint(&buf, "    mov rax, {d}\n", .{n});
                
                // CRÍTICO: Agregar código y verificar que realmente se agregó
                try self.text_section.appendSlice(self.allocator, code);
                const len_after = self.text_section.items.len;
                
                // Si no se agregó código, hay un problema con el allocator
                if (len_after == len_before) {
                    // appendSlice falló silenciosamente - retornar error explícito
                    return error.OutOfMemory;
                }
            },
            .Float => |f| {
                const float_name = try self.generateFloatLiteral(f);
                try self.generateLoadFloat(float_name);
                // Para comparaciones, necesitamos convertir float a int o comparar floats
                // Por ahora, asumimos que las comparaciones son con enteros
            },
            .Ident => |name| {
                // Cargar variable desde registro o stack usando register allocator si está disponible
                if (self.register_alloc) |*reg_alloc| {
                    if (reg_alloc.getLocation(name)) |var_location| {
                        switch (var_location) {
                            .Register => |reg| {
                                // Variable está en un registro callee-saved
                var buf: [128]u8 = undefined;
                                const code = try std.fmt.bufPrint(&buf, "    mov rax, {s}    ; Cargar variable {s} desde registro\n", .{ reg.name(), name });
                try self.text_section.appendSlice(self.allocator, code);
                            },
                            .Stack => |offset| {
                                // Variable está en stack
                                var buf: [128]u8 = undefined;
                                const code = try std.fmt.bufPrint(&buf, "    mov rax, [rbp-{d}]    ; Cargar variable {s} desde stack\n", .{ offset, name });
                                try self.text_section.appendSlice(self.allocator, code);
                            },
                        }
                    } else {
                        // Variable no encontrada en register allocator, intentar symbol_table
                        if (self.symbol_table.get(name)) |location| {
                            var buf: [128]u8 = undefined;
                            const code = try std.fmt.bufPrint(&buf, "    mov rax, [rbp-{d}]\n", .{location.offset});
                            try self.text_section.appendSlice(self.allocator, code);
                        } else {
                            // Variable no encontrada - error
                            var buf: [256]u8 = undefined;
                            const code = try std.fmt.bufPrint(&buf, "    ; ERROR: Variable '{s}' no encontrada\n    mov rax, 0\n", .{name});
                            try self.text_section.appendSlice(self.allocator, code);
                        }
                    }
                } else {
                    // Sin register allocator, usar tabla de símbolos tradicional
                    if (self.symbol_table.get(name)) |location| {
                        var buf: [128]u8 = undefined;
                        const code = try std.fmt.bufPrint(&buf, "    mov rax, [rbp-{d}]\n", .{location.offset});
                        try self.text_section.appendSlice(self.allocator, code);
                    } else {
                        // Variable no encontrada - error
                        var buf: [256]u8 = undefined;
                        const code = try std.fmt.bufPrint(&buf, "    ; ERROR: Variable '{s}' no encontrada\n    mov rax, 0\n", .{name});
                        try self.text_section.appendSlice(self.allocator, code);
                    }
                }
            },
            .BinaryOp => |bin| {
                // OPTIMIZACIÓN: Intentar optimizar la expresión binaria
                const optimized = optimizer.optimizeBinaryOp(self.allocator, bin);
                if (optimized) |opt_expr| {
                    // Expresión optimizada - generar código para la expresión optimizada
                    try self.generateExpr(opt_expr);
                    return;
                }
                
                // Verificar si podemos usar shift para multiplicación o división
                const left_val = switch (bin.left.*) {
                    .Number => |n| n,
                    else => null,
                };
                const right_val = switch (bin.right.*) {
                    .Number => |n| n,
                    else => null,
                };
                
                // OPTIMIZACIÓN: x * potencia_de_2 → shl x, log2(potencia)
                if (bin.op == .Mul) {
                    if (optimizer.canUseShiftForMul(bin.op, left_val, right_val)) |shift_amount| {
                        // Determinar qué operando es la variable (no constante)
                        if (right_val != null and left_val == null) {
                            // x * potencia_de_2 → shl x, shift_amount
                            try self.generateExpr(bin.left);
                            var shift_buf: [64]u8 = undefined;
                            const shift_code = try std.fmt.bufPrint(&shift_buf, "    shl rax, {d}\n", .{shift_amount});
                            try self.text_section.appendSlice(self.allocator, shift_code);
                            return;
                        } else if (left_val != null and right_val == null) {
                            // potencia_de_2 * x → shl x, shift_amount
                            try self.generateExpr(bin.right);
                            var shift_buf: [64]u8 = undefined;
                            const shift_code = try std.fmt.bufPrint(&shift_buf, "    shl rax, {d}\n", .{shift_amount});
                            try self.text_section.appendSlice(self.allocator, shift_code);
                            return;
                        }
                    }
                }
                
                // OPTIMIZACIÓN: x / potencia_de_2 → shr x, log2(potencia)
                if (bin.op == .Div) {
                    if (optimizer.canUseShiftForDiv(bin.op, right_val)) |shift_amount| {
                        // x / potencia_de_2 → shr x, shift_amount
                        try self.generateExpr(bin.left);
                        var shift_buf: [64]u8 = undefined;
                        const shift_code = try std.fmt.bufPrint(&shift_buf, "    shr rax, {d}\n", .{shift_amount});
                        try self.text_section.appendSlice(self.allocator, shift_code);
                        return;
                    }
                }
                
                // Generar left
                try self.generateExpr(bin.left);
                try self.text_section.appendSlice(self.allocator, "    push rax\n");
                
                // Generar right
                try self.generateExpr(bin.right);
                try self.text_section.appendSlice(self.allocator, "    pop rbx\n");
                
                // Generar operación
                switch (bin.op) {
                    .Add => {
                        try self.text_section.appendSlice(self.allocator, "    add rax, rbx\n");
                    },
                    .Sub => {
                        try self.text_section.appendSlice(self.allocator, "    sub rbx, rax\n    mov rax, rbx\n");
                    },
                    .Mul => {
                        try self.text_section.appendSlice(self.allocator, "    imul rax, rbx\n");
                    },
                    .Div => {
                        try self.text_section.appendSlice(self.allocator, 
                            \\    mov rdx, 0
                            \\    mov rcx, rax
                            \\    mov rax, rbx
                            \\    div rcx
                            \\
                        );
                    },
                    .Mod => {
                        // Módulo: RAX = RBX % RAX (resto de división)
                        try self.text_section.appendSlice(self.allocator,
                            \\    mov rdx, 0
                            \\    mov rcx, rax
                            \\    mov rax, rbx
                            \\    div rcx
                            \\    mov rax, rdx  ; resto (módulo) en RAX
                            \\
                        );
                    },
                    .Eq => {
                        try self.text_section.appendSlice(self.allocator,
                            \\    cmp rax, rbx
                            \\    sete al
                            \\    movzx rax, al
                            \\
                        );
                    },
                    .Le => {
                        try self.text_section.appendSlice(self.allocator,
                            \\    cmp rbx, rax
                            \\    setle al
                            \\    movzx rax, al
                            \\
                        );
                    },
                    .Lt => {
                        try self.text_section.appendSlice(self.allocator,
                            \\    cmp rbx, rax
                            \\    setl al
                            \\    movzx rax, al
                            \\
                        );
                    },
                    .Ge => {
                        try self.text_section.appendSlice(self.allocator,
                            \\    cmp rbx, rax
                            \\    setge al
                            \\    movzx rax, al
                            \\
                        );
                    },
                    .Gt => {
                        try self.text_section.appendSlice(self.allocator,
                            \\    cmp rbx, rax
                            \\    setg al
                            \\    movzx rax, al
                            \\
                        );
                    },
                    .Ne => {
                        try self.text_section.appendSlice(self.allocator,
                            \\    cmp rax, rbx
                            \\    setne al
                            \\    movzx rax, al
                            \\
                        );
                    },
                }
            },
            .String => |s| {
                // String literal - agregar a .data section
                const string_name = try std.fmt.allocPrint(self.allocator, "msg_{d}", .{self.float_count});
                self.float_count += 1;
                
                var buf: [256]u8 = undefined;
                const data_code = try std.fmt.bufPrint(&buf, "    {s}: db \"{s}\", 0xA\n", .{ string_name, s });
                try self.data_section.appendSlice(self.allocator, data_code);
                
                const len_name = try std.fmt.allocPrint(self.allocator, "{s}_len", .{string_name});
                const len_code = try std.fmt.bufPrint(&buf, "    {s}: equ $ - {s}\n", .{ len_name, string_name });
                try self.data_section.appendSlice(self.allocator, len_code);
                
                // Para strings, retornamos puntero (por ahora no implementado completamente)
                var code_buf: [128]u8 = undefined;
                const code = try std.fmt.bufPrint(&code_buf, "    lea rax, [rel {s}]\n", .{string_name});
                try self.text_section.appendSlice(self.allocator, code);
            },
        }
    }
    
    /// Generar código para imprimir un número desde rax
    fn generatePrintNumberFromRax(self: *NASMGenerator) !void {
        // Crear buffer temporal en stack para conversión
        // Usar espacio después de las variables (stack_size + 32)
        // Esto evita conflictos con variables existentes
        const stack_size = if (self.register_alloc) |*reg_alloc|
            reg_alloc.getStackSize()
        else
            self.symbol_table.getStackSize();
        const buffer_offset = stack_size + 32; // Buffer después de todas las variables
        
        // Algoritmo de conversión: dividir por 10 y guardar dígitos
        // Por simplicidad, usar un enfoque que funciona para números pequeños
        // Para números grandes, necesitaríamos una función más compleja
        
        var code_buf: [1024]u8 = undefined;
        const print_code = try std.fmt.bufPrint(&code_buf,
            \\    ; Convertir número en rax a string e imprimir
            \\    ; Buffer temporal en [rbp-{d}]
            \\    mov rsi, rbp
            \\    sub rsi, {d}        ; rsi = dirección del buffer
            \\    mov rdi, rsi        ; rdi = fin del buffer (empezamos desde el final)
            \\    add rdi, 31         ; último byte del buffer
            \\    mov byte [rdi], 0xA  ; newline al final
            \\    dec rdi
            \\    mov rcx, rax        ; rcx = número a convertir
            \\    mov rbx, 10         ; divisor
            \\
            \\    ; Si el número es 0, manejar caso especial
            \\    cmp rcx, 0
            \\    je .print_zero
            \\
            \\    ; Convertir dígitos
            \\.convert_loop:
            \\    mov rax, rcx
            \\    mov rdx, 0
            \\    div rbx            ; rax = cociente, rdx = resto (dígito)
            \\    add dl, '0'         ; convertir dígito a ASCII
            \\    mov [rdi], dl       ; guardar dígito
            \\    dec rdi
            \\    mov rcx, rax        ; siguiente iteración
            \\    cmp rcx, 0
            \\    jne .convert_loop
            \\
            \\    ; Calcular longitud
            \\    mov rax, rbp
            \\    sub rax, {d}
            \\    add rax, 31         ; fin del buffer
            \\    sub rax, rdi        ; longitud = fin - inicio
            \\    mov r8, rax         ; r8 = longitud
            \\    inc rdi             ; ajustar inicio (rdi apunta un byte antes)
            \\    jmp .print_done
            \\
            \\.print_zero:
            \\    mov byte [rdi], '0'
            \\    mov r8, 2           ; longitud: "0\n"
            \\    mov rdi, rbp
            \\    sub rdi, {d}
            \\    add rdi, 30
            \\
            \\.print_done:
            \\    ; Llamar WriteFile
            \\    mov rcx, [rbp+16]   ; stdout handle
            \\    mov rdx, rdi        ; buffer pointer
            \\    ; r8 ya tiene la longitud
            \\    lea r9, [rbp+24]    ; lpNumberOfBytesWritten
            \\    mov qword [rsp+32], 0  ; lpOverlapped = NULL
            \\    call WriteFile
            \\
        , .{ buffer_offset, buffer_offset, buffer_offset, buffer_offset });
        try self.text_section.appendSlice(self.allocator, print_code);
    }
    
    /// Generar código para un statement
    pub fn generateStatement(self: *NASMGenerator, stmt: statement_parser.Statement) !void {
        switch (stmt) {
            .Print => |expr| {
                // Para números enteros, convertir a string e imprimir
                switch (expr.*) {
                    .Number => |n| {
                        // Cargar número en rax
                        var load_buf: [64]u8 = undefined;
                        const load_code = try std.fmt.bufPrint(&load_buf, "    mov rax, {d}\n", .{n});
                        try self.text_section.appendSlice(self.allocator, load_code);
                        
                        // Imprimir desde rax
                        try self.generatePrintNumberFromRax();
                    },
                    .String => |s| {
                        // String literal
                        const string_name = try std.fmt.allocPrint(self.allocator, "msg_{d}", .{self.float_count});
                        self.float_count += 1;
                        
                        var buf: [256]u8 = undefined;
                        const data_code = try std.fmt.bufPrint(&buf, "    {s}: db \"{s}\", 0xA\n", .{ string_name, s });
                        try self.data_section.appendSlice(self.allocator, data_code);
                        
                        const len_name = try std.fmt.allocPrint(self.allocator, "{s}_len", .{string_name});
                        const len_code = try std.fmt.bufPrint(&buf, "    {s}: equ $ - {s}\n", .{ len_name, string_name });
                        try self.data_section.appendSlice(self.allocator, len_code);
                        
                        var write_buf: [512]u8 = undefined;
                        const write_code = try std.fmt.bufPrint(&write_buf,
                            \\    ; Print string
                            \\    mov rcx, [rbp+16]
                            \\    lea rdx, [rel {s}]
                            \\    mov r8, {s}_len
                            \\    lea r9, [rbp+24]
                            \\    mov qword [rsp+32], 0
                            \\    call WriteFile
                            \\
                        , .{ string_name, string_name });
                        try self.text_section.appendSlice(self.allocator, write_code);
                    },
                    .Ident => |name| {
                        // Cargar variable y convertir a string para imprimir (usar registro o stack)
                        var found = false;
                        if (self.register_alloc) |*reg_alloc| {
                            if (reg_alloc.getLocation(name)) |var_location| {
                                found = true;
                                switch (var_location) {
                                    .Register => |reg| {
                                        // Variable está en un registro callee-saved
                                        var load_buf: [128]u8 = undefined;
                                        const load_code = try std.fmt.bufPrint(&load_buf, "    mov rax, {s}    ; Cargar variable {s} desde registro\n", .{ reg.name(), name });
                                        try self.text_section.appendSlice(self.allocator, load_code);
                                    },
                                    .Stack => |offset| {
                                        // Variable está en stack
                                        var load_buf: [128]u8 = undefined;
                                        const load_code = try std.fmt.bufPrint(&load_buf, "    mov rax, [rbp-{d}]    ; Cargar variable {s} desde stack\n", .{ offset, name });
                                        try self.text_section.appendSlice(self.allocator, load_code);
                                    },
                                }
                                // Imprimir desde rax
                                try self.generatePrintNumberFromRax();
                            }
                        }
                        
                        if (!found) {
                            // Intentar con symbol_table tradicional
                            if (self.symbol_table.get(name)) |location| {
                                var load_buf: [128]u8 = undefined;
                                const load_code = try std.fmt.bufPrint(&load_buf, "    mov rax, [rbp-{d}]    ; Cargar variable {s}\n", .{ location.offset, name });
                                try self.text_section.appendSlice(self.allocator, load_code);
                                
                                // Imprimir desde rax
                                try self.generatePrintNumberFromRax();
                            } else {
                                // Variable no encontrada
                                var buf: [256]u8 = undefined;
                                const code = try std.fmt.bufPrint(&buf, "    ; ERROR: Variable '{s}' no encontrada\n", .{name});
                                try self.text_section.appendSlice(self.allocator, code);
                            }
                        }
                    },
                    .BinaryOp => {
                        // Evaluar expresión binaria y imprimir resultado
                        try self.generateExpr(expr);
                        // Imprimir desde rax
                        try self.generatePrintNumberFromRax();
                    },
                    else => {
                        // Para otras expresiones, evaluar y convertir resultado a string
                        try self.generateExpr(expr);
                        // Imprimir desde rax
                        try self.generatePrintNumberFromRax();
                    },
                }
            },
            .Let => |let_stmt| {
                // CRÍTICO: Verificar tamaño ANTES de generar expresión
                const text_len_before_expr = self.text_section.items.len;
                
                // Generar valor (resultado en rax)
                // IMPORTANTE: Asegurar que generateExpr realmente agregue código
                self.generateExpr(let_stmt.value) catch |err| {
                    // Si generateExpr falla, agregar código de error y continuar
                    _ = err; // Usar err para evitar warning
                    var error_buf: [256]u8 = undefined;
                    const error_code = try std.fmt.bufPrint(&error_buf, 
                        "    ; ERROR: generateExpr falló\n    mov rax, 0    ; Valor por defecto\n", .{});
                    try self.text_section.appendSlice(self.allocator, error_code);
                };
                
                // CRÍTICO: Verificar que generateExpr agregó código
                const text_len_after_expr = self.text_section.items.len;
                if (text_len_after_expr == text_len_before_expr) {
                    // generateExpr no agregó código - esto es un error crítico
                    // Agregar código de error y código mínimo para evitar crash
                    var error_buf: [256]u8 = undefined;
                    const error_code = try std.fmt.bufPrint(&error_buf, 
                        "    ; ERROR: generateExpr no agregó código para expresión\n    mov rax, 0    ; Valor por defecto\n", .{});
                    try self.text_section.appendSlice(self.allocator, error_code);
                }
                
                // Asignar espacio para la variable y guardar valor (usar registro o stack)
                if (self.register_alloc) |*reg_alloc| {
                    if (reg_alloc.getLocation(let_stmt.name)) |var_location| {
                        switch (var_location) {
                            .Register => |reg| {
                                // Variable está en un registro callee-saved
                                var buf: [128]u8 = undefined;
                                const code = try std.fmt.bufPrint(&buf, "    mov {s}, rax    ; Guardar variable {s} en registro\n", .{ reg.name(), let_stmt.name });
                                
                                // CRÍTICO: Verificar que appendSlice realmente agregue código
                                const len_before_assign = self.text_section.items.len;
                try self.text_section.appendSlice(self.allocator, code);
                                const len_after_assign = self.text_section.items.len;
                                if (len_after_assign == len_before_assign) {
                                    return error.OutOfMemory;
                                }
                            },
                            .Stack => |offset| {
                                // Variable está en stack
                                var buf: [128]u8 = undefined;
                                const code = try std.fmt.bufPrint(&buf, "    mov [rbp-{d}], rax    ; Guardar variable {s} en stack\n", .{ offset, let_stmt.name });
                                
                                // CRÍTICO: Verificar que appendSlice realmente agregue código
                                const len_before_assign = self.text_section.items.len;
                                try self.text_section.appendSlice(self.allocator, code);
                                const len_after_assign = self.text_section.items.len;
                                if (len_after_assign == len_before_assign) {
                                    return error.OutOfMemory;
                                }
                            },
                        }
                    } else {
                        // Variable no encontrada en register allocator, usar symbol_table
                        const location = try self.symbol_table.allocate(let_stmt.name);
                        var buf: [128]u8 = undefined;
                        const code = try std.fmt.bufPrint(&buf, "    mov [rbp-{d}], rax    ; Guardar variable {s}\n", .{ location.offset, let_stmt.name });
                        
                        const len_before_assign = self.text_section.items.len;
                        try self.text_section.appendSlice(self.allocator, code);
                        const len_after_assign = self.text_section.items.len;
                        if (len_after_assign == len_before_assign) {
                            return error.OutOfMemory;
                        }
                    }
                } else {
                    // Sin register allocator, usar tabla de símbolos tradicional
                    const location = try self.symbol_table.allocate(let_stmt.name);
                    var buf: [128]u8 = undefined;
                    const code = try std.fmt.bufPrint(&buf, "    mov [rbp-{d}], rax    ; Guardar variable {s}\n", .{ location.offset, let_stmt.name });
                    
                    // CRÍTICO: Verificar que appendSlice realmente agregue código
                    const len_before_assign = self.text_section.items.len;
                    try self.text_section.appendSlice(self.allocator, code);
                    const len_after_assign = self.text_section.items.len;
                    if (len_after_assign == len_before_assign) {
                        // appendSlice falló silenciosamente - esto es crítico
                        return error.OutOfMemory;
                    }
                }
            },
            .Assign => |assign_stmt| {
                // Generar valor (resultado en rax)
                try self.generateExpr(assign_stmt.value);
                
                // Obtener ubicación de variable existente o crear nueva
                const location = if (self.symbol_table.get(assign_stmt.name)) |loc|
                    loc
                else
                    try self.symbol_table.allocate(assign_stmt.name);
                
                var buf: [128]u8 = undefined;
                const code = try std.fmt.bufPrint(&buf, "    mov [rbp-{d}], rax    ; Asignar variable {s}\n", .{ location.offset, assign_stmt.name });
                try self.text_section.appendSlice(self.allocator, code);
            },
            .While => |while_stmt| {
                const loop_start = try self.newLabel();
                const loop_end = try self.newLabel();
                
                // Label de inicio del loop
                var label_buf: [128]u8 = undefined;
                const label_code = try std.fmt.bufPrint(&label_buf, "{s}:\n", .{loop_start});
                try self.text_section.appendSlice(self.allocator, label_code);
                
                // Generar condición
                // Para while, la condición debe evaluarse correctamente
                // Si es una comparación (<=, >=, <, >, ==, !=), generar comparación directa
                switch (while_stmt.condition.*) {
                    .BinaryOp => |bin_op| {
                        // Es una comparación binaria (suma <= limite)
                        switch (bin_op.op) {
                            .Le, .Lt, .Ge, .Gt, .Eq, .Ne => {
                                // Es una comparación - generar comparación directa
                                // Generar left
                                try self.generateExpr(bin_op.left);
                                try self.text_section.appendSlice(self.allocator, "    push rax\n");
                                
                                // Generar right
                                try self.generateExpr(bin_op.right);
                                try self.text_section.appendSlice(self.allocator, "    pop rbx\n");
                                try self.text_section.appendSlice(self.allocator, "    cmp rbx, rax\n");
                                
                                // Generar jump según operador
                                var jump_buf: [128]u8 = undefined;
                                const jump_code = switch (bin_op.op) {
                                    .Le => try std.fmt.bufPrint(&jump_buf, "    jg {s}\n", .{loop_end}), // si rbx > rax, salir
                                    .Lt => try std.fmt.bufPrint(&jump_buf, "    jge {s}\n", .{loop_end}), // si rbx >= rax, salir
                                    .Ge => try std.fmt.bufPrint(&jump_buf, "    jl {s}\n", .{loop_end}), // si rbx < rax, salir
                                    .Gt => try std.fmt.bufPrint(&jump_buf, "    jle {s}\n", .{loop_end}), // si rbx <= rax, salir
                                    .Eq => try std.fmt.bufPrint(&jump_buf, "    jne {s}\n", .{loop_end}), // si rbx != rax, salir
                                    .Ne => try std.fmt.bufPrint(&jump_buf, "    je {s}\n", .{loop_end}), // si rbx == rax, salir
                                    else => unreachable,
                                };
                                try self.text_section.appendSlice(self.allocator, jump_code);
                            },
                            else => {
                                // No es comparación, evaluar como expresión booleana
                                try self.generateExpr(while_stmt.condition);
                                try self.text_section.appendSlice(self.allocator, "    cmp rax, 0\n");
                                var jump_buf: [128]u8 = undefined;
                                const jump_code = try std.fmt.bufPrint(&jump_buf, "    je {s}\n", .{loop_end});
                                try self.text_section.appendSlice(self.allocator, jump_code);
                            },
                        }
                    },
                    else => {
                        // No es comparación binaria, evaluar como expresión booleana
                        try self.generateExpr(while_stmt.condition);
                        try self.text_section.appendSlice(self.allocator, "    cmp rax, 0\n");
                        var jump_buf: [128]u8 = undefined;
                        const jump_code = try std.fmt.bufPrint(&jump_buf, "    je {s}\n", .{loop_end});
                        try self.text_section.appendSlice(self.allocator, jump_code);
                    },
                }
                
                // Generar body
                for (while_stmt.body) |body_stmt| {
                    try self.generateStatement(body_stmt);
                }
                
                // Saltar de vuelta al inicio
                var jump_buf_2: [128]u8 = undefined;
                const jmp_code = try std.fmt.bufPrint(&jump_buf_2, "    jmp {s}\n", .{loop_start});
                try self.text_section.appendSlice(self.allocator, jmp_code);
                
                // Label de fin del loop
                const end_label_code = try std.fmt.bufPrint(&label_buf, "{s}:\n", .{loop_end});
                try self.text_section.appendSlice(self.allocator, end_label_code);
                
                // Liberar labels
                self.allocator.free(loop_start);
                self.allocator.free(loop_end);
            },
            .If => |if_stmt| {
                const else_label = try self.newLabel();
                const end_label = try self.newLabel();
                
                // Generar condición
                try self.generateExpr(if_stmt.condition);
                
                // Comparar con 0
                try self.text_section.appendSlice(self.allocator,
                    \\    cmp rax, 0
                    \\
                );
                
                if (if_stmt.else_body) |_| {
                    var jump_buf: [128]u8 = undefined;
                    const jump_code = try std.fmt.bufPrint(&jump_buf, "    je {s}\n", .{else_label});
                    try self.text_section.appendSlice(self.allocator, jump_code);
                } else {
                    var jump_buf: [128]u8 = undefined;
                    const jump_code = try std.fmt.bufPrint(&jump_buf, "    je {s}\n", .{end_label});
                    try self.text_section.appendSlice(self.allocator, jump_code);
                }
                
                // Then body
                for (if_stmt.then_body) |then_stmt| {
                    try self.generateStatement(then_stmt);
                }
                
                if (if_stmt.else_body) |else_body| {
                    var jump_buf: [128]u8 = undefined;
                    const jmp_code = try std.fmt.bufPrint(&jump_buf, "    jmp {s}\n", .{end_label});
                    try self.text_section.appendSlice(self.allocator, jmp_code);
                    
                    // Else label
                    var label_buf: [128]u8 = undefined;
                    const else_label_code = try std.fmt.bufPrint(&label_buf, "{s}:\n", .{else_label});
                    try self.text_section.appendSlice(self.allocator, else_label_code);
                    
                    // Else body
                    for (else_body) |else_stmt| {
                        try self.generateStatement(else_stmt);
                    }
                }
                
                // End label
                var label_buf: [128]u8 = undefined;
                const end_label_code = try std.fmt.bufPrint(&label_buf, "{s}:\n", .{end_label});
                try self.text_section.appendSlice(self.allocator, end_label_code);
                
                // Liberar labels
                self.allocator.free(else_label);
                self.allocator.free(end_label);
            },
            .Expr => |expr| {
                try self.generateExpr(expr);
            },
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
        
        // Calcular tamaño de stack necesario para variables
        const stack_size = if (self.register_alloc) |*reg_alloc|
            reg_alloc.getStackSize()
        else
            self.symbol_table.getStackSize();
        var stack_buf: [64]u8 = undefined;
        const stack_code = try std.fmt.bufPrint(&stack_buf, "    sub rsp, {d}\n", .{stack_size});
        try writer.writeAll(stack_code);
        
        try writer.writeAll("    ; Get stdout handle\n");
        try writer.writeAll("    mov ecx, -11\n");
        try writer.writeAll("    call GetStdHandle\n");
        try writer.writeAll("    mov [rbp+16], rax\n");
        
        // CRÍTICO: Verificar que text_section tiene contenido antes de escribir
        if (self.text_section.items.len == 0) {
            // text_section está vacío - esto es un problema crítico
            // Agregar código de error para identificar el problema
            try writer.writeAll("    ; ERROR: text_section está vacío - ningún código generado\n");
            try writer.writeAll("    ; Esto indica que generateStatement() no agregó código\n");
            // Retornar error para que el sistema use fallback a C
            return error.OutOfMemory; // Usar error estándar de Zig
        }
        
        // Código generado - escribir contenido de text_section
        try writer.writeAll(self.text_section.items);
        
        // Exit
        try writer.writeAll("    ; Exit process\n");
        try writer.writeAll("    mov ecx, 0\n");
        // Restaurar stack
        const final_stack_size = if (self.register_alloc) |*reg_alloc|
            reg_alloc.getStackSize()
        else
            self.symbol_table.getStackSize();
        var restore_stack_buf: [64]u8 = undefined;
        const restore_stack_code = try std.fmt.bufPrint(&restore_stack_buf, "    add rsp, {d}\n", .{final_stack_size});
        try writer.writeAll(restore_stack_code);
        
        // Restaurar registros callee-saved si se están usando
        if (self.register_alloc) |*reg_alloc| {
            // Verificar si hay variables en registros
            var has_registers = false;
            var it = reg_alloc.allocations.iterator();
            while (it.next()) |entry| {
                switch (entry.value_ptr.*) {
                    .Register => {
                        has_registers = true;
                        break;
                    },
                    .Stack => {},
                }
            }
            
            if (has_registers) {
                // Restaurar registros callee-saved al final
                try writer.writeAll("    pop r15\n");
                try writer.writeAll("    pop r14\n");
                try writer.writeAll("    pop r13\n");
                try writer.writeAll("    pop r12\n");
            }
        }
        
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
    const trimmed = std.mem.trim(u8, input, " \t\n\r");
    
    // Detectar si es un statement (contiene "while", "if", "let", "print", etc.)
    const is_statement = std.mem.indexOf(u8, trimmed, "while") != null or
                        std.mem.indexOf(u8, trimmed, "if") != null or
                        std.mem.indexOf(u8, trimmed, "let") != null or
                        std.mem.indexOf(u8, trimmed, "print") != null;
    
    var generator = NASMGenerator.init(allocator);
    defer generator.deinit();
    
    if (is_statement) {
        // Intentar parsear como statement (múltiples statements)
        var stmt_parser = statement_parser.StatementParser.init(allocator, trimmed);
        var parsed_any = false;
        
        // OPTIMIZACIÓN FASE 2: Crear tablas de optimización compartidas para todo el programa
        var constant_table = constant_propagation.ConstantTable.init(allocator);
        defer constant_table.deinit();
        
        var cse_table = cse.CSETable.init(allocator);
        defer cse_table.deinit();
        
        // LOGGING: Verificar input
        const debug_info = std.fmt.allocPrint(allocator, "DEBUG: Input length: {d}, trimmed length: {d}\n", .{ input.len, trimmed.len }) catch "";
        _ = debug_info; // Usar para evitar warning de unused
        
        // Parsear todos los statements primero (sin optimizaciones)
        var parsed_statements = std.ArrayList(statement_parser.Statement).init(allocator);
        defer parsed_statements.deinit();
        
        var statement_count: usize = 0;
        while (true) {
            // Skip whitespace antes de intentar parsear
            const old_pos = stmt_parser.pos;
            stmt_parser.skipWhitespace();
            
            // Si después de skip whitespace llegamos al final, terminamos
            if (stmt_parser.pos >= trimmed.len) {
                // LOGGING: Fin del input
                break;
            }
            
            // Si la posición no cambió después de skip, puede ser un problema
            if (stmt_parser.pos == old_pos and stmt_parser.pos >= trimmed.len) break;
            
            // Intentar parsear un statement
            const maybe_stmt = stmt_parser.parse() catch |parse_err| {
                // LOGGING: Error en parsing
                _ = parse_err;
                // Si falla el parsing, verificar si avanzamos
                if (stmt_parser.pos == old_pos) {
                    // No avanzó, probablemente terminamos
                    break;
                }
                // Si avanzó algo pero falló, puede ser un error real
                // Pero si ya parseamos algo, continuar intentando
                if (parsed_any) {
                    // Continuar intentando parsear más statements
                    if (stmt_parser.pos >= trimmed.len) break;
                    continue;
                }
                // Si no parseamos nada y falló, retornar error
                return -1;
            };
            if (maybe_stmt) |stmt| {
                parsed_any = true;
                statement_count += 1;
                
                // Guardar statement parseado para optimización posterior
                try parsed_statements.append(stmt);
                
                // Continuar parseando si hay más
                if (stmt_parser.pos >= trimmed.len) break;
            } else {
                // No hay más statements para parsear
                break;
            }
        }
        
        // CRÍTICO: Verificar si se parseó algo
        if (!parsed_any) {
            // No se parseó ningún statement - esto es el problema
            return -5; // Código de error: ningún statement parseado
        }
        
        // OPTIMIZACIÓN FASE 2: Aplicar optimizaciones a todos los statements parseados
        // 1. Primero aplicar CSE (Common Subexpression Elimination)
        var cse_optimized: []statement_parser.Statement = undefined;
        if (cse.applyCSE(allocator, parsed_statements.items)) |optimized| {
            cse_optimized = optimized;
        } else |err| {
            // Si falla CSE, usar statements originales (duplicar para mantener consistencia)
            _ = err;
            cse_optimized = allocator.dupe(statement_parser.Statement, parsed_statements.items) catch {
                return -6; // Error al duplicar statements
            };
        }
        defer allocator.free(cse_optimized);
        
        // 2. Luego aplicar Constant Propagation
        var cp_statements = std.ArrayList(statement_parser.Statement).init(allocator);
        defer cp_statements.deinit();
        
        for (cse_optimized) |stmt| {
            const cp_optimized = constant_propagation.propagateInStatement(
                allocator,
                stmt,
                &constant_table
            ) catch |err| {
                // Si falla CP, usar statement original
                _ = err;
                null;
            };
            
            if (cp_optimized) |opt_stmt| {
                try cp_statements.append(opt_stmt);
            } else {
                try cp_statements.append(stmt);
            }
            }
            
        // 3. Aplicar Loop Invariant Code Motion (LICM)
        var licm_optimized: []statement_parser.Statement = undefined;
        if (loop_optimizer.applyLICM(allocator, cp_statements.items)) |optimized| {
            licm_optimized = optimized;
        } else |err| {
            // Si falla LICM, usar statements anteriores (duplicar para mantener consistencia)
            _ = err;
            licm_optimized = allocator.dupe(statement_parser.Statement, cp_statements.items) catch {
                return -7; // Error al duplicar statements para LICM
            };
        }
        defer allocator.free(licm_optimized);
        
        // 4. Aplicar Constant Propagation nuevamente después de LICM (por si hay nuevas constantes)
        var cp2_statements = std.ArrayList(statement_parser.Statement).init(allocator);
        defer cp2_statements.deinit();
        
        for (licm_optimized) |stmt| {
            const cp_optimized = constant_propagation.propagateInStatement(
                allocator,
                stmt,
                &constant_table
            ) catch |err| {
                // Si falla CP, usar statement original
                _ = err;
                null;
            };
            
            if (cp_optimized) |opt_stmt| {
                try cp2_statements.append(opt_stmt);
            } else {
                try cp2_statements.append(stmt);
            }
        }
        
        // 5. Aplicar Register Allocation Inteligente
        generator.register_alloc = register_allocator.applyRegisterAllocation(
            allocator,
            cp2_statements.items
        ) catch |err| {
            // Si falla Register Allocation, continuar sin él
            _ = err;
            null;
        };
        
        // 6. Generar código para todos los statements optimizados
        for (cp2_statements.items) |stmt| {
            // CRÍTICO: Verificar tamaño de text_section ANTES de generar código
            const text_section_len_before_stmt = generator.text_section.items.len;
            
            // Generar código para el statement - SI FALLA, RETORNAR ERROR
            generator.generateStatement(stmt) catch |err| {
                // Si falla la generación, retornar error específico
                _ = err;
                return -4; // Error en generación de código
            };
            
            // CRÍTICO: Verificar que se agregó código a text_section DESPUÉS de generar
            const text_section_len_after_stmt = generator.text_section.items.len;
            
            if (text_section_len_after_stmt == text_section_len_before_stmt) {
                // No se agregó código a text_section - esto es un problema crítico
                // Retornar error específico para identificar el problema
                return -10; // Código de error: generateStatement no agregó código a text_section
            }
        }
        
        // CRÍTICO: Verificar que text_section tiene contenido después de generar
        if (generator.text_section.items.len == 0) {
            // Se parsearon statements pero text_section está vacío
            // Esto indica que generateStatement() no agregó código
            return -11; // Código de error: statements parseados pero text_section vacío
        }
    } else {
        // Parsear como expresión (código original)
        var parser = expr_parser.ExprParser.init(allocator, trimmed);
        const expr = parser.parse() catch {
            return -1;
        };
        
        if (expr == null) {
            return -1;
        }
    
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
    
    // CRÍTICO: Verificar que text_section tiene contenido ANTES de generar código completo
    const text_section_len_before = generator.text_section.items.len;
    
    generator.generateCompleteCode(writer) catch |err| {
        // Si falla, retornar error específico
        _ = err;
        return -6; // Error al generar código completo
    };
    
    // CRÍTICO: Verificar que se generó código (más que solo headers)
    // Headers son aproximadamente 50-100 bytes, código real debería ser mucho más
    const final_buffer_len = temp_buffer.items.len;
    
    // Verificar múltiples condiciones para detectar el problema
    if (final_buffer_len <= 150) {
        // Solo headers, sin código real
        // Esto significa que text_section estaba vacío o no se agregó código
        return -7; // Código de error: solo headers generados
    }
    
    // Verificar que text_section tenía contenido antes de generar código completo
    if (text_section_len_before == 0) {
        // text_section estaba vacío antes de generateCompleteCode
        // Esto indica que generateStatement() no agregó código
        return -8; // Código de error: text_section vacío antes de generateCompleteCode
    }
    
    // Verificar que el código generado incluye el contenido de text_section
    if (text_section_len_before > 0 and final_buffer_len <= 200) {
        // text_section tenía contenido pero el buffer final es muy pequeño
        // Esto indica un problema en generateCompleteCode
        return -9; // Código de error: text_section tenía contenido pero buffer final es pequeño
    }
    
    // Copiar al buffer de salida
    if (temp_buffer.items.len >= output_buffer_len) {
        return -1; // Buffer muy pequeño
    }
    @memcpy(output_buffer[0..temp_buffer.items.len], temp_buffer.items);
    output_buffer[temp_buffer.items.len] = 0; // Null terminator
    
    return @intCast(temp_buffer.items.len);
}
