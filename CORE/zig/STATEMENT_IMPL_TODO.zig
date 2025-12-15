// Este archivo contiene el código que necesita agregarse a nasm_generator.zig
// Función generateStatement y generateExpr que generan código NASM para statements

/// Generar código para una expresión que devuelve un valor en RAX
fn generateExpr(self: *NASMGenerator, expr: *expr_parser.Expr) !void {
    switch (expr.*) {
        .Number => |n| {
            var buf: [64]u8 = undefined;
            const code = try std.fmt.bufPrint(&buf, "    mov rax, {d}\n", .{n});
            try self.text_section.appendSlice(self.allocator, code);
        },
        .Float => |f| {
            const float_name = try self.generateFloatLiteral(f);
            try self.generateLoadFloat(float_name);
            // Para comparaciones, necesitamos convertir float a int o comparar floats
            // Por ahora, asumimos que las comparaciones son con enteros
        },
        .Ident => |name| {
            // Cargar variable desde stack (asumimos offset fijo por simplicidad)
            // TODO: Trackear offsets de variables
            var buf: [128]u8 = undefined;
            const code = try std.fmt.bufPrint(&buf, "    ; TODO: cargar variable {s}\n    mov rax, 0\n", .{name});
            try self.text_section.appendSlice(self.allocator, code);
        },
        .BinaryOp => |bin| {
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

/// Generar código para un statement
pub fn generateStatement(self: *NASMGenerator, stmt: statement_parser.Statement) !void {
    switch (stmt) {
        .Print => |expr| {
            // Para números enteros, convertir a string e imprimir
            switch (expr.*) {
                .Number => |n| {
                    // Convertir número a string e imprimir
                    var num_str_buf: [64]u8 = undefined;
                    const num_str = try std.fmt.bufPrint(&num_str_buf, "{d}\n", .{n});
                    
                    const string_name = try std.fmt.allocPrint(self.allocator, "msg_{d}", .{self.float_count});
                    self.float_count += 1;
                    
                    var buf: [256]u8 = undefined;
                    const data_code = try std.fmt.bufPrint(&buf, "    {s}: db \"{s}\"\n", .{ string_name, num_str });
                    try self.data_section.appendSlice(self.allocator, data_code);
                    
                    const len_name = try std.fmt.allocPrint(self.allocator, "{s}_len", .{string_name});
                    const len_code = try std.fmt.bufPrint(&buf, "    {s}: equ $ - {s}\n", .{ len_name, string_name });
                    try self.data_section.appendSlice(self.allocator, len_code);
                    
                    var write_buf: [512]u8 = undefined;
                    const write_code = try std.fmt.bufPrint(&write_buf,
                        \\    ; Print number
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
                else => {
                    // Para expresiones complejas, evaluar y convertir resultado a string
                    try self.generateExpr(expr);
                    // TODO: Convertir RAX a string e imprimir
                },
            }
        },
        .Let => |let_stmt| {
            // Generar valor
            try self.generateExpr(let_stmt.value);
            // TODO: Guardar en variable (necesita tracking de offsets)
            var buf: [256]u8 = undefined;
            const code = try std.fmt.bufPrint(&buf, "    ; TODO: guardar variable {s}\n", .{let_stmt.name});
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
            try self.generateExpr(while_stmt.condition);
            
            // Comparar con 0 y saltar si es falso
            try self.text_section.appendSlice(self.allocator,
                \\    cmp rax, 0
                \\
            );
            var jump_buf: [128]u8 = undefined;
            const jump_code = try std.fmt.bufPrint(&jump_buf, "    je {s}\n", .{loop_end});
            try self.text_section.appendSlice(self.allocator, jump_code);
            
            // Generar body
            for (while_stmt.body) |body_stmt| {
                try self.generateStatement(body_stmt);
            }
            
            // Saltar de vuelta al inicio
            const jmp_code = try std.fmt.bufPrint(&jump_buf, "    jmp {s}\n", .{loop_start});
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

