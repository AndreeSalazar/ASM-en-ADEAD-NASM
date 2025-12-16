# 🚀 Guía de Ejecución - Scripts de Testing

## ⚠️ Problema Común: Política de Ejecución de PowerShell

Si recibes el error:
```
El término '.\ejecutar_test_individual.ps1' no se reconoce como nombre de un cmdlet...
```

Esto significa que PowerShell tiene restricciones para ejecutar scripts. Hay varias soluciones:

---

## ✅ Solución 1: Usar Scripts Batch (Recomendado)

He creado scripts `.bat` que ejecutan PowerShell con la política correcta:

### Ejecutar un test individual:
```cmd
EJECUTAR-TEST.bat test_strings_basico.ad
```

### Ejecutar todos los tests:
```cmd
EJECUTAR-TODOS-TESTS.bat
```

### Verificar solo compilación:
```cmd
VERIFICAR-COMPILACION.bat
```

---

## ✅ Solución 2: Cambiar Política de Ejecución de PowerShell

Abre PowerShell como **Administrador** y ejecuta:

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

Luego puedes ejecutar los scripts normalmente:
```powershell
.\ejecutar_test_individual.ps1 test_strings_basico.ad
```

---

## ✅ Solución 3: Ejecutar con Bypass Temporal

Ejecuta PowerShell con bypass de política:

```powershell
powershell.exe -ExecutionPolicy Bypass -File ejecutar_test_individual.ps1 test_strings_basico.ad
```

---

## ✅ Solución 4: Ejecutar Directamente desde PowerShell

Si estás en PowerShell, usa la ruta completa:

```powershell
& ".\ejecutar_test_individual.ps1" test_strings_basico.ad
```

O con el operador de llamada:

```powershell
& .\ejecutar_test_individual.ps1 test_strings_basico.ad
```

---

## 📋 Verificar Directorio Actual

Asegúrate de estar en el directorio correcto:

```powershell
# Ver directorio actual
Get-Location

# Cambiar al directorio de tests
cd Ejemplos-Reales\ejemplos\basicos

# Verificar que los scripts existen
Get-ChildItem *.ps1
```

---

## 🔧 Verificar Política Actual

Para ver tu política de ejecución actual:

```powershell
Get-ExecutionPolicy -List
```

---

## 📝 Archivos Disponibles

### Scripts PowerShell (.ps1):
- `ejecutar_test_individual.ps1` - Ejecutar un test
- `ejecutar_tests_strings.ps1` - Ejecutar todos los tests
- `verificar_compilacion.ps1` - Solo verificar compilación

### Scripts Batch (.bat) - **NUEVOS**:
- `EJECUTAR-TEST.bat` - Ejecutar un test (usa PowerShell con bypass)
- `EJECUTAR-TODOS-TESTS.bat` - Ejecutar todos los tests
- `VERIFICAR-COMPILACION.bat` - Verificar compilación

---

## 🎯 Recomendación

**Usa los scripts `.bat`** - Son más fáciles y no requieren cambiar políticas de PowerShell.

---

**Última actualización:** Diciembre 2025

