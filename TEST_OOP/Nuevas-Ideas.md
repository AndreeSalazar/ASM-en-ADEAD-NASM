# 💡 Nuevas Ideas para Solucionar Problemas OOP

**Fecha:** 17 de Diciembre 2025  
**Objetivo:** Investigar y proponer soluciones para los problemas de generación de código

---

## 🔍 Análisis del Problema

### Problema Principal
Los statements `let` y `print` no se están generando en el main para test_6 y test_9, aunque sí funcionan en test_3.

### Comparación de Tests

**Test 3 (FUNCIONA):**
```ad
struct Persona {
    nombre
    edad
}

fn Persona_new(self, nombre, edad) {
    self.nombre = nombre
    self.edad = edad
}

let p = Persona.new("Juan", 25)
print p.nombre
print p.edad
```

**Test 6 (NO FUNCIONA):**
```ad
struct Calculadora {
}

fn Calculadora_sumar(a, b) {
    return a + b
}

fn Calculadora_new() {
    # Constructor vacío
}

let resultado = Calculadora.sumar(10, 20)
print resultado
```

**Diferencias Clave:**
1. Test 3 tiene constructor con `self` como primer parámetro
2. Test 6 tiene método estático sin `self` y constructor vacío sin `self`
3. Test 3 usa `Persona.new()` (constructor)
4. Test 6 usa `Calculadora.sumar()` (método estático)

---

## 💡 Ideas y Soluciones Propuestas

### Idea 1: Problema de Parsing - Comentarios o Líneas Vacías

**Hipótesis:** Los comentarios `#` o líneas vacías después de las funciones pueden estar causando que el parser ignore los statements siguientes.

**Solución Propuesta:**
```rust
// En el parser, verificar que los comentarios no interfieran
// Asegurar que ws_and_comments() maneje correctamente los comentarios
// Verificar que el parser no se detenga después de funciones
```

**Implementación:**
- Agregar debug para ver qué statements se parsean
- Verificar que los comentarios no causen problemas
- Asegurar que las líneas vacías se ignoren correctamente

---

### Idea 2: Problema de Orden de Procesamiento

**Hipótesis:** El orden en que se procesan los statements puede estar causando que algunos se pierdan.

**Solución Propuesta:**
```rust
// Verificar el orden de procesamiento:
// 1. Structs (registrar tipos)
// 2. Funciones (generar código)
// 3. Other statements (generar en main)

// Posible problema: Los statements pueden procesarse antes de que las funciones estén listas
```

**Implementación:**
- Agregar debug para ver el orden de procesamiento
- Verificar que todos los statements se procesen en el orden correcto
- Asegurar que los statements se agreguen a `other_statements` correctamente

---

### Idea 3: Problema con Métodos Estáticos - No Se Detectan Correctamente

**Hipótesis:** Los métodos estáticos (`Calculadora_sumar` sin `self`) pueden no estar siendo detectados correctamente como métodos de struct, causando que no se generen.

**Solución Propuesta:**
```rust
// Verificar detección de métodos estáticos:
// - Calculadora_sumar(a, b) -> método estático
// - Debe generarse como función global fn_Calculadora_sumar
// - Debe poder llamarse como Calculadora.sumar(10, 20)

// Posible problema: El método estático no se detecta como método de struct
// y por lo tanto no se genera correctamente
```

**Implementación:**
- Verificar que los métodos estáticos se detecten correctamente
- Asegurar que se generen como funciones globales
- Verificar que las llamadas `StructName.method()` funcionen

---

### Idea 4: Problema con Expresiones Call con Módulo

**Hipótesis:** `Calculadora.sumar(10, 20)` puede no estar parseándose correctamente como `Call { module: Some("Calculadora"), name: "sumar", args: [...] }`.

**Solución Propuesta:**
```rust
// Verificar parsing de StructName.method():
// - Calculadora.sumar(10, 20) debe parsearse como:
//   Call { module: Some("Calculadora"), name: "sumar", args: [Number(10), Number(20)] }

// Posible problema: El parser puede estar parseando esto incorrectamente
// o el backend no está procesando correctamente este tipo de Call
```

**Implementación:**
- Agregar debug para ver cómo se parsea `Calculadora.sumar(10, 20)`
- Verificar que se procese correctamente en `Expr::Call` con `module: Some(...)`
- Asegurar que se genere código para métodos estáticos

---

### Idea 5: Problema con Dead Code Elimination

**Hipótesis:** El dead code elimination puede estar eliminando código que parece no usarse.

**Solución Propuesta:**
```rust
// Verificar dead code elimination:
// - Las funciones pueden estar siendo eliminadas si no se detectan como usadas
// - Los statements pueden estar siendo eliminados si no se detectan como necesarios

// Posible problema: El análisis de uso puede no detectar correctamente
// que Calculadora.sumar() se usa en let resultado = Calculadora.sumar(10, 20)
```

**Implementación:**
- Verificar que el análisis de uso detecte correctamente las llamadas
- Asegurar que las funciones no se eliminen incorrectamente
- Verificar que los statements no se eliminen incorrectamente

---

### Idea 6: Problema con el Parser - Statements Después de Funciones

**Hipótesis:** El parser puede tener problemas parseando statements que vienen después de definiciones de funciones.

**Solución Propuesta:**
```rust
// Verificar parsing de statements después de funciones:
// - El parser puede estar deteniéndose después de funciones
// - Los statements pueden no estar siendo parseados correctamente

// Posible problema: El parser puede tener un problema con el orden
// o con cómo maneja los statements después de funciones
```

**Implementación:**
- Agregar debug para ver qué statements se parsean
- Verificar que el parser no se detenga después de funciones
- Asegurar que todos los statements se parseen correctamente

---

### Idea 7: Problema con el Backend - Statements No Se Agregan a other_statements

**Hipótesis:** Los statements pueden estar parseándose correctamente pero no agregándose a `other_statements`.

**Solución Propuesta:**
```rust
// Verificar agregado a other_statements:
// - Los statements Let y Print deben agregarse a other_statements
// - Deben procesarse en el main

// Posible problema: Los statements pueden estar siendo clasificados incorrectamente
// o no agregándose a other_statements
```

**Implementación:**
- Agregar debug para ver qué statements se agregan a `other_statements`
- Verificar que los statements se clasifiquen correctamente
- Asegurar que se procesen en el main

---

## 🛠️ Debug Personalizado Agregado

### Debug en Separación de Statements
```rust
eprintln!("[DEBUG] Total statements parseados: {}", program.statements.len());
eprintln!("[DEBUG] Statement {}: Let '{}' -> other_statements", i, name);
eprintln!("[DEBUG] Statement {}: Print -> other_statements", i);
eprintln!("[DEBUG] Structs: {}, Functions: {}, Other: {}", structs.len(), user_functions.len(), other_statements.len());
```

### Debug en Procesamiento de Main
```rust
eprintln!("[DEBUG] Procesando {} statements en main", other_statements.len());
eprintln!("[DEBUG] WARNING: No hay statements para procesar en main!");
eprintln!("[DEBUG] Procesando statement {} en main", i);
```

---

## 📋 Plan de Acción

### Fase 1: Investigación con Debug
1. ✅ Agregar debug personalizado al código
2. ✅ Crear sistema de debug inteligente estilo Python
3. ✅ Implementar análisis automático de problemas
4. ✅ Generar reportes detallados del proceso
5. ✅ Integrar debug en todo el flujo de compilación
6. ⏳ Compilar y ejecutar test_6 con debug para ver el análisis completo
7. ⏳ Analizar output del debug para identificar el problema exacto

### Fase 2: Corrección Basada en Debug
1. ⏳ Identificar la causa raíz del problema
2. ⏳ Aplicar la corrección correspondiente
3. ⏳ Verificar que test_6 y test_9 funcionen

### Fase 3: Verificación Completa
1. ⏳ Verificar que todos los tests funcionen
2. ⏳ Verificar que el código NASM sea puro y estándar
3. ⏳ Documentar las soluciones aplicadas

---

## 🎯 Próximos Pasos Inmediatos

1. **Compilar con debug:**
   ```bash
   cd CORE/rust
   cargo build --release
   cd ../../TEST_OOP
   ..\CORE\rust\target\release\adeadc.exe compile test_6_metodo_estatico.ad -o test_6.asm
   ```

2. **Analizar output del debug:**
   - Ver qué statements se parsean
   - Ver qué statements se agregan a `other_statements`
   - Ver qué statements se procesan en el main

3. **Aplicar corrección basada en findings:**
   - Si el problema es de parsing, corregir el parser
   - Si el problema es de procesamiento, corregir el backend
   - Si el problema es de detección, corregir la lógica de detección

---

## 📝 Notas Adicionales

### Observaciones
- Test 3 funciona correctamente, lo que sugiere que el sistema básico funciona
- Test 6 y Test 9 tienen problemas similares, lo que sugiere un problema común
- El código NASM generado es correcto cuando se genera, lo que sugiere que el problema está en la generación, no en el código generado

### Hipótesis Principal
El problema más probable es que los statements no se están parseando correctamente o no se están agregando a `other_statements`. El debug personalizado ayudará a identificar exactamente dónde está el problema.

---

**Última actualización:** 17 de Diciembre 2025

