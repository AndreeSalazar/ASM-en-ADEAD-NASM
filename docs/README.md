# 📚 Documentación de ADead

## 📋 Índice de Documentación

### ✅ Documentación Actualizada (Diciembre 2025)

1. **[FLUJO-ACTUAL.md](FLUJO-ACTUAL.md)** ⭐ **ACTUAL**
   - Flujo completo actual: ADead → Parser Manual → C → GCC/Clang → ASM → EXE
   - Documentación detallada del flujo funcional

2. **[ESTADO-ACTUAL.md](ESTADO-ACTUAL.md)** ⭐ **ACTUAL**
   - Estado completo del proyecto
   - Qué funciona y qué no funciona
   - Roadmap de lo que falta

3. **[CARACTERISTICAS-FUNCIONALES.md](CARACTERISTICAS-FUNCIONALES.md)** ⭐ **ACTUAL**
   - Lista completa de características implementadas
   - Ejemplos verificados
   - Comparativa funcional vs necesario

### 📁 Carpetas de Documentación

#### `/avances/` - Histórico de Avances
Documentos históricos sobre implementaciones anteriores:
- **ZIG-*.md** - Documentación sobre integración con Zig (obsoleto)
- **FLOAT-*.md** - Planes de implementación de floats (pendiente)
- **CAPACIDADES-ACTUALES.md** - Estado anterior (verificar si actual)

#### `/roadmap/` - Planificación
- **ROADMAP-PROFESIONAL.md** - Plan de desarrollo profesional
- **PROGRESO-SPRINT*.md** - Progreso de sprints anteriores

#### `/testing/` - Testing
- **ESTADO-TESTING.md** - Estado del sistema de testing
- **TESTING-*.md** - Documentación de testing específico

#### `/casos-de-uso/` - Casos de Uso
Documentación sobre posibles aplicaciones de ADead:
- Ciencias de datos
- Sistemas embedded
- Videojuegos
- Comunicaciones/redes
- Seguridad/criptografía
- Educación
- Herramientas/utilidades

#### `/aplicaciones/` - Aplicaciones Posibles
Ideas y planes para aplicaciones reales con ADead

#### `/windows/` - Windows Específico
Documentación sobre compatibilidad y problemas específicos de Windows

### ⚠️ Documentación Obsoleta

Los siguientes documentos pueden contener información obsoleta sobre Tree-sitter, Zig o arquitecturas anteriores:

- `FLUJO-COMPLETO.md` - Actualizado para indicar que es histórico
- `/avances/ZIG-*.md` - Documentación sobre Zig (no usado actualmente)
- `/avances/INTEGRACION-ZIG-*.md` - Integración con Zig (obsoleto)

**Nota:** La arquitectura actual usa **Parser Manual + Backend C**, no Tree-sitter/Zig/D.

### 🔍 Cómo Saber si un Documento Está Actualizado

**Documentos actuales:**
- ✅ Mencionan "Parser Manual" o "Backend C"
- ✅ Mencionan "GCC/Clang"
- ✅ Fecha: Diciembre 2025
- ✅ Describen flujo: ADead → C → ASM → EXE

**Documentos obsoletos:**
- ⚠️ Mencionan "Tree-sitter" como componente activo
- ⚠️ Mencionan "Zig" como parte del flujo principal
- ⚠️ Describen flujos complejos con múltiples lenguajes
- ⚠️ Fecha anterior a Diciembre 2025

---

## 📝 Convenciones de Documentación

### Estado de Características
- ✅ **Funcional** - Implementado, probado y funcionando
- 🔄 **En desarrollo** - Parcialmente implementado
- ❌ **No implementado** - Falta por implementar
- ⏳ **Pendiente** - Planificado pero no empezado

### Prioridades
- 🔴 **Crítico** - Necesario para desarrollo básico
- 🟠 **Esencial** - Necesario para desarrollo real
- 🟡 **Importante** - Mejora significativa
- 🔵 **Futuro** - Nice to have
