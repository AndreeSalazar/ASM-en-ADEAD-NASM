# 🔧 Compatibilidad de Tipos con NASM

**Documentación técnica sobre cómo los tipos de ADead se mapean a NASM x86-64**

---

## 📋 Resumen

El sistema de tipos de ADead está diseñado para mapear directamente a representaciones en NASM x86-64, respetando las convenciones de tamaño, alineación y registros.

---

## 🎯 Tipos Primitivos y su Mapeo NASM

### Enteros

| Tipo ADead | Tamaño | Registro NASM | Directiva .data | Ejemplo |
|------------|--------|---------------|-----------------|---------|
| `int8` / `uint8` | 1 byte | `AL`, `BL`, `CL`, `DL` | `db` | `mov al, 42` |
| `int16` / `uint16` | 2 bytes | `AX`, `BX`, `CX`, `DX` | `dw` | `mov ax, 1234` |
| `int32` / `uint32` | 4 bytes | `EAX`, `EBX`, `ECX`, `EDX` | `dd` | `mov eax, 123456` |
| `int64` / `uint64` | 8 bytes | `RAX`, `RBX`, `RCX`, `RDX` | `dq` | `mov rax, 12345678` |

**Nota:** En x86-64, `int64` es el tipo estándar (por defecto para literales enteros).

### Punto Flotante

| Tipo ADead | Tamaño | Registro NASM | Directiva .data | Ejemplo |
|------------|--------|---------------|-----------------|---------|
| `float32` | 4 bytes | `XMM0-XMM15` | `dd` | `movss xmm0, [value]` |
| `float64` | 8 bytes | `XMM0-XMM15` | `dq` | `movsd xmm0, [value]` |

**Nota:** NASM usa registros XMM para punto flotante SSE.

### Otros Primitivos

| Tipo ADead | Tamaño | Representación NASM | Ejemplo |
|------------|--------|---------------------|---------|
| `bool` | 1 byte | `db` (0=false, 1=true) | `mov al, 1` |
| `char` | 1 byte | `db` (código ASCII/Unicode) | `mov al, 'A'` |

---

## 🔧 Métodos Disponibles en `Type`

### `size_bytes() -> usize`

Obtiene el tamaño del tipo en bytes (crítico para alineación de stack en NASM):

```rust
Type::Int64.size_bytes()    // 8
Type::Int32.size_bytes()    // 4
Type::Int8.size_bytes()     // 1
Type::String.size_bytes()   // 16 (puntero + longitud)
```

### `align_bytes() -> usize`

Obtiene la alineación necesaria (x86-64 requiere alineación de 16 bytes en stack):

```rust
Type::Int64.align_bytes()   // 8
Type::Float64.align_bytes() // 8
// Tipos mayores se alinean a 16 bytes
```

### `nasm_register_hint() -> (&str, &str)`

Sugiere el registro y tamaño de operación NASM:

```rust
Type::Int64.nasm_register_hint()   // ("rax", "qword")
Type::Int32.nasm_register_hint()   // ("eax", "dword")
Type::Int8.nasm_register_hint()    // ("al", "byte")
Type::Float64.nasm_register_hint() // ("xmm0", "qword")
```

### `nasm_declaration(label: &str) -> String`

Genera la declaración NASM en `.data`:

```rust
Type::Int64.nasm_declaration("mi_var")   
// "mi_var: dq 0"

Type::Int8.nasm_declaration("mi_byte")   
// "mi_byte: db 0"

Type::String.nasm_declaration("mi_str")  
// "mi_str: dq 0  ; string pointer\n    dq 0  ; string length"
```

### `is_copy() -> bool`

Verifica si el tipo es Copy (se puede copiar directamente en NASM):

```rust
Type::Int64.is_copy()     // true (primitivos son Copy)
Type::String.is_copy()    // false (se mueve, no copia)
Type::Ref { .. }.is_copy() // true (la referencia es Copy)
```

---

## 📐 Alineación de Stack en NASM

x86-64 requiere que la stack esté alineada a **16 bytes**. Al asignar variables en el stack:

```asm
; Ejemplo: asignar espacio para diferentes tipos
sub rsp, 8    ; int64 (8 bytes, alineado)
sub rsp, 16   ; 2 x int64 (alineado a 16 bytes)
sub rsp, 32   ; string (16 bytes) + int64 (8 bytes) + padding (8 bytes)
```

El método `align_bytes()` ayuda a calcular esto correctamente.

---

## 🔄 Ejemplos de Uso en Backend

### Generar código para asignar variable

```rust
fn allocate_variable(typ: &Type, name: &str) -> String {
    let size = typ.size_bytes();
    let align = typ.align_bytes();
    
    // Calcular offset alineado
    let aligned_size = ((size + align - 1) / align) * align;
    
    format!("    sub rsp, {}  ; allocate {} (aligned to {})", 
            aligned_size, name, align)
}
```

### Generar código para mover valor a registro

```rust
fn load_to_register(typ: &Type, source: &str) -> String {
    let (reg, size) = typ.nasm_register_hint();
    
    match size {
        "byte" => format!("    mov {}, [{}]", reg, source),
        "word" => format!("    mov {}, [{}]", reg, source),
        "dword" => format!("    mov {}, [{}]", reg, source),
        "qword" => format!("    mov {}, [{}]", reg, source),
        _ => format!("    mov {}, [{}]", reg, source),
    }
}
```

---

## ⚠️ Consideraciones Importantes

### 1. Strings en NASM

Strings se representan como:
- **Puntero** (8 bytes) - dirección del string
- **Longitud** (8 bytes) - tamaño del string

Total: 16 bytes en x86-64.

### 2. Arrays

- **Tamaño fijo**: `Array<Type, N>` se declara con `times N`
- **Dinámico**: `Array<Type>` es puntero (8 bytes) + capacidad (8 bytes)

### 3. Punto Flotante

Usa registros XMM (SSE), no la FPU stack tradicional:
- `movss` para float32
- `movsd` para float64

### 4. Referencias

Todas las referencias (`&T`, `&mut T`) son punteros de 8 bytes en x86-64.

---

## ✅ Verificación de Compatibilidad

El sistema de tipos está diseñado para:
- ✅ Mapear directamente a tamaños de registro NASM
- ✅ Respetar alineación de stack x86-64 (16 bytes)
- ✅ Usar convenciones estándar de NASM (db, dw, dd, dq)
- ✅ Compatible con llamadas a funciones (ABI x86-64)
- ✅ Preparado para tipos futuros (Option, Result, etc.)

---

*Documentación técnica - Compatibilidad NASM*
*Última actualización: Diciembre 2025*

