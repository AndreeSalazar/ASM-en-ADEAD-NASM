# 💡 Ideas3 - Programación Orientada a Objetos (OOP) para ADead

**Documento de ideas para implementar características de Programación Orientada a Objetos en el lenguaje ADead**

> Este documento organiza todas las ideas OOP por categoría, prioridad y complejidad para facilitar la implementación completa de paradigma orientado a objetos.

**Última actualización:** Diciembre 2025

---

## 📑 Tabla de Contenidos

1. [📊 Estado Actual del Proyecto](#-estado-actual-del-proyecto)
2. [🎯 Próximos Pasos Críticos](#-próximos-pasos-críticos)
3. [✅ Lo que está Completado](#-lo-que-está-completado)
4. [📦 Fundamentos OOP (Estilo Rust Mejorado)](#-fundamentos-oop-estilo-rust-mejorado)
5. [🏛️ Clases y Objetos](#️-clases-y-objetos)
6. [🔒 Encapsulación](#-encapsulación)
7. [👨‍👩‍👧 Herencia](#👩‍👧-herencia)
8. [🎭 Polimorfismo](#-polimorfismo)
9. [💾 Memory Management](#-memory-management)
10. [🔧 Generics y Templates](#-generics-y-templates)
11. [🗺️ Roadmap OOP](#️-roadmap-oop)

---

## 📊 Estado Actual del Proyecto

### Resumen Ejecutivo

**Total de Ideas OOP:** 35+  
**Completadas:** 7 (20%)  
**Parcialmente Completadas:** 2 (6%)  
**Pendientes:** 26+ (74%)

### Progreso por Categoría

- 🔧 **Pre-requisitos Rust**: 3/4 (75%) - *Ownership, Types, Option/Result*
- 📦 **Fundamentos OOP**: 5/5 (100%) - *Structs/Classes, RAII, Ownership, Propiedades, Métodos* ✅
- 🏛️ **Clases y Objetos**: 0/6 (0%)
- 🔒 **Encapsulación**: 0/4 (0%) - *Incluye Module System*
- 👨‍👩‍👧 **Herencia**: 0/5 (0%)
- 🎭 **Polimorfismo**: 0/4 (0%)
- 💾 **Memory Management**: 1/3 (33%) - *RAII ✅, Smart Pointers pendiente*

### MVP Funcional Actual ✅

**Lenguaje Básico:**
- ✅ Parser funcional (chumsky)
- ✅ Generación de código NASM (Windows x64)
- ✅ CLI modular (compile, assemble, link, run)
- ✅ Variables: `let` y `let mut` (inmutabilidad por defecto)
- ✅ Funciones: `fn nombre(params) { body }`
- ✅ Control: `if/else`, `while`
- ✅ Operadores: aritméticos y de comparación
- ✅ Statements: `print`, `let`, `if`, `while`, `fn`, `return`

---

## 🎯 Próximos Pasos Críticos

### 🚨 Prioridad ALTA - Completar Fundación (Sprint Actual)

Estas son las funcionalidades **más críticas** que faltan para tener una base sólida:

#### 1. **O5 - Encapsulación (public/private/pub)** ⭐⭐⭐
**Por qué es crítico:** Sin encapsulación, no hay verdadero OOP. Necesario para:
- Control de acceso a campos y métodos
- Seguridad y organización de código
- Preparación para herencia y polimorfismo

**Estado:** ❌ Pendiente  
**Complejidad:** 🟡 Media  
**Esfuerzo:** 20-30 horas  
**Bloquea:** Herencia, Interfaces, Polimorfismo

#### 2. **Completar O0.1 - Type Checker** ⭐⭐⭐
**Por qué es crítico:** Actualmente solo tenemos el enum `Type` extendido, pero falta:
- Verificación de tipos en tiempo de compilación
- Type inference completo
- Mensajes de error de tipo claros

**Estado:** ⚠️ Parcial (enum Type ✅, type checker ❌)  
**Complejidad:** 🔴 Alta  
**Esfuerzo:** 30-40 horas  
**Bloquea:** Generics, Type safety avanzado

#### 3. **Completar O0.2 - Verificación de Ownership** ⚠️
**Por qué es importante:** Tenemos AST y parser, pero falta:
- Verificación completa de reglas de borrowing
- Lifetime tracking
- Prevención de use-after-move

**Estado:** ⚠️ Parcial (AST/parser ✅, verificación completa ❌)  
**Complejidad:** 🔴 Alta  
**Esfuerzo:** 20-30 horas  
**Importante para:** Memory safety, uso avanzado de structs

---

## ✅ Lo que está Completado

### Fase 1.1: Sistema de Tipos y Ownership (Pre-requisitos Rust-like)

#### ✅ O0.1 - Sistema de Tipos Robusto (PARCIAL - 70%)
- ✅ **Enum Type extendido:** Todos los tipos primitivos, Array, Tuple, Option, Result, Ref
- ✅ **Compatibilidad NASM:** Métodos `size_bytes()`, `nasm_register_hint()`, `nasm_declaration()`
- ⏳ **Pendiente:** Módulo `adead-typecheck` para type checking/inference completo
- 📝 **Archivo:** `crates/adead-common/src/lib.rs` ✅

#### ✅ O0.2 - Ownership y Borrowing Básico (PARCIAL - 60%)
- ✅ **AST extendido:** `Borrow` y `Deref` en AST
- ✅ **Parser funcional:** Parser para `&` y `&mut`, parámetros con borrowing
- ✅ **Módulo borrow:** `adead-borrow` creado con borrow checker básico
- ⏳ **Pendiente:** Verificación completa de reglas (no aliasing mutable, moves), lifetime tracking
- 📝 **Archivos:**
  - `crates/adead-parser/src/lib.rs` ✅ (AST extendido)
  - `crates/adead-borrow/src/lib.rs` ✅ (Borrow checker básico)
  - `crates/adead-backend/src/lib.rs` ✅ (Compatibilidad NASM)

#### ✅ O0.3 - Inmutabilidad por Defecto (100%)
- ✅ **Campo mutable:** `Stmt::Let` tiene campo `mutable: bool`
- ✅ **Parser:** Reconoce `let mut`
- ✅ **Verificación:** Borrow checker verifica que variables inmutables no pueden ser modificadas
- ✅ **Tests:** Tests completos para parser y verificación de mutabilidad
- 📝 **Archivos:**
  - `crates/adead-parser/src/lib.rs` ✅
  - `crates/adead-borrow/src/lib.rs` ✅
  - `crates/adead-backend/src/lib.rs` ✅

#### ✅ O0.4 - Option y Result Types (100%)
- ✅ **AST extendido:** `Some`, `None`, `Ok`, `Err`, `Match`, `Pattern`, `MatchArm`
- ✅ **Parser:** Soporte completo para Option/Result/match expressions
- ✅ **Backend:** Generación de código NASM para tagged unions
- ✅ **Match exhaustivo:** Saltos condicionales implementados
- ✅ **Tests:** Tests completos para parsing y generación de código
- 📝 **Archivos:**
  - `crates/adead-parser/src/lib.rs` ✅
  - `crates/adead-common/src/lib.rs` ✅
  - `crates/adead-borrow/src/lib.rs` ✅
  - `crates/adead-backend/src/lib.rs` ✅
  - `crates/adead-backend/tests/option_result_match.rs` ✅

### Fase 1.2: Estructuras de Datos (Fundación) ✅ **COMPLETADA**

#### ✅ O1 - Structs/Clases Básicas (100%)
- ✅ **AST extendido:** `Stmt::Struct`, `Expr::StructLiteral`, `Expr::FieldAccess`, `Expr::MethodCall`
- ✅ **Parser completo:** Definición de structs, literales, acceso a campos y llamadas a métodos
- ✅ **Campos inmutables por defecto:** Requieren `mut` para ser mutables
- ✅ **Sintaxis completa:** `struct Nombre { campo: tipo }`, `Nombre { campo: valor }`, `objeto.campo`, `objeto.metodo(args)`
- ✅ **Tests:** Tests completos para parsing
- 📝 **Ejemplos:** `Ejemplos-Reales/ejemplos/structs.ad`, `structs-metodos.ad`

#### ✅ O3 - Propiedades (Fields/Members) con Ownership (100%)
- ✅ **StructField:** Campo `mutable: bool` para tracking de ownership
- ✅ **Borrow checker:** Verifica acceso a campos
- ✅ **Backend:** Genera código NASM para acceso a campos (layout simplificado de 8 bytes por campo)

#### ✅ O4 - Métodos de Instancia (100% básico)
- ✅ **Parser:** Parser para `objeto.metodo(args)`
- ✅ **Backend:** Genera llamadas con `self` como primer argumento
- ⏳ **Pendiente:** Dispatch real de métodos y binding de `&self`/`&mut self` completo

### Fase 1.3: Inicialización y Limpieza (RAII como Rust) ✅ **COMPLETADA**

#### ✅ O2 - Constructores y Destructores (RAII automático) (100%)
- ✅ **Sintaxis `init()`:** Para constructores en structs
- ✅ **Sintaxis `destroy()`:** Para destructores (O2.1 - Drop Trait)
- ✅ **Generación NASM:** Funciones `StructName_init` y `StructName_destroy`
- ✅ **RAII automático:** Destructores llamados automáticamente al salir de scope
- ✅ **Orden LIFO:** Tracking correcto de variables con destructores
- ✅ **Tests:** 9 tests (3 parsing + 6 code generation)
- 📝 **Archivos:**
  - `crates/adead-parser/src/lib.rs` ✅
  - `crates/adead-backend/src/lib.rs` ✅
  - `crates/adead-parser/tests/raii_init_destroy.rs` ✅
  - `crates/adead-backend/tests/raii_init_destroy.rs` ✅
  - `Ejemplos-Reales/ejemplos/raii-init-destroy.ad` ✅
  - `docs/RAII-ANALISIS.md` ✅

#### ✅ O2.1 - Drop Trait (destrucción determinística) (100%)
- ✅ **Implementado como parte de O2:** `destroy()` con RAII automático
- ✅ **Documentación:** Análisis completo en `docs/RAII-ANALISIS.md`

---

## 📦 Fundamentos OOP (Estilo Rust Mejorado)

### ❌ O0.1 - Sistema de Tipos Robusto (Completar Type Checker)

**Estado:** ⚠️ **PARCIAL** (70% completado)  
**Complejidad:** 🔴 Alta | **Impacto:** 🔴 Alto | **Esfuerzo:** 30-40 horas  
**Prioridad:** 🔴 CRÍTICA

**✅ Completado:**
- Enum `Type` extendido con todos los tipos primitivos y compuestos
- Métodos `size_bytes()`, `align_bytes()`, `nasm_register_hint()`, `nasm_declaration()`

**❌ Pendiente:**
- Crear módulo `adead-typecheck` para verificación de tipos
- Implementar `TypeChecker` struct
- Implementar type inference completo
- Verificación de tipos en tiempo de compilación
- Mensajes de error claros

**📝 Ver sección detallada más abajo para plan completo.**

---

### ⚠️ O0.2 - Ownership y Borrowing Básico (Completar Verificación)

**Estado:** ⚠️ **PARCIAL** (60% completado)  
**Complejidad:** 🔴 Alta | **Impacto:** 🔴 Alto | **Esfuerzo:** 20-30 horas  
**Prioridad:** 🟡 IMPORTANTE (no bloquea OOP básico)

**✅ Completado:**
- AST extendido con `Borrow` y `Deref`
- Parser para `&` y `&mut`
- Módulo `adead-borrow` con borrow checker básico
- Verificación básica de mutabilidad

**❌ Pendiente:**
- Verificación completa de reglas de borrowing (no aliasing mutable)
- Lifetime tracking avanzado
- Prevención de use-after-move
- Verificación de moves en asignaciones

**📝 Ver sección detallada más abajo para plan completo.**

---

### ❌ O5 - Encapsulación (public/private/pub) ⭐⭐⭐ **PRÓXIMO CRÍTICO**

**Estado:** ❌ NO Implementado  
**Complejidad:** 🟡 Media | **Impacto:** 🔴 Alto | **Esfuerzo:** 20-30 horas  
**Prioridad:** 🔴 CRÍTICA - Necesario para OOP verdadero

**Descripción:** Control de visibilidad estilo Rust (`pub`, privado por defecto)

**Sintaxis Propuesta:**
```adead
class Banco {
    // Privado por defecto
    saldo: int64
    contraseña: string
    
    // Público
    pub nombre: string
    
    // Público dentro del crate/módulo
    pub(crate) tasa_interes: float64
    
    // Constructor público
    pub init(nombre: string) {
        self.nombre = nombre
        self.saldo = 0
    }
    
    // Método público
    pub fn depositar(&mut self, monto: int64) {
        self.saldo = self.saldo + monto
    }
    
    // Método privado
    fn verificar_contraseña(&self, pass: string) -> bool {
        return self.contraseña == pass
    }
}

let mut banco = Banco("Mi Banco")
banco.depositar(100)          // OK: público
// banco.contraseña = "123"   // Error: privado
```

**Checklist de Implementación:**
- [ ] Privado por defecto (más seguro que Python)
- [ ] Keyword `pub` para público
- [ ] `pub(crate)` para crate-level visibility (futuro)
- [ ] Verificación de acceso en compilación
- [ ] Errores claros de acceso denegado
- [ ] Tests para encapsulación

**Bloquea:** Herencia, Interfaces, Polimorfismo verdadero

**📝 Ver sección detallada más abajo.**

---

## 📊 Plan Detallado de Implementación

### Paso 1: O5 - Encapsulación (Sprint Actual - 2-3 semanas)

**Por qué empezar aquí:**
1. ✅ Base sólida ya existe (O1, O2, O3, O4 completados)
2. ✅ Necesario para herencia y polimorfismo
3. ✅ Complejidad media (manejable)
4. ✅ Impacto alto (verdadero OOP)

**Plan de Implementación:**

#### Fase 1: AST y Parser (Semana 1)

1. **Extender AST para visibility modifiers**
   - Agregar campo `visibility` a `StructField`
   - Agregar campo `visibility` a métodos (cuando se implementen completamente)
   - Enum `Visibility`: `Private`, `Public`, `PubCrate` (futuro)

2. **Actualizar parser**
   - Reconocer `pub` keyword antes de campos/métodos
   - Privado por defecto si no hay `pub`

3. **Tests de parsing**
   - Struct con campos públicos y privados
   - Métodos públicos y privados

#### Fase 2: Verificación de Acceso (Semana 2)

1. **Crear módulo de verificación de acceso**
   - Verificar acceso a campos desde diferentes scopes
   - Verificar acceso a métodos
   - Generar errores claros

2. **Integrar con borrow checker**
   - Verificar acceso antes de verificar borrowing
   - Mensajes de error combinados

#### Fase 3: Backend y Tests (Semana 3)

1. **Actualizar generación de código**
   - Los campos privados/públicos no cambian el código NASM (es verificación en compilación)
   - Preparar para futuras optimizaciones

2. **Tests completos**
   - Tests de acceso permitido
   - Tests de acceso denegado (debe generar error)
   - Tests de encapsulación con herencia (preparación)

---

### Paso 2: Completar O0.1 - Type Checker (Sprint Siguiente - 3-4 semanas)

**Por qué después:**
- No bloquea OOP básico actual
- Requiere más esfuerzo (alta complejidad)
- Mejora calidad pero no es crítico para funcionalidad básica

**Plan de Implementación:**

#### Fase 1: Crear Módulo Type Checker (Semana 1)

1. **Crear crate `adead-typecheck`**
2. **Implementar `TypeChecker` struct básico**
3. **Implementar `infer_expr_type()` para expresiones básicas**

#### Fase 2: Verificación de Statements (Semana 2)

1. **Implementar verificación para `let`**
2. **Implementar verificación para `if`, `while`**
3. **Implementar verificación para funciones**
4. **Manejo de scopes**

#### Fase 3: Integración y Mejoras (Semanas 3-4)

1. **Integrar en CLI**
2. **Mensajes de error claros**
3. **Type inference avanzado**
4. **Tests completos**

---

### Paso 3: Completar O0.2 - Verificación de Ownership (Sprint Paralelo - 2-3 semanas)

**Puede hacerse en paralelo con O5 o después**

1. **Verificación completa de reglas de borrowing**
2. **Lifetime tracking**
3. **Prevención de use-after-move**
4. **Tests exhaustivos**

---

## 🏛️ Clases y Objetos

### ❌ O6 - Métodos Estáticos ⭐⭐

**Estado:** ❌ Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🟡 Medio | **Esfuerzo:** 10-15 horas  
**Prioridad:** ⭐⭐ Media (después de O5)

**Sintaxis Propuesta:**
```adead
class Math {
    static fn max(a: int64, b: int64) -> int64 {
        if a > b {
            return a
        }
        return b
    }
}

let resultado = Math.max(10, 20)  // Sin instanciar
```

**Dependencias:** O5 (Encapsulación)

---

### ❌ O7 - Propiedades con Getters/Setters ⭐⭐

**Estado:** ❌ Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🟡 Medio | **Esfuerzo:** 15-20 horas  
**Prioridad:** ⭐⭐ Media

**Sintaxis Propuesta:**
```adead
class Persona {
    private _nombre: string
    
    property nombre: string {
        get {
            return self._nombre
        }
        set(valor: string) {
            if len(valor) > 0 {
                self._nombre = valor
            }
        }
    }
}

let p = Persona()
p.nombre = "Juan"        // Llama al setter
print p.nombre           // Llama al getter
```

**Dependencias:** O5 (Encapsulación)

---

## 🔒 Encapsulación

### ❌ O5 - Encapsulación con Visibility Modifiers ⭐⭐⭐ **PRÓXIMO**

**Ver sección "Próximos Pasos Críticos" arriba para detalles completos.**

---

### ❌ O5.1 - Module System ⭐⭐

**Estado:** ❌ Pendiente  
**Complejidad:** 🟡 Media | **Impacto:** 🟡 Medio | **Esfuerzo:** 15-25 horas  
**Prioridad:** ⭐⭐ Media (después de O5)

**Dependencias:** O5 (Encapsulación)

---

## 👨‍👩‍👧 Herencia

### ❌ O10 - Herencia Simple ⭐⭐⭐

**Estado:** ❌ Pendiente  
**Complejidad:** 🔴 Alta | **Impacto:** 🔴 Alto | **Esfuerzo:** 40-60 horas  
**Prioridad:** ⭐⭐⭐ Alta (después de O5)

**Dependencias:** O1, O3, O4, O5

**Bloqueado por:** O5 (Encapsulación) - necesario para verdadera herencia

---

## 🎭 Polimorfismo

### ❌ O14 - Métodos Virtuales y Override ⭐⭐⭐

**Estado:** ❌ Pendiente  
**Complejidad:** 🔴 Alta | **Impacto:** 🔴 Alto | **Esfuerzo:** 30-40 horas  
**Prioridad:** ⭐⭐⭐ Alta (después de O10)

**Dependencias:** O10 (Herencia)

---

### ❌ O15 - Interfaces/Traits ⭐⭐⭐

**Estado:** ❌ Pendiente  
**Complejidad:** 🔴 Alta | **Impacto:** 🔴 Alto | **Esfuerzo:** 40-60 horas  
**Prioridad:** ⭐⭐⭐ Alta (después de O10)

**Dependencias:** O10 (Herencia)

---

## 💾 Memory Management

### ✅ O23 - RAII (Resource Acquisition Is Initialization) ⭐⭐⭐

**Estado:** ✅ **COMPLETADO** (100%)  
**Ver:** O2 - Constructores y Destructores arriba

---

### ❌ O24 - Smart Pointers ⭐⭐

**Estado:** ❌ Pendiente  
**Complejidad:** 🔴 Alta | **Impacto:** 🟡 Medio | **Esfuerzo:** 30-40 horas  
**Prioridad:** ⭐⭐ Media (futuro)

**Dependencias:** O23 (RAII) ✅

---

## 🔧 Generics y Templates

### ❌ O26 - Generics/Templates Básicos ⭐⭐⭐

**Estado:** ❌ Pendiente  
**Complejidad:** 🔴 Alta | **Impacto:** 🔴 Alto | **Esfuerzo:** 50-70 horas  
**Prioridad:** ⭐⭐⭐ Alta (después de type checker completo)

**Dependencias:** O0.1 completo (Type Checker)

---

## 🗺️ Roadmap OOP Actualizado

### ✅ Sprint 0: Fundación Rust-like (COMPLETADO)

```
✅ O0.1 - Sistema de Tipos (enum Type) - 70%
✅ O0.2 - Ownership (AST/parser) - 60%
✅ O0.3 - Inmutabilidad por Defecto - 100%
✅ O0.4 - Option/Result Types - 100%
```

### ✅ Sprint 1: Estructuras Básicas (COMPLETADO)

```
✅ O1 - Structs/Clases Básicas - 100%
✅ O3 - Propiedades con Ownership - 100%
✅ O4 - Métodos de Instancia - 100%
✅ O2 - Constructores y RAII - 100%
✅ O2.1 - Drop Trait - 100%
```

### 🚨 Sprint Actual: Encapsulación (EN PROGRESO)

```
🎯 O5 - Encapsulación (public/private/pub) - 0% [PRÓXIMO]
   ⏳ O5.1 - Module System - 0% [Después de O5]
```

### 📅 Sprint Siguiente: Type Safety y Ownership Completo

```
⏳ Completar O0.1 - Type Checker - 30% [Pendiente]
⏳ Completar O0.2 - Verificación Ownership - 40% [Pendiente]
⏳ O6 - Métodos Estáticos - 0% [Después de O5]
⏳ O7 - Propiedades Getters/Setters - 0% [Después de O5]
```

### 📅 Sprint Futuro: Herencia y Polimorfismo

```
⏳ O10 - Herencia Simple - 0% [Requiere O5]
⏳ O12 - Constructor de Clase Padre - 0% [Requiere O10]
⏳ O13 - Clases Abstractas - 0% [Requiere O10]
⏳ O14 - Métodos Virtuales - 0% [Requiere O10]
⏳ O15 - Interfaces/Traits - 0% [Requiere O10]
```

### 📅 Sprint Avanzado: Generics y Características Avanzadas

```
⏳ O26 - Generics/Templates - 0% [Requiere type checker completo]
⏳ O19 - Operator Overloading - 0%
⏳ O24 - Smart Pointers - 0% [Requiere RAII ✅]
```

---

## 📊 Matriz de Dependencias

```
O0.1 (Type System) ──┐
                     ├──> O26 (Generics)
O0.2 (Ownership) ────┘

O1 (Structs) ──┬──> O3 (Propiedades) ──> O4 (Métodos)
               │
               └──> O2 (RAII) ✅
                    │
                    └──> O5 (Encapsulación) [PRÓXIMO] ──┬──> O10 (Herencia)
                                                         │
                                                         └──> O14 (Polimorfismo)
                                                              │
                                                              └──> O15 (Interfaces)
```

---

## 🎯 Recomendación de Implementación

### Orden Sugerido (Próximos 3-4 meses)

1. **Semana 1-3: O5 - Encapsulación** 🚨
   - Crítico para OOP verdadero
   - Base para herencia y polimorfismo
   - Complejidad media

2. **Semana 4-5: O6 - Métodos Estáticos**
   - Relativamente simple
   - Mejora usabilidad

3. **Semana 6-9: Completar O0.1 - Type Checker**
   - Mejora calidad del código
   - Necesario para generics
   - Alta complejidad

4. **Semana 10-12: O10 - Herencia Simple**
   - OOP verdadero
   - Requiere O5 ✅ (después de semana 3)
   - Alta complejidad

5. **Semana 13-15: O14 - Polimorfismo**
   - Completa OOP básico
   - Requiere O10
   - Alta complejidad

---

## 📚 Referencias y Documentación

### Documentos Relacionados

- **Análisis RAII:** `docs/RAII-ANALISIS.md` - Análisis completo de implementación O2
- **Respuestas:** `docs/respuesta.md`, `docs/respuestas_2.md` - Casos de uso y preguntas frecuentes
- **Ejemplos:** `Ejemplos-Reales/ejemplos/` - Ejemplos funcionales

### Tests Existentes

- `crates/adead-parser/tests/` - Tests de parsing
- `crates/adead-backend/tests/` - Tests de generación de código
- `crates/adead-borrow/tests/` - Tests de borrow checker

---

**¡Sigue construyendo!** 🚀

*Última actualización: Diciembre 2025*
