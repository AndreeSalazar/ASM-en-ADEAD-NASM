# 🏗️ Arquitectura Híbrida Zig + Rust

Documentación técnica de la arquitectura única de ADead.

## 🎯 Filosofía de Diseño

**Principio fundamental:** Cada lenguaje hace lo que mejor sabe hacer.

- **Zig:** Parsing eficiente y directo
- **Rust:** Seguridad, validación y generación de código

---

## 📐 Arquitectura Actual

### Flujo de Compilación

```
Archivo .ad
    ↓
┌─────────────────────────────────────┐
│  PARSER HÍBRIDO                     │
├─────────────────────────────────────┤
│  Zig Parser (Structs complejos)     │  ← Parsing rápido
│  +                                   │
│  Rust Parser (Resto del lenguaje)   │  ← Parsing robusto
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│  VALIDACIÓN                         │
├─────────────────────────────────────┤
│  Rust Borrow Checker                │  ← Seguridad de memoria
│  Type Checker                       │  ← Verificación de tipos
└─────────────────────────────────────┘
    ↓
┌─────────────────────────────────────┐
│  GENERACIÓN DE CÓDIGO               │
├─────────────────────────────────────┤
│  Rust Code Generator                │  ← NASM x86-64
│  (Windows + Linux backends)         │
└─────────────────────────────────────┘
    ↓
Archivo .asm (NASM)
    ↓
NASM Assembler
    ↓
Archivo .obj/.o
    ↓
Linker (gcc/ld)
    ↓
Ejecutable .exe
```

---

## 🔧 Componentes Principales

### 1. Zig Parser

**Responsabilidades:**
- Parsing de structs complejos
- Manejo de sintaxis multi-línea
- Detección de bloques `end`

**Ubicación:**
- `zig/src/parser_completo.zig`
- `rust/crates/adead-parser/src/zig_ffi_parser.rs` (FFI bridge)

**Ventajas:**
- Parsing más directo que Rust
- Mejor manejo de estructuras complejas
- Performance excelente

**Estado:** ✅ Implementado, con fallback Rust

---

### 2. Rust Parser

**Responsabilidades:**
- Parsing del resto del lenguaje (expresiones, statements)
- Integración con Zig parser
- Fallback si Zig no está disponible

**Ubicación:**
- `rust/crates/adead-parser/src/lib.rs`
- Usa `chumsky` como parser combinator

**Ventajas:**
- Robusto y completo
- Fácil de extender
- Buen error reporting

**Estado:** ✅ Implementado y funcional

---

### 3. Borrow Checker

**Responsabilidades:**
- Análisis de ownership
- Verificación de borrowing
- Detección de memory leaks potenciales

**Ubicación:**
- `rust/crates/adead-borrow/src/lib.rs`

**Features:**
- Tracking de variables y scopes
- Verificación de mutabilidad
- Detección de use-after-move
- Verificación de borrowing válido

**Estado:** ✅ Implementado

---

### 4. Code Generator

**Responsabilidades:**
- Generación de código NASM
- Manejo de calling conventions
- Gestión de stack frames
- Generación de tagged unions

**Ubicación:**
- `rust/crates/adead-backend/src/lib.rs`

**Backends:**
- Windows x64 (completo)
- Linux x64 (completo)

**Estado:** ✅ Implementado para ambos sistemas

---

## 🔗 Integración Zig + Rust

### FFI (Foreign Function Interface)

**Cómo funciona:**
1. Rust llama a funciones Zig compiladas como C library
2. Zig retorna estructuras parseadas
3. Rust convierte a tipos nativos de Rust

**Ventajas:**
- Aprovecha fortalezas de ambos lenguajes
- Fallback automático si Zig falla
- Performance óptimo

**Desafíos:**
- Linking en Windows (resuelto con config)
- Marshalling de datos
- Gestión de memoria compartida

---

## 📊 Comparación de Parsers

| Aspecto | Zig Parser | Rust Parser |
|---------|------------|-------------|
| **Parsing de structs** | ⭐⭐⭐⭐⭐ Excelente | ⭐⭐⭐ Bueno |
| **Parsing de expresiones** | ⭐⭐ Básico | ⭐⭐⭐⭐⭐ Excelente |
| **Error reporting** | ⭐⭐⭐ Bueno | ⭐⭐⭐⭐ Muy bueno |
| **Extensibilidad** | ⭐⭐⭐ Bueno | ⭐⭐⭐⭐⭐ Excelente |
| **Performance** | ⭐⭐⭐⭐⭐ Excelente | ⭐⭐⭐⭐ Muy bueno |

**Conclusión:** Combinación óptima de ambos.

---

## 🎯 Ventajas de la Arquitectura Híbrida

1. **Performance:** Parsing rápido con Zig
2. **Seguridad:** Validación robusta con Rust
3. **Flexibilidad:** Fallback automático
4. **Mantenibilidad:** Cada componente en su lenguaje ideal
5. **Extensibilidad:** Fácil agregar features

---

## 🔮 Futuro

### Mejoras Planificadas

1. **Parser Zig mejorado:**
   - Soporte para más construcciones
   - Mejor error reporting
   - Optimizaciones adicionales

2. **Integración más profunda:**
   - Compartir más código entre parsers
   - Optimizaciones de FFI
   - Mejor manejo de errores

3. **Backend adicionales:**
   - ARM64
   - WebAssembly (WASM)
   - Otros targets

---

**Última actualización:** Diciembre 2025

