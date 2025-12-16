# 🚀 Guía de Uso: Compilación Paralela de ADead

## 📋 Descripción

El sistema de compilación paralela permite compilar múltiples archivos `.ad` simultáneamente, aprovechando múltiples núcleos de CPU y un sistema de caching inteligente.

## 🎯 Ventajas

- ⚡ **Compilación rápida**: Múltiples archivos en paralelo
- 💾 **Caching inteligente**: No recompila archivos sin cambios
- 🔄 **Optimizaciones automáticas**: CSE, Constant Propagation, Loop Invariant, etc.
- 📊 **Estadísticas detalladas**: Tiempo, éxitos, fallos

## 📝 Uso Básico

### Opción 1: Script Helper (Recomendado)

Desde el directorio `Ejemplos-Reales\compilados`:

```powershell
# Compilar múltiples archivos
.\compile-parallel-local.ps1 test_simple.ad test_ctfe.ad test_cse.ad

# Con directorio de salida personalizado
.\compile-parallel-local.ps1 test_simple.ad test_ctfe.ad -OutputDir .\mi_output

# Limpiar cache antes de compilar
.\compile-parallel-local.ps1 test_simple.ad test_ctfe.ad -ClearCache
```

### Opción 2: Comando Directo

```powershell
# Desde cualquier directorio
$baseDir = "C:\Users\andre\OneDrive\Documentos\ASM en ADEAD"
$exePath = "$baseDir\CORE\rust\target\release\adeadc.exe"

# Compilar archivos
& $exePath compile-parallel archivo1.ad archivo2.ad archivo3.ad -o .\output_parallel
```

### Opción 3: Script desde Raíz del Proyecto

```powershell
# Desde el directorio raíz del proyecto
.\compile-parallel.ps1 Ejemplos-Reales\compilados\test_simple.ad Ejemplos-Reales\compilados\test_ctfe.ad -OutputDir .\output
```

## 🔧 Parámetros Disponibles

### Comando `compile-parallel`

```
Usage: adeadc.exe compile-parallel [OPTIONS] [INPUTS]...

Arguments:
  [INPUTS]...  Archivos de entrada (.ad) - múltiples archivos

Options:
  -o, --output-dir <OUTPUT_DIR>  Directorio de salida [opcional: usa directorio actual]
      --cache-dir <CACHE_DIR>    Directorio de cache [opcional: usa .adead_cache]
      --clear-cache              Limpiar cache antes de compilar
  -h, --help                     Print help
```

## 📊 Ejemplo de Salida

```
🚀 Compilación paralela de 3 archivo(s)
🚀 Compilando 3 archivo(s) en paralelo...
   🔷 Paso 1: D Language - CTFE y optimización compile-time...
   ⚡ Paso 2: Zig - Generación ASM directo...
   🔒 Paso 3: Rust - Limpieza y optimización de ASM...
   ✅ Pipeline completo: ASM virgen y limpio generado

📊 Resumen de compilación paralela:
   ✅ Exitosas: 3
   ❌ Fallidas: 0
   ⏱️  Tiempo total: 459 ms

📦 Estadísticas del cache:
   Total de entradas: 3
   Entradas válidas: 3
```

## 💡 Consejos

1. **Primera ejecución**: Toma más tiempo porque compila todo
2. **Ejecuciones siguientes**: Más rápidas gracias al cache
3. **Archivos modificados**: Solo se recompilan los que cambiaron
4. **Cache persistente**: El cache se mantiene entre ejecuciones (en memoria por ahora)

## 🐛 Solución de Problemas

### Error: "Archivo no encontrado"

**Problema**: Estás ejecutando desde el directorio incorrecto.

**Solución**: 
```powershell
# Navega al directorio donde están los archivos
cd Ejemplos-Reales\compilados
.\compile-parallel-local.ps1 test_simple.ad test_ctfe.ad
```

### Error: "No se encontró el ejecutable"

**Problema**: El proyecto no está compilado.

**Solución**:
```powershell
cd CORE\rust
cargo build --release
```

### Error: "La ruta de salida es un archivo, no un directorio"

**Problema**: Existe un archivo con el mismo nombre que el directorio de salida.

**Solución**: Usa un nombre diferente para el directorio de salida:
```powershell
.\compile-parallel-local.ps1 test_simple.ad -OutputDir .\output_nuevo
```

## 🎓 Ejemplos Prácticos

### Compilar todos los archivos de prueba

```powershell
cd Ejemplos-Reales\compilados
.\compile-parallel-local.ps1 test_*.ad
```

### Compilar con cache personalizado

```powershell
$exePath = "CORE\rust\target\release\adeadc.exe"
& $exePath compile-parallel test1.ad test2.ad --cache-dir .\.mi_cache
```

### Compilar y limpiar cache

```powershell
.\compile-parallel-local.ps1 test_simple.ad test_ctfe.ad -ClearCache
```

## 📈 Rendimiento

- **Compilación secuencial**: ~1100 ms para 3 archivos
- **Compilación paralela**: ~450 ms para 3 archivos
- **Mejora**: ~2.4x más rápido

## 🔮 Próximas Mejoras

- [ ] Cache persistente en disco (entre ejecuciones)
- [ ] Compilación incremental automática
- [ ] Estadísticas más detalladas
- [ ] Soporte para wildcards (*.ad)

---

**Creado por:** Eddi Andreé Salazar Matos  
**Fecha:** Diciembre 2025  
**Proyecto:** ADead - ASM en estilo Python

