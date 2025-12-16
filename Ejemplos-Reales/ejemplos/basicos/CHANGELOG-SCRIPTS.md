# 📝 Changelog - Scripts de Testing

## Versión 2.0 - Correcciones y Mejoras

### ✅ Problemas Corregidos

1. **Backend incorrecto**
   - ❌ Antes: `--backend cpp` (no soporta strings avanzados completamente)
   - ✅ Ahora: `--backend auto` (selecciona automáticamente el mejor pipeline)

2. **Numeración inconsistente**
   - ❌ Antes: "[1/3]" luego "[2/4]", "[3/4]", "[4/4]"
   - ✅ Ahora: "[1/4]", "[2/4]", "[3/4]", "[4/4]", "[5/5]" (consistente)

3. **Manejo de errores**
   - ❌ Antes: Errores ocultos con `Out-Null`
   - ✅ Ahora: Muestra mensajes de error detallados

4. **Path del compilador**
   - ❌ Antes: Solo path relativo `..\..\..\`
   - ✅ Ahora: Resolución robusta con fallback a path relativo

5. **Verificación de archivos generados**
   - ❌ Antes: No verificaba si los archivos se generaron correctamente
   - ✅ Ahora: Verifica existencia de ASM, OBJ y EXE antes de continuar

6. **Manejo de stderr**
   - ❌ Antes: stderr redirigido a `Out-Null`
   - ✅ Ahora: Captura y muestra stderr para debugging

### 🆕 Mejoras Agregadas

1. **Mensajes informativos**
   - Muestra tamaño de archivos generados
   - Muestra exit code del programa ejecutado
   - Muestra primeros 3-5 líneas de errores para debugging

2. **Mejor detección de herramientas**
   - Verifica existencia de NASM y GCC antes de usar
   - Muestra mensajes claros si faltan herramientas

3. **Manejo de excepciones mejorado**
   - Captura excepciones con stack trace
   - Muestra detalles del error

### 📋 Archivos Modificados

- ✅ `ejecutar_test_individual.ps1` - Script individual corregido
- ✅ `ejecutar_tests_strings.ps1` - Script batch corregido
- ✅ `verificar_compilacion.ps1` - Script de verificación corregido

### 🔧 Uso Actualizado

Los scripts ahora funcionan correctamente con:
- Backend automático (`--backend auto`)
- Manejo robusto de paths
- Mensajes de error detallados
- Verificación de archivos generados

---

**Fecha:** Diciembre 2025  
**Versión:** 2.0

