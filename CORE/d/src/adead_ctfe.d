/**
 * ADead CTFE Module - Optimización Compile-Time Completa
 * 
 * Este módulo implementa CTFE (Compile-Time Function Execution) avanzado
 * para optimizar expresiones constantes ANTES de generar código ASM.
 * 
 * Funcionalidades:
 * - Evaluación de expresiones aritméticas simples: 5 + 3 → 8
 * - Evaluación de expresiones complejas: (5 + 3) * 2 → 16
 * - Múltiples operadores: 10 + 5 * 2 → 20 (con precedencia)
 * - Eliminación de código muerto
 * - Optimización de expresiones anidadas
 * 
 * Autor: Eddi Andreé Salazar Matos
 * Fecha: Diciembre 2025
 */

module adead_ctfe;

import std.stdio;
import std.string;
import std.conv;
import std.algorithm;
import std.regex;

// ============================================================
// Estructuras para representar expresiones
// ============================================================

enum TokenType {
    Number,
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    LeftParen,
    RightParen,
    Whitespace
}

struct Token {
    TokenType type;
    string value;
    size_t position;
}

// ============================================================
// Parser de expresiones
// ============================================================

/**
 * Tokeniza una expresión matemática
 */
pure Token[] tokenize(string expr) {
    Token[] tokens;
    size_t pos = 0;
    
    while (pos < expr.length) {
        char c = expr[pos];
        
        if (c >= '0' && c <= '9') {
            // Número
            size_t start = pos;
            while (pos < expr.length && (expr[pos] >= '0' && expr[pos] <= '9')) {
                pos++;
            }
            tokens ~= Token(TokenType.Number, expr[start..pos], start);
        } else if (c == '+') {
            tokens ~= Token(TokenType.Plus, "+", pos);
            pos++;
        } else if (c == '-') {
            tokens ~= Token(TokenType.Minus, "-", pos);
            pos++;
        } else if (c == '*') {
            tokens ~= Token(TokenType.Multiply, "*", pos);
            pos++;
        } else if (c == '/') {
            tokens ~= Token(TokenType.Divide, "/", pos);
            pos++;
        } else if (c == '%') {
            tokens ~= Token(TokenType.Modulo, "%", pos);
            pos++;
        } else if (c == '(') {
            tokens ~= Token(TokenType.LeftParen, "(", pos);
            pos++;
        } else if (c == ')') {
            tokens ~= Token(TokenType.RightParen, ")", pos);
            pos++;
        } else if (c == ' ' || c == '\t') {
            // Ignorar whitespace
            pos++;
        } else {
            // Carácter desconocido, avanzar
            pos++;
        }
    }
    
    return tokens;
}

/**
 * Evalúa una expresión matemática en compile-time
 * Soporta: +, -, *, /, %, paréntesis, precedencia de operadores
 */
pure long evaluateExpression(string expr) {
    // Limpiar expresión
    expr = strip(expr);
    if (expr.length == 0) return 0;
    
    // Tokenizar
    Token[] tokens = tokenize(expr);
    if (tokens.length == 0) return 0;
    
    // Evaluar usando algoritmo de precedencia
    return evaluateTokens(tokens);
}

/**
 * Evalúa tokens con precedencia de operadores
 * Precedencia: () > * / % > + -
 */
pure long evaluateTokens(Token[] tokens) {
    if (tokens.length == 0) return 0;
    if (tokens.length == 1 && tokens[0].type == TokenType.Number) {
        return to!long(tokens[0].value);
    }
    
    // Primero: evaluar paréntesis
    Token[] processed;
    size_t i = 0;
    while (i < tokens.length) {
        if (tokens[i].type == TokenType.LeftParen) {
            // Encontrar paréntesis de cierre
            int depth = 1;
            size_t start = i + 1;
            size_t j = i + 1;
            while (j < tokens.length && depth > 0) {
                if (tokens[j].type == TokenType.LeftParen) depth++;
                else if (tokens[j].type == TokenType.RightParen) depth--;
                j++;
            }
            
            if (depth == 0) {
                // Evaluar contenido del paréntesis
                Token[] subExpr = tokens[start..j-1];
                long result = evaluateTokens(subExpr);
                processed ~= Token(TokenType.Number, to!string(result), i);
                i = j;
                continue;
            }
        }
        processed ~= tokens[i];
        i++;
    }
    
    // Segundo: evaluar * / %
    Token[] multDiv;
    i = 0;
    while (i < processed.length) {
        if (i + 2 < processed.length &&
            processed[i].type == TokenType.Number &&
            (processed[i+1].type == TokenType.Multiply ||
             processed[i+1].type == TokenType.Divide ||
             processed[i+1].type == TokenType.Modulo) &&
            processed[i+2].type == TokenType.Number) {
            
            long left = to!long(processed[i].value);
            long right = to!long(processed[i+2].value);
            long result = 0;
            
            switch (processed[i+1].type) {
                case TokenType.Multiply:
                    result = left * right;
                    break;
                case TokenType.Divide:
                    if (right != 0) result = left / right;
                    else return 0; // División por cero
                    break;
                case TokenType.Modulo:
                    if (right != 0) result = left % right;
                    else return 0;
                    break;
                default:
                    break;
            }
            
            multDiv ~= Token(TokenType.Number, to!string(result), i);
            i += 3;
        } else {
            multDiv ~= processed[i];
            i++;
        }
    }
    
    // Tercero: evaluar + -
    long result = 0;
    if (multDiv.length > 0 && multDiv[0].type == TokenType.Number) {
        result = to!long(multDiv[0].value);
    }
    
    i = 1;
    while (i < multDiv.length) {
        if (i + 1 < multDiv.length &&
            (multDiv[i].type == TokenType.Plus || multDiv[i].type == TokenType.Minus) &&
            multDiv[i+1].type == TokenType.Number) {
            
            long value = to!long(multDiv[i+1].value);
            if (multDiv[i].type == TokenType.Plus) {
                result += value;
            } else {
                result -= value;
            }
            i += 2;
        } else {
            i++;
        }
    }
    
    return result;
}

/**
 * Optimiza una expresión constante en compile-time
 * Input: "5 + 3"
 * Output: "8"
 */
pure string optimizeConstExpr(string expr) {
    // Limpiar expresión
    expr = strip(expr);
    
    // Intentar evaluar
    try {
        long result = evaluateExpression(expr);
        return to!string(result);
    } catch (Exception e) {
        // Si falla, retornar original
        return expr;
    }
}

/**
 * Optimiza múltiples expresiones en un string
 * Busca patrones como "5 + 3", "10 * 2", etc. y los reemplaza
 * NOTA: No es pure porque usa funciones de string que pueden tener efectos secundarios
 */
string optimizeSource(string source) {
    string result = source;
    bool changed = true;
    int iterations = 0;
    const int MAX_ITERATIONS = 10; // Evitar loops infinitos
    
    while (changed && iterations < MAX_ITERATIONS) {
        changed = false;
        iterations++;
        
        // Buscar expresiones simples manualmente (sin regex para evitar problemas con pure)
        // Patrón: "número operador número"
        
        size_t pos = 0;
        while (pos < result.length) {
            // Buscar inicio de número
            if (result[pos] >= '0' && result[pos] <= '9') {
                size_t num1_start = pos;
                while (pos < result.length && result[pos] >= '0' && result[pos] <= '9') {
                    pos++;
                }
                size_t num1_end = pos;
                
                // Buscar espacios y operador
                while (pos < result.length && (result[pos] == ' ' || result[pos] == '\t')) {
                    pos++;
                }
                
                if (pos < result.length && (result[pos] == '+' || result[pos] == '-' || 
                    result[pos] == '*' || result[pos] == '/' || result[pos] == '%')) {
                    char op = result[pos];
                    pos++;
                    
                    // Buscar espacios
                    while (pos < result.length && (result[pos] == ' ' || result[pos] == '\t')) {
                        pos++;
                    }
                    
                    // Buscar segundo número
                    if (pos < result.length && result[pos] >= '0' && result[pos] <= '9') {
                        size_t num2_start = pos;
                        while (pos < result.length && result[pos] >= '0' && result[pos] <= '9') {
                            pos++;
                        }
                        size_t num2_end = pos;
                        
                        // Evaluar expresión
                        try {
                            long left = to!long(result[num1_start..num2_start]);
                            long right = to!long(result[num2_start..num2_end]);
                            
                            long result_val = 0;
                            switch (op) {
                                case '+': result_val = left + right; break;
                                case '-': result_val = left - right; break;
                                case '*': result_val = left * right; break;
                                case '/': if (right != 0) result_val = left / right; else { pos = num2_end; continue; } break;
                                case '%': if (right != 0) result_val = left % right; else { pos = num2_end; continue; } break;
                                default: { pos = num2_end; continue; }
                            }
                            
                            // Reemplazar
                            string num1_str = result[num1_start..num1_end];
                            string num2_str = result[num2_start..num2_end];
                            string op_str = [op];
                            string pattern = num1_str ~ (op == '+' ? " + " : op == '-' ? " - " : op == '*' ? " * " : op == '/' ? " / " : " % ") ~ num2_str;
                            string alt_pattern1 = num1_str ~ op_str ~ num2_str;
                            string alt_pattern2 = num1_str ~ " " ~ op_str ~ " " ~ num2_str;
                            
                            string replacement = to!string(result_val);
                            
                            if (result.indexOf(pattern) != -1) {
                                result = replace(result, pattern, replacement);
                                changed = true;
                                break;
                            } else if (result.indexOf(alt_pattern1) != -1) {
                                result = replace(result, alt_pattern1, replacement);
                                changed = true;
                                break;
                            } else if (result.indexOf(alt_pattern2) != -1) {
                                result = replace(result, alt_pattern2, replacement);
                                changed = true;
                                break;
                            }
                        } catch (Exception) {
                            // Error al convertir, continuar
                        }
                    }
                }
            } else {
                pos++;
            }
        }
    }
    
    return result;
}

// ============================================================
// FFI: Funciones exportadas para Rust
// ============================================================

extern(C) {
    import core.stdc.stdlib;
    import core.stdc.string;
    
    /**
     * Optimiza expresiones constantes en compile-time
     * Retorna string optimizado (caller debe liberar con d_free_string)
     */
    export const(char)* d_optimize_const_expr(const(char)* expr) {
        if (!expr) return null;
        
        // Convertir C string a D string
        size_t len = strlen(expr);
        char[] dstr = new char[len];
        for (size_t i = 0; i < len; i++) {
            dstr[i] = expr[i];
        }
        string dString = cast(string)dstr;
        
        // Optimizar
        string optimized = optimizeConstExpr(dString);
        
        // Convertir a C string
        char* cstr = cast(char*)malloc(optimized.length + 1);
        if (!cstr) return null;
        
        for (size_t i = 0; i < optimized.length; i++) {
            cstr[i] = optimized[i];
        }
        cstr[optimized.length] = '\0';
        
        return cast(const(char)*)cstr;
    }
    
    /**
     * Optimiza código fuente completo
     * Busca y reemplaza todas las expresiones constantes
     */
    export const(char)* d_optimize_source(const(char)* source) {
        if (!source) return null;
        
        // Convertir C string a D string
        size_t len = strlen(source);
        char[] dstr = new char[len];
        for (size_t i = 0; i < len; i++) {
            dstr[i] = source[i];
        }
        string dString = cast(string)dstr;
        
        // Optimizar
        string optimized = optimizeSource(dString);
        
        // Convertir a C string
        char* cstr = cast(char*)malloc(optimized.length + 1);
        if (!cstr) return null;
        
        for (size_t i = 0; i < optimized.length; i++) {
            cstr[i] = optimized[i];
        }
        cstr[optimized.length] = '\0';
        
        return cast(const(char)*)cstr;
    }
    
    /**
     * Libera memoria de string C
     */
    export void d_free_string(const(char)* str) {
        if (str) {
            free(cast(void*)str);
        }
    }
}

// ============================================================
// Tests
// ============================================================

unittest {
    // Test expresión simple
    assert(evaluateExpression("5 + 3") == 8);
    assert(evaluateExpression("10 * 2") == 20);
    assert(evaluateExpression("15 - 7") == 8);
    assert(evaluateExpression("20 / 4") == 5);
    
    // Test expresión compleja
    assert(evaluateExpression("(5 + 3) * 2") == 16);
    assert(evaluateExpression("10 + 5 * 2") == 20); // Precedencia
    
    // Test optimización de string
    assert(optimizeConstExpr("5 + 3") == "8");
    assert(optimizeConstExpr("10 * 2") == "20");
}

