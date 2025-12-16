# 🚀 Instrucciones Rápidas - Solución de Problemas

## ⚠️ Si los scripts no funcionan

### Paso 1: Verificar que el compilador existe

```cmd
cd CORE\rust
cargo build --release
```

Luego verificar:
```cmd
dir target\release\adeadc.exe
```

Si no existe, puede estar en:
```cmd
dir target\release\*.exe
```

### Paso 2: Buscar el compilador automáticamente

```cmd
cd Ejemplos-Reales\ejemplos\basicos
BUSCAR-COMPILADOR.bat
```

Este script buscará el compilador en todas las ubicaciones posibles.

### Paso 3: Compilar manualmente un test

```cmd
COMPILAR-MANUAL.bat test_strings_basico.ad
```

Este script:
- Busca el compilador automáticamente
- Compila el test
- Muestra instrucciones para ensamblar

### Paso 4: Usar el script más simple

```cmd
ejecutar_test_simple.bat test_strings_basico.ad
```

Este script busca el compilador en múltiples ubicaciones automáticamente.

---

## 🔍 Diagnóstico Completo

Ejecuta el diagnóstico completo:

```cmd
cd Ejemplos-Reales\ejemplos\basicos
DIAGNOSTICO.bat
```

---

## 📝 Compilación Manual Paso a Paso

Si nada funciona, compila manualmente:

### 1. Compilar el proyecto
```cmd
cd CORE\rust
cargo build --release
```

### 2. Verificar el ejecutable
```cmd
dir target\release\*.exe
```

El ejecutable debería llamarse `adeadc.exe` o similar.

### 3. Compilar un test
```cmd
cd ..\..\Ejemplos-Reales\ejemplos\basicos
CORE\rust\target\release\adeadc.exe compile test_strings_basico.ad --backend auto -o test.asm
```

### 4. Ensamblar (si tienes NASM)
```cmd
nasm -f win64 test.asm -o test.obj
```

### 5. Linkear (si tienes GCC)
```cmd
gcc test.obj -o test.exe
```

### 6. Ejecutar
```cmd
test.exe
```

---

## 🆘 Si el compilador tiene otro nombre

Si el ejecutable tiene otro nombre (no `adeadc.exe`), busca:

```cmd
cd CORE\rust
dir /s /b target\release\*.exe
```

Luego modifica los scripts para usar el nombre correcto.

---

## ✅ Verificación Rápida

```cmd
REM 1. Verificar que compilaste correctamente
cd CORE\rust
cargo build --release
dir target\release\*.exe

REM 2. Buscar compilador
cd ..\..\Ejemplos-Reales\ejemplos\basicos
BUSCAR-COMPILADOR.bat

REM 3. Probar compilación
COMPILAR-MANUAL.bat test_strings_basico.ad
```

---

**Última actualización:** Diciembre 2025

