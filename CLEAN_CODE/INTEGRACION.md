# 🔗 Integración de CLEAN CODE con ADead

## Integración en el Proyecto Rust

### 1. Agregar como Dependencia

Edita `CORE/rust/Cargo.toml`:

```toml
[dependencies]
clean-code = { path = "../../CLEAN_CODE" }
```

### 2. Usar en el CLI

Edita `CORE/rust/crates/adead-cli/src/main.rs`:

```rust
use clean_code::AsmCleaner;

// En la función compile
fn compile_with_clean(input: &str, output: &str, clean: bool) -> Result<(), Box<dyn Error>> {
    // ... generar ASM sucio ...
    
    if clean {
        let cleaner = AsmCleaner::new();
        let clean_asm = cleaner.clean(&dirty_asm)?;
        std::fs::write(output, clean_asm)?;
    } else {
        std::fs::write(output, dirty_asm)?;
    }
    
    Ok(())
}
```

### 3. Agregar Flag --clean

```rust
// En clap::App
.arg(
    Arg::with_name("clean")
        .long("clean")
        .help("Aplica limpieza de ASM (CLEAN CODE)")
)
```

## Integración en el Pipeline

### Flujo Actual
```
ADead (.ad) → Parser → C Generator → GCC/Clang → ASM → EXE
```

### Flujo con CLEAN CODE
```
ADead (.ad) → Parser → C Generator → GCC/Clang → ASM Sucio → CLEAN CODE → ASM Virgen → NASM → EXE
```

## Uso desde CLI

```powershell
# Compilar con limpieza automática
.\target\release\adeadc.exe compile ejemplo.ad --clean -o ejemplo.asm

# Solo limpiar ASM existente
.\target\release\adeadc.exe clean ejemplo.asm -o ejemplo_clean.asm
```

## Benchmarking

Para medir las mejoras:

```rust
use clean_code::AsmCleaner;

let dirty = std::fs::read_to_string("dirty.asm")?;
let cleaner = AsmCleaner::new();
let clean = cleaner.clean(&dirty)?;

println!("Original: {} líneas", dirty.lines().count());
println!("Limpio: {} líneas", clean.lines().count());
println!("Reducción: {:.2}%", 
    (1.0 - clean.lines().count() as f64 / dirty.lines().count() as f64) * 100.0);
```

## Próximos Pasos

1. ✅ Integrar en CLI con flag `--clean`
2. ⏳ Agregar soporte para Agner Fog's objconv
3. ⏳ Análisis de data flow avanzado
4. ⏳ Optimizaciones específicas x86_64
5. ⏳ Benchmarking automático

