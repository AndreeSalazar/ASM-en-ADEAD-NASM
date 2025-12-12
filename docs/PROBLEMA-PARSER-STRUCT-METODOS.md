# Análisis: Problema con Parser de Structs con Métodos

**Fecha:** Diciembre 2025  
**Estado:** Identificado - Solución en progreso

## 🔍 Problema Identificado

El parser de structs con métodos (`init` y `destroy`) tiene un problema de diseño:

### Parser Actual

El parser en `crates/adead-parser/src/lib.rs` (líneas 357-400):

```rust
let struct_stmt = just("struct")
    .padded()
    .ignore_then(ident.clone())
    .then(
        just("{")
            .padded()
            .ignore_then(
                struct_field
                    .separated_by(just(",").padded())
                    .allow_trailing(),
            )
            .then_ignore(just("}").padded()),  // <-- CIERRA aquí
    )
    .then(  // <-- Espera métodos FUERA del bloque
        struct_method
            .repeated()
            .collect::<Vec<_>>(),
    )
```

### Formato Esperado por el Parser Actual

```
struct Banco {
    campo1: tipo1,
    campo2: tipo2
}
init(...) { ... }  // <-- Métodos FUERA del bloque
destroy() { ... }
```

### Formato que Estamos Usando (Más Natural)

```
struct Banco {
    campo1: tipo1,
    campo2: tipo2
    init(...) { ... }  // <-- Métodos DENTRO del bloque
    destroy() { ... }
}
```

## ✅ Lo que Funciona

- ✅ Structs simples sin métodos funcionan correctamente
- ✅ El parser parsea campos correctamente
- ✅ El parser parsea métodos cuando están fuera del bloque

## ❌ Lo que No Funciona

- ❌ Structs con métodos dentro del bloque `{}`
- ❌ Formato actual usado en ejemplos y tests
- ❌ `raii-init-destroy.ad` no compila con el parser actual

## 🔧 Solución Recomendada

**Estrategia:** Modificar el parser para aceptar una secuencia de elementos dentro del bloque `{}`, donde cada elemento puede ser:
1. Un campo (tiene `:` seguido de tipo)
2. Un método (empieza con `init` o `destroy` seguido de `(`)

### Implementación Propuesta

```rust
// Parser que distingue entre campo y método
let struct_item = struct_method
    .map(|m| ElementType::Method(m))
    .or(struct_field
        .map(|f| ElementType::Field(f)));

let struct_stmt = just("struct")
    .padded()
    .ignore_then(ident.clone())
    .then(
        just("{")
            .padded()
            .ignore_then(
                struct_item
                    .separated_by(just(",").padded().or_not())  // Coma opcional
                    .allow_trailing()
                    .collect::<Vec<_>>()
            )
            .then_ignore(just("}").padded()),
    )
    .map(|(name, items)| {
        // Separar campos y métodos
        let mut fields = Vec::new();
        let mut init = None;
        let mut destroy = None;
        
        for item in items {
            match item {
                ElementType::Field(f) => fields.push(f),
                ElementType::Method((name, m)) => {
                    if name == "init" { init = Some(m); }
                    else if name == "destroy" { destroy = Some(m); }
                }
            }
        }
        
        Stmt::Struct { name, fields, init, destroy }
    });
```

**Ventajas:**
- ✅ Más natural y legible
- ✅ Compatible con tests existentes
- ✅ Respeta el formato intuitivo (métodos dentro del struct)
- ✅ Compatible con NASM (no afecta generación de código)

## 📝 Archivos Afectados

- `crates/adead-parser/src/lib.rs` - Parser de structs (líneas 357-400)
- `Ejemplos-Reales/ejemplos/encapsulacion.ad` - Ejemplo actual
- `Ejemplos-Reales/ejemplos/raii-init-destroy.ad` - También afectado

## 🎯 Próximos Pasos

1. ✅ **Implementar parser que acepta métodos dentro del bloque** (en progreso)
2. ⏳ Probar con todos los tests existentes
3. ⏳ Verificar que genera código NASM correctamente
4. ⏳ Actualizar ejemplos si es necesario

## 💡 Nota sobre NASM

**El formato interno del struct NO afecta la generación de código NASM.** El parser solo estructura el AST; el backend genera el mismo código ASM independientemente de si los métodos están dentro o fuera del bloque en el código fuente.

**Respeta ASM en NASM:** ✅ La generación de código NASM no cambia, solo mejora la UX del parser.
