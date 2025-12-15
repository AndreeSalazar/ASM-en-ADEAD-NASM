# ADead D Language Module

Módulo de metaprogramming avanzado en D Language para ADead.

## Características

- ✅ **CTFE (Compile-Time Function Execution)** - Ejecuta código en compile-time
- ✅ **Templates avanzados** - Generación de código ASM con templates
- ✅ **Validación en compile-time** - Verifica tipos y expresiones antes de ejecutar
- ✅ **Optimización automática** - Evalúa constantes en compile-time
- ✅ **Generación directa de ASM** - Crea código NASM puro

## Requisitos

- **D Language** (DMD o LDC)
  - DMD: https://dlang.org/download.html
  - LDC: https://github.com/ldc-developers/ldc/releases

## Compilación

### Windows (PowerShell)
```powershell
cd d
.\build.ps1
```

### Linux/Mac
```bash
cd d
chmod +x build.sh
./build.sh
```

## Integración con Rust

El módulo compila a un objeto `.obj` (Windows) o `.o` (Linux) que puede enlazarse con Rust:

```rust
// En Cargo.toml o build.rs
#[link(name = "adead_d")]
extern "C" {
    fn parseAndValidateExpr(source: *const c_char) -> *mut Expr;
    fn generateASMFromExpr(expr: *const Expr) -> *const c_char;
}
```

## Uso

Ver `src/adead_metaprog.d` para ejemplos de uso de:
- Templates para generación de ASM
- CTFE para validación
- Optimización de expresiones

## Autor

Eddi Andreé Salazar Matos  
Diciembre 2025

