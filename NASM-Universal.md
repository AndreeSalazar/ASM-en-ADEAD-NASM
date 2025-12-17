# 🎯 NASM Universal - Guía de Generación y Patrones

**Propósito:** Documento base para mantener consistencia en la generación de código NASM  
**Última actualización:** Diciembre 2025  
**Estado:** ✅ Base establecida + Mejoras implementadas

---

## 📊 Estado de Implementación Actual

| Componente | Estado | Notas |
|------------|--------|-------|
| Arrays (10 métodos) | ✅ 100% | Optimizado con `rep movsq` |
| Strings (6 métodos) | ✅ 100% | ASCII-only |
| Funciones ABI-safe | ✅ 100% | Windows x64 compliant |
| For Loops | ✅ Backend | Falta parser sintáctico |
| Break/Continue | ✅ Backend | Falta parser sintáctico |
| LoopContext | ✅ 100% | Loops anidados soportados |
| Stdlib (11 funciones) | ✅ 100% | min, max, abs, pow, clamp, sign, gcd, lcm, factorial, is_even, is_odd |
| Debug Symbols | ✅ 100% | En todos los statements |
| Dead Code Elimination | ✅ 100% | DependencyGraph + UsageAnalyzer |

---

## 📋 Tabla de Contenidos

1. [Principios Fundamentales](#principios-fundamentales)
2. [Convenciones de Generación](#convenciones-de-generación)
3. [Patrones de Funciones Helper](#patrones-de-funciones-helper)
4. [ABI Compliance](#abi-compliance)
5. [Dead Code Elimination](#dead-code-elimination)
6. [Runtime Boundary](#runtime-boundary)
7. [Debug Symbols](#debug-symbols)
8. [Ownership y Memory Management](#ownership-y-memory-management)
9. [Error Handling](#error-handling)
10. [Estructuras de Datos](#estructuras-de-datos)
11. [Guía para Nuevas Funcionalidades](#guía-para-nuevas-funcionalidades)
12. [Checklist de Próximos Pasos](#checklist-de-próximos-pasos)

---

## 🎯 Principios Fundamentales

### **1. Generación Directa**
- ✅ **SIEMPRE** generar NASM directamente desde el AST
- ❌ **NUNCA** pasar por capas intermedias (C++, Rust, etc.)
- ✅ Control total sobre el código generado

### **2. Consistencia**
- ✅ Usar los mismos patrones en todas las funciones
- ✅ Seguir las mismas convenciones de nombres
- ✅ Mantener estructura uniforme

### **3. ABI Compliance**
- ✅ **SIEMPRE** cumplir con Windows x64 ABI
- ✅ Stack alignment a 16 bytes
- ✅ Shadow space de 32 bytes
- ✅ Preservar registros no volátiles

### **4. Dead Code Elimination**
- ✅ **SIEMPRE** verificar si una función se usa antes de generarla
- ✅ Usar `DependencyGraph` para rastrear dependencias
- ✅ Solo generar código necesario

### **5. Runtime Boundary**
- ✅ Marcar claramente qué es runtime vs código usuario
- ✅ Separar visualmente en el código generado
- ✅ Documentar con comentarios

---

## 📐 Convenciones de Generación

### **Estructura de Archivo NASM**

```asm
default rel                    ; Direccionamiento relativo
section .text                  ; Código ejecutable

; ============================================
; RUNTIME: Extern Declarations
; ============================================
extern GetStdHandle
extern WriteFile
extern ExitProcess
extern VirtualAlloc
extern VirtualFree

global main

; ============================================
; RUNTIME: Sistema de Panic (si se usa)
; ============================================
; Solo se genera si deps.uses_panic() == true

; ============================================
; RUNTIME: Funciones Helper de Array (si se usan)
; ============================================
; Solo se genera si deps.uses_arrays() == true

; ============================================
; RUNTIME: Funciones Helper de String (si se usan)
; ============================================
; Solo se genera si deps.uses_strings() == true

; ============================================
; RUNTIME: Librería Estándar (Stdlib)
; ============================================
; Funciones predefinidas (min, max, abs, pow)

; ============================================
; RUNTIME BOUNDARY END: Código Generado del Usuario
; ============================================

; Funciones definidas por el usuario
; (generadas antes de main)

main:
    ; Código del programa principal
    ; ...
    call ExitProcess

section .data
    ; Datos estáticos (strings, constantes)
```

### **Convenciones de Nombres**

| Tipo | Convención | Ejemplo |
|------|------------|---------|
| **Funciones Helper** | `tipo_operacion` | `array_get`, `string_concat` |
| **Funciones de Usuario** | `nombre_funcion` | `factorial`, `suma` |
| **Labels Internos** | `.label_local` | `.capacity_ok`, `.copy_done` |
| **Variables Globales** | `nombre_variable` | `stdout_handle` |
| **Constantes** | `NOMBRE_CONSTANTE` | `MEM_COMMIT`, `PAGE_READWRITE` |
| **Mensajes de Error** | `panic_msg_tipo` | `panic_msg_out_of_bounds` |

### **Comentarios en NASM**

```asm
; ============================================
; Sección: Descripción
; ============================================
; Para secciones grandes

; Función: Descripción breve
; Parámetros: RCX = ..., RDX = ...
; Retorna: RAX = ...
; Para funciones

    ; Operación específica
    ; Para operaciones dentro de funciones

    ; ADead: line X - statement original
    ; Para debug symbols
```

---

## 🔧 Patrones de Funciones Helper

### **Estructura Estándar de Función Helper**

```asm
nombre_funcion:
    ; ============================================
    ; PROLOGUE ABI-SAFE
    ; ============================================
    push rbp
    mov rbp, rsp
    
    ; Preservar registros no volátiles si se usan
    push rbx      ; Si se usa RBX
    push r12      ; Si se usa R12
    push r13      ; Si se usa R13
    push r14      ; Si se usa R14
    push r15      ; Si se usa R15
    
    ; Reservar shadow space (32 bytes) si se llama a funciones externas
    sub rsp, 32
    
    ; Alinear stack a 16 bytes
    and rsp, -16
    
    ; ============================================
    ; CUERPO DE LA FUNCIÓN
    ; ============================================
    ; Parámetros disponibles:
    ; - RCX = primer parámetro
    ; - RDX = segundo parámetro
    ; - R8 = tercer parámetro
    ; - R9 = cuarto parámetro
    ; - [rbp + 16 + 8*N] = parámetros adicionales (N = 0, 1, 2...)
    
    ; Lógica de la función
    ; ...
    
    ; ============================================
    ; EPILOGUE ABI-SAFE
    ; ============================================
    ; Restaurar stack
    mov rsp, rbp
    
    ; Restaurar registros preservados
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    
    ; Restaurar RBP
    pop rbp
    
    ; Retornar
    ret
```

### **Función Helper con VirtualAlloc/VirtualFree**

```asm
funcion_que_alloca:
    ; PROLOGUE (con shadow space)
    push rbp
    mov rbp, rsp
    push rbx
    push r12
    sub rsp, 32      ; Shadow space
    and rsp, -16     ; Alinear stack
    
    ; Preservar parámetros
    mov r12, rcx     ; Preservar en registro no volátil
    
    ; Llamar a VirtualAlloc
    ; Stack ya está alineado
    mov rcx, 0       ; lpAddress (NULL = auto)
    mov rdx, 64      ; dwSize
    mov r8, 0x1000   ; MEM_COMMIT
    mov r9, 0x04     ; PAGE_READWRITE
    call VirtualAlloc
    
    ; Usar resultado
    mov [r12 + 0], rax  ; Guardar puntero
    
    ; EPILOGUE
    mov rsp, rbp
    pop r12
    pop rbx
    pop rbp
    ret
```

### **Función Helper Simple (Sin Shadow Space)**

```asm
funcion_simple:
    ; PROLOGUE (sin shadow space, sin llamadas externas)
    push rbp
    mov rbp, rsp
    push rbx      ; Solo si se usa
    
    ; Cuerpo de la función
    ; ...
    
    ; EPILOGUE
    pop rbx
    pop rbp
    ret
```

### **Función Helper con Panic**

```asm
funcion_con_panic:
    ; PROLOGUE
    push rbp
    mov rbp, rsp
    push r12
    sub rsp, 32
    and rsp, -16
    
    ; Verificar null pointer
    test rcx, rcx
    jz panic_null_pointer
    
    ; Verificar bounds
    mov r12, rcx
    cmp rdx, [r12 + 8]  ; Comparar índice con length
    jge panic_out_of_bounds
    cmp rdx, 0
    jl panic_out_of_bounds
    
    ; Lógica normal
    ; ...
    
    ; EPILOGUE
    mov rsp, rbp
    pop r12
    pop rbp
    ret
```

---

## 🛡️ ABI Compliance

### **Windows x64 Calling Convention**

#### **Parámetros**
- **Primeros 4 parámetros enteros:** RCX, RDX, R8, R9
- **Parámetros adicionales:** Stack `[rbp + 16 + 8*N]` donde N = 0, 1, 2...
- **Primeros 4 parámetros float:** XMM0, XMM1, XMM2, XMM3
- **Parámetros float adicionales:** Stack

#### **Valor de Retorno**
- **Entero:** RAX
- **Float:** XMM0
- **Estructura grande:** Puntero en RAX

#### **Shadow Space**
- **SIEMPRE** reservar 32 bytes antes de cada `call`
- Incluso si la función no tiene parámetros
- Stack debe estar alineado a 16 bytes ANTES de reservar shadow space

#### **Stack Alignment**
- **SIEMPRE** alinear stack a 16 bytes antes de `call`
- Usar: `and rsp, -16`
- Verificar: `(rsp % 16) == 0` antes de cada `call`

#### **Registros Preservados (Callee-Saved)**
- **Deben preservarse:** RBX, RBP, RDI, RSI, R12-R15, XMM6-XMM15
- **Pueden modificarse:** RAX, RCX, RDX, R8-R11, XMM0-XMM5

### **Patrón ABI-Safe**

```rust
// En CodeGenerator
fn generate_abi_prologue(&mut self, needs_shadow_space: bool) {
    self.text_section.push("    push rbp".to_string());
    self.text_section.push("    mov rbp, rsp".to_string());
    
    if needs_shadow_space {
        self.text_section.push("    sub rsp, 32  ; Shadow space".to_string());
    }
    
    // Alinear stack
    self.text_section.push("    and rsp, -16  ; Alinear a 16 bytes".to_string());
}

fn generate_abi_epilogue(&mut self, needs_shadow_space: bool) {
    self.text_section.push("    mov rsp, rbp".to_string());
    
    if needs_shadow_space {
        // Shadow space ya está incluido en rbp
    }
    
    self.text_section.push("    pop rbp".to_string());
    self.text_section.push("    ret".to_string());
}

fn ensure_stack_alignment_before_call(&mut self, context: &str) {
    // Verificar que stack está alineado
    // Si no, ajustar antes de call
    self.text_section.push(format!("    ; Verificar stack alignment antes de {}", context));
    // Stack debe estar alineado después de prologue
}
```

---

## 🗑️ Dead Code Elimination

### **Sistema de Dependency Graph**

```rust
// Estructura
pub struct DependencyGraph {
    used_functions: HashSet<String>,
    dependencies: HashMap<String, Vec<String>>,
}

// Uso
let mut deps = DependencyGraph::new();
UsageAnalyzer::analyze_program(program, &mut deps);

// Generar solo si se usa
if deps.should_generate("array_get") {
    self.generate_array_get();
}
```

### **Patrón para Funciones Helper**

```rust
fn generate_array_helpers_nasm_selective(&mut self, deps: &DependencyGraph) {
    // Cada función tiene su condicional
    if deps.should_generate("array_new") {
        self.text_section.push("array_new:".to_string());
        // ... código de array_new
        self.text_section.push("".to_string());
    }
    
    if deps.should_generate("array_get") {
        self.text_section.push("array_get:".to_string());
        // ... código de array_get
        self.text_section.push("".to_string());
    }
    
    // ... etc
}
```

### **Mapeo de Dependencias**

```rust
// En DependencyGraph::new()
deps.insert("array_append".to_string(), vec!["panic_null_pointer".to_string()]);
deps.insert("array_get".to_string(), vec!["panic_out_of_bounds".to_string(), "panic_null_pointer".to_string()]);
```

### **Análisis de Uso**

```rust
// En UsageAnalyzer
fn analyze_expr(expr: &Expr, deps: &mut DependencyGraph) {
    match expr {
        Expr::Index { .. } => {
            deps.mark_used("array_get");
        }
        Expr::ArrayLiteral(_) => {
            deps.mark_used("array_from_values");
        }
        Expr::Call { name, .. } => {
            match name.as_str() {
                "array_append" => deps.mark_used("array_append"),
                // ... etc
            }
        }
        // ... etc
    }
}
```

---

## 🏗️ Runtime Boundary

### **Marcado Visual**

```asm
; ============================================
; RUNTIME: Sistema de Panic
; ============================================
; Estas funciones son parte del runtime de ADead
; NO son código generado del usuario

panic_out_of_bounds:
    ; ...

; ============================================
; RUNTIME: Funciones Helper de Array
; ============================================
; Funciones helper del runtime para operaciones con arrays

array_new:
    ; ...

; ============================================
; RUNTIME: Librería Estándar (Stdlib)
; ============================================
; Funciones predefinidas disponibles en todos los programas

stdlib_min:
    ; ...

; ============================================
; RUNTIME BOUNDARY END: Código Generado del Usuario
; ============================================
; A partir de aquí, código generado del usuario

usuario_factorial:
    ; ...
```

### **Comentarios en Código Rust**

```rust
// ============================================
// RUNTIME BOUNDARY: Funciones Helper del Runtime
// ============================================
// Estas funciones son parte del runtime de ADead
// NO son código generado del usuario, son helpers del sistema
// SOLO se generan si se usan (dead code elimination)

if deps.uses_panic() {
    self.generate_panic_system();
}

// ============================================
// RUNTIME BOUNDARY END: Código Generado del Usuario
// ============================================
```

---

## 🔍 Debug Symbols

### **Formato de Comentarios**

```asm
; ADead: line X - statement original
```

### **Aplicación**

```rust
// En CodeGenerator
fn add_debug_comment(&mut self, comment: &str) {
    if self.current_line > 0 {
        self.text_section.push(format!("    ; ADead: line {} - {}", self.current_line, comment));
    } else {
        self.text_section.push(format!("    ; ADead: {}", comment));
    }
}

// Uso
self.current_line = 5;
self.add_debug_comment("let x = 5");
// Genera: ; ADead: line 5 - let x = 5
```

### **Aplicar a Statements**

```rust
match stmt {
    Stmt::Let { name, value, .. } => {
        self.current_line = stmt.line_number;
        self.add_debug_comment(&format!("let {} = ...", name));
        // ... generar código
    }
    Stmt::Print(expr) => {
        self.current_line = stmt.line_number;
        self.add_debug_comment("print ...");
        // ... generar código
    }
    // ... etc
}
```

---

## 💾 Ownership y Memory Management

### **Reglas de Ownership**

| Operación | Ownership | Quién Libera |
|-----------|-----------|--------------|
| `array_new()` | Creador | Caller debe llamar `array_free()` |
| `array_from_values()` | Creador | Caller debe llamar `array_free()` |
| `array_append()` | Mutación | Mismo ownership, no libera |
| `string_concat()` | Creador | Caller debe llamar `string_free()` |
| `string_slice()` | Creador | Caller debe llamar `string_free()` |

### **Patrón de Liberación**

```asm
; Ejemplo: Liberar array antes de salir
mov rcx, arr_pointer
call array_free

; Verificar error (opcional)
cmp rax, 0
jne .error_handler
```

### **RAII (Resource Acquisition Is Initialization)**

```rust
// En CodeGenerator
// Rastrear variables que necesitan liberación
self.variables_to_destroy.push(("arr", "Array".to_string()));

// Antes de ExitProcess, liberar todas
for (var_name, struct_type) in &self.variables_to_destroy {
    match struct_type.as_str() {
        "Array" => {
            // Generar código para array_free
        }
        "String" => {
            // Generar código para string_free
        }
    }
}
```

---

## ⚠️ Error Handling

### **Sistema de Panic**

```asm
; ============================================
; RUNTIME: Sistema de Panic
; ============================================

panic_out_of_bounds:
    ; Imprimir mensaje de error
    mov rcx, -11  ; STD_OUTPUT_HANDLE
    call GetStdHandle
    mov r12, rax  ; Preservar handle
    
    ; Escribir mensaje
    mov rcx, r12
    lea rdx, [rel panic_msg_out_of_bounds]
    mov r8, panic_msg_out_of_bounds_len
    lea r9, [rsp + 32]  ; lpNumberOfBytesWritten (en shadow space)
    mov qword [rsp + 32], 0  ; Reservar espacio
    call WriteFile
    
    ; Salir con error
    mov rcx, 1  ; Exit code = 1 (error)
    call ExitProcess

panic_null_pointer:
    ; Similar a panic_out_of_bounds
    ; ...
```

### **Uso de Panic**

```asm
; En funciones helper
array_get:
    ; Verificar null pointer
    test rcx, rcx
    jz panic_null_pointer
    
    ; Verificar bounds
    mov r12, rcx
    cmp rdx, [r12 + 8]  ; índice vs length
    jge panic_out_of_bounds
    cmp rdx, 0
    jl panic_out_of_bounds
    
    ; Lógica normal
    ; ...
```

### **Mensajes de Error en .data**

```asm
section .data
    panic_msg_out_of_bounds: db "Error: Array index out of bounds", 0xA, 0
    panic_msg_out_of_bounds_len equ $ - panic_msg_out_of_bounds
    
    panic_msg_null_pointer: db "Error: Null pointer dereference", 0xA, 0
    panic_msg_null_pointer_len equ $ - panic_msg_null_pointer
```

---

## 📦 Estructuras de Datos

### **Array Structure**

```asm
; Estructura Array (24 bytes):
; [offset + 0]  : data (qword) - puntero a memoria dinámica
; [offset + 8]  : length (qword) - número de elementos
; [offset + 16] : capacity (qword) - capacidad total

; Acceso:
mov rax, [array_ptr + 0]   ; data
mov rax, [array_ptr + 8]   ; length
mov rax, [array_ptr + 16]  ; capacity
```

### **String Structure**

```asm
; Estructura String (32 bytes):
; [offset + 0]  : data (qword) - puntero a memoria dinámica (char*)
; [offset + 8]  : length (qword) - número de caracteres
; [offset + 16] : capacity (qword) - capacidad total
; [offset + 24] : hash (qword) - hash cacheado (0 = no calculado)

; Acceso:
mov rax, [string_ptr + 0]   ; data
mov rax, [string_ptr + 8]   ; length
mov rax, [string_ptr + 16]  ; capacity
mov rax, [string_ptr + 24]  ; hash
```

### **Patrón de Allocación**

```asm
; 1. Allocar estructura
mov rcx, 0
mov rdx, 24  ; Tamaño de Array struct
mov r8, 0x1000  ; MEM_COMMIT
mov r9, 0x04  ; PAGE_READWRITE
call VirtualAlloc
mov r12, rax  ; Preservar puntero a struct

; 2. Allocar data buffer
mov rcx, 0
mov rdx, 32  ; Tamaño de data (capacity * sizeof(element))
mov r8, 0x1000
mov r9, 0x04
call VirtualAlloc

; 3. Inicializar struct
mov [r12 + 0], rax  ; data = puntero a buffer
mov qword [r12 + 8], 0  ; length = 0
mov qword [r12 + 16], 4  ; capacity = 4
```

### **Patrón de Liberación**

```asm
; 1. Liberar data buffer
mov rcx, [array_ptr + 0]  ; data pointer
test rcx, rcx
jz .skip_data_free
mov rdx, 0
mov r8, 0x8000  ; MEM_RELEASE
call VirtualFree

.skip_data_free:
; 2. Liberar struct
mov rcx, array_ptr
mov rdx, 0
mov r8, 0x8000
call VirtualFree
```

---

## 🚀 Guía para Nuevas Funcionalidades

### **Checklist para Implementar Nueva Funcionalidad**

#### **1. Análisis y Diseño**
- [ ] Definir estructura de datos (si aplica)
- [ ] Definir funciones helper necesarias
- [ ] Mapear dependencias en `DependencyGraph`
- [ ] Documentar ownership rules

#### **2. Implementación en Parser**
- [ ] Agregar variantes al AST (`Expr` o `Stmt`)
- [ ] Implementar parser para nueva sintaxis
- [ ] Agregar tests de parsing

#### **3. Implementación en CodeGenerator**
- [ ] Agregar caso en `generate_expr()` o `generate_stmt()`
- [ ] Implementar funciones helper en NASM
- [ ] Agregar condicionales para dead code elimination
- [ ] Agregar debug symbols

#### **4. Integración con Dependency Graph**
- [ ] Agregar dependencias en `DependencyGraph::new()`
- [ ] Agregar detección en `UsageAnalyzer::analyze_expr()`
- [ ] Verificar que se genera solo si se usa

#### **5. Testing**
- [ ] Crear test básico
- [ ] Verificar generación NASM
- [ ] Verificar ejecución correcta
- [ ] Verificar dead code elimination

#### **6. Documentación**
- [ ] Actualizar `meta.md`
- [ ] Agregar ejemplos de uso
- [ ] Documentar ownership rules
- [ ] Documentar error handling

### **Ejemplo: Implementar Nueva Función Helper**

```rust
// 1. Agregar a DependencyGraph
deps.insert("nueva_funcion".to_string(), vec!["dependencia1".to_string()]);

// 2. Agregar detección en UsageAnalyzer
match name.as_str() {
    "nueva_funcion" => deps.mark_used("nueva_funcion"),
    // ...
}

// 3. Implementar función helper
fn generate_nueva_funcion(&mut self) {
    self.text_section.push("nueva_funcion:".to_string());
    self.generate_abi_prologue(true);  // Con shadow space si necesita
    
    // Lógica de la función
    // ...
    
    self.generate_abi_epilogue(true);
    self.text_section.push("".to_string());
}

// 4. Agregar condicional en función selectiva
if deps.should_generate("nueva_funcion") {
    self.generate_nueva_funcion();
}
```

### **Ejemplo: Implementar Nueva Estructura de Datos**

```rust
// 1. Definir estructura
// Estructura NuevaStruct (X bytes):
// [offset + 0]  : campo1 (qword)
// [offset + 8]  : campo2 (qword)
// ...

// 2. Implementar funciones helper
fn generate_nueva_struct_new(&mut self) {
    // Allocar memoria
    // Inicializar campos
}

fn generate_nueva_struct_free(&mut self) {
    // Liberar memoria
}

// 3. Agregar a DependencyGraph
deps.insert("nueva_struct_new".to_string(), vec![]);
deps.insert("nueva_struct_free".to_string(), vec![]);

// 4. Agregar detección en UsageAnalyzer
match expr {
    Expr::NuevaStructLiteral(_) => {
        deps.mark_used("nueva_struct_new");
    }
    // ...
}
```

---

## 📝 Convenciones de Código

### **Orden de Secciones en Archivo NASM**

1. `default rel`
2. `section .text`
3. `extern` declarations
4. `global` declarations
5. Sistema de panic (si se usa)
6. Funciones helper de arrays (si se usan)
7. Funciones helper de strings (si se usan)
8. Librería estándar
9. **RUNTIME BOUNDARY END**
10. Funciones de usuario
11. `main`
12. `section .data`
13. Mensajes de error (si se usan)
14. Strings literales

### **Orden de Operaciones en Funciones**

1. Prologue ABI-safe
2. Preservar parámetros en registros no volátiles
3. Validaciones (null pointer, bounds, etc.)
4. Lógica principal
5. Preparar valor de retorno
6. Epilogue ABI-safe

### **Uso de Registros**

| Registro | Uso Recomendado | Preservar? |
|----------|----------------|------------|
| **RAX** | Valor de retorno, cálculos temporales | ❌ No |
| **RCX** | Primer parámetro, contador | ❌ No |
| **RDX** | Segundo parámetro, datos temporales | ❌ No |
| **R8** | Tercer parámetro, datos temporales | ❌ No |
| **R9** | Cuarto parámetro, datos temporales | ❌ No |
| **R10** | Datos temporales | ❌ No |
| **R11** | Datos temporales | ❌ No |
| **RBX** | Preservar datos entre llamadas | ✅ Sí |
| **RBP** | Stack frame pointer | ✅ Sí |
| **RDI** | Preservar datos | ✅ Sí |
| **RSI** | Preservar datos | ✅ Sí |
| **R12-R15** | Preservar datos | ✅ Sí |

### **Patrón de Preservación de Registros**

```asm
funcion:
    push rbp
    mov rbp, rsp
    push r12  ; Preservar R12
    push r13  ; Preservar R13
    
    ; Usar R12 y R13 libremente
    mov r12, rcx  ; Preservar parámetro
    mov r13, rdx  ; Preservar parámetro
    
    ; Llamar a función externa (puede modificar R12, R13)
    call alguna_funcion
    
    ; R12 y R13 siguen intactos
    
    ; Restaurar
    pop r13
    pop r12
    pop rbp
    ret
```

---

## 🔄 Optimizaciones Comunes

### **1. Rep Movsq para Copias Rápidas**

```asm
; Copiar array de elementos qword
mov rdi, destino
mov rsi, fuente
mov rcx, count  ; Número de elementos (qwords)
cld  ; Clear direction flag (forward)
rep movsq  ; Copiar 8 bytes a la vez - MUCHO MÁS RÁPIDO
```

### **2. Arena Allocator para Arrays**

```asm
; En lugar de VirtualAlloc para cada append:
; 1. Pre-reservar capacidad mayor
; 2. Duplicar capacity cuando se llena
; 3. Usar rep movsq para copia rápida
```

### **3. Dead Code Elimination**

```rust
// Solo generar funciones usadas
if deps.should_generate("funcion") {
    self.generate_funcion();
}
```

### **4. Register Optimization**

```rust
// Solo preservar registros que realmente se usan
// Analizar uso de registros por función
// Optimizar prologue/epilogue
```

---

## 🎯 Patrones para Futuras Implementaciones

### **Módulos**

```rust
// Generar NASM por módulo
fn generate_module(&mut self, module: &Module) -> Result<String> {
    // Namespace: math.sqrt() → math_sqrt
    let prefix = module.name.clone();
    
    for func in &module.functions {
        let name = format!("{}_{}", prefix, func.name);
        self.generate_function_with_name(func, &name);
    }
}

// Generar extern para funciones importadas
fn generate_extern_declarations(&mut self, imports: &[Import]) {
    for import in imports {
        for func in &import.functions {
            let name = format!("{}_{}", import.module, func.name);
            self.text_section.push(format!("extern {}", name));
        }
    }
}
```

### **For Loops**

```rust
// for i in 0..10
fn generate_for_range(&mut self, var: &str, start: i64, end: i64) {
    let loop_label = self.new_label();
    let end_label = self.new_label();
    
    // Inicializar contador
    self.text_section.push(format!("    mov {}, {}", var, start));
    
    // Label de loop
    self.text_section.push(format!("{}:", loop_label));
    
    // Verificar condición
    self.text_section.push(format!("    cmp {}, {}", var, end));
    self.text_section.push(format!("    jge {}", end_label));
    
    // Cuerpo del loop (se genera después)
    // ...
    
    // Incrementar contador
    self.text_section.push(format!("    inc {}", var));
    self.text_section.push(format!("    jmp {}", loop_label));
    
    // Label de fin
    self.text_section.push(format!("{}:", end_label));
}
```

### **Break/Continue**

```rust
// Sistema de labels para break/continue
struct LoopContext {
    break_label: String,
    continue_label: String,
}

// En generate_while o generate_for
let loop_ctx = LoopContext {
    break_label: self.new_label(),
    continue_label: self.new_label(),
};
self.loop_stack.push(loop_ctx);

// En generate_stmt para break/continue
match stmt {
    Stmt::Break => {
        let ctx = self.loop_stack.last().unwrap();
        self.text_section.push(format!("    jmp {}", ctx.break_label));
    }
    Stmt::Continue => {
        let ctx = self.loop_stack.last().unwrap();
        self.text_section.push(format!("    jmp {}", ctx.continue_label));
    }
}
```

---

## 📚 Referencias y Recursos

### **Documentos Relacionados**

- `meta.md` - Roadmap completo del proyecto
- `CORE/rust/crates/adead-backend/ERROR-CONVENTION.md` - Convención de errores
- `CORE/rust/crates/adead-backend/OWNERSHIP-RULES.md` - Reglas de ownership
- `CORE/rust/crates/adead-backend/RUNTIME-BOUNDARY.md` - Runtime boundary
- `OBJETIVO-OPTIMIZACION-FINAL.md` - Optimizaciones avanzadas

### **Archivos Clave**

- `CORE/rust/crates/adead-backend/src/lib.rs` - Generador principal
- `CORE/rust/crates/adead-backend/src/dependency_graph.rs` - Dead code elimination
- `CORE/rust/crates/adead-backend/src/usage_analyzer.rs` - Análisis de uso
- `CORE/rust/crates/adead-backend/src/stdlib.rs` - Librería estándar

### **Convenciones Externas**

- **Windows x64 ABI:** https://docs.microsoft.com/en-us/cpp/build/x64-calling-convention
- **NASM Manual:** https://www.nasm.us/docs.php
- **x86-64 Instruction Reference:** https://www.felixcloutier.com/x86/

---

## ✅ Checklist de Verificación

Antes de implementar cualquier nueva funcionalidad, verificar:

- [ ] ¿Sigue las convenciones de nombres?
- [ ] ¿Cumple con ABI compliance?
- [ ] ¿Tiene dead code elimination?
- [ ] ¿Está marcado correctamente el runtime boundary?
- [ ] ¿Tiene debug symbols?
- [ ] ¿Documenta ownership rules?
- [ ] ¿Maneja errores correctamente?
- [ ] ¿Preserva registros correctamente?
- [ ] ¿Alinea stack correctamente?
- [ ] ¿Reserva shadow space cuando es necesario?

---

## 📋 Checklist de Próximos Pasos

### **Prioridad 1: Parser Sintáctico** 🔥
- [ ] Parser para `for VAR in START..END { BODY }`
- [ ] Parser para keyword `break`
- [ ] Parser para keyword `continue`
- [ ] Parser para `for item in arr { }`

### **Prioridad 2: Operadores Lógicos** 🔥
- [ ] Agregar `BinOp::And` (&&)
- [ ] Agregar `BinOp::Or` (||)
- [ ] Agregar `Expr::Not` (!)
- [ ] Short-circuit evaluation en NASM
- [ ] Tests de operadores lógicos

### **Prioridad 3: Módulos** ⚡
- [ ] Generación NASM por módulo separado
- [ ] Namespaces: `math.sqrt()` → `math_sqrt`
- [ ] Generar `extern` para funciones importadas
- [ ] Generar `global` para funciones exportadas
- [ ] Integrar con Zig linker para múltiples .obj
- [ ] Resolución de dependencias circulares

### **Prioridad 4: Matemáticas FPU/SSE** ⚡
- [ ] `sqrt(x)` usando FPU/SSE
- [ ] `sin(x)`, `cos(x)`, `tan(x)`
- [ ] `log(x)`, `exp(x)`
- [ ] `floor(x)`, `ceil(x)`, `round(x)`
- [ ] Constantes: PI, E, TAU

### **Prioridad 5: OOP Básico** 📘
- [ ] Clases con campos en NASM
- [ ] Métodos de instancia
- [ ] vtable para polimorfismo
- [ ] Herencia simple

### **Prioridad 6: Operaciones Avanzadas** 📘
- [ ] `s.split(delim)`
- [ ] `s.join(arr)`
- [ ] `s.replace(old, new)`
- [ ] `arr.map(fn)`
- [ ] `arr.filter(fn)`
- [ ] `arr.reduce(fn, init)`

---

## 🎯 Resumen Ejecutivo

Este documento establece la base para:

1. **Consistencia:** Todos los cambios futuros siguen los mismos patrones
2. **Calidad:** ABI compliance, dead code elimination, error handling
3. **Mantenibilidad:** Código claro, documentado, trazable
4. **Escalabilidad:** Fácil agregar nuevas funcionalidades siguiendo los patrones
5. **Optimización:** Dead code elimination, register optimization, memory pooling

---

## 🚀 Logros Recientes (Diciembre 2025)

### **Implementado Esta Sesión:**
- ✅ `Stmt::For`, `Stmt::Break`, `Stmt::Continue` en AST
- ✅ Generación NASM completa para for/break/continue
- ✅ `LoopContext` para manejo de loops anidados
- ✅ Funciones stdlib: `clamp`, `sign`, `gcd`, `lcm`, `factorial`, `is_even`, `is_odd`
- ✅ Optimización `rep movsq` para copia rápida de arrays
- ✅ Debug symbols consistentes (`add_debug_comment()`)
- ✅ Actualización del borrow checker
- ✅ Actualización del usage analyzer

### **Próximo Objetivo Inmediato:**
Parser sintáctico para `for i in 0..10 { }`, `break`, `continue`

---

**Última actualización:** Diciembre 2025  
**Mantener actualizado:** Cada vez que se agregue nueva funcionalidad, actualizar este documento

---

**Este documento es la base para todos los cambios futuros. Consultar antes de implementar cualquier nueva funcionalidad.**

