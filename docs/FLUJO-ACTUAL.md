# 🔄 Flujo Actual de Compilación ADead - Diciembre 2025

## 📋 Resumen Ejecutivo

**Flujo Actual Implementado y Funcional:**
```
ADead → Parser Manual → C → GCC/Clang → ASM → EXE
```

## 🔍 Flujo Detallado Paso a Paso

### 1️⃣ **ADead Source (.ad)**

Archivo fuente del lenguaje ADead:
```adead
print "Iniciando test..."
let suma = 0
let limite = 10

while suma <= limite {
    if suma % 5 == 0 {
        print suma
    }
    suma = suma + 1
}
```

### 2️⃣ **Parser Manual (Regex + Recursión)**

**Responsabilidad:** Parsing directo y simple de código ADead

**Archivo:** `CORE/rust/crates/adead-parser/src/c_manual_parser.rs`

**Características:**
- ✅ Parsea `while` loops completos
- ✅ Parsea `if` statements
- ✅ Extrae bloques anidados correctamente
- ✅ Maneja expresiones aritméticas
- ✅ Sin dependencias externas complejas

**Proceso:**
1. Extrae `while` loops usando regex y conteo de llaves
2. Parsea condiciones (`suma <= limite`, `suma % 5 == 0`)
3. Parsea cuerpos de bloques recursivamente
4. Genera AST interno de Rust

**Ventajas:**
- ⚡ Simple y directo
- ✅ Control total del parsing
- ✅ Fácil de debuggear
- ✅ Sin overhead de bibliotecas externas

### 3️⃣ **Generador de Código C**

**Responsabilidad:** Convertir AST interno a código C válido

**Archivo:** `CORE/rust/crates/adead-parser/src/c_generator.rs`

**Proceso:**
1. Recibe AST interno (`Program`, `Stmt`, `Expr`)
2. Genera código C válido:
   - Headers estándar (`stdio.h`, `stdlib.h`, `stdint.h`)
   - Función `main()` automática
   - Variables con tipos correctos (`int64_t`)
   - Estructuras de control (`while`, `if`)
   - Operaciones aritméticas y comparaciones
   - `printf` con `fflush(stdout)` para output en tiempo real

**Ventajas:**
- ✅ Código C estándar y válido
- ✅ Compilable con cualquier GCC/Clang
- ✅ Output en tiempo real

### 4️⃣ **GCC/Clang (Compilador C)**

**Responsabilidad:** Compilar código C a ASM y EXE

**Proceso:**
1. **Genera ASM**: `gcc -S -masm=intel -O2 -o output.asm input.c`
   - Formato GAS (GNU Assembler)
   - Sintaxis Intel (legible)
   - Optimización nivel 2

2. **Compila EXE**: `gcc -O2 -o output.exe input.c`
   - Genera ejecutable directamente
   - Optimizado con -O2
   - Sin dependencias externas

**Ventajas:**
- ✅ Aprovecha optimizaciones probadas de GCC/Clang
- ✅ Genera ASM limpio y optimizado
- ✅ Compatible con Windows/Linux

### 5️⃣ **Ejecutable (.exe)**

**Resultado:**
- Ejecutable nativo Windows
- Sin dependencias externas
- Performance optimizada
- Output en tiempo real

## 📊 Diagrama de Flujo Completo

```
┌─────────────────────────────────────────┐
│  ADead Source (.ad)                    │
│  • Sintaxis estilo Python              │
│  • while/if/print/let                  │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│  PARSER MANUAL                          │
│  ┌───────────────────────────────────┐ │
│  │ • extract_while_loops()           │ │
│  │ • extract_if_statements()         │ │
│  │ • parse_while_from_text()         │ │
│  │ • parse_if_from_text()            │ │
│  │ • parse_expr_from_text()          │ │
│  └───────────────────────────────────┘ │
│  → AST interno (Program, Stmt, Expr)   │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│  GENERADOR DE CÓDIGO C                  │
│  ┌───────────────────────────────────┐ │
│  │ • generate_c_code()               │ │
│  │ • generate_stmt()                 │ │
│  │ • generate_expr()                 │ │
│  │ • Headers: stdio.h, stdint.h      │ │
│  │ • Función main() automática       │ │
│  │ • fflush(stdout) en cada printf  │ │
│  └───────────────────────────────────┘ │
│  → Código C válido y completo          │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│  GCC/CLANG                              │
│  ┌───────────────────────────────────┐ │
│  │ Paso 1: Generar ASM               │ │
│  │ gcc -S -masm=intel -O2            │ │
│  │ → output.asm (GAS, Intel syntax)  │ │
│  │                                   │ │
│  │ Paso 2: Compilar EXE              │ │
│  │ gcc -O2 -o output.exe input.c     │ │
│  │ → output.exe (ejecutable)         │ │
│  └───────────────────────────────────┘ │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│  EJECUTABLE (.exe)                      │
│  • Código optimizado                   │
│  • Sin dependencias                    │
│  • Output en tiempo real               │
└──────────────┬──────────────────────────┘
               │
               ▼
         ⚡ CPU Directo ⚡
```

## 🔧 Componentes Técnicos

### Parser Manual (`c_manual_parser.rs`)

```rust
pub struct CManualParser {
    source: String,
    position: usize,
}

// Funciones principales:
pub fn parse_program(source: &str) -> Result<Program, ...>
pub fn parse_while_from_text(while_text: &str) -> Result<(Expr, Vec<Stmt>), ...>
pub fn parse_if_from_text(if_text: &str) -> Result<(Expr, Vec<Stmt>), ...>
pub fn extract_while_loops(source: &str) -> Vec<(usize, usize, String)>
pub fn parse_expr_from_text(text: &str) -> Result<Expr, ...>
```

### Generador de C (`c_generator.rs`)

```rust
pub struct CGenerator {
    output: String,
    indent_level: usize,
}

pub fn generate_c_code(program: &Program) -> String {
    // Genera código C completo desde AST
}
```

### Compilador C (`c_compiler.rs`)

```rust
pub fn find_c_compiler() -> Option<String> {
    // Busca GCC/Clang en el sistema
}
```

## ✅ Ventajas del Flujo Actual

1. **Simplicidad:** Flujo directo sin capas innecesarias
2. **Control:** Parser manual = control total
3. **Optimización:** GCC/Clang -O2 = código optimizado automáticamente
4. **Compatibilidad:** Funciona con cualquier GCC/Clang
5. **Output en Tiempo Real:** fflush permite ver progreso mientras ejecuta
6. **ASM Limpio:** Genera código assembly legible y optimizado

## 📊 Ejemplo Completo

**Input ADead:**
```adead
let suma = 0
let limite = 10

while suma <= limite {
    if suma % 5 == 0 {
        print suma
    }
    suma = suma + 1
}
```

**Código C Generado:**
```c
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>

int main(void) {
    int64_t suma = 0LL;
    int64_t limite = 10LL;
    while ((suma <= limite)) {
        if (((suma % 5LL) == 0LL)) {
            printf("%ld\n", suma); fflush(stdout);
        }
        suma = (suma + 1LL);
    }
    return 0;
}
```

**ASM Generado (GCC):**
```asm
.file   "test.c"
.intel_syntax noprefix
.text
.section .rdata,"dr"
.LC0:
    .ascii "%ld\12\0"
.text.startup
main:
    push    rbx
    sub     rsp, 32
    mov     ebx, 0
    jmp     .L3
.L2:
    add     rbx, 1
.L3:
    cmp     rbx, 10
    jg      .L6
    mov     rax, rbx
    ; ... código optimizado para módulo ...
    call    __mingw_printf
    add     rbx, 1
    cmp     rbx, 11
    jne     .L3
.L6:
    xor     eax, eax
    add     rsp, 32
    pop     rbx
    ret
```

**Resultado Ejecutable:**
```
0
5
10
```

## ✅ Estado del Flujo

**Estado:** ✅ **COMPLETO Y FUNCIONAL**

- ✅ Parser manual funciona correctamente
- ✅ Generación de C funciona correctamente
- ✅ Compilación con GCC/Clang funciona
- ✅ Ejecutables funcionan correctamente
- ✅ Output en tiempo real funciona
- ✅ Ejemplos verificados y funcionando

