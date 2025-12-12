# ⚡ Comandos Fáciles - ADead

Guía ultra-simplificada para ejecutar código ADead en segundos.

---

## 🚀 Ejecutar Cualquier Archivo .ad

### Comando Básico (TODO de una vez)

```powershell
.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\hello.ad
```

**Eso es todo.** Esto compila, ensambla, enlaza y ejecuta automáticamente.

---

## 📝 Ejemplos Rápidos

### Hello World
```powershell
.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\hello.ad
```

### Factorial
```powershell
.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\factorial.ad
```

### Conditional (If/Else)
```powershell
.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\conditional.ad
```

### Loop (While)
```powershell
.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\loop.ad
```

### Loop Infinito
```powershell
.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\loop-infinito.ad
```

---

## ▶️ Ejecutar el .exe Ya Generado

Si ya compilaste antes, simplemente ejecuta el `.exe`:

```powershell
.\Ejemplos-Reales\compilados\hello.exe
```

---

## 🔍 Ver el Proceso (Paso a Paso)

Si quieres ver cada paso del proceso:

```powershell
.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\hello.ad --keep-temp
```

Esto muestra cada paso y guarda los archivos `.asm` y `.obj` para revisarlos.

---

## 📁 Dónde Están los Ejecutables

Todos los `.exe` se guardan en:
```
Ejemplos-Reales\compilados\
```

Ejemplos:
- `Ejemplos-Reales\compilados\hello.exe`
- `Ejemplos-Reales\compilados\factorial.exe`
- `Ejemplos-Reales\compilados\conditional.exe`
- etc.

---

## 💡 Tips Rápidos

1. **Ejecutar rápido:** Usa `run` (hace todo automático)
2. **Ver código ASM:** Agrega `--keep-temp` y revisa `Ejemplos-Reales\compilados\*.asm`
3. **Ejecutar .exe:** Directamente `.\Ejemplos-Reales\compilados\nombre.exe`
4. **Tu propio código:** Crea `Ejemplos-Reales\ejemplos\mi-codigo.ad` y ejecútalo igual

---

## 🎯 Plantilla para Tu Código

1. Crea un archivo en `Ejemplos-Reales\ejemplos\tu-archivo.ad`
2. Escribe tu código ADead
3. Ejecuta:
   ```powershell
   .\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\tu-archivo.ad
   ```
4. ¡Listo! Tu programa se ejecuta automáticamente.

---

## ⚡ Atajo de PowerShell (Opcional)

Si usas PowerShell frecuentemente, puedes crear un alias:

```powershell
Set-Alias adead ".\target\release\adeadc.exe"
```

Luego solo ejecutas:
```powershell
adead run Ejemplos-Reales\ejemplos\hello.ad
```

---

## 📋 Resumen Ultra-Rápido

**Para ejecutar cualquier .ad:**
```powershell
.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\NOMBRE.ad
```

**Para ejecutar el .exe:**
```powershell
.\Ejemplos-Reales\compilados\NOMBRE.exe
```

**¡Eso es todo!** 🎉

