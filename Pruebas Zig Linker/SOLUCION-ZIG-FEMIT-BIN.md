# Solución: Error "unrecognized file extension" con Zig -femit-bin

## Problema

El error ocurría porque `-femit-bin` necesita el **signo igual (=)** para especificar el path del archivo de salida.

**Comando incorrecto:**
```rust
cmd.arg("-femit-bin").arg(exe_file);  // ❌ Zig lo interpreta como dos argumentos separados
```

Esto genera:
```
zig build-exe -femit-bin programa.exe obj1.obj
```

Zig interpreta `programa.exe` como un archivo de entrada (porque no reconoce la extensión .exe como entrada válida).

## Solución

Usar el formato `-femit-bin=path` con el signo igual:

**Comando correcto:**
```rust
let emit_bin_arg = format!("-femit-bin={}", exe_file.display());
cmd.arg(&emit_bin_arg);
```

Esto genera:
```
zig build-exe -femit-bin=programa.exe obj1.obj  // ✅ Correcto
```

## Cambios Aplicados

**Antes:**
```rust
cmd.arg("-femit-bin")  // ❌ Argumento separado
    .arg(exe_file);
```

**Después:**
```rust
let emit_bin_arg = format!("-femit-bin={}", exe_file.display());
cmd.arg(&emit_bin_arg);  // ✅ Un solo argumento con =
```

## Sintaxis Correcta de Zig

Zig requiere que las opciones con valores usen el signo `=`:

- ✅ `-femit-bin=path` (correcto)
- ❌ `-femit-bin path` (incorrecto - Zig lo interpreta como dos argumentos)

## Estado

✅ **Corregido** - Ahora usa `-femit-bin=path` con el signo igual.

---

**Fecha:** Diciembre 2025

