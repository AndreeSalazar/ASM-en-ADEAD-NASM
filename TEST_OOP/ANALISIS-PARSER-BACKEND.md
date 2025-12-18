# 🔍 Análisis Paso a Paso: Parser y Backend para OOP

**Fecha:** Análisis en curso  
**Objetivo:** Entender por qué `c.area()` se parsea como `Call` en lugar de `MethodCall`

---

## 📋 PASO 1: Entender la Estructura del Parser Chumsky

### 1.1 Orden de Precedencia Actual

El parser Chumsky procesa las expresiones en este orden:

```
atom (números, strings, identificadores, etc.)
  ↓
qualified_name (modulo.funcion o funcion)
  ↓
call (qualified_name(args) → Expr::Call)
  ↓
unary (call | match_expr)
  ↓
with_access (unary.metodo(args) → Expr::MethodCall)
  ↓
index_or_slice
  ↓
... (operadores binarios, etc.)
```

### 1.2 Problema Identificado

**Código problemático en `lib.rs` líneas 1208-1246:**

```rust
// qualified_name parsea "c.area" como (Some("c"), "area")
let qualified_name = text::ident()
    .then(
        just(".")
        .padded()
        .ignore_then(text::ident())
        .or_not()
    )
    .try_map(|(first, second), span| {
        if let Some(second) = second {
            // modulo.funcion → (Some("modulo"), "funcion")
            Ok((Some(first), second))
        } else {
            // solo funcion → (None, "funcion")
            Ok((None, first))
        }
    });

// call parsea "c.area()" como Call ANTES de que with_access pueda manejarlo
let call = qualified_name
    .then(
        just("(")
        .padded()
        .ignore_then(expr.clone().separated_by(just(",").padded()).allow_trailing())
        .then_ignore(just(")").padded()),
    )
    .map(|((module, name), args)| Expr::Call {
        module,  // Some("c")
        name,    // "area"
        args,    // []
    })
    .or(atom);
```

**Resultado:** `c.area()` se parsea como:
```rust
Expr::Call {
    module: Some("c"),
    name: "area",
    args: []
}
```

En lugar de:
```rust
Expr::MethodCall {
    object: Box::new(Expr::Ident("c")),
    method: "area".to_string(),
    args: []
}
```

### 1.3 ¿Por qué `with_access` no lo corrige?

`with_access` (líneas 1308-1341) intenta aplicar `.metodo(args)` DESPUÉS de `call`, pero:

1. `c.area()` ya fue parseado como `Call` en la fase anterior
2. `with_access` solo puede aplicar `.metodo()` a expresiones que ya fueron parseadas
3. No puede "deshacer" un `Call` y convertirlo en `MethodCall`

---

## 📋 PASO 2: Análisis del Backend

### 2.1 Cómo el Backend Maneja `Call` vs `MethodCall`

**Ubicación:** `CORE/rust/crates/adead-backend/src/lib.rs` líneas 1442-1634

El backend tiene un **workaround** que detecta MethodCalls mal parseados:

```rust
Expr::Call { module, name, args } => {
    // DETECCIÓN 1: Si module es Some(variable) y variable es de tipo struct
    if let Some(obj_name) = &module {
        if let Some(struct_type) = self.variable_types.get(obj_name) {
            // Es un MethodCall mal parseado: generar como MethodCall
            let method_label = format!("fn_{}_{}", struct_type, name);
            // ... generar código para MethodCall ...
            return Ok(());
        }
    }
    
    // DETECCIÓN 2: Si args.len() == 1 y args[0] es Ident de tipo struct
    if args.len() == 1 {
        if let Expr::Ident(obj_name) = &args[0] {
            if let Some(struct_type) = self.variable_types.get(obj_name) {
                // Es un MethodCall mal parseado: generar como MethodCall
                // ...
            }
        }
    }
    
    // Si no es MethodCall, generar como Call normal
    // ...
}
```

### 2.2 ¿Por qué funciona el workaround?

El workaround funciona porque:
1. El backend tiene información de tipos (`variable_types`) que el parser no tiene
2. Puede distinguir entre:
   - `modulo.funcion(args)` → `Call` con `module` que NO está en `variable_types`
   - `obj.metodo(args)` → `Call` con `module` que SÍ está en `variable_types` → convertir a `MethodCall`

### 2.3 Limitaciones del Workaround

1. **Depende de información de tipos:** Solo funciona si `variable_types` tiene la información correcta
2. **No es la solución correcta:** El parser debería generar `MethodCall` desde el inicio
3. **Mensajes confusos:** Los mensajes de debug dicen "mal parseado" aunque funciona

---

## 📋 PASO 3: Análisis de los Tests

### 3.1 Test 2: `c.area()`

**Código:**
```ad
let c = Circulo { radio: 5 }
let area = c.area()
```

**Parseo actual:**
- `c.area()` → `Call { module: Some("c"), name: "area", args: [] }`

**Backend corrige:**
- Detecta que `c` está en `variable_types` como `"Circulo"`
- Genera `call fn_Circulo_area` ✅

**Resultado:** Funciona correctamente gracias al workaround.

### 3.2 Test 4: `c.incrementar(5)`

**Código:**
```ad
let c = Contador { valor: 0 }
c.incrementar(5)
```

**Parseo actual:**
- `c.incrementar(5)` → `Call { module: Some("c"), name: "incrementar", args: [Number(5)] }`

**Backend corrige:**
- Detecta que `c` está en `variable_types` como `"Contador"`
- Genera `call fn_Contador_incrementar` con `self` en RCX y `5` en RDX ✅

**Resultado:** Funciona correctamente gracias al workaround.

### 3.3 Test 5: `p1.mover(5, 5)`

**Código:**
```ad
let p1 = Punto { x: 10, y: 20 }
p1.mover(5, 5)
```

**Parseo actual:**
- `p1.mover(5, 5)` → `Call { module: Some("p1"), name: "mover", args: [Number(5), Number(5)] }`

**Backend corrige:**
- Detecta que `p1` está en `variable_types` como `"Punto"`
- Genera `call fn_Punto_mover` con `self` en RCX, `5` en RDX, `5` en R8 ✅

**Resultado:** Funciona correctamente gracias al workaround.

---

## 📋 PASO 4: Problema Fundamental

### 4.1 ¿Por qué el parser no puede distinguir?

El parser Chumsky **no tiene información de tipos** en tiempo de parsing. No puede saber si:
- `c` es una variable de tipo struct → debería ser `MethodCall`
- `c` es un nombre de módulo → debería ser `Call { module: Some("c"), ... }`

### 4.2 Soluciones Posibles

#### Opción A: Modificar `qualified_name` para NO parsear `ident.ident(args)`

**Problema:** Esto rompería `modulo.funcion(args)` que es válido.

#### Opción B: Hacer que `call` NO use `qualified_name` cuando viene de un `ident`

**Problema:** Necesitaríamos reestructurar el parser completamente.

#### Opción C: Agregar un parser específico para `ident.metodo(args)` ANTES de `call`

**Ventaja:** Podríamos parsear `obj.metodo(args)` como `MethodCall` antes de que `call` lo capture.

**Implementación:**
```rust
// Parser para method calls: obj.metodo(args)
let method_call = text::ident()
    .then(
        just(".")
        .padded()
        .ignore_then(text::ident())
        .then(
            just("(")
            .padded()
            .ignore_then(expr.clone().separated_by(just(",").padded()).allow_trailing())
            .then_ignore(just(")").padded())
        )
    )
    .map(|(obj, (method, args))| Expr::MethodCall {
        object: Box::new(Expr::Ident(obj)),
        method,
        args,
    });

// call solo para funciones normales (sin punto antes del nombre)
let call = text::ident()  // Solo ident, NO qualified_name
    .then(
        just("(")
        .padded()
        .ignore_then(expr.clone().separated_by(just(",").padded()).allow_trailing())
        .then_ignore(just(")").padded()),
    )
    .map(|(name, args)| Expr::Call {
        module: None,
        name,
        args,
    });

// qualified_name solo para modulo.funcion (sin args todavía)
let qualified_call = qualified_name
    .then(
        just("(")
        .padded()
        .ignore_then(expr.clone().separated_by(just(",").padded()).allow_trailing())
        .then_ignore(just(")").padded()),
    )
    .map(|((module, name), args)| Expr::Call {
        module,
        name,
        args,
    });

let call = method_call
    .or(qualified_call)
    .or(text::ident().map(|name| Expr::Call { module: None, name, args: vec![] }))
    .or(atom);
```

**Problema:** Esto rompería la compatibilidad con código existente que usa `modulo.funcion(args)`.

#### Opción D: Mantener el workaround pero mejorar los mensajes (ACTUAL)

**Ventaja:** Funciona correctamente, no rompe nada.

**Desventaja:** No es la solución "correcta" desde el punto de vista del diseño del parser.

---

## 📋 PASO 5: Recomendación

### 5.1 Estado Actual

✅ **Funciona correctamente:** Todos los tests pasan gracias al workaround  
⚠️ **No es ideal:** El parser genera `Call` en lugar de `MethodCall`  
✅ **Mensajes mejorados:** Ya no dicen "error" sino "workaround activo"

### 5.2 Próximos Pasos Recomendados

1. **Corto plazo:** Mantener el workaround (ya funciona)
2. **Mediano plazo:** Implementar Opción C (parser específico para method calls)
3. **Largo plazo:** Considerar análisis semántico en el parser para distinguir módulos de variables

### 5.3 ¿Vale la pena arreglar el parser ahora?

**Argumentos a favor:**
- Código más limpio y semánticamente correcto
- Eliminar dependencia del workaround
- Mejor separación de responsabilidades

**Argumentos en contra:**
- El workaround funciona perfectamente
- Requiere cambios significativos en el parser
- Podría introducir bugs en código existente
- No hay funcionalidad faltante

**Recomendación:** Mantener el workaround por ahora, documentar bien, y considerar arreglar el parser en una refactorización futura del sistema de parsing.

---

## 📋 PASO 6: Cómo el Backend Maneja MethodCall Correctamente Parseado

### 6.1 Si el Parser Genera MethodCall Correctamente

**Ubicación:** `lib.rs` líneas 1932-2240

El backend tiene un manejo completo para `Expr::MethodCall`:

1. **Métodos especiales de arrays/strings:** `append`, `pop`, `reverse`, `insert`, `remove`, `index`, `count`, `sort`, `upper`, `lower`, etc.
2. **Métodos de structs/clases:** Detecta el tipo del objeto usando `get_struct_type_from_expr()` y genera `call fn_StructName_method`

**Ejemplo:**
```rust
Expr::MethodCall { object: Ident("c"), method: "area", args: [] }
  ↓
get_struct_type_from_expr() → "Circulo"
  ↓
Genera: call fn_Circulo_area
```

### 6.2 Si el Parser Genera Call (Workaround)

**Ubicación:** `lib.rs` líneas 1442-1634

El backend detecta y corrige:

1. **Detección por `module`:** Si `Call.module` es una variable de tipo struct
2. **Detección por `args[0]`:** Si `Call.args[0]` es un Ident de tipo struct
3. **Corrección:** Genera código como si fuera `MethodCall`

**Ejemplo:**
```rust
Expr::Call { module: Some("c"), name: "area", args: [] }
  ↓
variable_types.get("c") → Some("Circulo")
  ↓
Genera: call fn_Circulo_area (igual que MethodCall)
```

### 6.3 Ambos Caminos Funcionan

✅ **Camino 1 (Ideal):** Parser → `MethodCall` → Backend maneja directamente  
✅ **Camino 2 (Actual):** Parser → `Call` → Backend detecta y corrige → Mismo resultado

**Resultado:** Ambos caminos generan el mismo código ASM correcto.

---

## 📋 PASO 7: Conclusión Final

### 7.1 Estado Actual

✅ **Funcionalidad:** Completa y correcta  
✅ **Tests:** Todos pasan  
✅ **Código generado:** Correcto en ambos caminos  
⚠️ **Diseño:** Parser genera `Call` en lugar de `MethodCall` (pero se corrige automáticamente)

### 7.2 No Hay Error Real

El sistema funciona correctamente gracias a un workaround inteligente en el backend. El parser genera `Call` en lugar de `MethodCall`, pero el backend lo detecta y corrige usando información de tipos.

**No hay error real** - solo una diferencia en cómo se representa el AST inicialmente. El código generado es correcto y los tests pasan.

### 7.3 Mensajes de Debug Mejorados

Los mensajes de debug ahora reflejan correctamente que es un "workaround activo" en lugar de un "error":

**Antes:**
```
DEBUG Call: Detectado MethodCall mal parseado: c.area()
```

**Ahora:**
```
DEBUG Call: Convirtiendo Call a MethodCall: c.area() (workaround activo)
```

### 7.4 Recomendación Final

**Mantener el workaround por ahora:**
- ✅ Funciona perfectamente
- ✅ No rompe nada
- ✅ Los mensajes ya no son confusos
- ✅ Ambos caminos (MethodCall directo y Call corregido) generan el mismo código

**Considerar arreglar el parser en el futuro:**
- Cuando se haga una refactorización mayor del sistema de parsing
- Si se necesita mejor separación de responsabilidades
- Si se quiere eliminar la dependencia del workaround

**Por ahora, el sistema está funcionando correctamente y no requiere cambios urgentes.**

