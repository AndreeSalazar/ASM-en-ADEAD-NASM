# Ideas y Ejecución - Pruebas Zig Linker

**Fecha:** Diciembre 2025  
**Estado:** ✅ **ÉXITO COMPLETO** - Ejemplo funcional al 100%

---

## 🎯 Resumen del Éxito

### ✅ Lo que Funciona Perfectamente

1. **Compilación Completa del Pipeline**
   - ✅ ADead → NASM (generación directa)
   - ✅ NASM → .obj (ensamblado)
   - ✅ Zig Linker → .exe (linkeo)
   - ✅ Ejecución exitosa del programa

2. **Ejemplo Funcional: `test_simple.ad`**
   ```adead
   let x = 5
   let y = 10
   let z = x + y
   print z
   ```
   - ✅ Variables numéricas funcionan correctamente
   - ✅ Operaciones aritméticas (`+`) funcionan
   - ✅ `print` de variables numéricas funciona
   - ✅ Conversión número → string en runtime funciona
   - ✅ WriteFile funciona correctamente
   - ✅ Salida: `15` (correcto)

3. **Correcciones Críticas Implementadas**
   - ✅ **Bug 1:** `print` de variables numéricas ahora detecta tipo correctamente
   - ✅ **Bug 2:** Función helper `int_to_str_runtime` se genera correctamente
   - ✅ **Bug 3:** Stack alignment y shadow space correctos
   - ✅ **Bug 4:** Registros preservados correctamente (ABI-safe)

---

## 🔧 Problemas Resueltos

### 1. Error de Tipo en `print`
**Problema:** El código asumía que todas las variables eran strings, causando crash cuando se imprimía un número.

**Solución:** Implementada detección de tipo con `is_string_expr()`:
- Si es string → usa lógica de String struct
- Si es numérico → evalúa expresión y convierte a string

### 2. Función Helper No Generada
**Problema:** `int_to_str_runtime_0` se llamaba pero no se generaba en el código NASM.

**Solución:** Agregada generación completa de la función helper inline después de la llamada a `WriteFile`, incluyendo:
- Manejo de números negativos
- Conversión decimal
- Reversión de string
- Cálculo de longitud
- Restauración correcta de registros

### 3. Access Violation (0xC0000005)
**Problema:** El código intentaba acceder a memoria inválida al tratar un número como puntero.

**Solución:** Corrección completa del flujo de `print` para variables numéricas.

---

## 💡 Ideas para el Futuro

### 🚀 Corto Plazo (Inmediato)

#### 1. Expandir Ejemplos de Prueba
- [ ] **Ejemplo con strings:** `let s = "hola"; print s`
- [ ] **Ejemplo con arrays:** `let arr = [1, 2, 3]; print arr[0]`
- [ ] **Ejemplo con funciones:** `fn suma(a, b) { return a + b }; print suma(5, 3)`
- [ ] **Ejemplo con múltiples prints:** Varios `print` en secuencia
- [ ] **Ejemplo con expresiones complejas:** `print (x + y) * 2`

#### 2. Mejorar Output de `print`
- [ ] **Agregar newline automático:** Actualmente solo agrega `\n` al final
- [ ] **Soporte para múltiples argumentos:** `print x, y, z`
- [ ] **Formato mejorado:** `print "x =", x` (concatenación automática)
- [ ] **Escape sequences:** `print "Hola\nMundo"` con `\n` real

#### 3. Optimizaciones de Código Generado
- [ ] **Reutilizar función helper:** En lugar de generar `int_to_str_runtime` múltiples veces, generar una sola vez
- [ ] **Optimizar conversión:** Para números pequeños (< 1000), usar lookup table
- [ ] **Dead code elimination:** Eliminar funciones helper no usadas

### 🎯 Mediano Plazo (1-2 semanas)

#### 4. Mejorar Sistema de Tipos
- [ ] **Type inference mejorado:** Detectar automáticamente si una variable es string o número
- [ ] **Type annotations:** `let x: int = 5` o `let s: string = "hola"`
- [ ] **Type checking:** Error en compile-time si se intenta `string + int` sin conversión

#### 5. Expansión de Operadores
- [ ] **Operadores aritméticos:** `-`, `*`, `/`, `%`
- [ ] **Operadores de comparación:** `==`, `!=`, `<`, `>`, `<=`, `>=`
- [ ] **Operadores lógicos:** `&&`, `||`, `!`
- [ ] **Operador de concatenación:** `+` para strings (ya funciona parcialmente)

#### 6. Estructuras de Control
- [ ] **If/Else completo:** `if x > 5 { print "mayor" } else { print "menor" }`
- [ ] **Loops:** `for i in 0..10 { print i }`
- [ ] **While loops:** `while x > 0 { x = x - 1 }`

### 🌟 Largo Plazo (1-2 meses)

#### 7. Sistema de Módulos Completo
- [ ] **Múltiples archivos:** `import math; print math.sqrt(16)`
- [ ] **Namespaces:** Cada módulo tiene su propio namespace
- [ ] **Resolución de dependencias:** Detectar y resolver dependencias entre módulos
- [ ] **Linking automático:** Zig linker maneja múltiples `.obj` automáticamente

#### 8. Debugging y Desarrollo
- [ ] **Debug symbols:** Información de debugging en `.pdb` (ya se genera parcialmente)
- [ ] **Trazabilidad:** Comentarios `; ADead: line X` en código NASM generado
- [ ] **Error messages mejorados:** Mensajes de error más descriptivos
- [ ] **Warnings:** Advertencias para código potencialmente problemático

#### 9. Librería Estándar Expandida
- [ ] **Math functions:** `sqrt`, `sin`, `cos`, `log`, etc.
- [ ] **String functions:** `len`, `upper`, `lower`, `slice` (ya parcialmente implementado)
- [ ] **Array functions:** `append`, `pop`, `reverse`, `sort` (ya parcialmente implementado)
- [ ] **IO functions:** `read`, `read_line`, `write_file`

#### 10. Optimizaciones Avanzadas
- [ ] **Constant folding:** `let x = 5 + 3` → `let x = 8` en compile-time
- [ ] **Dead code elimination:** Eliminar código no alcanzable
- [ ] **Register allocation:** Optimizar uso de registros
- [ ] **Inlining:** Inline funciones pequeñas

---

## 📚 Lecciones Aprendidas

### ✅ Lo que Funcionó Bien

1. **Pipeline Modular:** Separar compilación, ensamblado y linkeo permite debugging más fácil
2. **Zig como Linker:** Zig funciona perfectamente como linker, simplificando el proceso
3. **Generación Directa NASM:** Evitar C++ intermedio hace el código más limpio y predecible
4. **Detección de Tipo:** `is_string_expr()` es una solución simple pero efectiva

### ⚠️ Problemas Encontrados

1. **Asunciones Incorrectas:** Asumir que todas las variables son strings causó bugs
2. **Funciones Helper:** Necesitan generarse en el lugar correcto del código
3. **Stack Management:** Requiere cuidado especial en Windows x64 ABI
4. **Type System:** El sistema de tipos necesita ser más robusto

### 🎓 Mejores Prácticas

1. **Siempre verificar tipo:** No asumir el tipo de una variable
2. **Generar funciones helper antes de usarlas:** O usar `jmp` para saltar sobre ellas
3. **Documentar convenciones:** Especialmente para ABI y stack alignment
4. **Probar con ejemplos simples primero:** `test_simple.ad` fue perfecto para debugging

---

## 🔍 Análisis Técnico del Ejemplo Actual

### Flujo de Ejecución

```
1. main() se ejecuta
2. Variables creadas en stack:
   - x = 5 en [rbp - 8]
   - y = 10 en [rbp - 16]
   - z = 15 en [rbp - 24]
3. print z:
   a. Detecta que z es numérico (no string)
   b. Evalúa expresión: carga z → RAX = 15
   c. Reserva buffer en stack: [rbp - 32]
   d. Llama int_to_str_runtime_0:
      - Convierte 15 → "15\n"
      - Retorna longitud en RAX
      - Retorna buffer en RDX
   e. Llama WriteFile con buffer y longitud
   f. Imprime "15" en stdout
4. ExitProcess(0)
```

### Estructura del Código Generado

```
section .text
  ; Runtime helpers (arrays, strings)
  ; Stdlib functions
  ; int_to_str_runtime_0 (generada inline)
  ; main()
```

### Registros Usados

- **RAX:** Valor de retorno, resultado de expresiones
- **RBX:** Registro temporal preservado
- **RCX:** Primer parámetro (WriteFile: stdout handle)
- **RDX:** Segundo parámetro (WriteFile: buffer pointer)
- **R8:** Tercer parámetro (WriteFile: length)
- **R9:** Cuarto parámetro (WriteFile: lpNumberOfBytesWritten)
- **RBP:** Base pointer (stack frame)
- **RSP:** Stack pointer

---

## 🗺️ Roadmap Sugerido

### Fase 1: Consolidación (Esta semana)
- [x] Ejemplo básico funcionando
- [ ] Ejemplos con strings
- [ ] Ejemplos con arrays
- [ ] Documentación completa del pipeline

### Fase 2: Expansión (Próximas 2 semanas)
- [ ] Funciones de usuario
- [ ] Estructuras de control básicas
- [ ] Operadores adicionales
- [ ] Mejor manejo de errores

### Fase 3: Módulos (Próximo mes)
- [ ] Sistema de módulos básico
- [ ] Resolución de dependencias
- [ ] Linking de múltiples módulos
- [ ] Namespaces

### Fase 4: Optimización (Mes 2)
- [ ] Optimizaciones de código
- [ ] Dead code elimination
- [ ] Register allocation
- [ ] Inlining

---

## 📝 Notas Técnicas Importantes

### Windows x64 ABI
- **Shadow space:** Siempre reservar 32 bytes antes de `call`
- **Stack alignment:** RSP debe estar alineado a 16 bytes antes de `call`
- **Registros preservados:** RBX, RBP, RDI, RSI, R12-R15 deben preservarse
- **Parámetros:** RCX, RDX, R8, R9 (primeros 4), luego stack

### Conversión Número → String
- **Algoritmo:** División por 10, obtener dígitos en reverso
- **Buffer:** 24 bytes suficiente para int64 (incluyendo signo negativo y newline)
- **Reversión:** Necesaria porque los dígitos se generan al revés
- **Longitud:** Calculada como `fin - inicio` del buffer

### Generación de Funciones Helper
- **Inline:** Generadas directamente en el código, no como funciones separadas
- **Labels:** Usar `new_label()` para evitar colisiones
- **Jump:** Usar `jmp label_end` antes de la función para saltar sobre ella
- **End label:** Siempre generar `label_end:` después de la función

---

## 🎉 Conclusión

El ejemplo `test_simple.ad` demuestra que **ADead funciona completamente** para casos básicos:
- ✅ Variables numéricas
- ✅ Operaciones aritméticas
- ✅ Print de números
- ✅ Pipeline completo de compilación

**Este es un hito importante** que demuestra la viabilidad del proyecto y establece una base sólida para futuras mejoras.

---

**Última actualización:** Diciembre 2025  
**Estado:** ✅ Funcional y listo para expansión

