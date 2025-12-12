# 📁 Ejemplos Reales - ADead

Esta carpeta contiene ejemplos prácticos de código ADead organizados en subcarpetas.

## 📂 Estructura de Carpetas

```
Ejemplos-Reales/
├── ejemplos/          # Código fuente (.ad)
├── ejemplos-con-asm/  # Ejemplos con ASM generado (educativo)
├── compilados/        # Ejecutables (.exe) y objetos (.obj)
└── documentacion/     # Documentación (.md)
```

### 📖 Descripción de Carpetas

- **`ejemplos/`** - Código fuente en ADead (.ad) para ejecutar
- **`ejemplos-con-asm/`** - Ejemplos educativos que muestran el código fuente junto con el ASM generado automáticamente
- **`compilados/`** - Archivos compilados (.exe, .obj) generados al ejecutar
- **`documentacion/`** - Guías y documentación completa

## 🚀 Ejecutar un Ejemplo

### Comando Correcto

**Desde la carpeta raíz del proyecto:**
```powershell
.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\hello.ad
```

Este comando automáticamente:
1. ✅ Compila `hello.ad` → `hello.asm`
2. ✅ Ensambla `hello.asm` → `hello.obj`
3. ✅ Enlaza `hello.obj` → `hello.exe`
4. ✅ Ejecuta `hello.exe`
5. ✅ Guarda los archivos generados en `compilados\`

### Ejecutar el .exe Directamente

Una vez generado, puedes ejecutar el `.exe` directamente:

```powershell
# Desde PowerShell
.\Ejemplos-Reales\compilados\hello.exe

# O hacer doble clic en hello.exe desde el explorador
```

---

## 📝 Ejemplos Disponibles

### En `ejemplos/` (Código fuente)

- **`hello.ad`** - Hola Mundo básico con múltiples prints
- **`conditional.ad`** - Ejemplos de if/else condicionales
- **`loop.ad`** - Ejemplos de bucles while
- **`factorial.ad`** - Cálculo de factorial
- **`loop-infinito.ad`** - Bucle infinito con print (usa `Ctrl+C` para detener)

### En `ejemplos-con-asm/` (Educativo)

Esta carpeta contiene ejemplos que muestran cómo el código ADead se traduce automáticamente a ASM (NASM):

- **`hello.ad` + `hello.asm`** - Muestra cómo se traducen los prints a Windows API
- **`loop-infinito.ad` + `loop-infinito.asm`** - Muestra cómo se implementan los bucles
- **`conditional.ad` + `conditional.asm`** - Muestra cómo se traducen if/else
- **`loop.ad` + `loop.asm`** - Muestra bucles con condición

📖 **Revisa `ejemplos-con-asm/README.md` para más detalles educativos.**

---

## 💡 Opciones Útiles

### Mantener Archivos Temporales

Si quieres ver los archivos `.asm` y `.obj` generados:
```powershell
.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\hello.ad --keep-temp
```

### Solo Compilar (sin ejecutar)

```powershell
.\target\release\adeadc.exe compile Ejemplos-Reales\ejemplos\hello.ad -o hello.asm
```

---

## 📚 Documentación

Revisa la carpeta `documentacion/` para:
- **README.md** - Información general
- **COMANDOS.md** - Comandos detallados
- **EJEMPLOS.md** - Ejemplos de código

---

## 🔧 Troubleshooting

**Si el linking falla:**
- Verifica que tienes `gcc` instalado: `gcc --version`
- Verifica que tienes `nasm` instalado: `nasm --version`
- El compilador usará automáticamente tu MinGW64 de MSYS2

**Si quieres ver más detalles:**
- Usa `--keep-temp` para ver los archivos intermedios
- Revisa el archivo `.asm` generado para debugging

---

## ✅ Resumen

**Comando único:**
```powershell
.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\hello.ad
```

**Los archivos compilados se guardan en:** `Ejemplos-Reales\compilados\`

¡Así de simple! 🎉
