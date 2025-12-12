# Changelog

Todos los cambios notables de este proyecto serán documentados en este archivo.

El formato está basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.0.0/),
y este proyecto adhiere a [Semantic Versioning](https://semver.org/lang/es/).

## [0.1.0] - 2025-12-11

### Añadido
- ✨ Parser completo con chumsky para:
  - Print statements con strings
  - Variables (`let`)
  - Asignaciones
  - Condicionales (`if/else`)
  - Loops (`while`)
  - Funciones con parámetros y return
  - Expresiones aritméticas (+, -, *, /)
  - Comparaciones (==, !=, <, <=, >, >=)
  
- 🏗️ Backend NASM:
  - Generación de código NASM x86_64
  - System V ABI compliance
  - Stack frame management
  - String handling en data section
  - Labels y jumps para control de flujo
  
- 🛠️ CLI tool:
  - Comando `compile`
  - Flags `-o` (output) y `--run` (auto-assemble)
  
- 📚 Documentación:
  - README completo
  - Gramática formal
  - Tutorial de 5 minutos
  - Documento de diseño técnico
  - Guía de contribución
  
- 📦 Ejemplos:
  - Hello World
  - Condicionales
  - Loops
  - Factorial
  
- 🔧 CI/CD:
  - GitHub Actions workflow
  - Tests automatizados
  
- 🇵🇪 Proyecto iniciado por Eddi Andreé Salazar Matos

---

**Desarrollador:** Eddi Andreé Salazar Matos  
**Fecha de lanzamiento inicial:** 11 de Diciembre de 2025  
🇵🇪 *Perú*

