# 🐍 Roadmap: ADead → Python-Like

**Objetivo:** Hacer ADead lo más similar posible a Python manteniendo rendimiento nativo

---

## 🎯 Características Python que ADead DEBE tener

### 1. ✅ Sintaxis Básica (YA IMPLEMENTADO)

```python
# Variables
let x = 10
let mut y = 20

# Tipos básicos
let entero = 42
let texto = "Hola"
let lista = [1, 2, 3]

# Control de flujo
if x > 5 {
    print "mayor"
}

for i in 0..10 {
    print i
}

while x < 100 {
    x += 1
}

# Funciones
fn suma(a, b) {
    return a + b
}

# Comentarios
# Esto es un comentario
```

---

### 2. 🔥 Sintaxis Python-Style (ALTA PRIORIDAD)

#### 2.1 Indentación Opcional (Mantener llaves)
```python
# ADead actual (con llaves)
if x > 5 {
    print "mayor"
}

# Futuro: Permitir ambos estilos
if x > 5:
    print "mayor"
```

#### 2.2 Operadores Python
```python
# Potencia
x = 2 ** 10  # ⏳ PENDIENTE

# División entera
x = 10 // 3  # ⏳ PENDIENTE

# Operadores compuestos
x += 5   # ⏳ PENDIENTE
x -= 3   # ⏳ PENDIENTE
x *= 2   # ⏳ PENDIENTE
x /= 4   # ⏳ PENDIENTE

# Operadores lógicos (ya funcionan)
if x > 5 && y < 10 {  # ✅
    print "ok"
}

# Futuro: Sintaxis Python alternativa
if x > 5 and y < 10:  # ⏳ PENDIENTE
    print "ok"
```

#### 2.3 String Formatting
```python
# Actual
print "Hola " + nombre

# Futuro: f-strings
print f"Hola {nombre}, tienes {edad} años"  # ⏳ PENDIENTE

# Futuro: format()
print "Hola {}, tienes {} años".format(nombre, edad)  # ⏳ PENDIENTE
```

#### 2.4 Múltiple Asignación
```python
# Tuple unpacking
let a, b = 10, 20  # ⏳ PENDIENTE
let x, y, z = punto.coords()  # ⏳ PENDIENTE

# Swap
a, b = b, a  # ⏳ PENDIENTE
```

---

### 3. 🔥 Tipos de Datos Python (ALTA PRIORIDAD)

#### 3.1 Bool Nativo
```python
let verdadero = True   # ⏳ PENDIENTE (actualmente: true)
let falso = False      # ⏳ PENDIENTE (actualmente: false)

# Valores truthy/falsy
if lista {  # Lista vacía = False
    print "tiene elementos"
}

if texto {  # String vacío = False
    print "tiene texto"
}
```

#### 3.2 None
```python
let valor = None  # ⏳ PENDIENTE

fn buscar(lista, item) {
    for i, x in enumerate(lista) {
        if x == item {
            return i
        }
    }
    return None
}
```

#### 3.3 Diccionarios
```python
# Crear diccionario
let persona = {
    "nombre": "Juan",
    "edad": 25,
    "ciudad": "Lima"
}  # ⏳ PENDIENTE

# Acceso
print persona["nombre"]
persona["edad"] = 26

# Métodos
persona.keys()
persona.values()
persona.items()
persona.get("nombre", "default")
```

#### 3.4 Tuples
```python
# Crear tuple
let punto = (10, 20)  # ⏳ PENDIENTE
let rgb = (255, 128, 0)

# Acceso
let x = punto[0]
let y = punto[1]

# Inmutables
# punto[0] = 15  # ERROR
```

#### 3.5 Sets
```python
# Crear set
let numeros = {1, 2, 3, 4, 5}  # ⏳ PENDIENTE

# Operaciones
numeros.add(6)
numeros.remove(3)
numeros.contains(4)

# Operaciones de conjuntos
let a = {1, 2, 3}
let b = {3, 4, 5}
let union = a | b
let interseccion = a & b
let diferencia = a - b
```

---

### 4. 🔥 OOP Python-Style (PARCIALMENTE IMPLEMENTADO)

#### 4.1 Clases (Mejorar sintaxis)
```python
# Actual (funciona)
struct Persona {
    nombre
    edad
}

fn Persona_new(self, nombre, edad) {
    self.nombre = nombre
    self.edad = edad
}

# Futuro: Sintaxis más Python
class Persona:
    def __init__(self, nombre, edad):
        self.nombre = nombre
        self.edad = edad
    
    def saludar(self):
        print f"Hola, soy {self.nombre}"
```

#### 4.2 Herencia
```python
class Empleado extends Persona:  # ⏳ PENDIENTE
    def __init__(self, nombre, edad, salario):
        super().__init__(nombre, edad)
        self.salario = salario
    
    def trabajar(self):
        print f"{self.nombre} está trabajando"
```

#### 4.3 Properties
```python
class Circulo:
    def __init__(self, radio):
        self._radio = radio
    
    @property
    def area(self):
        return 3.14159 * self._radio ** 2
    
    @property
    def radio(self):
        return self._radio
    
    @radio.setter
    def radio(self, valor):
        if valor > 0:
            self._radio = valor

# Uso
c = Circulo(5)
print c.area  # Llamada como propiedad, no método
c.radio = 10  # Setter
```

#### 4.4 Métodos Especiales
```python
class Vector:
    def __init__(self, x, y):
        self.x = x
        self.y = y
    
    def __add__(self, other):  # Sobrecarga +
        return Vector(self.x + other.x, self.y + other.y)
    
    def __str__(self):  # Conversión a string
        return f"Vector({self.x}, {self.y})"
    
    def __len__(self):  # len(vector)
        return 2
    
    def __getitem__(self, index):  # vector[0]
        if index == 0: return self.x
        if index == 1: return self.y

# Uso
v1 = Vector(1, 2)
v2 = Vector(3, 4)
v3 = v1 + v2  # Usa __add__
print v3  # Usa __str__
```

---

### 5. 🔥 Funciones Avanzadas (PENDIENTE)

#### 5.1 Parámetros por Defecto
```python
def saludar(nombre, saludo="Hola"):  # ⏳ PENDIENTE
    print f"{saludo}, {nombre}"

saludar("Mundo")  # "Hola, Mundo"
saludar("Mundo", "Buenos días")  # "Buenos días, Mundo"
```

#### 5.2 Parámetros Nombrados
```python
def crear_persona(nombre, edad, ciudad="Lima"):
    return Persona(nombre, edad, ciudad)

# Llamada con nombres
p = crear_persona(nombre="Juan", edad=25)  # ⏳ PENDIENTE
p = crear_persona(edad=30, nombre="María")  # Orden diferente
```

#### 5.3 *args y **kwargs
```python
def sumar(*numeros):  # ⏳ PENDIENTE
    total = 0
    for n in numeros:
        total += n
    return total

print sumar(1, 2, 3, 4, 5)  # 15

def configurar(**opciones):
    for key, value in opciones.items():
        print f"{key} = {value}"

configurar(debug=True, timeout=30)
```

#### 5.4 Lambdas
```python
# Lambda simple
doble = lambda x: x * 2  # ⏳ PENDIENTE
suma = lambda a, b: a + b

# Con map/filter
numeros = [1, 2, 3, 4, 5]
dobles = list(map(lambda x: x * 2, numeros))
pares = list(filter(lambda x: x % 2 == 0, numeros))
```

#### 5.5 Decoradores
```python
@memoize  # ⏳ PENDIENTE
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

@timer
def operacion_lenta():
    # código lento
    pass

# Decorador con parámetros
@retry(max_attempts=3)
def api_call():
    # llamada a API
    pass
```

---

### 6. 🔥 Comprehensions (PENDIENTE)

#### 6.1 List Comprehension
```python
# Básica
cuadrados = [x**2 for x in range(10)]  # ⏳ PENDIENTE

# Con condición
pares = [x for x in range(20) if x % 2 == 0]

# Anidada
matriz = [[i*j for j in range(5)] for i in range(5)]
```

#### 6.2 Dict Comprehension
```python
# Crear diccionario
cuadrados_dict = {x: x**2 for x in range(10)}  # ⏳ PENDIENTE

# Con condición
pares_dict = {x: x**2 for x in range(20) if x % 2 == 0}
```

#### 6.3 Set Comprehension
```python
# Crear set
numeros_unicos = {x % 10 for x in range(100)}  # ⏳ PENDIENTE
```

---

### 7. 🔥 Control de Flujo Avanzado (PENDIENTE)

#### 7.1 Match/Switch
```python
match valor:  # ⏳ PENDIENTE
    case 1:
        print "uno"
    case 2:
        print "dos"
    case 3..10:
        print "entre 3 y 10"
    case _:
        print "otro"
```

#### 7.2 For con Iterables
```python
# Iterar lista
for item in lista:  # 🔄 EN PROGRESO
    print item

# Iterar diccionario
for key, value in diccionario.items():  # ⏳ PENDIENTE
    print f"{key} = {value}"

# Enumerate
for i, item in enumerate(lista):  # ⏳ PENDIENTE
    print f"{i}: {item}"

# Zip
for a, b in zip(lista1, lista2):  # ⏳ PENDIENTE
    print f"{a} - {b}"

# Range con step
for i in range(0, 10, 2):  # ⏳ PENDIENTE
    print i  # 0, 2, 4, 6, 8
```

#### 7.3 Operador Ternario
```python
# Python style
resultado = "par" if x % 2 == 0 else "impar"  # ⏳ PENDIENTE

# Uso en expresiones
max_val = a if a > b else b
```

#### 7.4 With Statement
```python
with open("archivo.txt", "r") as f:  # ⏳ PENDIENTE
    contenido = f.read()
    print contenido
# Archivo se cierra automáticamente
```

---

### 8. 🔥 Manejo de Errores Python-Style (PENDIENTE)

#### 8.1 Try/Except
```python
try:  # ⏳ PENDIENTE
    resultado = 10 / 0
except ZeroDivisionError as e:
    print f"Error: {e}"
except Exception as e:
    print f"Error general: {e}"
finally:
    print "Limpieza"
```

#### 8.2 Raise
```python
def dividir(a, b):
    if b == 0:
        raise ValueError("No se puede dividir por cero")  # ⏳ PENDIENTE
    return a / b
```

#### 8.3 Assert
```python
assert x > 0, "x debe ser positivo"  # ⏳ PENDIENTE
assert len(lista) > 0, "Lista vacía"
```

---

### 9. 🔥 Módulos e Imports Python-Style (PENDIENTE)

#### 9.1 Import Básico
```python
import math  # ⏳ PENDIENTE
import os
import sys

x = math.sqrt(16)
```

#### 9.2 From Import
```python
from math import sqrt, sin, cos  # ⏳ PENDIENTE
from os import path

x = sqrt(16)  # Sin prefijo
```

#### 9.3 Import As
```python
import math as m  # ⏳ PENDIENTE
from math import sqrt as raiz

x = m.sqrt(16)
y = raiz(25)
```

#### 9.4 Import All
```python
from math import *  # ⏳ PENDIENTE (no recomendado)
```

---

### 10. 🔥 Built-in Functions Python (PENDIENTE)

```python
# Funciones que ADead DEBE tener

# Tipos
int(x)      # ⏳ PENDIENTE
float(x)    # ⏳ PENDIENTE
str(x)      # ⏳ PENDIENTE
bool(x)     # ⏳ PENDIENTE
list(x)     # ⏳ PENDIENTE
dict(x)     # ⏳ PENDIENTE
tuple(x)    # ⏳ PENDIENTE
set(x)      # ⏳ PENDIENTE

# Matemáticas
abs(x)      # ✅ YA EXISTE
min(a, b)   # ✅ YA EXISTE
max(a, b)   # ✅ YA EXISTE
pow(x, y)   # ✅ YA EXISTE
round(x)    # ⏳ PENDIENTE
sum(lista)  # ⏳ PENDIENTE

# Secuencias
len(x)      # ✅ YA EXISTE
range(n)    # ✅ YA EXISTE
enumerate(x) # ⏳ PENDIENTE
zip(a, b)   # ⏳ PENDIENTE
sorted(x)   # ⏳ PENDIENTE
reversed(x) # ⏳ PENDIENTE

# Funcionales
map(f, x)    # ⏳ PENDIENTE
filter(f, x) # ⏳ PENDIENTE
reduce(f, x) # ⏳ PENDIENTE
all(x)       # ⏳ PENDIENTE
any(x)       # ⏳ PENDIENTE

# I/O
print(x)     # ✅ YA EXISTE
input(msg)   # ⏳ PENDIENTE
open(file)   # ⏳ PENDIENTE

# Utilidades
type(x)      # ⏳ PENDIENTE
isinstance(x, T) # ⏳ PENDIENTE
hasattr(x, a)    # ⏳ PENDIENTE
getattr(x, a)    # ⏳ PENDIENTE
setattr(x, a, v) # ⏳ PENDIENTE
```

---

## 📋 Plan de Implementación Priorizado

### Sprint 1: Sintaxis Python-Like (1 semana)
```
□ Operador ** (potencia)
□ Operador // (división entera)
□ Operadores compuestos (+=, -=, *=, /=)
□ True/False (bool nativo)
□ None (valor nulo)
□ Operadores lógicos alternativos (and, or, not)
```

### Sprint 2: Tipos Python (2 semanas)
```
□ Diccionarios básicos
□ Tuples básicos
□ Sets básicos
□ Type conversions (int(), str(), etc.)
□ Truthy/Falsy values
```

### Sprint 3: Funciones Avanzadas (2 semanas)
```
□ Parámetros por defecto
□ Parámetros nombrados
□ *args (variádicos)
□ Lambdas básicas
□ Decoradores básicos
```

### Sprint 4: Comprehensions (1 semana)
```
□ List comprehension
□ Dict comprehension
□ Set comprehension
□ Generator expressions
```

### Sprint 5: Control Avanzado (1 semana)
```
□ Match/Switch
□ Operador ternario
□ For con enumerate/zip
□ With statement
```

### Sprint 6: Manejo de Errores (1 semana)
```
□ Try/Except/Finally
□ Raise
□ Assert
□ Custom exceptions
```

### Sprint 7: Módulos (1 semana)
```
□ Import básico
□ From import
□ Import as
□ Namespaces
```

### Sprint 8: Built-ins (1 semana)
```
□ Type conversions
□ Funciones funcionales (map, filter, reduce)
□ Enumerate, zip, sorted
□ Input/Output mejorado
```

---

## 🎯 Diferencias Aceptables con Python

ADead NO necesita ser 100% idéntico a Python. Diferencias aceptables:

### ✅ Mantener en ADead:
- **Llaves `{}`** - Más claro que indentación
- **`let` keyword** - Declaración explícita de variables
- **`fn` keyword** - Declaración explícita de funciones
- **Tipos estáticos opcionales** - Mejor rendimiento
- **Sin GIL** - Mejor concurrencia
- **Sin runtime** - Ejecutables standalone

### ❌ NO implementar de Python:
- **GC (Garbage Collector)** - Usar RAII/ownership
- **Dynamic typing total** - Usar inferencia + anotaciones
- **Metaclasses** - Demasiado complejo
- **Multiple inheritance** - Usar interfaces/traits
- **Global Interpreter Lock** - No necesario

---

## 📊 Comparación Python vs ADead

| Característica | Python | ADead Actual | ADead Meta |
|----------------|--------|--------------|------------|
| Sintaxis | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Tipos de datos | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| OOP | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Funciones | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Módulos | ⭐⭐⭐⭐⭐ | ⭐ | ⭐⭐⭐⭐⭐ |
| Performance | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Tamaño binario | ⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Startup time | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |

---

**Objetivo Final:** ADead con 95% de la sintaxis de Python y 500% del rendimiento.

**Desarrollado por:** Eddi Andreé Salazar Matos  
**Fecha:** 18 de Diciembre 2025
