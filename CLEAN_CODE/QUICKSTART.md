# ⚡ Quickstart - CLEAN CODE

## Instalación Rápida

```bash
cd CLEAN_CODE
cargo build --release
```

## Uso Básico

### Como Biblioteca Rust

```rust
use clean_code::{AsmCleaner, OptimizationLevel};

// Básico (default)
let cleaner = AsmCleaner::new();

// Avanzado
let cleaner = AsmCleaner::with_level(OptimizationLevel::Advanced);

// EXTREMO 🔥
let cleaner = AsmCleaner::with_level(OptimizationLevel::Extreme);

let dirty_asm = std::fs::read_to_string("dirty.asm")?;
let clean_asm = cleaner.clean(&dirty_asm)?;
std::fs::write("clean.asm", clean_asm)?;
```

### Ejemplo Completo

```rust
use clean_code::AsmCleaner;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ASM sucio
    let dirty = r#"
        mov rax, rax
        mov rbx, rbx
        jmp label1
    label1:
        mov rax, 5
    "#;
    
    // Limpiar
    let cleaner = AsmCleaner::new();
    let clean = cleaner.clean(dirty)?;
    
    println!("{}", clean);
    Ok(())
}
```

## Optimizaciones Aplicadas

✅ **Movimientos redundantes** - `mov rax, rax` → eliminado  
✅ **Saltos innecesarios** - `jmp label` seguido de `label:` → eliminado  
✅ **Simplificación** - `mov reg, 0` + `add reg, val` → `mov reg, val`  
✅ **Push/pop innecesarios** - `push reg` + `pop reg` → eliminado  
✅ **Nops múltiples** - Secuencias de `nop` → eliminadas  
✅ **Dead code** - Labels no referenciados → eliminados  
✅ **Código inalcanzable** - Después de `ret` → eliminado  

## Resultados Esperados

- **30-70% menos instrucciones** en código simple
- **ASM más legible** y fácil de optimizar
- **Mejor uso de registros**
- **Código más compacto**

## Próximos Pasos

1. Integrar en CLI con flag `--clean`
2. Agregar soporte para Agner Fog's objconv
3. Optimizaciones avanzadas x86_64

---

**Stack:** Rust + Regex + Peephole Optimizations  
**Objetivo:** ASM virgen puro directo al CPU ⚡

