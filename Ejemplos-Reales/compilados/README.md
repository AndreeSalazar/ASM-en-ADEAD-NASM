# 📁 Compilados - Archivos Generados

Esta carpeta contiene los archivos compilados y generados por ADead, organizados en subcarpetas para mejor mantenimiento.

## 📂 Estructura

```
compilados/
├── fuentes/          # Archivos fuente ADead (.ad)
├── ejecutables/      # Ejecutables compilados (.exe)
├── temporales/       # Archivos temporales (.c, .asm) - pueden eliminarse
└── README.md         # Este archivo
```

## 📝 Descripción de Carpetas

### `fuentes/`
Contiene todos los archivos fuente `.ad` que pueden ser compilados.

**Archivos disponibles:**
- `test_array_completo.ad` - Test completo de todas las operaciones de arrays
- `test_array_avanzado.ad` - Test avanzado de arrays (append, length, asignación)
- `test_pop_simple.ad` - Test simple de pop()
- `test_simple_append.ad` - Test simple de append()
- `test_array.ad` - Test básico de arrays
- `test_10.ad` - Test con while e if
- `100mil_optimizado.ad` - Loop optimizado hasta 100k
- `1_billon_optimizado.ad` - Loop optimizado hasta 1 billón
- `1_billón.ad` - Variante del loop de 1 billón

### `ejecutables/`
Contiene los ejecutables compilados listos para ejecutar.

**Ejecutables disponibles:**
- `test_array_completo.exe` - Test completo de arrays (pop, insert, remove, index, count, sort, reverse)
- `test_array_avanzado.exe` - Test avanzado de arrays
- `test_pop_simple.exe` - Test simple de pop()
- `test_simple_append.exe` - Test simple de append()
- `test_10_c.exe` - Test con while e if
- `100mil_optimizado_c.exe` - Loop hasta 100k
- `1_billon_optimizado_c.exe` - Loop hasta 1 billón

### `temporales/`
Contiene archivos temporales generados durante la compilación:
- Archivos `.c` - Código C intermedio generado
- Archivos `.asm` - Código Assembly generado

**Nota:** Estos archivos pueden eliminarse de forma segura. Se regeneran automáticamente al compilar.

## 🚀 Cómo Usar

### Compilar un archivo fuente:
```powershell
cd "C:\Users\andre\OneDrive\Documentos\ASM en ADEAD"
.\CORE\rust\target\release\adeadc.exe compile "Ejemplos-Reales\compilados\fuentes\test_array_completo.ad" --backend c -o "Ejemplos-Reales\compilados\ejecutables\test_array_completo.exe"
```

### Ejecutar un programa:
```powershell
cd "Ejemplos-Reales\compilados\ejecutables"
.\test_array_completo.exe
```

## 🧹 Limpieza

Para limpiar archivos temporales:
```powershell
cd "Ejemplos-Reales\compilados\temporales"
Remove-Item *.c, *.asm
```

## 📊 Estadísticas

- **Archivos fuente:** 9 archivos `.ad`
- **Ejecutables:** 7 ejecutables `.exe`
- **Archivos temporales:** 11 archivos (`.c` y `.asm`)

---

**Última actualización:** Diciembre 2025  
**Organización:** Archivos organizados por tipo para mejor mantenimiento

