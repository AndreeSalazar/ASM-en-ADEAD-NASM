# 🎯 Plan de Implementación OOP Completo

**Fecha:** Diciembre 2025  
**Prioridad:** NASM (Backend NASM es prioridad máxima)

---

## ✅ Estado Actual

### Completado:
1. ✅ Parser arreglado - `obj.metodo()` se parsea como `MethodCall`
2. ✅ Workaround eliminado
3. ✅ Debug eliminado
4. ✅ NASM estándar verificado
5. ✅ Métodos de instancia funcionando

### Pendiente:
1. ⚠️ Constructor real (`Persona.new()`) - Error: "undefined variable: Persona"
2. ❌ Métodos estáticos
3. ❌ Visibilidad
4. ❌ Herencia
5. ❌ super.metodo()
6. ❌ Vtables
7. ❌ Polimorfismo
8. ❌ Interfaces/Traits

---

## 🔍 Problema Actual: Constructor

**Error:** `undefined variable: Persona (variables must be declared with 'let')`

**Causa:** Cuando se procesa `Persona.new()`, el backend está tratando de evaluar `Persona` como una variable en lugar de reconocerlo como un struct.

**Solución necesaria:**
1. Verificar que `Persona.new()` se parsea correctamente como `Call { module: Some("Persona"), name: "new", args: [...] }`
2. Asegurar que los structs se registren ANTES de procesar cualquier código que los use
3. Verificar que la detección de constructor funcione correctamente

---

## 📋 Plan de Implementación

### FASE 1: Arreglar Constructor (PRIORIDAD ALTA) ⚡

**Tareas:**
1. Debuggear por qué `Persona.new()` genera error "undefined variable"
2. Verificar orden de procesamiento de structs
3. Arreglar detección de constructor
4. Probar Test 3

**Archivos a modificar:**
- `CORE/rust/crates/adead-backend/src/lib.rs` (líneas 164-200, 1462-1527)

---

### FASE 2: OOP Intermedio

#### 2.1 Métodos Estáticos
- Sintaxis: `StructName.metodo_estatico(args)`
- Parser: Detectar `Call { module: Some("StructName"), name: "metodo", args }` donde `name != "new"`
- Backend: Generar `call fn_StructName_metodo_estatico` (sin `self`)

#### 2.2 Visibilidad
- Sintaxis: `pub fn` (público), `fn` (privado)
- Parser: Ya existe `Visibility` enum
- Backend: Validar acceso (por ahora, solo registrar, validación futura)

#### 2.3 Getters/Setters
- Sintaxis: `obj.get_campo()`, `obj.set_campo(valor)`
- Backend: Generar métodos automáticos si no existen

---

### FASE 3: OOP Avanzado

#### 3.1 Herencia
- Sintaxis: `struct Hijo extends Padre { ... }`
- Parser: Ya existe `parent: Option<String>` en Struct
- Backend: 
  - Incluir campos del padre en el hijo
  - Calcular offsets correctos
  - Generar constructores que llamen al constructor del padre

#### 3.2 super.metodo()
- Sintaxis: `super.metodo(args)`
- Parser: Agregar `Expr::SuperCall { method, args }`
- Backend: Llamar al método del padre

#### 3.3 Vtables
- Estructura: `[vtable_ptr, campo1, campo2, ...]`
- Generar vtable para cada clase
- Llamadas virtuales: `call [obj + 0]` -> `call [vtable + offset]`

#### 3.4 Polimorfismo Dinámico
- Usar vtables para dispatch dinámico
- Override de métodos

#### 3.5 Interfaces/Traits
- Sintaxis: `interface Nombre { fn metodo() }`
- Parser: Agregar `Stmt::Interface`
- Backend: Verificar que las clases implementen todos los métodos

---

## 🎯 Prioridades

1. **ALTA:** Arreglar constructor (`Persona.new()`)
2. **ALTA:** Métodos estáticos
3. **MEDIA:** Visibilidad
4. **MEDIA:** Herencia básica
5. **BAJA:** super.metodo()
6. **BAJA:** Vtables
7. **BAJA:** Polimorfismo
8. **BAJA:** Interfaces

---

## 📝 Notas

- **NASM es prioridad máxima:** Todo debe generar código NASM estándar y universal
- **Sin ruido:** Eliminar todos los mensajes de debug
- **Paso a paso:** Implementar una funcionalidad a la vez, probar, luego continuar

---

**Próximo paso:** Debuggear y arreglar el constructor


