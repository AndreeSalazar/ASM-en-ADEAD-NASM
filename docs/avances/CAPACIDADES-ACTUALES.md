# 🚀 Capacidades Actuales de ADead

Documentación completa de todas las features implementadas y funcionalidades disponibles.

## ✅ Features Implementadas (100% Funcionales)

### 1. Sistema de Tipos

**Tipos primitivos disponibles:**
- ✅ `int8`, `int16`, `int32`, `int64` - Enteros con signo
- ✅ `uint8`, `uint16`, `uint32`, `uint64` - Enteros sin signo
- ⏳ `float32`, `float64` - Punto flotante (en roadmap)
- ⏳ `bool` - Booleano (en roadmap)
- ⏳ `char` - Carácter (en roadmap)
- ✅ `string` - Strings básicos
- ✅ `array<T>` - Arrays de cualquier tipo
- ✅ `Option<T>` - Valores opcionales
- ✅ `Result<T, E>` - Manejo de errores

**Tipos compuestos:**
- ✅ `Struct` - Estructuras personalizadas
- ✅ `Tuple` - Tuplas (en AST, pendiente implementación completa)

---

### 2. Variables y Asignación

**Funcionalidades:**
- ✅ Declaración: `let x = 10`
- ✅ Variables mutables: `let mut x = 10`
- ✅ Asignación: `x = 20`
- ✅ Type inference automático
- ✅ Scope local y global

**Ejemplo:**
```adead
let x = 10
let mut y = 20
y = 30
```

---

### 3. Operadores

**Aritméticos:**
- ✅ `+` - Suma
- ✅ `-` - Resta
- ✅ `*` - Multiplicación
- ✅ `/` - División

**Comparación:**
- ✅ `==` - Igualdad
- ✅ `!=` - Desigualdad
- ✅ `<` - Menor que
- ✅ `<=` - Menor o igual
- ✅ `>` - Mayor que
- ✅ `>=` - Mayor o igual

**Lógicos:**
- ⏳ `&&` - AND (pendiente)
- ⏳ `||` - OR (pendiente)
- ⏳ `!` - NOT (pendiente)

---

### 4. Control de Flujo

**Condicionales:**
- ✅ `if` / `else` - Condicionales completos
- ✅ Expresiones booleanas en condiciones

**Loops:**
- ✅ `while` - Loops condicionales
- ⏳ `for` - Loops iterativos (pendiente)
- ⏳ `break` / `continue` (pendiente)

**Ejemplo:**
```adead
if x > 5 {
    print "x es mayor que 5"
} else {
    print "x es menor o igual a 5"
}

while i < 10 {
    i = i + 1
}
```

---

### 5. Funciones

**Funcionalidades:**
- ✅ Definición: `fn nombre(params) { ... }`
- ✅ Parámetros múltiples
- ✅ Return values: `return valor`
- ✅ Recursión

**Ejemplo:**
```adead
fn factorial(n: int64) -> int64 {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}
```

---

### 6. Arrays

**Funcionalidades:**
- ✅ Literales: `[1, 2, 3]`
- ✅ Indexación: `arr[0]`
- ✅ Indexación anidada: `matriz[i][j]`
- ✅ Arrays vacíos: `[]`
- ✅ Stack-allocated (tamaño fijo)
- ⏳ Heap-allocated (dinámico, pendiente)
- ⏳ Funciones de array (len, push, pop - pendiente)

**Ejemplo:**
```adead
let numeros = [10, 20, 30]
let primero = numeros[0]
let segundo = numeros[1]
```

---

### 7. Structs y OOP

**Funcionalidades:**
- ✅ Definición de structs
- ✅ Campos públicos y privados (`pub` / `private`)
- ✅ Constructores: `init()`
- ✅ Destructores: `destroy()`
- ✅ Métodos: `impl Struct { fn metodo() {} }`
- ✅ RAII (Resource Acquisition Is Initialization)
- ✅ Encapsulación completa

**Ejemplo:**
```adead
struct Persona {
    pub nombre: string
    edad: int64
}

impl Persona {
    pub init(nombre: string, edad: int64) {
        self.nombre = nombre
        self.edad = edad
    }
    
    destroy() {
        print "Liberando recursos"
    }
}
```

---

### 8. Manejo de Errores

**Option:**
- ✅ `Some(valor)` - Valor presente
- ✅ `None` - Sin valor
- ✅ Pattern matching con `match`

**Result:**
- ✅ `Ok(valor)` - Éxito
- ✅ `Err(error)` - Error
- ✅ Pattern matching con `match`
- ✅ Operador `?` - Propagación automática de errores

**Errores estándar:**
- ✅ `FileError` - Errores de archivos
- ✅ `ParseError` - Errores de parsing
- ✅ `MathError` - Errores matemáticos
- ✅ `ValueError` - Errores de valores
- ✅ `IOError` - Errores de I/O

**Ejemplo:**
```adead
let resultado: Result<int64, FileError> = leer_archivo("data.txt")?

match resultado {
    Ok(valor) => print "Éxito: " + valor
    Err(error) => print "Error: " + error.message
}
```

---

### 9. Ownership y Borrowing

**Funcionalidades:**
- ✅ Borrow checker (borrowing analyzer)
- ✅ Referencias: `&T` y `&mut T`
- ✅ Dereferenciación: `*expr`
- ✅ Tracking de ownership
- ✅ Detección de use-after-move
- ✅ Verificación de mutabilidad

**Ejemplo:**
```adead
let x = 10
let y = &x  // Referencia inmutable
let z = &mut x  // Referencia mutable (si x es mut)
```

---

### 10. I/O Básico

**Funcionalidades:**
- ✅ `print "texto"` - Imprimir strings
- ⏳ `print numero` - Imprimir números (pendiente)
- ⏳ Input del usuario (pendiente)
- ⏳ File I/O (pendiente)

---

### 11. Compilación

**Backend:**
- ✅ Generación de código NASM para Windows x64
- ✅ Generación de código NASM para Linux x64
- ✅ Calling conventions correctas
- ✅ Stack frame management
- ✅ Tagged unions para Option/Result

**CLI:**
- ✅ `compile` - Compilar .ad a .asm
- ✅ `assemble` - Ensamblar .asm a .obj/.o
- ✅ `link` - Enlazar .obj/.o a .exe/ejecutable
- ✅ `run` - Compilar, ensamblar, enlazar y ejecutar

---

## ⏳ Features en Desarrollo

### Sprint 1 (89% completo)
- ✅ Manejo de errores - 100%
- ✅ Arrays básicos - 100%
- ⏳ Import básico - 0% (pendiente)

### Próximos Sprints
- ⏳ Strings completos (parsing, búsqueda, manipulación)
- ⏳ Tipos float32/float64
- ⏳ Bool como tipo nativo
- ⏳ Funciones matemáticas (std.math)
- ⏳ Funciones de array (std.array)
- ⏳ Sistema de módulos completo

---

## 📊 Métricas de Implementación

| Categoría | Completado | Pendiente | % |
|-----------|-----------|-----------|---|
| Tipos básicos | 8/12 | 4 | 67% |
| Control de flujo | 2/4 | 2 | 50% |
| Funciones | 1/1 | 0 | 100% |
| Arrays | 1/2 | 1 | 50% |
| OOP | 1/1 | 0 | 100% |
| Errores | 1/1 | 0 | 100% |
| Ownership | 1/1 | 0 | 100% |
| I/O | 1/4 | 3 | 25% |
| Módulos | 0/1 | 1 | 0% |

**Total aproximado: 65% del MVP completo**

---

## 🎯 Ejemplos Funcionales

### Hello World
```adead
print "Hola Mundo"
```

### Factorial Recursivo
```adead
fn factorial(n: int64) -> int64 {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}

let resultado = factorial(5)
```

### Arrays y Errores
```adead
let numeros = [1, 2, 3, 4, 5]
let primero = numeros[0]

let resultado: Result<int64, MathError> = dividir(10, 2)?
match resultado {
    Ok(valor) => print "Éxito"
    Err(error) => print "Error"
}
```

---

**Última actualización:** Diciembre 2025

