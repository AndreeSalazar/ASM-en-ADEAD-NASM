# Solución: Error "unrecognized parameter: '-o'" con Zig

## Problema

El error ocurría porque Zig `build-exe` **no acepta la opción `-o`** para especificar el archivo de salida.

**Comando incorrecto:**
```rust
zig build-exe obj1.obj -target x86_64-windows -lc -o programa.exe  // ❌ -o no existe
```

## Solución

Zig usa `-femit-bin=path` para especificar el archivo de salida, o simplemente infiere el nombre del primer archivo.

**Comando correcto:**
```rust
zig build-exe -target x86_64-windows -lc -femit-bin=programa.exe obj1.obj  // ✅ Correcto
```

## Cambios Aplicados

**Antes:**
```rust
cmd.arg("-target")
    .arg("x86_64-windows")
    .arg("-lc")
    .arg("-o")  // ❌ No existe en Zig
    .arg(exe_file);
```

**Después:**
```rust
cmd.arg("-target")
    .arg("x86_64-windows")
    .arg("-lc")
    .arg("-femit-bin")  // ✅ Opción correcta
    .arg(exe_file);
```

## Orden Correcto de Argumentos

Zig requiere que las opciones vengan **antes** de los archivos:

```
zig build-exe [opciones] [archivos]
```

**Ejemplo:**
```
zig build-exe -target x86_64-windows -lc -femit-bin=programa.exe obj1.obj obj2.obj
```

## Estado

✅ **Corregido** - El comando de Zig ahora usa `-femit-bin` en lugar de `-o`.

---

**Fecha:** Diciembre 2025

