<div align="center">

# 🇵🇪 .ad — ADead

**ASM is dead (but powerful)**

Simple sintaxis estilo Python • Rendimiento nativo

🎨 **Icono personalizado para archivos `.ad`** - Identidad visual única en Windows

**Desarrollado por:** Eddi Andreé Salazar Matos  
**Fecha:** 11 de Diciembre de 2025

</div>

## 🚀 Quickstart

### Requisitos

**Linux (recomendado):**
- Rust (última versión estable)
- NASM (`nasm` en PATH)
- binutils (`ld` en PATH)

**Windows:**
- Rust (última versión estable)
- NASM (`nasm` en PATH)
- MinGW/MSYS2 con `gcc` o binutils con `ld`
- ⚠️ **Nota importante:** El código generado usa syscalls de Linux. Para ejecutar en Windows necesitas:
  - WSL (Windows Subsystem for Linux) - **Recomendado**
  - O usar herramientas de Linux (MSYS2 puede funcionar con algunas limitaciones)

### Instalación

```bash
# Clonar el repo
git clone https://github.com/tuusuario/adead.git
cd adead

# Compilar
cargo build --release

# El binario estará en: target/release/adeadc
```

### Uso

#### 🎨 Icono Personalizado (Windows)

Los archivos `.ad` incluyen un icono personalizado en Windows Explorer. El icono se aplica automáticamente al instalar o mediante scripts de configuración.

#### 🚀 Método Simple: Un Solo Comando

**Desde cualquier lugar:**
```powershell
# Windows - Genera .exe y ejecuta automáticamente
.\target\release\adeadc.exe run Ejemplos-Reales\hello.ad

# Linux/Mac - Genera ejecutable y ejecuta
./target/release/adeadc run examples/hello.ad
```

El comando `run` automáticamente:
1. ✅ Compila el `.ad` a `.asm`
2. ✅ Ensambla a `.obj` (Windows) o `.o` (Linux)
3. ✅ Enlaza a `.exe` (Windows) o ejecutable (Linux)
4. ✅ Ejecuta el programa
5. ✅ Limpia archivos temporales (a menos que uses `--keep-temp`)

**El `.exe` se genera en la misma carpeta que el archivo `.ad`**

#### Opciones Avanzadas

```powershell
# Mantener archivos temporales para debugging
.\target\release\adeadc.exe run Ejemplos-Reales\hello.ad --keep-temp

# Solo compilar a ASM (sin ejecutar)
.\target\release\adeadc.exe compile Ejemplos-Reales\hello.ad -o hello.asm
```

#### Ejemplos Reales

Los ejemplos funcionales están en la carpeta `Ejemplos-Reales/`:

```powershell
# Ejecutar desde la raíz
.\run.ps1 Ejemplos-Reales\hello.ad

# O desde la carpeta Ejemplos-Reales
cd Ejemplos-Reales
.\ejecutar.ps1 hello.ad
```

Ver [Ejemplos-Reales/README.md](Ejemplos-Reales/README.md) para más detalles.

## 📝 Ejemplos

### Hello World

```adead
print "Hola Mundo"
```

### Variables y Aritmética

```adead
let x = 10
let y = 20
let sum = x + y
```

### Condicionales

```adead
if x > 5 {
    print "x is greater than 5"
} else {
    print "x is less than or equal to 5"
}
```

### Loops

```adead
let i = 0
while i < 10 {
    print "Iteration: "
    print i
    i = i + 1
}
```

### Funciones

```adead
fn add(a, b) {
    return a + b
}

let result = add(5, 3)
```

## 🏗️ Arquitectura

### Arquitectura Híbrida Zig + Rust

**Filosofía:** Cada lenguaje hace lo que mejor sabe
- **Zig:** Parsing eficiente y directo (compensa debilidades de Rust en parsing complejo)
- **Rust:** Validación de memoria, borrow checking y generación de código NASM (fortalezas de Rust)

### Proceso de Compilación Completo

```
Source (.ad) 
  → Zig Parser (parsing de structs complejos) ⚡
  → Rust Parser (resto del lenguaje)
  → Rust Validator (borrow checker, type checking)
  → Rust Code Generator (NASM)
  → compile → .asm (Assembly)
  → assemble → .obj/.o (Object file)
  → link → .exe (Ejecutable)
  → run → Ejecutar programa
```

**Flujo Detallado:**
```
.ad → Zig (Parsing structs) + Rust (Parsing resto + Validación + Codegen) 
  → .asm (NASM) → NASM → .obj → link → .exe → ✅ Ejecución
```

### Comandos Modulares

Puedes ejecutar cada paso por separado para mayor control:

```powershell
# 1. Compilar a Assembly
.\target\release\adeadc.exe compile Ejemplos-Reales\hello.ad

# 2. Ensamblar a objeto
.\target\release\adeadc.exe assemble Ejemplos-Reales\hello.asm

# 3. Enlazar a ejecutable
.\target\release\adeadc.exe link Ejemplos-Reales\hello.obj

# 4. Ejecutar
.\target\release\adeadc.exe run Ejemplos-Reales\hello.exe

# O todo en uno:
.\target\release\adeadc.exe run Ejemplos-Reales\hello.ad
```

## 📚 Documentación

### Guías Principales
- [Comandos Fáciles](Fácil_Comando.md) - ⚡ Guía rápida para empezar
- [Roadmap Profesional](docs/ROADMAP-PROFESIONAL.md) - 🎯 Plan completo para hacer ADead profesional

### Documentación Técnica
- [Arquitectura](docs/ARQUITECTURA.md) - Zig + Rust integrados
- [Flujo de Compilación](docs/FLUJO-COMPILACION.md) - Proceso completo
- [Ideas OOP](ideas3.md) - Programación Orientada a Objetos
- [Ideas Futuras](ideas2.md) - Roadmap e ideas de desarrollo

### Referencias
- [Gramática](docs/grammar.md)
- [Tutorial](docs/tutorial.md)
- [Diseño Técnico](docs/design.md)
- [Uso Rápido](USO-RAPIDO.md)
- [Contribuir](CONTRIBUTING.md)
- [Autores](AUTHORS.md)
- [Changelog](CHANGELOG.md)

## 🛠️ Estado del Proyecto

**MVP Funcional** ✅

- ✅ **Parser completo:** Zig + Rust integrados
  - Zig: Parsing eficiente de structs complejos
  - Rust: Parsing del resto + validación + codegen
- ✅ **OOP Básico:** Structs, métodos, `init`/`destroy`, encapsulación (`pub`/`private`)
- ✅ Generación NASM para x86_64 Windows/Linux
- ✅ CLI tool modular (compile, assemble, link, run)
- ✅ Ejemplos funcionales (hello, factorial, conditional, loop, structs, encapsulación, RAII)
- ✅ Icono personalizado para archivos `.ad` en Windows
- ✅ Compilación completa funcional en Windows con MinGW/MSYS2
- ✅ **Flujo completo:** `.ad → Zig/Rust → ASM → NASM → .exe` funcionando

**Mejoras Recientes:**

- ✅ Proceso de compilación modularizado
- ✅ Mejor manejo de errores y diagnósticos
- ✅ Soporte robusto para Windows con MinGW64
- ✅ Identidad visual con iconos personalizados

**Completado Recientemente:**

- ✅ Parsing híbrido Zig + Rust (Zig compensa parsing complejo de structs)
- ✅ Encapsulación (public/private) - O5 completado
- ✅ RAII (init/destroy) - O2 completado
- ✅ Structs con campos y métodos
- ✅ Codegen de strings en struct literals
- ✅ Flujo completo funcionando: `.ad → Zig + Rust → ASM → .exe`

**🚀 Próximos Pasos (Roadmap Profesional):**

**Críticos (Sprint 1 - Mes 1):**
- [ ] Manejo de errores completo (Option/Result funcionales)
- [ ] Arrays básicos
- [ ] Import básico (módulos simples)

**Esenciales (Sprint 2-3 - Mes 2-3):**
- [ ] Librería estándar mínima (`std.string`, `std.math`, `std.array`)
- [ ] Sistema de módulos completo
- [ ] Tipos nativos: Bool, Float

**Profesionales (Sprint 4-6 - Mes 4-6):**
- [ ] Package Manager
- [ ] Interoperabilidad C/Rust
- [ ] Pipeline optimizado (caching, incremental)
- [ ] Documentación oficial completa

📖 **Ver [docs/ROADMAP-PROFESIONAL.md](docs/ROADMAP-PROFESIONAL.md) para detalles completos.**

## 🧪 Testing

```bash
cargo test --workspace
```

## 🤝 Contribuir

¡Las contribuciones son bienvenidas! Por favor lee [CONTRIBUTING.md](CONTRIBUTING.md) para más detalles.

## 👨‍💻 Autor

**Eddi Andreé Salazar Matos**

- Proyecto iniciado: 11 de Diciembre de 2025
- ⚡ Lenguaje .ad - Simple y poderoso

Para más información, ver [AUTHORS.md](AUTHORS.md)

## 📄 Licencia

MIT License - ver [LICENSE](LICENSE) para más detalles.

Copyright (c) 2025 Eddi Andreé Salazar Matos

## 🎯 Roadmap

### ✅ Completado (MVP)
1. ✅ **MVP**: print/let/if/while/func + tests
2. ✅ **OOP Básico**: Structs, métodos, RAII, encapsulación
3. ✅ **Arquitectura**: Zig + Rust integrados para parsing y codegen
4. ✅ **Flujo completo**: `.ad → Zig + Rust → ASM → .exe` funcionando

### 🚀 Próximos Pasos (Roadmap Profesional)
1. 🔴 **Manejo de errores completo**: Option/Result funcionales
2. 🔴 **Arrays y tipos básicos**: Arrays, Bool, Float nativos
3. 🔴 **Sistema de módulos**: Import/export, proyectos multi-archivo
4. 🔴 **Librería estándar**: `std.string`, `std.math`, `std.array`
5. 🟡 **Package Manager**: Ecosistema distribuido
6. 🟡 **Interoperabilidad**: C/Rust FFI
7. 🟡 **Optimizaciones**: Compilación incremental, caching, flags
8. 🟡 **Documentación**: Guías oficiales completas

📖 **Ver [docs/ROADMAP-PROFESIONAL.md](docs/ROADMAP-PROFESIONAL.md) para el plan detallado de 6 meses.**

---

<div align="center">

**Hecho con ❤️ en Rust por Eddi Andreé Salazar Matos**

⚡ *ADead - Simple syntax, powerful performance* ⚡

*11 de Diciembre de 2025*

</div>

