/**
 * Módulo D → Zig → ASM Directo
 * 
 * Este módulo implementa el flujo: ADead → D → Zig → ASM
 * 
 * Proceso:
 * 1. D Language parsea y procesa el código ADead (metaprogramming)
 * 2. Genera código Zig intermedio
 * 3. Zig compila el código Zig a NASM directamente
 * 4. Resultado: ASM puro para la CPU
 * 
 * Autor: Eddi Andreé Salazar Matos
 * Fecha: Diciembre 2025
 */

module adead_d_to_zig;

import std.stdio;
import std.string;
import std.conv;
import std.file;
import std.process;

// ============================================================
// Generación de código Zig desde AST de D
// ============================================================

/**
 * Genera código Zig desde una expresión AST de D
 */
string generateZigCode(Expr* expr) {
    if (!expr) return "";
    
    string zigCode = "";
    
    switch (expr.kind) {
        case ExprKind.Number:
            zigCode ~= expr.value;
            break;
            
        case ExprKind.Identifier:
            zigCode ~= expr.value;
            break;
            
        case ExprKind.BinaryOp:
            zigCode ~= "(" ~ generateZigCode(expr.left);
            switch (expr.op) {
                case OpType.Add: zigCode ~= " + "; break;
                case OpType.Sub: zigCode ~= " - "; break;
                case OpType.Mul: zigCode ~= " * "; break;
                case OpType.Div: zigCode ~= " / "; break;
                case OpType.Mod: zigCode ~= " % "; break;
                default: zigCode ~= " ??? "; break;
            }
            zigCode ~= generateZigCode(expr.right) ~ ")";
            break;
            
        default:
            zigCode ~= "// expresión no soportada";
            break;
    }
    
    return zigCode;
}

/**
 * Genera un programa Zig completo desde código ADead
 */
string generateZigProgram(string adeadSource) {
    string zigCode = `const std = @import("std");

pub fn main() void {
    // Código generado desde ADead
    `;
    
    // Procesar el código ADead y generar Zig
    // Por ahora, generación básica
    zigCode ~= `    // TODO: Procesar código ADead y convertir a Zig
    
`;
    
    zigCode ~= `}
`;
    
    return zigCode;
}

/**
 * Compila código Zig a NASM directamente
 * Retorna la ruta del archivo ASM generado
 */
string compileZigToNASM(string zigCode, string outputName) {
    // Crear archivo temporal Zig
    string zigFile = outputName ~ ".zig";
    std.file.write(zigFile, zigCode);
    
    // Compilar Zig a NASM usando build.zig
    // Zig puede generar ASM directamente con: zig build-lib -target x86_64-windows --output-format nasm
    
    string[] args = [
        "zig",
        "build-lib",
        zigFile,
        "-target", "x86_64-windows",
        "-fno-strip",
        "-dynamic",
        "--name", outputName,
        "-femit-asm=" ~ outputName ~ ".asm"
    ];
    
    auto result = std.process.executeShell(args.join(" "));
    
    if (result.status != 0) {
        stderr.writeln("Error compilando Zig: ", result.output);
        return "";
    }
    
    return outputName ~ ".asm";
}

// ============================================================
// FFI: Funciones exportadas
// ============================================================

extern(C) {
    /**
     * Procesa código ADead y genera ASM vía Zig
     * Flujo: ADead → D (parse) → Zig (codegen) → NASM
     */
    const(char)* adeadToASMViaZig(const(char)* adeadSource) {
        if (!adeadSource) return null;
        
        // 1. Parsear código ADead (usando D)
        // Por ahora, asumimos que ya tenemos el código
        
        // 2. Generar código Zig
        size_t len = 0;
        while (adeadSource[len] != '\0') len++;
        char[] dstr = new char[len];
        for (size_t i = 0; i < len; i++) {
            dstr[i] = adeadSource[i];
        }
        string adeadStr = cast(string)dstr;
        
        string zigCode = generateZigProgram(adeadStr);
        
        // 3. Compilar Zig a NASM
        string asmFile = compileZigToNASM(zigCode, "adead_output");
        
        if (asmFile.length == 0) {
            return null;
        }
        
        // 4. Leer archivo ASM generado
        auto asmContent = read(asmFile);
        
        // Asignar memoria para el string C
        auto cstr = cast(char*)malloc(asmContent.length + 1);
        if (!cstr) return null;
        
        for (size_t i = 0; i < asmContent.length; i++) {
            cstr[i] = asmContent[i];
        }
        cstr[asmContent.length] = '\0';
        
        return cast(const(char)*)cstr;
    }
    
    /**
     * Parsea código ADead y genera código Zig intermedio
     */
    const(char)* adeadToZig(const(char)* adeadSource) {
        if (!adeadSource) return null;
        
        // Convertir C string a D string
        size_t len = 0;
        while (adeadSource[len] != '\0') len++;
        char[] dstr = new char[len];
        for (size_t i = 0; i < len; i++) {
            dstr[i] = adeadSource[i];
        }
        string adeadStr = cast(string)dstr;
        
        // Generar código Zig
        string zigCode = generateZigProgram(adeadStr);
        
        // Convertir a C string
        auto cstr = cast(char*)malloc(zigCode.length + 1);
        if (!cstr) return null;
        
        for (size_t i = 0; i < zigCode.length; i++) {
            cstr[i] = zigCode[i];
        }
        cstr[zigCode.length] = '\0';
        
        return cast(const(char)*)cstr;
    }
}

