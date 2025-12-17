# 🧪 Tests de ADead

## Archivos de Prueba

| Archivo | Descripción | Estado |
|---------|-------------|--------|
| `01_arrays_basico.ad` | Arrays básicos, append, len | ✅ Funciona |
| `02_arrays_avanzado.ad` | sort, reverse, insert, index, count | ✅ Funciona |
| `03_strings.ad` | Concatenación, upper, lower, slice | ✅ Funciona |
| `04_funciones.ad` | Funciones, recursión | ✅ Funciona |
| `05_control_flujo.ad` | If/else, while | ✅ Funciona |
| `06_stdlib.ad` | min, max, abs, pow, clamp, etc. | ✅ Funciona |
| `07_for_loops.ad` | For loops con rangos | ✅ Funciona |
| `08_break_continue.ad` | Break/Continue en loops | ✅ Funciona |
| `09_logical_operators.ad` | Operadores &&, \|\|, ! | ✅ Funciona |

### Tests Unitarios

| Archivo | Descripción |
|---------|-------------|
| `test_basic.ad` | Print simple |
| `test_for_simple.ad` | For básico |
| `test_for_print.ad` | For con print |
| `test_logical_simple.ad` | Variables e if |
| `test_and.ad` | Operador && |
| `test_or_simple.ad` | Operador \|\| |
| `test_not_simple.ad` | Operador ! |
| `test_not_expr.ad` | NOT con expresiones |
| `test_or_not.ad` | Combinación OR y NOT |

## Cómo Ejecutar

```powershell
# Compilar el proyecto primero
cd CORE\rust
cargo build --release

# Luego compilar un test
cd ..\..\Test
..\CORE\rust\target\release\adeadc.exe build 01_arrays_basico.ad -o test.exe

# Ejecutar
.\test.exe
```

## Sintaxis Soportada

```python
# Variables
let x = 10

# Arrays
let arr = [1, 2, 3]
arr.append(4)
arr.sort()
print arr[0]

# Strings
let s = "hola" + " mundo"
print s.upper()

# Funciones
fn suma(a, b) {
    return a + b
}

# Control de flujo
if x > 5 {
    print 1
}

while x > 0 {
    x = x - 1
}

# For loops
for i in 0..10 {
    print i
}

# Break/Continue
for i in 0..100 {
    if i == 50 {
        break
    }
    if i % 2 == 0 {
        continue
    }
    print i
}

# Operadores lógicos
if x > 0 && x < 10 {
    print "en rango"
}

if !error || success {
    print "ok"
}
```

## Próximo Objetivo

```python
# For con iterables (PENDIENTE)
for item in arr {
    print item
}

# Módulos (PENDIENTE)
import math
print math.sqrt(16)
```
