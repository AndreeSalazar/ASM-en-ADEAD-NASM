# 📁 Estructura Organizada - CLEAN_CODE Examples

## ✅ Organización Completada

La carpeta `examples` ahora está organizada de forma clara y estructurada.

## 🗂️ Estructura de Carpetas

```
examples/
│
├── README.md                    # Guía principal - EMPIEZA AQUÍ
├── comparar.ps1                 # Script único para comparar TODO
│
├── ASM/                        # Archivos ASM organizados
│   ├── dirty.asm               # ASM sucio (original Clang)
│   ├── basic.asm               # ASM limpio - nivel básico
│   ├── advanced.asm            # ASM limpio - nivel avanzado
│   └── extreme.asm              # ASM limpio - nivel extremo
│
├── OBJ/                        # Objetos compilados
│   ├── dirty.obj               # Objeto del ASM sucio
│   ├── basic.obj               # Objeto del ASM básico
│   ├── advanced.obj            # Objeto del ASM avanzado
│   └── extreme.obj             # Objeto del ASM extremo
│
├── EXE/                        # Ejecutables
│   └── test_array_funcional.exe # Ejecutable funcional
│
├── CODIGO/                     # Código fuente
│   └── test_array_original.c   # Código C original
│
├── docs/                       # Documentación
│   ├── RESULTADOS.md           # Resultados detallados
│   ├── IDEAS.md                # Ideas para mejoras
│   └── SOLUCION.md             # Soluciones independientes
│
└── scripts/                    # Scripts adicionales (opcionales)
    └── (scripts de desarrollo)
```

## 🚀 Uso Rápido

### Para comparar los 4 elementos:

```powershell
.\comparar.ps1
```

**Eso es todo.** Un solo comando muestra:
- ✅ Compilación y ejecución del programa
- ✅ Comparación de los 4 archivos ASM
- ✅ Comparación de objetos compilados
- ✅ Reducciones y estadísticas

## 📊 Archivos Principales

### 1. `README.md`
**Guía principal** - Explica toda la estructura y cómo usar todo.

### 2. `comparar.ps1`
**Script único principal** - Ejecuta la comparación completa.

### 3. Carpeta `ASM/`
Contiene los 4 archivos ASM para comparar:
- `dirty.asm` - Original sin limpiar
- `basic.asm` - Limpio básico
- `advanced.asm` - Limpio avanzado
- `extreme.asm` - Limpio extremo

### 4. Carpeta `OBJ/`
Contiene los objetos compilados desde los ASM.

### 5. Carpeta `EXE/`
Contiene el ejecutable funcional que demuestra que el código funciona.

### 6. Carpeta `CODIGO/`
Contiene el código fuente C original.

### 7. Carpeta `docs/`
Contiene documentación adicional (opcional).

## 📈 Resultados Esperados

Al ejecutar `.\comparar.ps1` verás:

```
+----------------------+----------+----------+----------+----------+
| Version              | ASM (B)  | Lineas   | OBJ (B)  | EXE (B)  |
+----------------------+----------+----------+----------+----------+
| Ejecutable           |     N/A |     N/A |     N/A |   264279 |
| Sucio                |     4249 |      204 |     N/A |     N/A |
| Basico               |      582 |       28 |     N/A |     N/A |
| Avanzado             |      582 |       28 |     N/A |     N/A |
| Extremo              |      531 |       26 |     N/A |     N/A |
| Sucio (OBJ)          |     N/A |     N/A |     1669 |     N/A |
| Basico (OBJ)         |     N/A |     N/A |      428 |     N/A |
| Avanzado (OBJ)       |     N/A |     N/A |      428 |     N/A |
| Extremo (OBJ)        |     N/A |     N/A |      428 |     N/A |
+----------------------+----------+----------+----------+----------+

REDUCCION vs ASM Sucio:
  Extremo: -87.5% (4,249 -> 531 bytes)
  Extremo: -87.3% (204 -> 26 líneas)

REDUCCION vs OBJ Sucio:
  Extremo (OBJ): -74.4% (1,669 -> 428 bytes)
```

## 🎯 Ventajas de Esta Organización

1. ✅ **Clara y estructurada** - Cada tipo de archivo en su carpeta
2. ✅ **Fácil de navegar** - Encuentras todo rápidamente
3. ✅ **Un solo script** - `comparar.ps1` hace todo
4. ✅ **Sin confusión** - Archivos organizados por tipo
5. ✅ **Fácil de mantener** - Estructura lógica

## 📝 Notas

- **Archivos antiguos** fueron movidos a `scripts/` y `docs/`
- **Solo necesitas** `comparar.ps1` para comparar
- **Lee `README.md`** para más detalles

---

**Estructura creada:** Diciembre 2025  
**Organización:** Completa y lista para usar ✅

