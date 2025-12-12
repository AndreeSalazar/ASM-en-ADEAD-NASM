# Diseño Técnico de ADead

**Autor:** Eddi Andreé Salazar Matos  
**Fecha:** 11 de Diciembre de 2025  
🇵🇪 *Proyecto peruano* 🇵🇪

## Filosofía

ADead busca ser un lenguaje simple y directo que compila a código NASM sin overhead. El objetivo es proporcionar una sintaxis amigable estilo Python mientras mantiene el control y rendimiento del ensamblador.

## Arquitectura

### Pipeline de Compilación

```
Source (.ad)
  ↓
Lexer (implícito en parser)
  ↓
Parser (chumsky) → AST
  ↓
Code Generator → NASM (.asm)
  ↓
NASM → Object (.o)
  ↓
LD → Ejecutable
```

### Componentes

1. **adead-common**: Tipos compartidos, errores
2. **adead-parser**: Parser recursivo descendente usando chumsky
3. **adead-backend**: Generador de código NASM
4. **adead-cli**: Interfaz de línea de comandos

## Decisiones de Diseño

### Sistema de Tipos (MVP)

- **Tipos primitivos**: `int64`, `string`
- **Inferencia**: Simple, basada en literales
- **Sin tipos explícitos**: En MVP, todo se infiere

### Gestión de Memoria

- **Stack-based**: Variables locales en el stack
- **Sin GC**: Sin garbage collector, gestión manual
- **ABI System V**: Usa convenciones System V AMD64

### Generación de Código

- **NASM directo**: Genera código NASM, no IR intermedia (en MVP)
- **Registro allocation**: Simple, usa stack para spills
- **Sin optimizaciones**: MVP sin optimizaciones avanzadas

## Tradeoffs

### Ventajas

- ✅ Sintaxis simple y legible
- ✅ Control total sobre el código generado
- ✅ Compilación rápida
- ✅ Sin runtime dependencies

### Limitaciones (MVP)

- ⚠️ Windows x64 (soporte completo), Linux en desarrollo
- ⚠️ Tipos básicos (int64, string, structs)
- ⚠️ Sin gestión de memoria avanzada (stack-based)
- ⚠️ Sin optimizaciones avanzadas

## Roadmap Técnico

### Fase 1: MVP ✅

- Parser básico
- Generación NASM
- Variables, condicionales, loops
- Funciones simples

### Fase 2: Mejoras

- IR intermedia
- Optimizaciones (const folding, dead code)
- Mejor registro allocation
- Arrays y strings avanzados

### Fase 3: Expansión

- Múltiples targets (Windows, macOS)
- Interoperabilidad con C
- SIMD/intrinsics
- Tooling (LSP, formatter)

## Convenciones de Código

### NASM

- Usa posición relativa (`[rel label]`)
- Stack frame con `rbp`
- System V ABI para llamadas

### Rust

- Error handling con `anyhow`/`thiserror`
- Tests unitarios para cada componente
- Documentación inline para funciones públicas

## Rendimiento

El objetivo no es competir con compiladores optimizados como GCC o Clang, sino proporcionar:

1. Compilación rápida
2. Código legible
3. Control sobre el output

El código generado es funcional pero no está optimizado en el MVP.

