# Módulos Zig para ADead 🦎

Este directorio contiene código Zig real que trabaja junto con Rust para generar código NASM.

## Módulos

### `parser.zig`
Parser eficiente de parámetros de funciones.
- Parsing rápido y directo
- Manejo eficiente de memoria
- Sin overhead

### `codegen.zig`
Generador de código NASM.
- Generación directa de instrucciones
- Optimizaciones específicas
- Control total del output

### `main.zig`
Módulo principal que exporta funciones para FFI con Rust.

## Tests

Ejecutar tests:

```bash
zig test parser.zig
zig test codegen.zig
```

## Build

Compilar biblioteca estática:

```bash
zig build
```

Esto genera `zig-out/lib/libadead_zig.a` que Rust puede linkear.

