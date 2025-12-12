# Código Rust para ADead 🦀

Este directorio contiene todo el código Rust del proyecto.

## Estructura

```
rust/
├── crates/
│   ├── adead-cli/         # CLI principal
│   ├── adead-parser/      # Parser Chumsky (alto nivel)
│   ├── adead-borrow/      # Borrow checker
│   ├── adead-backend/     # Code generator (orquestación)
│   └── adead-common/      # Utilidades compartidas
├── Cargo.toml             # Workspace de Rust
└── Cargo.lock
```

## Compilación

```bash
cd rust/
cargo build --release
```

## Tests

```bash
cd rust/
cargo test --workspace
```

## Integración con Zig

Rust trabaja con Zig a través de FFI. Ver `../ffi/` para más detalles.

