# 🧪 Tests de Strings Avanzados - ADead

## 📋 Archivos de Prueba

### Tests Básicos

1. **`test_strings_basico.ad`**
   - Crear string y imprimir
   - Verifica: `let s = "hola"` y `print s`

2. **`test_strings_concat.ad`**
   - Concatenación básica
   - Verifica: `s1 + s2` funciona correctamente

3. **`test_strings_slice.ad`**
   - Slicing básico
   - Verifica: `s[0:4]` funciona correctamente

4. **`test_strings_upper.ad`**
   - Método upper()
   - Verifica: `s.upper()` convierte a mayúsculas

5. **`test_strings_lower.ad`**
   - Método lower()
   - Verifica: `s.lower()` convierte a minúsculas

6. **`test_strings_len.ad`**
   - Longitud de string
   - Verifica: `len(s)` retorna longitud correcta

### Tests Avanzados

7. **`test_strings_completo.ad`**
   - Todas las funcionalidades combinadas
   - Verifica: concat, upper, lower, slice, len

8. **`test_strings_concatenacion_multiple.ad`**
   - Concatenación múltiple
   - Verifica: `s1 + s2 + s3` funciona

9. **`test_strings_slicing_avanzado.ad`**
   - Múltiples slices del mismo string
   - Verifica: slicing múltiple funciona

10. **`test_strings_metodos_combinados.ad`**
    - Métodos combinados
    - Verifica: upper() + lower() + slice

11. **`test_strings_len_completo.ad`**
    - len() con diferentes strings
    - Verifica: len() funciona con diferentes longitudes

12. **`test_strings_operaciones_complejas.ad`**
    - Operaciones complejas encadenadas
    - Verifica: múltiples operaciones funcionan juntas

13. **`test_strings_variables.ad`**
    - Variables de tipo String
    - Verifica: asignar y reutilizar strings

14. **`test_strings_print_expresiones.ad`**
    - Print con expresiones directamente
    - Verifica: `print "hola" + "mundo"` funciona

15. **`test_strings_comparacion.ad`**
    - Comparación con arrays
    - Verifica: len() funciona tanto para arrays como strings

---

## 🚀 Cómo Ejecutar los Tests

### Opción 1: Script Automático (Recomendado)

```powershell
cd Ejemplos-Reales\ejemplos\basicos
.\ejecutar_tests_strings.ps1
```

### Opción 2: Manual

```powershell
# Compilar un test específico
cd CORE\rust\target\release
.\adeadc.exe compile ..\..\..\..\Ejemplos-Reales\ejemplos\basicos\test_strings_basico.ad --backend cpp -o test_strings_basico.asm

# Compilar ASM a EXE
nasm -f win64 test_strings_basico.asm -o test_strings_basico.obj
gcc test_strings_basico.obj -o test_strings_basico.exe

# Ejecutar
.\test_strings_basico.exe
```

---

## ✅ Resultados Esperados

### test_strings_basico.ad
```
hola
```

### test_strings_concat.ad
```
holamundo
```

### test_strings_slice.ad
```
hola
mundo
```

### test_strings_upper.ad
```
hola mundo
HOLA MUNDO
```

### test_strings_lower.ad
```
HOLA MUNDO
hola mundo
```

### test_strings_len.ad
```
hola
4
```

### test_strings_completo.ad
```
Hola
Mundo
Hola Mundo
HOLA MUNDO
hola mundo
HOLA
4
```

---

## 🔧 Requisitos

- **Compilador ADead:** `CORE\rust\target\release\adeadc.exe`
- **NASM:** Para ensamblar código ASM
- **GCC:** Para linkear objetos a ejecutables
- **Windows x64:** Sistema operativo objetivo

---

## 📝 Notas

- Los tests verifican que las funciones helper NASM funcionen correctamente
- Cada test es independiente y puede ejecutarse por separado
- Los tests cubren todas las funcionalidades principales de strings
- Si un test falla, revisar el código ASM generado para debugging

---

**Última actualización:** Diciembre 2025  
**Estado:** Tests listos para ejecutar

