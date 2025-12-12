# FFI Bridge Rust ↔ Zig 🔗

Este directorio contiene el código que permite que Rust y Zig trabajen juntos.

## Estructura

```
ffi/
├── rust_side/             # Código Rust para FFI
│   └── zig_ffi.rs         # Bindings Rust → Zig
└── README.md              # Este archivo
```

## Cómo Funciona

### Rust → Zig

Rust llama funciones de Zig a través de FFI usando:
- Funciones exportadas con `extern "C"`
- Structs C-compatibles con `#[repr(C)]`
- Manejo seguro de memoria

### Ejemplo

```rust
// En Rust
use ffi::rust_side::zig_ffi::parse_params_with_zig;

let params = parse_params_with_zig("nombre: string, edad: int64")?;
```

```zig
// En Zig
pub export fn parse_params_ffi(input: []const u8) ParamList {
    return parseParams(allocator, input);
}
```

## Integración

El código FFI se integra en:
- `rust/crates/adead-parser/` - Para parsing eficiente
- `rust/crates/adead-backend/` - Para code generation

## Build

1. Compilar Zig: `cd ../zig && zig build`
2. Compilar Rust: `cd ../rust && cargo build --release`

Rust automáticamente linkea la biblioteca de Zig.

