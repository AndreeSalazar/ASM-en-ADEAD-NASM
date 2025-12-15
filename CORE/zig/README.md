# Módulos Zig para ADead 🦎

Este directorio contiene todo el código Zig que trabaja junto con Rust para generar código NASM.

## Estructura

```
zig/
├── src/
│   ├── parser.zig         # Parser eficiente de parámetros
│   ├── codegen.zig        # Generador de código NASM
│   └── main.zig           # Exportaciones FFI
├── build.zig              # Build system de Zig
├── build.zig.zon          # Dependencias Zig
└── README.md              # Este archivo
```

## Compilación

```bash
cd zig/
zig build
```

Esto genera `zig-out/lib/libadead_zig.a` que Rust puede linkear.

## Tests

```bash
cd zig/
zig test src/parser.zig
zig test src/codegen.zig
```

## Integración con Rust

Rust linkea esta biblioteca estática a través de FFI. Ver `../ffi/` para más detalles.

