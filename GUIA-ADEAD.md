# 🎯 ADead: Python Style → NASM Directo

**Guía Completa de Implementación**  
**Última actualización:** Diciembre 2025  
**Objetivo:** Lenguaje completo con sintaxis Python que genera NASM optimizado

---

## 📊 ESTADO GENERAL

```
Progreso Total: █████████░ 87% → Meta: 100% Producción

✅ COMPLETADO (87%)          🔄 EN PROGRESO           ⏳ PENDIENTE (13%)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Tipos: int, string, array    For con iterables       Floats (FPU)
Variables let/mut            Métodos estáticos       Bool nativo
Arrays (10 métodos)          RAII/Destructores       Diccionarios
Strings (6 métodos)          Herencia básica         Tuples
Funciones ABI-safe                                    Sets
Stdlib (11 funciones)                                 Polimorfismo
Control flow completo                                 Match/Switch
For loops (range)                                     Try/Catch
Break/Continue                                        Módulos/Import
Operadores lógicos                                    Lambdas
Comentarios (#)                                       Generics
Dead Code Elimination                                 Async/Await
Debug Symbols                                         File I/O
Structs/OOP Básico ✅                                 Decoradores
Campos y acceso ✅                                    Type hints
Métodos instancia ✅                                  Comprehensions
Constructores ✅
Métodos con params ✅
```

---

## 🎯 VISIÓN DEL PRODUCTO FINAL

```python
# ADead: Sintaxis Python, Rendimiento ASM

# === TIPOS DE DATOS ===
let entero = 42
let decimal = 3.14159
let texto = "Hola Mundo"
let booleano = true
let nulo = None
let lista = [1, 2, 3, 4, 5]
let tupla = (10, 20, 30)
let diccionario = {"nombre": "ADead", "version": 1}
let conjunto = {1, 2, 3}

# === ARITMÉTICA COMPLETA ===
let suma = 10 + 5
let resta = 10 - 5
let mult = 10 * 5
let div = 10 / 5
let modulo = 10 % 3
let potencia = 2 ** 10
let div_entera = 10 // 3

# Operadores compuestos
x += 5
x -= 3
x *= 2
x /= 4

# Operadores bitwise
let and_bit = a & b
let or_bit = a | b
let xor_bit = a ^ b
let not_bit = ~a
let shift_left = a << 2
let shift_right = a >> 2

# === MATEMÁTICAS AVANZADAS ===
import math

let raiz = math.sqrt(16)
let seno = math.sin(3.14159)
let coseno = math.cos(0)
let logaritmo = math.log(100)
let potencia = math.pow(2, 10)
let absoluto = math.abs(-42)
let redondeo = math.round(3.7)
let piso = math.floor(3.9)
let techo = math.ceil(3.1)

# === OOP COMPLETO ===
class Persona {
    # Constructor
    fn new(nombre, edad) {
        self.nombre = nombre
        self.edad = edad
    }
    
    # Método público
    fn saludar(self) {
        print "Hola, soy " + self.nombre
    }
    
    # Método privado
    fn _validar_edad(self) {
        return self.edad >= 0
    }
    
    # Método estático
    fn static crear_anonimo() {
        return Persona("Anónimo", 0)
    }
}

# Herencia
class Empleado extends Persona {
    fn new(nombre, edad, salario) {
        super.new(nombre, edad)
        self.salario = salario
    }
    
    fn trabajar(self) {
        print self.nombre + " está trabajando"
    }
}

# Interfaces/Traits
trait Serializable {
    fn to_string(self) -> string
    fn from_string(data: string) -> Self
}

class Producto implements Serializable {
    fn new(nombre, precio) {
        self.nombre = nombre
        self.precio = precio
    }
    
    fn to_string(self) -> string {
        return self.nombre + ":" + str(self.precio)
    }
}

# === CONTROL DE FLUJO AVANZADO ===

# Match/Switch
match valor {
    1 => print "uno"
    2 => print "dos"
    3..10 => print "entre 3 y 10"
    _ => print "otro"
}

# For con iterables
for item in lista {
    print item
}

for key, value in diccionario {
    print key + " = " + str(value)
}

for i, item in enumerate(lista) {
    print str(i) + ": " + str(item)
}

# Comprensiones de lista
let cuadrados = [x ** 2 for x in 0..10]
let pares = [x for x in lista if x % 2 == 0]

# === FUNCIONES AVANZADAS ===

# Parámetros por defecto
fn saludar(nombre, saludo = "Hola") {
    print saludo + ", " + nombre
}

# Parámetros nombrados
saludar(nombre = "Mundo", saludo = "Hola")

# Funciones variádicas
fn sumar(*numeros) {
    let total = 0
    for n in numeros {
        total += n
    }
    return total
}

# Lambdas
let doble = |x| x * 2
let suma = |a, b| a + b
lista.map(|x| x * 2)
lista.filter(|x| x > 5)

# Closures
fn crear_contador() {
    let count = 0
    return || {
        count += 1
        return count
    }
}

# Decoradores
@memoize
fn fibonacci(n) {
    if n <= 1 { return n }
    return fibonacci(n - 1) + fibonacci(n - 2)
}

# === MANEJO DE ERRORES ===

# Try/Catch
try {
    let resultado = dividir(10, 0)
} catch DivisionError as e {
    print "Error: " + e.message
} finally {
    print "Limpieza"
}

# Result/Option types
fn dividir(a, b) -> Result<int, Error> {
    if b == 0 {
        return Err("División por cero")
    }
    return Ok(a / b)
}

let resultado = dividir(10, 2)
match resultado {
    Ok(valor) => print valor
    Err(msg) => print "Error: " + msg
}

# Option
fn buscar(lista, valor) -> Option<int> {
    for i, item in enumerate(lista) {
        if item == valor {
            return Some(i)
        }
    }
    return None
}

# === MÓDULOS E IMPORTS ===

# Importar módulo completo
import math
import io
import net

# Importar funciones específicas
from math import sqrt, sin, cos
from io import read_file, write_file

# Alias
import math as m
from math import sqrt as raiz

# Módulos propios
# archivo: utils.ad
module utils {
    fn helper() {
        print "Helper function"
    }
}

# archivo: main.ad
import utils
utils.helper()

# === I/O DE ARCHIVOS ===

# Leer archivo
let contenido = read_file("datos.txt")

# Escribir archivo
write_file("salida.txt", "Contenido")

# Append
append_file("log.txt", "Nueva línea\n")

# Context manager
with open("archivo.txt", "r") as f {
    let lineas = f.readlines()
    for linea in lineas {
        print linea
    }
}

# === ASYNC/AWAIT ===

async fn fetch_data(url) {
    let response = await http.get(url)
    return response.body
}

async fn main() {
    let data = await fetch_data("https://api.example.com")
    print data
}

# Múltiples tareas
let tasks = [
    fetch_data("url1"),
    fetch_data("url2"),
    fetch_data("url3")
]
let results = await async.all(tasks)

# === GENERICS ===

fn max<T: Comparable>(a: T, b: T) -> T {
    if a > b { return a }
    return b
}

class Stack<T> {
    fn new() {
        self.items = []
    }
    
    fn push(self, item: T) {
        self.items.append(item)
    }
    
    fn pop(self) -> Option<T> {
        if len(self.items) == 0 {
            return None
        }
        return Some(self.items.pop())
    }
}

# === TIPOS Y ANOTACIONES ===

# Anotaciones de tipo
let x: int = 42
let s: string = "hola"
let arr: [int] = [1, 2, 3]
let dict: {string: int} = {"a": 1}

# Funciones tipadas
fn sumar(a: int, b: int) -> int {
    return a + b
}

# Type aliases
type Punto = (int, int)
type Callback = fn(int) -> int

let p: Punto = (10, 20)
let cb: Callback = |x| x * 2
```

---

## 📋 ROADMAP DETALLADO

### FASE 1: FUNDAMENTOS ✅ COMPLETADA

| Característica | Estado | Descripción |
|----------------|--------|-------------|
| Variables | ✅ | `let x = 5`, `let mut y = 10` |
| Enteros | ✅ | Literales, aritmética básica |
| Strings | ✅ | Literales, concatenación, métodos |
| Arrays | ✅ | Literales, acceso, 10 métodos |
| Funciones | ✅ | Declaración, llamadas, recursión |
| If/Else | ✅ | Condicionales simples y anidados |
| While | ✅ | Loops básicos |
| For (range) | ✅ | `for i in 0..10` |
| Break/Continue | ✅ | Control de loops |
| Operadores lógicos | ✅ | `&&`, `||`, `!` |
| Comentarios | ✅ | `# comentario` |
| Stdlib básica | ✅ | min, max, abs, pow, etc. |

---

### FASE 2: TIPOS DE DATOS 🔥 PRIORIDAD ALTA

| Característica | Estado | Tiempo Est. | Descripción |
|----------------|--------|-------------|-------------|
| **Floats (FPU)** | ⏳ | 1 semana | Decimales con x87/SSE |
| **Bool nativo** | ⏳ | 2 horas | `true`, `false` como tipos |
| **None/null** | ⏳ | 2 horas | Valor nulo |
| **Tuples** | ⏳ | 1 día | `(a, b, c)` inmutables |
| **Diccionarios** | ⏳ | 3 días | `{"key": value}` hash maps |
| **Sets** | ⏳ | 2 días | `{1, 2, 3}` conjuntos |

#### Implementación Floats (FPU)
```asm
; x87 FPU para floats
fld qword [valor1]      ; Cargar float
fadd qword [valor2]     ; Sumar
fstp qword [resultado]  ; Guardar

; SSE para operaciones vectoriales
movsd xmm0, [valor1]
addsd xmm0, [valor2]
movsd [resultado], xmm0
```

---

### FASE 3: ARITMÉTICA COMPLETA 🔥 PRIORIDAD ALTA

| Característica | Estado | Tiempo Est. | Descripción |
|----------------|--------|-------------|-------------|
| **Potencia** | ⏳ | 2 horas | `2 ** 10` operador |
| **División entera** | ⏳ | 1 hora | `10 // 3` |
| **Operadores compuestos** | ⏳ | 3 horas | `+=`, `-=`, `*=`, `/=` |
| **Bitwise** | ⏳ | 4 horas | `&`, `|`, `^`, `~`, `<<`, `>>` |
| **Negación unaria** | ⏳ | 1 hora | `-x` |
| **Math avanzado** | ⏳ | 1 semana | sqrt, sin, cos, log (FPU) |

#### Implementación Bitwise
```asm
; AND bitwise
mov rax, [a]
and rax, [b]
mov [resultado], rax

; Shift left
mov rax, [valor]
mov cl, [cantidad]
shl rax, cl
```

---

### FASE 4: OOP BÁSICO ✅ COMPLETADA (18 Dic 2025)

| Característica | Estado | Tests | Descripción |
|----------------|--------|-------|-------------|
| **Structs** | ✅ | test_1 | Estructuras de datos con campos |
| **Struct Literals** | ✅ | test_1 | `Punto { x: 10, y: 20 }` |
| **Acceso a Campos** | ✅ | test_1 | `obj.campo` lectura y escritura |
| **Asignación Campos** | ✅ | test_1 | `obj.campo = valor` |
| **Constructores** | ✅ | test_3 | `Struct.new(params)` funcional |
| **Métodos Instancia** | ✅ | test_2 | `obj.metodo()` con self |
| **Métodos con Params** | ✅ | test_4 | Múltiples parámetros |
| **Return Values** | ✅ | test_2,4 | Métodos retornan valores |

#### Estado Actual de OOP (18 Diciembre 2025)

**✅ COMPLETAMENTE IMPLEMENTADO Y VERIFICADO:**
- ✅ Structs con campos múltiples
- ✅ Struct literals: `Punto { x: 10, y: 20 }`
- ✅ Acceso a campos: `obj.campo` (lectura)
- ✅ Asignación a campos: `obj.campo = valor` (escritura)
- ✅ Constructores: `Struct.new(params)` completamente funcional
- ✅ Métodos de instancia: `obj.metodo()` con parámetro `self`
- ✅ Métodos con parámetros: `obj.metodo(a, b, c)`
- ✅ Return values desde métodos
- ✅ Múltiples instancias independientes
- ✅ Layout en memoria correcto (stack con offsets negativos)
- ✅ Windows x64 ABI compliance total

**Tests Verificados:**
- ✅ test_1_struct_basico.ad - Structs y campos (Output: 10, 20, 30, 40)
- ✅ test_2_metodo_simple.ad - Métodos instancia (Output: 78)
- ✅ test_3_constructor_simple.ad - Constructores (Output: 100, 25)
- ✅ test_4_metodo_con_params.ad - Métodos params (Output: 8)

**📋 Ver:** `TEST_OOP/RESUMEN-FINAL-OOP.md` para documentación completa

#### Implementación Clases (Actual)
```asm
; Estructura en memoria (stack-based):
; [rbp - N]    campo1    (offset 0)
; [rbp - N-8]  campo2    (offset 8)
; [rbp - N-16] campo3    (offset 16)
; ...

; Futuro (con heap allocation):
; [+0]  vtable_ptr  (puntero a tabla de métodos)
; [+8]  campo1
; [+16] campo2
; ...

; Constructor (new)
Persona_new:
    push rbp
    mov rbp, rsp
    sub rsp, 32
    
    ; Allocar memoria para objeto
    mov rcx, 24         ; Tamaño del objeto
    call malloc
    
    ; Inicializar vtable
    lea rdx, [Persona_vtable]
    mov [rax], rdx
    
    ; Inicializar campos
    mov rdx, [rbp+16]   ; nombre
    mov [rax+8], rdx
    mov rdx, [rbp+24]   ; edad
    mov [rax+16], rdx
    
    mov rsp, rbp
    pop rbp
    ret
```

---

### FASE 5: OOP AVANZADO 🔥 SIGUIENTE PRIORIDAD

| Característica | Estado | Tiempo Est. | Descripción |
|----------------|--------|-------------|-------------|
| **Métodos estáticos** | 🔄 | 4 horas | `Struct.metodo()` - Infraestructura lista, ajuste parser |
| **Visibilidad** | ⏳ | 1 día | `_privado`, público por defecto |
| **Herencia simple** | ⏳ | 3 días | `class B extends A`, campos heredados |
| **super.metodo()** | ⏳ | 1 día | Llamadas a métodos del padre |
| **Interfaces/Traits** | ⏳ | 4 días | `class X implements Y`, contratos |
| **Polimorfismo** | ⏳ | 2 días | Vtables, dispatch dinámico |
| **Properties** | ⏳ | 2 días | Getters/setters automáticos |
| **Operadores** | ⏳ | 2 días | Sobrecarga de operadores |

---

### FASE 6: CONTROL AVANZADO

| Característica | Estado | Tiempo Est. | Descripción |
|----------------|--------|-------------|-------------|
| **For iterables** | 🔄 | 4 horas | `for item in arr` |
| **Match/Switch** | ⏳ | 1 día | Pattern matching |
| **Ternario** | ⏳ | 2 horas | `x if cond else y` |
| **List comprehension** | ⏳ | 2 días | `[x*2 for x in arr]` |

#### Implementación Match
```asm
; match valor { 1 => ..., 2 => ..., _ => ... }
match_start:
    mov rax, [valor]
    cmp rax, 1
    je .case_1
    cmp rax, 2
    je .case_2
    jmp .default
.case_1:
    ; código caso 1
    jmp .end
.case_2:
    ; código caso 2
    jmp .end
.default:
    ; código default
.end:
```

---

### FASE 7: FUNCIONES AVANZADAS

| Característica | Estado | Tiempo Est. | Descripción |
|----------------|--------|-------------|-------------|
| **Parámetros default** | ⏳ | 3 horas | `fn f(x, y = 10)` |
| **Parámetros nombrados** | ⏳ | 4 horas | `f(y = 5, x = 3)` |
| **Variádicos** | ⏳ | 1 día | `fn f(*args)` |
| **Lambdas** | ⏳ | 2 días | `|x| x * 2` |
| **Closures** | ⏳ | 3 días | Captura de variables |
| **Decoradores** | ⏳ | 2 días | `@decorator` |

---

### FASE 8: MANEJO DE ERRORES

| Característica | Estado | Tiempo Est. | Descripción |
|----------------|--------|-------------|-------------|
| **Try/Catch** | ⏳ | 3 días | Excepciones |
| **Result type** | ⏳ | 2 días | `Ok(v)` / `Err(e)` |
| **Option type** | ⏳ | 1 día | `Some(v)` / `None` |
| **Propagación** | ⏳ | 1 día | `?` operator |
| **Finally** | ⏳ | 1 día | Cleanup garantizado |

---

### FASE 9: MÓDULOS E IMPORTS

| Característica | Estado | Tiempo Est. | Descripción |
|----------------|--------|-------------|-------------|
| **import básico** | ⏳ | 2 días | `import modulo` |
| **from import** | ⏳ | 1 día | `from m import f` |
| **Alias** | ⏳ | 2 horas | `import m as alias` |
| **Namespaces** | ⏳ | 2 días | Aislamiento de nombres |
| **Módulos propios** | ⏳ | 2 días | Definir módulos |

---

### FASE 10: I/O Y SISTEMA

| Característica | Estado | Tiempo Est. | Descripción |
|----------------|--------|-------------|-------------|
| **read_file** | ⏳ | 1 día | Leer archivos |
| **write_file** | ⏳ | 1 día | Escribir archivos |
| **stdin/stdout** | ⏳ | 4 horas | I/O consola |
| **Args CLI** | ⏳ | 2 horas | Argumentos línea comandos |
| **Env vars** | ⏳ | 2 horas | Variables de entorno |
| **Context managers** | ⏳ | 2 días | `with open() as f` |

---

### FASE 11: GENERICS Y TIPOS

| Característica | Estado | Tiempo Est. | Descripción |
|----------------|--------|-------------|-------------|
| **Anotaciones** | ⏳ | 2 días | `let x: int = 5` |
| **Funciones tipadas** | ⏳ | 2 días | `fn f(a: int) -> int` |
| **Generics básicos** | ⏳ | 1 semana | `fn f<T>(x: T)` |
| **Type aliases** | ⏳ | 4 horas | `type Punto = (int, int)` |
| **Constraints** | ⏳ | 3 días | `<T: Comparable>` |

---

### FASE 12: ASYNC/CONCURRENCIA (FUTURO)

| Característica | Estado | Tiempo Est. | Descripción |
|----------------|--------|-------------|-------------|
| **async/await** | ⏳ | 2 semanas | Asincronía |
| **Channels** | ⏳ | 1 semana | Comunicación |
| **Mutex/Lock** | ⏳ | 3 días | Sincronización |
| **Threads** | ⏳ | 1 semana | Multihilo |

---

## 🎯 PLAN DE ACCIÓN INMEDIATO

### Sprint 1: Tipos y Aritmética (2 semanas)
```
Semana 1:
├── [ ] Bool nativo (true/false)
├── [ ] None/null
├── [ ] Operador ** (potencia)
├── [ ] Operador // (división entera)
├── [ ] Operadores compuestos (+=, -=, etc.)
└── [ ] Operadores bitwise (&, |, ^, ~, <<, >>)

Semana 2:
├── [ ] Floats básicos (FPU x87)
├── [ ] Aritmética float (+, -, *, /)
├── [ ] Comparaciones float
├── [ ] Math.sqrt, Math.abs (float)
└── [ ] Tuples básicos
```

### Sprint 2: OOP (2 semanas)
```
Semana 3:
├── [ ] Structs básicos
├── [ ] Campos y acceso (struct.campo)
├── [ ] Constructor (new)
├── [ ] Métodos simples
└── [ ] self/this

Semana 4:
├── [ ] Herencia básica (extends)
├── [ ] super.method()
├── [ ] Métodos estáticos
├── [ ] Visibilidad (_privado)
└── [ ] Polimorfismo básico
```

### Sprint 3: Control y Funciones (2 semanas)
```
Semana 5:
├── [ ] For con iterables
├── [ ] Match/Switch básico
├── [ ] Parámetros default
├── [ ] Parámetros nombrados
└── [ ] Ternario (x if cond else y)

Semana 6:
├── [ ] Lambdas básicas
├── [ ] Closures simples
├── [ ] List comprehension básica
├── [ ] Result type
└── [ ] Option type
```

### Sprint 4: Módulos e I/O (2 semanas)
```
Semana 7:
├── [ ] import básico
├── [ ] from import
├── [ ] Namespaces
├── [ ] Módulos propios
└── [ ] Alias (as)

Semana 8:
├── [ ] read_file / write_file
├── [ ] stdin / stdout mejorado
├── [ ] Args CLI
├── [ ] Env vars
└── [ ] Try/Catch básico
```

---

## 📁 ARCHIVOS CLAVE

```
CORE/rust/crates/
├── adead-parser/src/
│   ├── lib.rs              ← Parser principal
│   ├── lexer.rs            ← Tokenización
│   ├── ast.rs              ← Definiciones AST
│   └── types.rs            ← Sistema de tipos (NUEVO)
├── adead-backend/src/
│   ├── lib.rs              ← Generador NASM principal
│   ├── stdlib.rs           ← Librería estándar
│   ├── oop.rs              ← Generación OOP (NUEVO)
│   ├── fpu.rs              ← Operaciones FPU (NUEVO)
│   ├── dependency_graph.rs ← Dead code elimination
│   └── usage_analyzer.rs   ← Análisis de uso
├── adead-borrow/src/
│   └── lib.rs              ← Borrow checker
└── adead-cli/src/
    └── main.rs             ← CLI
```

---

## 🔧 GUÍA TÉCNICA

### Windows x64 ABI
```asm
; Parámetros: RCX, RDX, R8, R9 (primeros 4)
; Retorno: RAX (int), XMM0 (float)
; Preservar: RBX, RBP, R12-R15, RDI, RSI
; Shadow space: 32 bytes antes de cada call
; Stack: Alineado a 16 bytes
```

### Estructuras de Datos
```asm
; Array (24 bytes)
; [+0]  data     (puntero)
; [+8]  length   (qword)
; [+16] capacity (qword)

; String (32 bytes)
; [+0]  data     (puntero)
; [+8]  length   (qword)
; [+16] capacity (qword)
; [+24] hash     (qword)

; Object (variable)
; [+0]  vtable   (puntero a métodos)
; [+8]  field1
; [+16] field2
; ...
```

### FPU (Floats)
```asm
; x87 Stack-based
fld qword [valor]       ; Push to FPU stack
fadd qword [otro]       ; Add
fstp qword [result]     ; Pop and store

; SSE (preferido)
movsd xmm0, [valor]     ; Load double
addsd xmm0, [otro]      ; Add
movsd [result], xmm0    ; Store
```

---

## ✅ CHECKLIST PARA NUEVAS CARACTERÍSTICAS

```
□ Definir sintaxis (cómo se escribe)
□ Definir AST (representación interna)
□ Agregar al Lexer (tokenización)
□ Agregar al Parser (parsing)
□ Agregar al Backend (generación NASM)
□ Agregar a DependencyGraph (dead code)
□ Agregar a UsageAnalyzer (uso de variables)
□ Agregar al Borrow Checker (ownership)
□ Verificar ABI compliance
□ Agregar debug symbols
□ Escribir tests
□ Documentar
```

---

## 📊 MÉTRICAS DE ÉXITO

| Métrica | Actual | Meta |
|---------|--------|------|
| Características implementadas | 80% | 100% |
| Tests pasando | 100% | 100% |
| Documentación | 60% | 100% |
| Ejemplos reales | 10 | 50+ |
| Benchmark vs C | ~90% | 95%+ |
| Tamaño ejecutable | <50KB | <100KB |
| Tiempo compilación | <1s | <2s |

---

## 🎯 OBJETIVO FINAL

```
                    ADead: El Lenguaje Completo
    ┌─────────────────────────────────────────────────────┐
    │  Sintaxis Python   →   NASM Optimizado   →   .exe  │
    └─────────────────────────────────────────────────────┘
    
    ✓ Sintaxis familiar (Python-like)
    ✓ Tipos: int, float, bool, string, array, dict, class
    ✓ OOP completo: clases, herencia, interfaces
    ✓ Funcional: lambdas, closures, map/filter
    ✓ Errores: try/catch, Result, Option
    ✓ Módulos: import, namespaces
    ✓ I/O: archivos, consola, red
    ✓ Performance: ASM nativo, sin runtime
    ✓ Tamaño: ejecutables pequeños (<100KB)
```

**Resultado:** Un lenguaje de programación completo con la simplicidad de Python y el rendimiento de Assembly, listo para desarrollo real de aplicaciones.

---

## 📅 TIMELINE ESTIMADO

| Fase | Duración | Acumulado |
|------|----------|-----------|
| Tipos y Aritmética | 2 semanas | 2 semanas |
| OOP Básico | 2 semanas | 4 semanas |
| Control y Funciones | 2 semanas | 6 semanas |
| Módulos e I/O | 2 semanas | 8 semanas |
| Generics y Tipos | 2 semanas | 10 semanas |
| Pulido y Docs | 2 semanas | **12 semanas** |

**Tiempo total estimado:** ~3 meses para producto completo

---

**Mantener este documento actualizado con cada avance.**
