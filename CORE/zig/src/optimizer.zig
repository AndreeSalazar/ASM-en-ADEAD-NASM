// Optimizador de expresiones para Zig
// Implementa strength reduction y otras optimizaciones comunes

const std = @import("std");
const expr_parser = @import("expr_parser.zig");

/// Optimizar una expresión binaria usando strength reduction
/// Retorna una expresión optimizada si es posible, o null si no se puede optimizar
pub fn optimizeBinaryOp(allocator: std.mem.Allocator, bin: expr_parser.Expr.BinaryOp) ?*expr_parser.Expr {
    // Solo optimizar si ambos operandos son constantes (Number)
    const left_is_const = switch (bin.left.*) {
        .Number => true,
        else => false,
    };
    const right_is_const = switch (bin.right.*) {
        .Number => true,
        else => false,
    };
    
    // Si ambos son constantes, la optimización se hace en CTFE (D Language)
    // Aquí nos enfocamos en optimizaciones cuando uno es constante
    
    switch (bin.op) {
        .Mul => {
            // x * 2 → shl x, 1
            // x * 4 → shl x, 2
            // x * 8 → shl x, 3
            // x * 16 → shl x, 4
            // x * 1 → x
            // x * 0 → 0
            
            if (right_is_const) {
                const right_val = bin.right.*.Number;
                
                // x * 0 → 0
                if (right_val == 0) {
                    const zero_expr = allocator.create(expr_parser.Expr) catch return null;
                    zero_expr.* = expr_parser.Expr{ .Number = 0 };
                    return zero_expr;
                }
                
                // x * 1 → x
                if (right_val == 1) {
                    return bin.left;
                }
                
                // x * potencia_de_2 → shl x, log2(potencia)
                // Retornar null indica que se debe usar shift en vez de mul
                // El generador de código manejará esto con canUseShiftForMul
                if (right_val > 0 and std.math.isPowerOfTwo(@as(u64, @intCast(right_val)))) {
                    return null; // Se manejará en generateExpr con optimización
                }
            }
            
            if (left_is_const) {
                const left_val = bin.left.*.Number;
                
                // 0 * x → 0
                if (left_val == 0) {
                    const zero_expr = allocator.create(expr_parser.Expr) catch return null;
                    zero_expr.* = expr_parser.Expr{ .Number = 0 };
                    return zero_expr;
                }
                
                // 1 * x → x
                if (left_val == 1) {
                    return bin.right;
                }
                
                // potencia_de_2 * x → shl x, log2(potencia)
                // Retornar null indica que se debe usar shift en vez de mul
                if (left_val > 0 and std.math.isPowerOfTwo(@as(u64, @intCast(left_val)))) {
                    return null; // Se manejará en generateExpr con optimización
                }
            }
        },
        .Add => {
            // x + 0 → x
            // 0 + x → x
            
            if (right_is_const and bin.right.*.Number == 0) {
                return bin.left;
            }
            
            if (left_is_const and bin.left.*.Number == 0) {
                return bin.right;
            }
        },
        .Sub => {
            // x - 0 → x
            // 0 - x → -x (no optimizamos esto por ahora)
            
            if (right_is_const and bin.right.*.Number == 0) {
                return bin.left;
            }
        },
        .Div => {
            // x / 1 → x
            // x / potencia_de_2 → shr x, log2(potencia)
            
            if (right_is_const) {
                const right_val = bin.right.*.Number;
                
                // x / 1 → x
                if (right_val == 1) {
                    return bin.left;
                }
                
                // x / potencia_de_2 → shr x, log2(potencia)
                // Retornar null indica que se debe usar shift en vez de div
                if (right_val > 0 and std.math.isPowerOfTwo(@as(u64, @intCast(right_val)))) {
                    return null; // Se manejará en generateExpr con optimización
                }
            }
        },
        else => {},
    }
    
    return null; // No optimizable
}

/// Verificar si una multiplicación puede usar shift
pub fn canUseShiftForMul(op: expr_parser.BinOp, left_val: ?i64, right_val: ?i64) ?u6 {
    if (op != .Mul) return null;
    
    if (right_val) |val| {
        if (val > 0 and std.math.isPowerOfTwo(@as(u64, @intCast(val)))) {
            const shift_amount = @as(u64, @intCast(val));
            return @as(u6, @intCast(std.math.log2_int(u64, shift_amount)));
        }
    }
    
    if (left_val) |val| {
        if (val > 0 and std.math.isPowerOfTwo(@as(u64, @intCast(val)))) {
            const shift_amount = @as(u64, @intCast(val));
            return @as(u6, @intCast(std.math.log2_int(u64, shift_amount)));
        }
    }
    
    return null;
}

/// Verificar si una división puede usar shift
pub fn canUseShiftForDiv(op: expr_parser.BinOp, right_val: ?i64) ?u6 {
    if (op != .Div) return null;
    
    if (right_val) |val| {
        if (val > 0 and std.math.isPowerOfTwo(@as(u64, @intCast(val)))) {
            const shift_amount = @as(u64, @intCast(val));
            return @as(u6, @intCast(std.math.log2_int(u64, shift_amount)));
        }
    }
    
    return null;
}

