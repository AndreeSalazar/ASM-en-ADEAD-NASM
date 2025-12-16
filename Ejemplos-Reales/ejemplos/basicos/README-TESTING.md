# 🧪 Guía de Testing - Strings Avanzados

## 📋 Archivos de Prueba Creados

### Tests Básicos (15 tests)
1. `test_strings_basico.ad` - Crear y imprimir string
2. `test_strings_concat.ad` - Concatenación básica
3. `test_strings_slice.ad` - Slicing básico
4. `test_strings_upper.ad` - Método upper()
5. `test_strings_lower.ad` - Método lower()
6. `test_strings_len.ad` - Longitud len()
7. `test_strings_completo.ad` - Todas las funcionalidades
8. `test_strings_concatenacion_multiple.ad` - Concatenación múltiple
9. `test_strings_slicing_avanzado.ad` - Slicing avanzado
10. `test_strings_metodos_combinados.ad` - Métodos combinados
11. `test_strings_len_completo.ad` - len() completo
12. `test_strings_operaciones_complejas.ad` - Operaciones complejas
13. `test_strings_variables.ad` - Variables de tipo String
14. `test_strings_print_expresiones.ad` - Print con expresiones
15. `test_strings_comparacion.ad` - Comparación con arrays

### Tests Avanzados (10 tests adicionales)
16. `test_strings_inmutabilidad.ad` - Verificar inmutabilidad
17. `test_strings_edge_cases.ad` - Casos límite
18. `test_strings_while_loop.ad` - Strings en loops
19. `test_strings_if_condition.ad` - Strings en condiciones
20. `test_strings_anidados.ad` - Operaciones anidadas
21. `test_strings_vs_arrays.ad` - Comparación strings vs arrays
22. `test_strings_print_multiple.ad` - Print múltiple
23. `test_strings_concatenacion_literales.ad` - Concatenación con literales
24. `test_strings_slice_completo.ad` - Slicing completo
25. `test_strings_metodos_encadenados.ad` - Métodos encadenados
26. `test_strings_complejidad.ad` - Complejidad máxima
27. `test_strings_ascii.ad` - Conversión ASCII
28. `test_strings_numeros.ad` - Strings con números
29. `test_strings_especiales.ad` - Caracteres especiales
30. `test_strings_performance.ad` - Test de performance

**Total: 30 archivos de prueba**

---

## 🚀 Scripts de Ejecución

### 1. `ejecutar_tests_strings.ps1`
Ejecuta todos los tests automáticamente y muestra resultados.

**Uso:**
```powershell
.\ejecutar_tests_strings.ps1
```

**Características:**
- Compila todos los tests
- Ensambla y linkea (si NASM/GCC están disponibles)
- Ejecuta cada test
- Muestra resumen final

### 2. `ejecutar_test_individual.ps1`
Ejecuta un test individual específico.

**Uso:**
```powershell
.\ejecutar_test_individual.ps1 test_strings_basico.ad
```

**Características:**
- Compila un test específico
- Ensambla y linkea
- Ejecuta y muestra output

### 3. `verificar_compilacion.ps1`
Solo verifica que todos los tests compilen correctamente (no ejecuta).

**Uso:**
```powershell
.\verificar_compilacion.ps1
```

**Características:**
- Verifica compilación de todos los tests
- No requiere NASM/GCC
- Rápido para verificar sintaxis

---

## 📊 Categorías de Tests

### ✅ Funcionalidad Básica
- Crear strings
- Imprimir strings
- Variables de tipo String

### ✅ Operaciones
- Concatenación (`s1 + s2`)
- Slicing (`s[0:4]`)
- Longitud (`len(s)`)

### ✅ Métodos
- `s.upper()` - Mayúsculas
- `s.lower()` - Minúsculas

### ✅ Casos Especiales
- Strings vacíos
- Slices al límite
- Caracteres especiales
- Operaciones anidadas

### ✅ Integración
- Strings en loops
- Strings en condiciones
- Comparación con arrays
- Print con expresiones

---

## 🔧 Requisitos para Ejecutar Tests

### Mínimos (Solo Compilación)
- ✅ Compilador ADead: `CORE\rust\target\release\adeadc.exe`
- ✅ Rust instalado (para compilar el compilador)

### Completos (Compilación + Ejecución)
- ✅ Compilador ADead
- ✅ NASM (para ensamblar ASM)
- ✅ GCC (para linkear objetos)
- ✅ Windows x64

---

## 📝 Cómo Compilar el Compilador

Si el compilador no existe:

```powershell
cd CORE\rust
cargo build --release
```

Esto generará: `target\release\adeadc.exe`

---

## 🎯 Resultados Esperados

Cada test tiene un resultado esperado específico. Ver `README-STRINGS.md` para detalles.

**Ejemplo:**
- `test_strings_basico.ad` → Imprime: `hola`
- `test_strings_concat.ad` → Imprime: `holamundo`
- `test_strings_slice.ad` → Imprime: `hola` y `mundo`

---

## 🐛 Debugging

Si un test falla:

1. **Verificar compilación:**
   ```powershell
   .\verificar_compilacion.ps1
   ```

2. **Compilar manualmente:**
   ```powershell
   .\ejecutar_test_individual.ps1 test_strings_basico.ad
   ```

3. **Revisar código ASM generado:**
   - Abrir `test_strings_basico.asm`
   - Verificar que las funciones helper estén presentes
   - Verificar que las llamadas sean correctas

4. **Revisar errores de compilación:**
   - Ejecutar compilador con verbose
   - Revisar mensajes de error

---

## 📈 Cobertura de Tests

### Funcionalidades Cubiertas:
- ✅ Creación de strings (`let s = "hola"`)
- ✅ Concatenación (`s1 + s2`)
- ✅ Slicing (`s[0:4]`)
- ✅ Métodos (`s.upper()`, `s.lower()`)
- ✅ Longitud (`len(s)`)
- ✅ Variables de tipo String
- ✅ Print con strings
- ✅ Operaciones anidadas
- ✅ Casos límite
- ✅ Integración con loops y condiciones

### Funcionalidades NO Cubiertas (Futuro):
- ⚠️ Conversión número a string (runtime)
- ⚠️ Interpolación de strings
- ⚠️ Escape sequences avanzadas
- ⚠️ Unicode/UTF-8 completo

---

**Última actualización:** Diciembre 2025  
**Total de tests:** 30 archivos  
**Estado:** Listos para ejecutar

