# 🔧 Solución de Problemas - Scripts de Testing

## ⚠️ Problemas Comunes y Soluciones

### Problema 1: Scripts `.ps1` no funcionan

**Síntomas:**
- Error: "no se reconoce como nombre de un cmdlet"
- Error: "cannot be loaded because running scripts is disabled"

**Soluciones:**

#### Opción A: Usar scripts `.bat` simplificados (Recomendado)
```cmd
ejecutar_test_simple.bat test_strings_basico.ad
```

#### Opción B: Cambiar política de PowerShell
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

#### Opción C: Ejecutar PowerShell con bypass
```powershell
powershell.exe -ExecutionPolicy Bypass -File ejecutar_test_individual.ps1 test_strings_basico.ad
```

---

### Problema 2: Compilador no encontrado

**Síntomas:**
- Error: "Compilador no encontrado en: ..."

**Solución:**

1. Verificar que el compilador existe:
```cmd
cd CORE\rust
dir target\release\adeadc.exe
```

2. Si no existe, compilar:
```cmd
cd CORE\rust
cargo build --release
```

3. Ejecutar diagnóstico:
```cmd
cd Ejemplos-Reales\ejemplos\basicos
DIAGNOSTICO.bat
```

---

### Problema 3: Archivos de test no encontrados

**Síntomas:**
- Error: "Archivo no encontrado: test_strings_basico.ad"

**Solución:**

1. Verificar que estás en el directorio correcto:
```cmd
cd Ejemplos-Reales\ejemplos\basicos
dir test_strings_*.ad
```

2. Si no existen, verificar que los archivos fueron creados correctamente.

---

### Problema 4: NASM/GCC no encontrados

**Síntomas:**
- Advertencia: "NASM no encontrado" o "GCC no encontrado"

**Solución:**

1. Verificar que están instalados:
```cmd
nasm -v
gcc --version
```

2. Si no están instalados:
   - **NASM**: Descargar de https://www.nasm.us/
   - **GCC**: Instalar MinGW-w64 o usar Visual Studio Build Tools

3. Agregar al PATH si es necesario.

---

### Problema 5: Compilación falla

**Síntomas:**
- Error: "Compilación falló"
- No se genera archivo `.asm`

**Solución:**

1. Verificar el código fuente del test:
```cmd
type test_strings_basico.ad
```

2. Intentar compilar manualmente:
```cmd
CORE\rust\target\release\adeadc.exe compile test_strings_basico.ad --backend auto -o test.asm
```

3. Revisar errores del compilador.

---

## 🛠️ Scripts de Diagnóstico

### DIAGNOSTICO.bat
Ejecuta un diagnóstico completo del entorno:
```cmd
DIAGNOSTICO.bat
```

Verifica:
- ✅ Directorio actual
- ✅ Archivos de test
- ✅ Compilador
- ✅ NASM
- ✅ GCC
- ✅ PowerShell

---

## 📋 Scripts Disponibles

### Scripts Batch (Más Confiables)

1. **`ejecutar_test_simple.bat`** ⭐ **RECOMENDADO**
   - Script simplificado sin dependencias de PowerShell
   - Funciona en cualquier Windows
   - Uso: `ejecutar_test_simple.bat test_strings_basico.ad`

2. **`EJECUTAR-TEST.bat`**
   - Ejecuta el script PowerShell con bypass
   - Uso: `EJECUTAR-TEST.bat test_strings_basico.ad`

3. **`DIAGNOSTICO.bat`**
   - Diagnóstico completo del entorno
   - Uso: `DIAGNOSTICO.bat`

### Scripts PowerShell

1. **`ejecutar_test_individual.ps1`**
   - Requiere política de ejecución modificada
   - Uso: `powershell.exe -ExecutionPolicy Bypass -File ejecutar_test_individual.ps1 test_strings_basico.ad`

---

## ✅ Verificación Rápida

Ejecuta estos comandos para verificar que todo está bien:

```cmd
REM 1. Verificar compilador
cd CORE\rust
dir target\release\adeadc.exe

REM 2. Verificar tests
cd ..\..\Ejemplos-Reales\ejemplos\basicos
dir test_strings_*.ad

REM 3. Ejecutar diagnóstico
DIAGNOSTICO.bat

REM 4. Probar compilación manual
CORE\rust\target\release\adeadc.exe compile test_strings_basico.ad --backend auto -o test.asm
```

---

## 🎯 Solución Rápida

Si nada funciona, usa el script más simple:

```cmd
cd Ejemplos-Reales\ejemplos\basicos
ejecutar_test_simple.bat test_strings_basico.ad
```

Este script:
- ✅ No requiere PowerShell
- ✅ Busca el compilador automáticamente
- ✅ Muestra errores claros
- ✅ Funciona en cualquier Windows

---

**Última actualización:** Diciembre 2025

