# Pruebas: Linking con Zig

**Objetivo:** Verificar que el linking con Zig funciona correctamente para convertir .obj a .exe

---

## ✅ Funcionalidades Implementadas

### **1. Comando `build`**
Compila y linkea completo: `.ad` → `.asm` → `.obj` → `.exe`

**Uso:**
```bash
adeadc build test.ad --linker zig
```

**Opciones:**
- `--backend auto|nasm|cpp|c` - Backend a usar
- `--linker zig|gcc|clang|auto` - Linker a usar
- `--output programa.exe` - Nombre del ejecutable
- `--assemble-only` - Solo ensamblar (.asm → .obj), no linkear

### **2. Comando `link`**
Linkea múltiples archivos .obj a .exe (preparado para módulos)

**Uso:**
```bash
adeadc link mod1.obj mod2.obj main.obj --output programa.exe --linker zig
```

**Opciones:**
- `--linker zig|gcc|clang|auto` - Linker a usar
- `--output programa.exe` - Nombre del ejecutable (requerido)

### **3. Comando `assemble`**
Ensambla archivo .asm a .obj

**Uso:**
```bash
adeadc assemble test.asm --output test.obj
```

---

## 🔧 Detección Automática

El CLI detecta automáticamente qué linker está disponible:
1. **Zig** (prioridad alta - recomendado)
2. **GCC** (fallback)
3. **Clang** (fallback)

Si no encuentra ninguno, muestra un error claro.

---

## 📋 Ejemplos de Uso

### Compilar y linkear completo
```bash
adeadc build test.ad --linker zig
```

### Solo compilar a ASM
```bash
adeadc compile test.ad
```

### Solo ensamblar
```bash
adeadc assemble test.asm
```

### Linkear múltiples módulos (futuro)
```bash
adeadc link mod1.obj mod2.obj main.obj --output programa.exe --linker zig
```

---

## ✅ Estado

- ✅ Detección automática de Zig/GCC/Clang
- ✅ Linking con Zig implementado
- ✅ Soporte para múltiples .obj (preparado para módulos)
- ✅ Comando `build` completo
- ✅ Comando `link` para múltiples archivos
- ✅ Comando `assemble` para ensamblar

---

**Fecha:** Diciembre 2025

