/**
 * ADead Metaprogramming Module - D Language
 * 
 * Este módulo proporciona metaprogramming avanzado para ADead:
 * - CTFE (Compile-Time Function Execution)
 * - Templates para generación de código ASM
 * - Validación en compile-time
 * - Generación directa de código NASM
 * 
 * Autor: Eddi Andreé Salazar Matos
 * Fecha: Diciembre 2025
 */

module adead_metaprog;

import std.stdio;
import std.string;
import std.conv;

// Exportar para uso en otros módulos
// Nota: adead_d_to_zig puede no estar disponible, comentado temporalmente
// public import adead_d_to_zig;

// ============================================================
// Estructuras básicas para AST
// ============================================================

enum ExprKind {
    Number,
    Identifier,
    BinaryOp,
    UnaryOp,
    Call
}

enum OpType {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge
}

struct Expr {
    ExprKind kind;
    string value;
    OpType op;
    Expr* left;
    Expr* right;
    string type;
}

// ============================================================
// Templates para generación de código ASM
// ============================================================

/**
 * Template para generar instrucción MOV en ASM
 */
template GenerateMovASM(string reg, string val) {
    enum GenerateMovASM = "    mov " ~ reg ~ ", " ~ val ~ "\n";
}

/**
 * Template para generar instrucciones aritméticas
 */
template GenerateArithASM(OpType op, string reg1, string reg2) {
    static if (op == OpType.Add) {
        enum GenerateArithASM = "    add " ~ reg1 ~ ", " ~ reg2 ~ "\n";
    } else static if (op == OpType.Sub) {
        enum GenerateArithASM = "    sub " ~ reg1 ~ ", " ~ reg2 ~ "\n";
    } else static if (op == OpType.Mul) {
        enum GenerateArithASM = "    imul " ~ reg1 ~ ", " ~ reg2 ~ "\n";
    } else static if (op == OpType.Div) {
        enum GenerateArithASM = "    mov rcx, " ~ reg2 ~ "\n" ~
                                "    mov rax, " ~ reg1 ~ "\n" ~
                                "    cqo\n" ~
                                "    idiv rcx\n";
    } else static if (op == OpType.Mod) {
        enum GenerateArithASM = "    mov rcx, " ~ reg2 ~ "\n" ~
                                "    mov rax, " ~ reg1 ~ "\n" ~
                                "    cqo\n" ~
                                "    idiv rcx\n" ~
                                "    mov rax, rdx\n";
    } else {
        enum GenerateArithASM = "    ; operación no soportada\n";
    }
}

/**
 * Template para generar comparaciones
 */
template GenerateCmpASM(OpType op, string reg1, string reg2) {
    static if (op == OpType.Eq) {
        enum GenerateCmpASM = "    cmp " ~ reg1 ~ ", " ~ reg2 ~ "\n" ~
                              "    je .L_equal\n";
    } else static if (op == OpType.Ne) {
        enum GenerateCmpASM = "    cmp " ~ reg1 ~ ", " ~ reg2 ~ "\n" ~
                              "    jne .L_not_equal\n";
    } else static if (op == OpType.Lt) {
        enum GenerateCmpASM = "    cmp " ~ reg1 ~ ", " ~ reg2 ~ "\n" ~
                              "    jl .L_less\n";
    } else static if (op == OpType.Le) {
        enum GenerateCmpASM = "    cmp " ~ reg1 ~ ", " ~ reg2 ~ "\n" ~
                              "    jle .L_less_equal\n";
    } else static if (op == OpType.Gt) {
        enum GenerateCmpASM = "    cmp " ~ reg1 ~ ", " ~ reg2 ~ "\n" ~
                              "    jg .L_greater\n";
    } else static if (op == OpType.Ge) {
        enum GenerateCmpASM = "    cmp " ~ reg1 ~ ", " ~ reg2 ~ "\n" ~
                              "    jge .L_greater_equal\n";
    } else {
        enum GenerateCmpASM = "    ; comparación no soportada\n";
    }
}

// ============================================================
// CTFE: Validación en compile-time
// ============================================================

/**
 * Valida que una expresión tenga tipos compatibles
 * Se ejecuta en compile-time (CTFE)
 */
pure bool validateExprTypes(Expr* expr) {
    if (!expr) return false;
    
    switch (expr.kind) {
        case ExprKind.Number:
        case ExprKind.Identifier:
            return true;
            
        case ExprKind.BinaryOp:
            if (!expr.left || !expr.right) return false;
            // Validar que los tipos sean compatibles
            if (expr.left.type != expr.right.type && 
                expr.left.type != "" && 
                expr.right.type != "") {
                return false;
            }
            // Validar recursivamente
            return validateExprTypes(expr.left) && 
                   validateExprTypes(expr.right);
                   
        default:
            return false;
    }
}

/**
 * Optimiza una expresión en compile-time
 */
pure Expr* optimizeExpr(Expr* expr) {
    if (!expr) return null;
    
    // Si es una operación con constantes, evaluar en compile-time
    if (expr.kind == ExprKind.BinaryOp && 
        expr.left && expr.right &&
        expr.left.kind == ExprKind.Number &&
        expr.right.kind == ExprKind.Number) {
        
        // Crear expresión optimizada con resultado
        auto result = new Expr();
        result.kind = ExprKind.Number;
        result.type = expr.left.type;
        
        // Evaluar en compile-time
        long leftVal = 0;
        long rightVal = 0;
        try {
            leftVal = to!long(expr.left.value);
            rightVal = to!long(expr.right.value);
        } catch(Throwable) {
            return expr; // No se puede convertir, retornar original
        }
        
        long resultVal = 0;
        
        switch (expr.op) {
            case OpType.Add: resultVal = leftVal + rightVal; break;
            case OpType.Sub: resultVal = leftVal - rightVal; break;
            case OpType.Mul: resultVal = leftVal * rightVal; break;
            case OpType.Div: if (rightVal != 0) resultVal = leftVal / rightVal; else return expr; break;
            case OpType.Mod: if (rightVal != 0) resultVal = leftVal % rightVal; else return expr; break;
            default: return expr; // No optimizable
        }
        
        result.value = to!string(resultVal);
        return result;
    }
    
    return expr;
}

// ============================================================
// Generación de código ASM
// ============================================================

/**
 * Genera código NASM para una expresión
 */
string generateExprASM(Expr* expr, ref int labelCounter) {
    if (!expr) return "";
    
    string asmCode = "";
    
    switch (expr.kind) {
        case ExprKind.Number:
            // Usar string directamente, no template
            asmCode ~= "    mov rax, " ~ expr.value ~ "\n";
            break;
            
        case ExprKind.Identifier:
            // Cargar variable desde stack (asumiendo offset en rbp)
            asmCode ~= "    mov rax, [rbp - " ~ expr.value ~ "]\n";
            break;
            
        case ExprKind.BinaryOp:
            // Generar código para left
            asmCode ~= generateExprASM(expr.left, labelCounter);
            asmCode ~= "    push rax\n";
            
            // Generar código para right
            asmCode ~= generateExprASM(expr.right, labelCounter);
            asmCode ~= "    mov rbx, rax\n";
            asmCode ~= "    pop rax\n";
            
            // Generar operación basada en el tipo
            switch (expr.op) {
                case OpType.Add:
                    asmCode ~= "    add rax, rbx\n";
                    break;
                case OpType.Sub:
                    asmCode ~= "    sub rax, rbx\n";
                    break;
                case OpType.Mul:
                    asmCode ~= "    imul rax, rbx\n";
                    break;
                case OpType.Div:
                    asmCode ~= "    mov rcx, rbx\n";
                    asmCode ~= "    mov rax, rax\n";
                    asmCode ~= "    cqo\n";
                    asmCode ~= "    idiv rcx\n";
                    break;
                case OpType.Mod:
                    asmCode ~= "    mov rcx, rbx\n";
                    asmCode ~= "    mov rax, rax\n";
                    asmCode ~= "    cqo\n";
                    asmCode ~= "    idiv rcx\n";
                    asmCode ~= "    mov rax, rdx\n";
                    break;
                default:
                    asmCode ~= "    ; operación no soportada\n";
                    break;
            }
            break;
            
        default:
            asmCode ~= "    ; expresión no soportada\n";
            break;
    }
    
    return asmCode;
}

// ============================================================
// FFI: Funciones exportadas para Rust
// ============================================================

extern(C) {
    /**
     * Parsea una expresión simple y la valida
     */
    Expr* parseAndValidateExpr(const(char)* source) {
        // Implementación básica - expandir según necesidades
        if (!source) return null;
        
        // Convertir C string a D string
        size_t len = 0;
        while (source[len] != '\0') len++;
        
        char[] dstr = new char[len];
        for (size_t i = 0; i < len; i++) {
            dstr[i] = source[i];
        }
        string dString = cast(string)dstr;
        
        auto expr = new Expr();
        expr.kind = ExprKind.Number;
        expr.value = dString.dup;
        expr.type = "int".dup;
        
        // Validar en compile-time si es posible
        if (!validateExprTypes(expr)) {
            freeExpr(expr);
            return null;
        }
        
        return expr;
    }
    
    /**
     * Genera código ASM para una expresión
     */
    const(char)* generateASMFromExpr(Expr* expr) {
        if (!expr) return null;
        
        int labelCounter = 0;
        string asmCode = generateExprASM(expr, labelCounter);
        
        // Asignar memoria para el string (caller debe liberar)
        auto cstr = cast(char*)malloc(asmCode.length + 1);
        if (!cstr) return null;
        
        for (size_t i = 0; i < asmCode.length; i++) {
            cstr[i] = asmCode[i];
        }
        cstr[asmCode.length] = '\0';
        
        return cast(const(char)*)cstr;
    }
    
    /**
     * Optimiza una expresión en compile-time
     */
    Expr* optimizeExprCTFE(Expr* expr) {
        return optimizeExpr(expr);
    }
    
    /**
     * Libera memoria de una expresión
     */
    void freeExpr(Expr* expr) {
        if (expr) {
            if (expr.left) freeExpr(expr.left);
            if (expr.right) freeExpr(expr.right);
            free(expr);
        }
    }
    
    /**
     * Libera memoria de un string C
     */
    void freeCString(const(char)* str) {
        if (str) {
            free(cast(void*)str);
        }
    }
}

// Funciones auxiliares para C compatibility
extern(C) {
    void* malloc(ulong size);
    void free(void* ptr);
    void* memcpy(void* dest, void* src, ulong n);
}

