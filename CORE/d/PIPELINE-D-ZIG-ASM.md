# Pipeline: ADead → D → Zig → ASM Directo

## 🔄 Flujo Completo

```
┌─────────────────────────────────────┐
│  ADead Source (.ad)                 │
│  • Sintaxis estilo Python           │
│  • Simple y legible                 │
└──────────────┬──────────────────────┘
               │
               ↓
┌─────────────────────────────────────┐
│  D Language (Metaprogramming)       │
│  • Parse del código ADead           │
│  • CTFE: Validación compile-time    │
│  • Templates: Generación de código  │
│  • Genera código Zig intermedio     │
└──────────────┬──────────────────────┘
               │ (Código Zig generado)
               ↓
┌─────────────────────────────────────┐
│  Zig (Codegen a NASM)               │
│  • Recibe código Zig                │
│  • Genera NASM directamente         │
│  • Sin overhead de capas            │
│  • Máxima eficiencia                │
└──────────────┬──────────────────────┘
               │ (Código NASM)
               ↓
┌─────────────────────────────────────┐
│  NASM (Assembly x86_64)             │
│  • Código ASM puro                  │
│  • Optimizado                       │
└──────────────┬──────────────────────┘
               │
               ↓
        ⚡ CPU Directo ⚡
```

## 🎯 Ventajas de este Pipeline

1. **Sin Rust:** Bypass completo de Rust, evita problemas de linking
2. **Metaprogramming Poderoso:** D Language hace parsing y validación avanzada
3. **Generación Directa:** Zig genera ASM sin capas intermedias
4. **Máxima Performance:** Código optimizado directamente para CPU
5. **Flujo Limpio:** Menos capas = menos errores

## 📋 Implementación

### Módulo D (`d/src/adead_d_to_zig.d`)
- Parsea código ADead
- Genera código Zig usando templates
- Llama a Zig para compilar a NASM

### Módulo Rust (`rust/.../d_zig_asm.rs`)
- Wrapper FFI para usar funciones D
- Integración con el compilador principal

### Módulo Zig (`zig/src/nasm_generator.zig`)
- Recibe código Zig generado por D
- Genera NASM directamente

## 🔧 Uso

```rust
use adead_parser::d_zig_asm::compile_adead_to_asm_via_zig;

let adead_code = r#"
    let x = 42
    print x
"#;

if let Some(asm_code) = compile_adead_to_asm_via_zig(adead_code) {
    // asm_code contiene código NASM puro
    println!("{}", asm_code);
}
```

## 🚀 Estado

- ✅ Estructura de módulos creada
- ⏳ Implementación en progreso
- ⏳ Testing pendiente

