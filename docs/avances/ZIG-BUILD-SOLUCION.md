# Solución: Compilación Zig 0.16.0

## Problema

Zig 0.16.0 dev.1484 tiene un problema con la API de `build.zig`:
```
error: no field or member function named 'addStaticLibrary' in 'Build'
note: use '.*' to dereference pointer
```

## Solución Temporal: Compilación Manual

Hasta que se resuelva la incompatibilidad de la API, se puede compilar manualmente:

### Windows/Linux
```bash
cd zig
zig build-lib src/expr_parser.zig --name adead_zig --library c -fno-strip -O Debug
```

### Script Automatizado

Usar `zig/build-manual.ps1` (Windows) o crear equivalente en bash:

```powershell
.\zig\build-manual.ps1
```

Este script:
1. Compila la biblioteca con `zig build-lib`
2. La copia a `zig-out/lib/adead_zig.lib`
3. Limpia archivos temporales

## Solución Definitiva (Pendiente)

Cuando Zig 0.16.0 estable esté disponible, actualizar `build.zig` con la API correcta.

**Código actual (no funciona en 0.16.0-dev):**
```zig
const lib = b.addStaticLibrary(.{
    .name = "adead_zig",
    .root_source_file = .{ .path = "src/main.zig" },
    .target = target,
    .optimize = optimize,
});
```

**API esperada (cuando esté disponible):**
- Verificar documentación oficial de Zig 0.16.0
- O usar versión estable (0.13.x, 0.14.x, etc.)

## Estado Actual

✅ **Parser Zig:** Completo y funcional (`expr_parser.zig`)
✅ **FFI:** Exportado correctamente (`parse_expr_ffi`)
✅ **Rust Wrapper:** Listo para usar (`zig_expr_parser.rs`)
✅ **Compilación:** Manual con `zig build-lib` (temporal)
⏳ **build.zig:** Pendiente actualización cuando API se estabilice

## Uso

1. Compilar Zig manualmente:
   ```bash
   cd zig
   .\build-manual.ps1  # Windows
   # O manualmente:
   zig build-lib src/expr_parser.zig --name adead_zig --library c
   ```

2. Compilar Rust (detectará la biblioteca automáticamente):
   ```bash
   cd rust
   cargo build
   ```

3. Listo para usar:
   ```rust
   // En adead-parser/src/lib.rs
   if let Some(zig_expr) = zig_expr_parser::parse_expr_with_zig("2 + 5") {
       // Usar expresión parseada por Zig
   }
   ```

---

**Nota:** Esta es una solución temporal. Una vez que Zig 0.16.0 estable esté disponible, se actualizará `build.zig` para usar la API oficial.

