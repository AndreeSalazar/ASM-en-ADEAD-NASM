# 🚀 Cómo Ejecutar CLEAN_CODE

## Método 1: Ejemplo Predefinido (Recomendado)

### Paso 1: Navegar a la carpeta CLEAN_CODE

```powershell
cd CLEAN_CODE
```

### Paso 2: Ejecutar el ejemplo

```powershell
# Compilar y ejecutar el ejemplo
cargo run --example clean_real_example --release
```

**Esto automáticamente:**
- ✅ Lee `examples/test_array_CLANG_dirty.asm`
- ✅ Aplica limpieza en 3 niveles (Básico, Avanzado, EXTREMO)
- ✅ Genera 3 archivos limpios en `examples/`
- ✅ Muestra estadísticas de reducción

---

## Método 2: Crear tu Propio Script

### Crear archivo `mi_limpieza.rs`:

```rust
use clean_code::{AsmCleaner, OptimizationLevel};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Leer tu archivo ASM
    let dirty_asm = fs::read_to_string("tu_archivo.asm")?;
    
    // Limpiar con nivel EXTREMO
    let cleaner = AsmCleaner::with_level(OptimizationLevel::Extreme);
    let clean_asm = cleaner.clean(&dirty_asm)?;
    
    // Guardar resultado
    fs::write("tu_archivo_limpio.asm", clean_asm)?;
    
    println!("✅ Limpieza completada!");
    Ok(())
}
```

### Ejecutar:

```powershell
# Agregar a Cargo.toml:
# [[example]]
# name = "mi_limpieza"
# path = "mi_limpieza.rs"

cargo run --example mi_limpieza --release
```

---

## Método 3: Usar como Biblioteca en tu Código

### En tu `Cargo.toml`:

```toml
[dependencies]
clean-code = { path = "../CLEAN_CODE" }
```

### En tu código Rust:

```rust
use clean_code::{AsmCleaner, OptimizationLevel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dirty_asm = std::fs::read_to_string("input.asm")?;
    
    // Nivel EXTREMO
    let cleaner = AsmCleaner::with_level(OptimizationLevel::Extreme);
    let clean_asm = cleaner.clean(&dirty_asm)?;
    
    std::fs::write("output.asm", clean_asm)?;
    Ok(())
}
```

---

## Comandos Rápidos

```powershell
# Desde la raíz del proyecto
cd CLEAN_CODE

# Ejecutar ejemplo predefinido
cargo run --example clean_real_example --release

# Solo compilar (sin ejecutar)
cargo build --release

# Ejecutar tests
cargo test

# Ver ayuda
cargo run --example clean_real_example -- --help
```

---

## Resultados Esperados

Al ejecutar `clean_real_example`, verás:

```
🧹 CLEAN CODE - Limpiando ASM real de Clang

📊 Estadísticas Originales:
   Líneas: 204
   Tamaño: 4249 bytes

🔧 Aplicando limpieza...

✅ Resultados:

┌─────────────┬──────────┬──────────┬─────────────┐
│ Nivel       │ Líneas   │ Tamaño   │ Reducción   │
├─────────────┼──────────┼──────────┼─────────────┤
│ Original    │      204 │     4249 │ 0%          │
│ Básico      │       28 │      582 │      86.3%  │
│ Avanzado    │       28 │      582 │      86.3%  │
│ EXTREMO 🔥  │       26 │      531 │      87.3%  │
└─────────────┴──────────┴──────────┴─────────────┘

📁 Archivos generados:
   - examples/test_array_CLANG_cleaned_basic.asm
   - examples/test_array_CLANG_cleaned_advanced.asm
   - examples/test_array_CLANG_cleaned_extreme.asm
```

---

## Solución de Problemas

### Error: "No such file or directory"
- Asegúrate de estar en la carpeta `CLEAN_CODE`
- Verifica que `examples/test_array_CLANG_dirty.asm` existe

### Error: "could not compile"
- Ejecuta `cargo clean` y luego `cargo build --release`
- Verifica que tienes Rust instalado: `rustc --version`

### Error: "example not found"
- Verifica que `Cargo.toml` tiene la sección `[[example]]`
- El nombre debe coincidir: `--example clean_real_example`

---

**¡Listo!** Ahora puedes limpiar cualquier ASM sucio con CLEAN_CODE 🔥

