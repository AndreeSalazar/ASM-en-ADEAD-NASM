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

### Proceso de Compilación Modular

```
Source (.ad) 
  → Lexer (tokens)
  → Parser (AST)
  → Code Generator (NASM)
  → compile → .asm (Assembly)
  → assemble → .obj/.o (Object file)
  → link → .exe (Ejecutable)
  → run → Ejecutar programa
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

- [Gramática](docs/grammar.md)
- [Tutorial](docs/tutorial.md)
- [Diseño Técnico](docs/design.md)
- [Uso Rápido](USO-RAPIDO.md) - Guía rápida de comandos
- [Comandos Fáciles](Fácil_Comando.md) - Comandos simplificados
- [Ideas Futuras](ideas2.md) - Roadmap e ideas de desarrollo
- [Contribuir](CONTRIBUTING.md)
- [Autores](AUTHORS.md)
- [Changelog](CHANGELOG.md)

## 🛠️ Estado del Proyecto

**MVP Funcional** ✅

- ✅ Parser completo (print, let, if, while, funciones)
- ✅ Generación NASM para x86_64 Windows/Linux
- ✅ CLI tool modular (compile, assemble, link, run)
- ✅ Ejemplos básicos
- ✅ Icono personalizado para archivos `.ad` en Windows
- ✅ Compilación completa funcional en Windows con MinGW/MSYS2

**Mejoras Recientes:**

- ✅ Proceso de compilación modularizado
- ✅ Mejor manejo de errores y diagnósticos
- ✅ Soporte robusto para Windows con MinGW64
- ✅ Identidad visual con iconos personalizados

**En desarrollo:**

- [ ] Sistema de tipos más robusto
- [ ] Optimizaciones (const folding, dead code elimination)
- [ ] Registro allocation mejorado
- [ ] Arrays y strings
- [ ] Interoperabilidad con C
- [ ] Syntax highlighting para editores
- [ ] LSP (Language Server Protocol) para IDEs

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

1. ✅ **MVP**: print/let/if/while/func + tests
2. 🔄 **Reg Alloc**: mejor asignación de registros
3. 📅 **IR y optimizaciones**: const-fold, dead-code
4. 📅 **Calls & extern**: interoperabilidad con C
5. 📅 **SIMD/intrinsics**: operaciones optimizadas
6. 📅 **Multi-target**: Windows/Mac
7. 📅 **Tooling**: LSP, formatter, playground web

---

<div align="center">

**Hecho con ❤️ en Rust por Eddi Andreé Salazar Matos**

⚡ *ADead - Simple syntax, powerful performance* ⚡

*11 de Diciembre de 2025*

</div>

