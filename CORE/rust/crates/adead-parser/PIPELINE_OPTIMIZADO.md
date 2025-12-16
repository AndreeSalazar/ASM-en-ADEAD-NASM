# 🚀 Pipeline Optimizado: D → Zig → Rust → ASM Virgen

## Descripción

Este pipeline implementa la arquitectura mejorada propuesta en `datos.md`:

```
ADead → Parser → D (CTFE) → Zig (ASM Directo) → Rust (Limpieza) → ASM Virgen
```

## Componentes

### 1. D Language (CTFE)
- **Función**: Optimización compile-time
- **Qué hace**: 
  - Evalúa constantes en compile-time (ej: `5 + 3` → `8`)
  - Elimina código muerto antes de generar código
  - Simplifica expresiones complejas

### 2. Zig (ASM Directo)
- **Función**: Generación de ASM sin pasar por C
- **Qué hace**:
  - Genera ASM directamente desde el código optimizado
  - Evita overhead de frame pointers innecesarios
  - Mejor control sobre registros CPU

### 3. Rust (Limpieza)
- **Función**: Post-procesamiento y limpieza de ASM
- **Qué hace**:
  - Elimina metadatos SEH (Windows)
  - Elimina frame pointers innecesarios
  - Optimiza movimientos redundantes
  - Elimina código muerto
  - Limpia saltos innecesarios

## Uso

### Desde CLI

```bash
# Usar pipeline optimizado
adeadc compile programa.ad --backend optimized -o programa.asm

# O usar alias corto
adeadc compile programa.ad --backend opt -o programa.asm
```

### Desde Código Rust

```rust
use adead_parser::optimized_pipeline::OptimizedPipeline;

let source = "let x = 5 + 3\nprint x";
let asm = OptimizedPipeline::process_complete(source, "programa.ad")?;
```

## Fallback

Si algún componente no está disponible, el pipeline hace fallback automático:

1. Si D no está disponible → Continúa sin optimización CTFE
2. Si Zig no está disponible → Usa C → GCC/Clang como fallback
3. Si Rust falla → Retorna error (no debería pasar)

## Beneficios Esperados

- ✅ ASM 30-50% más limpio
- ✅ Menos instrucciones innecesarias
- ✅ Mejor performance
- ✅ Código más pequeño

## Estado Actual

- ✅ Módulo de limpieza ASM implementado
- ✅ Pipeline optimizado implementado
- ✅ Integración en CLI completada
- ⚠️ D Language CTFE: Implementación básica (mejoras pendientes)
- ⚠️ Zig ASM Directo: Usa generador existente (mejoras pendientes)

## Próximos Pasos

1. Mejorar integración D Language para CTFE más agresivo
2. Mejorar generación ASM directa en Zig
3. Optimizaciones adicionales en limpieza Rust
4. Tests exhaustivos del pipeline completo

