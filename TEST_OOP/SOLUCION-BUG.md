# 🔧 Solución al Bug: `fn_c_area` en lugar de `fn_Circulo_area`

## Problema Identificado

El parser Chumsky está parseando `c.area()` como `Call { module: None, name: "area", args: [Ident("c")] }` en lugar de `MethodCall { object: Ident("c"), method: "area", args: [] }`.

## Evidencia

1. **Debug muestra:** `Discriminant(7)` que es `Call`, no `MethodCall` (que debería ser ~19)
2. **Código generado:** `call fn_c_area` en lugar de `call fn_Circulo_area`
3. **No aparece debug de `MethodCall`:** El código nunca llega a `Expr::MethodCall` en el match

## Causa Raíz

El parser Chumsky está parseando `c.area()` como una llamada a función `area(c)` en lugar de un método `c.area()`.

## Solución Implementada

Agregado código en `Expr::Call` para detectar MethodCalls mal parseados:
- Si `Call` tiene 1 argumento que es `Ident`
- Y ese `Ident` es una variable de tipo struct en `variable_types`
- Entonces generar como `MethodCall` en lugar de `Call`

## Código Agregado

```rust
// En Expr::Call, detectar MethodCall mal parseado
if args.len() == 1 {
    if let Expr::Ident(obj_name) = &args[0] {
        if let Some(struct_type) = self.variable_types.get(obj_name) {
            // Es un MethodCall mal parseado: generar como MethodCall
            // ... código para generar fn_StructName_method ...
        }
    }
}
```

## Próximos Pasos

1. Verificar que la solución funciona
2. Si funciona, considerar arreglar el parser Chumsky para que genere `MethodCall` correctamente
3. Continuar con los tests 3-5





