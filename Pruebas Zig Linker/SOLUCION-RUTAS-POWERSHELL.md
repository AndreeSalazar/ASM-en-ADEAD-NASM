# Solución: Error de rutas en PowerShell

## Problema

El error ocurría porque el script intentaba ejecutar `.\$exe_file` cuando `$exe_file` ya contenía una ruta absoluta, generando rutas inválidas como `.\C:\Users\...`.

**Error:**
```
El término '.\C:\Users\andre\OneDrive\Documentos\ASM-en-ADEAD-NASM\Pruebas Zig Linker\test_simple.exe' no se reconoce
```

## Solución

### 1. Normalizar rutas con `Resolve-Path`
Usar `Resolve-Path` para obtener la ruta absoluta correcta:

```powershell
$exe_path = Resolve-Path $exe_file -ErrorAction SilentlyContinue
if (-not $exe_path) {
    $exe_path = $exe_file
}
```

### 2. Ejecutar con manejo de errores
Usar `try-catch` para manejar errores de ejecución:

```powershell
try {
    $exec_output = & $exe_path 2>&1
    $exec_exit = $LASTEXITCODE
} catch {
    Write-Host "ERROR al ejecutar: $_" -ForegroundColor Red
    exit 1
}
```

### 3. Verificar existencia antes de ejecutar
Verificar que el archivo existe antes de intentar ejecutarlo:

```powershell
if (-not (Test-Path $exe_file)) {
    Write-Host "ERROR: Ejecutable no encontrado: $exe_file" -ForegroundColor Red
    exit 1
}
```

### 4. Búsqueda alternativa de ejecutable
Si no se encuentra en la ubicación esperada, buscar en el directorio del test:

```powershell
$test_dir = Split-Path -Parent $test_file
$exe_in_dir = Join-Path $test_dir (Split-Path -Leaf $exe_file)
if (Test-Path $exe_in_dir) {
    $exe_file = $exe_in_dir
}
```

## Cambios Aplicados

**Antes:**
```powershell
$exec_output = & ".\$exe_file" 2>&1  # ❌ Genera .\C:\Users\...
```

**Después:**
```powershell
$exe_path = Resolve-Path $exe_file -ErrorAction SilentlyContinue
if (-not $exe_path) {
    $exe_path = $exe_file
}
try {
    $exec_output = & $exe_path 2>&1  # ✅ Ruta correcta
    $exec_exit = $LASTEXITCODE
} catch {
    Write-Host "ERROR al ejecutar: $_" -ForegroundColor Red
    exit 1
}
```

## Mejoras Adicionales

1. ✅ Verificación de existencia del ejecutable
2. ✅ Normalización de rutas con `Resolve-Path`
3. ✅ Manejo de errores con `try-catch`
4. ✅ Búsqueda alternativa si no se encuentra en ubicación esperada
5. ✅ Mensajes de error más descriptivos

## Estado

✅ **Corregido** - Los scripts ahora manejan rutas correctamente y son más robustos.

---

**Fecha:** Diciembre 2025

