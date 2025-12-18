# 🔍 Investigación del Bug: `fn_c_area` en lugar de `fn_Circulo_area`

## Problema Identificado

El código genera `call fn_c_area` en lugar de `call fn_Circulo_area` cuando se llama `c.area()`.

## Debug Encontrado

### Orden de Procesamiento (CORRECTO):
1. **Statement 0:** `struct Circulo { ... }` 
   - `variable_types` antes: `[]`
   - `variable_types` después: `[]` ✅ (correcto, no registra variables)

2. **Statement 1:** `let c = Circulo { radio: 5 }`
   - `variable_types` antes: `[]`
   - `variable_types` después: `["c"]` ✅ (correcto, registra `c -> Circulo`)

3. **Statement 2:** `let area = c.area()`
   - `variable_types` antes: `["c"]` ✅ (correcto, `c` está disponible)
   - `variable_types` después: `["c"]` ✅ (correcto)

### Problema:
- Cuando se procesa `let area = c.area()`, se llama a `generate_expr_windows(value)` donde `value` es `Expr::MethodCall { object: c, method: "area", args: [] }`
- El debug muestra: `DEBUG generate_expr_windows: Procesando expresión de tipo Discriminant(7)` (que es `MethodCall`)
- **PERO** no aparece el debug de `MethodCall` que agregamos en la línea 1836
- Esto significa que el código **NO está llegando** a `Expr::MethodCall` en `generate_expr_windows`

## Hipótesis

El código está usando otro camino o hay un problema con el match. El código genera `call fn_c_area` pero no veo dónde se genera ese nombre específico.

## Próximos Pasos

1. Verificar si hay otro lugar donde se procesa `MethodCall`
2. Verificar si el código está usando `generate_expr` en lugar de `generate_expr_windows`
3. Agregar más debug para rastrear el flujo completo
4. Encontrar dónde se genera `fn_c_area` específicamente

## Nota

El debug muestra que `variable_types` tiene `["c"]` cuando se procesa el statement 2, pero cuando se busca `c` en `get_struct_type_from_expr`, `variable_types` está vacío. Esto sugiere que hay un problema con el contexto o que `variable_types` se está limpiando en algún lugar.





