# 🚀 Uso Rápido - ADead

Guía completa y resumida para usar el compilador ADead.

---

## 📋 Índice

1. [Método Combinado (Todo de una vez)](#método-combinado)
2. [Métodos Separados (Paso a paso)](#métodos-separados)
3. [Ejecutar .exe Generados](#ejecutar-exe-generados)
4. [Ubicación de Archivos](#ubicación-de-archivos)

---

## 🔄 Método Combinado

### Compilar, Ensamblar, Enlazar y Ejecutar todo junto

```powershell
.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\hello.ad
```

**Este comando hace todo automáticamente:**
1. ✅ Compila `.ad` → `.asm` (código NASM)
2. ✅ Ensambla `.asm` → `.obj` (archivo objeto)
3. ✅ Enlaza `.obj` → `.exe` (ejecutable)
4. ✅ Ejecuta el programa automáticamente

**Con archivos temporales (para debugging):**
```powershell
.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\hello.ad --keep-temp
```
Mantiene los archivos `.asm` y `.obj` para revisarlos.

---

## 🔧 Métodos Separados

### Paso 1: Compilar (.ad → .asm)

Genera código NASM desde tu código ADead:

```powershell
.\target\release\adeadc.exe compile Ejemplos-Reales\ejemplos\hello.ad
```

**Con salida personalizada:**
```powershell
.\target\release\adeadc.exe compile Ejemplos-Reales\ejemplos\hello.ad -o Ejemplos-Reales\compilados\hello.asm
```

**Resultado:** Genera `hello.asm` con código NASM.

---

### Paso 2: Ensamblar (.asm → .obj)

Convierte código NASM en archivo objeto:

```powershell
.\target\release\adeadc.exe assemble Ejemplos-Reales\compilados\hello.asm
```

**Con salida personalizada:**
```powershell
.\target\release\adeadc.exe assemble Ejemplos-Reales\compilados\hello.asm -o Ejemplos-Reales\compilados\hello.obj
```

**Resultado:** Genera `hello.obj` (Windows) o `hello.o` (Linux).

---

### Paso 3: Enlazar (.obj → .exe)

Convierte archivo objeto en ejecutable:

```powershell
.\target\release\adeadc.exe link Ejemplos-Reales\compilados\hello.obj
```

**Con salida personalizada:**
```powershell
.\target\release\adeadc.exe link Ejemplos-Reales\compilados\hello.obj -o Ejemplos-Reales\compilados\hello.exe
```

**Resultado:** Genera `hello.exe` (Windows) o `hello` (Linux).

---

## ▶️ Ejecutar .exe Generados

Una vez que tienes el `.exe`, puedes ejecutarlo de varias formas:

### Opción 1: Desde PowerShell

```powershell
# Desde la raíz del proyecto
.\Ejemplos-Reales\compilados\hello.exe

# O con ruta completa si hay espacios
& "Ejemplos-Reales\compilados\hello.exe"
```

### Opción 2: Desde CMD

```cmd
Ejemplos-Reales\compilados\hello.exe
```

### Opción 3: Doble clic

Haz doble clic en `hello.exe` desde el explorador de archivos.

### Opción 4: Desde la carpeta compilados

```powershell
cd Ejemplos-Reales\compilados
.\hello.exe
```

---

## 📁 Ubicación de Archivos

### Estructura de Carpetas

```
Ejemplos-Reales/
├── ejemplos/          # Código fuente .ad
│   ├── hello.ad
│   ├── factorial.ad
│   └── ...
├── compilados/        # Archivos generados (.asm, .obj, .exe)
│   ├── hello.asm
│   ├── hello.obj
│   ├── hello.exe      # ✅ Ejecutables finales
│   └── ...
└── documentacion/     # Documentación adicional
```

### Dónde se guardan los archivos

| Tipo | Ubicación por defecto |
|------|----------------------|
| **Código fuente** | `Ejemplos-Reales/ejemplos/*.ad` |
| **ASM generado** | `Ejemplos-Reales/compilados/*.asm` |
| **Archivo objeto** | `Ejemplos-Reales/compilados/*.obj` |
| **Ejecutable** | `Ejemplos-Reales/compilados/*.exe` |

**Nota:** Si el archivo `.ad` está en `ejemplos/`, los archivos generados van a `compilados/`. Si no, se guardan en la misma carpeta que el `.ad`.

---

## 🎯 Ejemplos Rápidos

### Ejemplo 1: Todo de una vez
```powershell
.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\hello.ad
```

### Ejemplo 2: Paso a paso (útil para estudiar ASM)
```powershell
# Compilar
.\target\release\adeadc.exe compile Ejemplos-Reales\ejemplos\hello.ad -o Ejemplos-Reales\compilados\hello.asm

# Ensamblar
.\target\release\adeadc.exe assemble Ejemplos-Reales\compilados\hello.asm

# Enlazar
.\target\release\adeadc.exe link Ejemplos-Reales\compilados\hello.obj

# Ejecutar
.\Ejemplos-Reales\compilados\hello.exe
```

### Ejemplo 3: Solo compilar para ver el ASM
```powershell
.\target\release\adeadc.exe compile Ejemplos-Reales\ejemplos\hello.ad -o hello.asm
code hello.asm  # Abre en editor
```

---

## 💡 Tips

- **Para debugging:** Usa `--keep-temp` para mantener archivos intermedios
- **Para estudiar ASM:** Usa métodos separados y revisa los `.asm` generados
- **Para ejecutar rápido:** Usa el método combinado `run`
- **Los .exe:** Nunca se eliminan, siempre están disponibles para ejecutar

---

¡Listo! Ahora puedes compilar y ejecutar programas ADead de todas las formas posibles. 🎉
