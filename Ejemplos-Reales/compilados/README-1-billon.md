# 1_billón.ad

Este archivo suma desde 1 hasta 1 billón (1,000,000,000) imprimiendo cada número.

## Problema Actual

El compilador actual tiene un bug donde no reconoce los operadores de comparación (`<`, `<=`, `>`, `>=`, `==`, `!=`) en las condiciones de `while`.

**Error:**
```
Parse error: found "<" but expected one of end of input, ...
```

## Solución Temporal

Una vez que se corrija el parser de comparaciones en `rust/crates/adead-parser/src/lib.rs`, este archivo debería funcionar correctamente.

## Cómo Debería Funcionar

1. Inicia con `suma = 1`
2. Mientras `suma <= limite` (1 billón):
   - Imprime el valor actual de `suma`
   - Incrementa `suma = suma + 1`
3. Cuando `suma` llegue a 1,000,000,001, el loop se detiene
4. Imprime "Llegamos a 1 billon!"

## Fix Necesario en el Parser

El problema está en que los operadores de comparación necesitan `.padded()` para reconocer espacios alrededor de ellos. El fix ya está aplicado en el código fuente, pero necesita recompilarse.

