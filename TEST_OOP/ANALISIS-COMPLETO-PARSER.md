# 🔍 Análisis Completo del Parser - Problema Identificado

**Fecha:** 17 de Diciembre 2025  
**Objetivo:** Identificar por qué los statements Let/Print no se generan después de funciones

---

## 📊 Análisis del Código del Parser

### 1. Estructura del Parser Principal

**Ubicación:** `CORE/rust/crates/adead-parser/src/lib.rs` - Función `program_parser()`

```rust
fn program_parser() -> impl Parser<char, Program, Error = Simple<char>> {
    ws_and_comments()
        .ignore_then(
            stmt_parser()
                .padded()
                .then_ignore(ws_and_comments())
                .repeated()
        )
        .then_ignore(end().or_not())
        .map(|stmts| Program {
            statements: stmts,
        })
}
```

**Análisis:**
- ✅ El parser usa `.repeated()` para parsear múltiples statements
- ✅ Usa `ws_and_comments()` para ignorar whitespace y comentarios entre statements
- ✅ Debería parsear TODOS los statements en secuencia

---

### 2. Orden de Precedencia en `stmt_parser()`

**Ubicación:** Línea ~959-973

```rust
while_stmt
    .or(for_stmt)
    .or(break_stmt)
    .or(continue_stmt)
    .or(if_stmt)
    .or(class_stmt)
    .or(struct_stmt)
    .or(import_stmt)
    .or(print)          // ← Print está aquí
    .or(let_stmt)       // ← Let está aquí
    .or(fn_stmt)        // ← Funciones están DESPUÉS de let/print
    .or(return_stmt)
    .or(field_assign_stmt)
    .or(assign_stmt)
    .or(expr_stmt)
```

**⚠️ PROBLEMA IDENTIFICADO:**

El orden de precedencia puede estar causando problemas:
- `print` y `let_stmt` están ANTES de `fn_stmt`
- Esto debería estar bien, pero puede haber un problema con cómo se manejan los comentarios después de funciones

---

### 3. Parser de Print Statement

**Ubicación:** Línea ~582-609

```rust
let print = just("print")
    .padded()
    .ignore_then(
        none_of("\n")
            .repeated()
            .at_least(1)
            .collect::<String>()
            .padded()
            .try_map({
                let expr_clone = expr.clone();
                move |expr_str: String, span| {
                    let trimmed = expr_str.trim();
                    // ... parsing de expresión
                }
            })
    )
    .map(Stmt::Print)
```

**Análisis:**
- ✅ El parser de print parece correcto
- ✅ Usa `none_of("\n")` para capturar hasta el final de línea
- ⚠️ Puede tener problemas si hay comentarios después

---

### 4. Parser de Let Statement

**Ubicación:** Línea ~611-621

```rust
let let_stmt = just("let")
    .padded()
    .then(just("mut").padded().or_not())
    .then(ident.clone())
    .then_ignore(just("=").padded())
    .then(expr.clone())
    .map(|(((_, mutable), name), value)| Stmt::Let {
        mutable: mutable.is_some(),
        name,
        value,
    });
```

**Análisis:**
- ✅ El parser de let parece correcto
- ✅ Maneja `let mut` correctamente
- ⚠️ Puede tener problemas con expresiones complejas como `Calculadora.sumar(10, 20)`

---

### 5. Parser de Funciones

**Ubicación:** Línea ~742-773

```rust
let fn_stmt = just("pub")
    .padded()
    .or_not()
    .then(just("fn")
        .padded()
        .ignore_then(ident.clone())
        .then(
            just("(")
                .padded()
                .ignore_then(
                    fn_param
                        .separated_by(just(",").padded())
                        .allow_trailing(),
                )
                .then_ignore(just(")").padded()),
        )
        .then(
            just("{")
                .padded()
                .ignore_then(stmt.clone().repeated())
                .then_ignore(just("}").padded()),
        ))
    .map(|(visibility, ((name, params), body))| Stmt::Fn { ... });
```

**Análisis:**
- ✅ El parser de funciones parece correcto
- ✅ Maneja el cuerpo con `stmt.clone().repeated()`
- ⚠️ Puede estar consumiendo más de lo necesario después del cierre `}`

---

## 🚨 Problemas Identificados

### Problema 1: Comentarios Después de Funciones

**Hipótesis:** Los comentarios `#` después de funciones pueden estar interfiriendo con el parsing de statements siguientes.

**Evidencia:**
```ad
fn Calculadora_new() {
    # Constructor vacío
}

let resultado = Calculadora.sumar(10, 20)  # ← Este statement no se parsea
```

**Posible Causa:**
- El parser puede estar consumiendo el comentario pero no avanzando correctamente
- `ws_and_comments()` puede no estar funcionando correctamente después de funciones

---

### Problema 2: Expresiones Call con Módulo

**Hipótesis:** `Calculadora.sumar(10, 20)` puede no estar parseándose correctamente como expresión.

**Evidencia:**
```ad
let resultado = Calculadora.sumar(10, 20)  # ← Expresión compleja
```

**Posible Causa:**
- El parser de expresiones puede no reconocer `StructName.method()` correctamente
- Puede estar parseándose como algo diferente

---

### Problema 3: Orden de Precedencia

**Hipótesis:** El orden de precedencia puede estar causando que el parser se detenga después de funciones.

**Evidencia:**
- `fn_stmt` está DESPUÉS de `let_stmt` y `print`
- Esto debería estar bien, pero puede haber un problema con cómo se manejan los fallbacks

---

## 🔧 Soluciones Propuestas

### Solución 1: Agregar Debug al Parser

**Implementación:**
```rust
// En parse_with_dir, después de parsear:
eprintln!("[PARSER-INFO] Programa parseado: {} statements", program.statements.len());
let let_count = program.statements.iter().filter(|s| matches!(s, Stmt::Let { .. })).count();
let print_count = program.statements.iter().filter(|s| matches!(s, Stmt::Print(_))).count();
eprintln!("[PARSER-INFO] Desglose: {} let, {} print", let_count, print_count);
```

**Estado:** ✅ Implementado

---

### Solución 2: Verificar Parsing de Expresiones Call

**Implementación:**
- Verificar que `Calculadora.sumar(10, 20)` se parsea como `Expr::Call { module: Some("Calculadora"), name: "sumar", args: [...] }`
- Agregar debug para ver qué expresión se genera

---

### Solución 3: Mejorar Manejo de Comentarios

**Implementación:**
- Verificar que `ws_and_comments()` funciona correctamente después de funciones
- Agregar debug para ver qué se consume después de cada statement

---

## 📋 Próximos Pasos

1. **Ejecutar con debug activado** para ver qué statements se parsean
2. **Analizar el output** para identificar exactamente dónde falla
3. **Aplicar corrección** basada en los findings
4. **Verificar** que test_6 y test_9 funcionen correctamente

---

## 🎯 Análisis del Código de Test_6

**Código fuente:**
```ad
struct Calculadora {
}

# Método estático (sin self)
fn Calculadora_sumar(a, b) {
    return a + b
}

# Método de instancia (con self)
fn Calculadora_new() {
    # Constructor vacío
}

let resultado = Calculadora.sumar(10, 20)
print resultado
```

**Statements Esperados:**
1. `Stmt::Struct { name: "Calculadora", ... }`
2. `Stmt::Fn { name: "Calculadora_sumar", ... }`
3. `Stmt::Fn { name: "Calculadora_new", ... }`
4. `Stmt::Let { name: "resultado", value: Expr::Call { module: Some("Calculadora"), name: "sumar", ... } }`
5. `Stmt::Print(Expr::Ident("resultado"))`

**Total Esperado:** 5 statements

**Si solo se parsean 3 statements (struct + 2 funciones), el problema está en el parser.**
**Si se parsean 5 statements pero no se generan, el problema está en el backend.**

---

**Última actualización:** 17 de Diciembre 2025


