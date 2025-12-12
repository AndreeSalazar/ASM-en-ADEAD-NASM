# 🚀 Comenzar con Fase 1.1 - Guía de Inicio Rápido

**Pasos concretos para comenzar a implementar Fase 1.1**

---

## 📋 Checklist de Preparación

Antes de empezar, asegúrate de tener:

- [ ] Rust instalado y actualizado
- [ ] NASM instalado y en PATH
- [ ] gcc/ld instalado (MinGW para Windows)
- [ ] Compilador actual funcionando: `cargo build --release`
- [ ] Ejemplos corriendo: `.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\hello.ad`

---

## 🎯 Objetivo: Implementar Fase 1.1

**Componentes:**
1. O0.1 - Sistema de Tipos Robusto
2. O0.2 - Ownership y Borrowing
3. O0.3 - Inmutabilidad por Defecto
4. O0.4 - Option/Result Types

---

## 📝 Paso 1: Extender Sistema de Tipos (O0.1)

### Archivo a Modificar: `crates/adead-common/src/lib.rs`

**Estado Actual:**
```rust
pub enum Type {
    Int64,
    String,
    Void,
}
```

**Acción:** Extender con todos los tipos necesarios (ver `IMPLEMENTACION-Fase-1.1.md`)

**Orden de implementación:**
1. Agregar tipos primitivos primero (int8-64, uint8-64, float32/64, bool, char)
2. Compilar y verificar que no rompe nada
3. Agregar tipos compuestos (Array, Tuple)
4. Agregar Option/Result (preparación para O0.4)
5. Agregar Ref (preparación para O0.2)

---

## 📝 Paso 2: Inmutabilidad (O0.3) - MÁS FÁCIL PRIMERO

### Archivo a Modificar: `crates/adead-parser/src/lib.rs`

**Estado Actual:**
```rust
Stmt::Let {
    name: String,
    value: Expr,
}
```

**Cambio:**
```rust
Stmt::Let {
    mutable: bool,  // Nuevo campo
    name: String,
    value: Expr,
}
```

**Por qué primero:** Es el cambio más simple, no depende de otros sistemas.

**Acciones:**
1. Agregar campo `mutable: bool` a `Stmt::Let`
2. Modificar parser para detectar `let mut`
3. Agregar verificación: no permitir mutar variables inmutables
4. Tests

---

## 📝 Paso 3: Option/Result Types (O0.4)

### Archivos a Modificar/Crear:
- `crates/adead-parser/src/lib.rs` - AST
- `crates/adead-common/src/lib.rs` - Ya extendido en Paso 1

**Por qué tercero:** Depende de tipos (Paso 1), pero es más simple que ownership.

**Acciones:**
1. Extender AST con `Some`, `None`, `Ok`, `Err`
2. Parser para estos constructores
3. Parser para `match` expressions
4. Type checking básico

---

## 📝 Paso 4: Ownership (O0.2) - MÁS COMPLEJO

### Archivos a Crear/Modificar:
- `crates/adead-parser/src/lib.rs` - AST con Borrow/Deref
- `crates/adead-borrow/` - Módulo nuevo completo

**Por qué último:** Es el más complejo, requiere los demás sistemas.

**Acciones:**
1. Crear crate `adead-borrow`
2. Extender AST con `Borrow`, `Deref`
3. Parser para `&` y `&mut`
4. Implementar borrow checker
5. Integrar con type checker

---

## 🧪 Estrategia de Testing Incremental

### Por Cada Paso:

1. **Compila primero:**
   ```bash
   cargo build
   ```

2. **Tests existentes deben pasar:**
   ```bash
   cargo test
   ```

3. **Ejemplos existentes deben funcionar:**
   ```bash
   cargo run --release -- run Ejemplos-Reales/ejemplos/hello.ad
   ```

4. **Agregar tests nuevos:**
   ```rust
   #[test]
   fn test_nueva_funcionalidad() {
       // ...
   }
   ```

---

## 📚 Documentación de Referencia

### Para Implementación:
- `IMPLEMENTACION-Fase-1.1.md` - Guía técnica detallada
- `IMPLEMENTACION-Guia-Desarrollo.md` - Guía general

### Para Usuarios (futuro):
- `01-Basico-Tipos.md` - Tipos básicos
- `02-Basico-Variables.md` - Variables e inmutabilidad
- `05-Intermedio-Ownership.md` - Ownership
- `06-Intermedio-Option-Result.md` - Option/Result

---

## 🎯 Primer Paso Concreto

### Empezar con O0.3 (Inmutabilidad) - El más simple

1. Abre `crates/adead-parser/src/lib.rs`
2. Busca `Stmt::Let`
3. Agrega campo `mutable: bool`
4. Modifica parser para detectar `let mut`
5. Compila: `cargo build`
6. Prueba con ejemplo:

```adead
let mut x = 10
x = 20
print x
```

---

## 💡 Tips

- **Un cambio a la vez**: Implementa una cosa, testea, commit
- **Tests primero (TDD)**: Escribe tests antes de implementar cuando sea posible
- **Compila frecuentemente**: No dejes errores acumularse
- **Documenta decisiones**: Comenta por qué haces algo de cierta manera

---

## 🐛 Si Algo Sale Mal

1. **Compilación falla:**
   - Revisa errores del compilador
   - Verifica que no rompiste sintaxis existente
   - Prueba con `cargo clean && cargo build`

2. **Tests fallan:**
   - Revisa qué tests fallan
   - Verifica si tu cambio afecta código existente
   - Ajusta tests o código según corresponda

3. **Ejemplos no funcionan:**
   - Verifica que no rompiste parser existente
   - Revisa mensajes de error
   - Prueba con ejemplos simples primero

---

## 📞 Próximos Pasos

Después de completar Fase 1.1:

1. ✅ Sistema de tipos robusto funcionando
2. ✅ Ownership system funcionando
3. ✅ Option/Result funcionando
4. ✅ Inmutabilidad funcionando

**Siguiente:** Fase 1.2 - OOP Básico (Structs/Clases con ownership)

---

*Guía de inicio - Fase 1.1*
*Última actualización: Diciembre 2025*

