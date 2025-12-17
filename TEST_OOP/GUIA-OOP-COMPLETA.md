# 🏗️ Guía Completa de OOP en ADead

## De Básico a Avanzado: Todo sobre Programación Orientada a Objetos

**Versión:** 1.0  
**Última actualización:** Diciembre 2025  
**Autor:** Eddi Andreé Salazar Matos

---

## 📑 Tabla de Contenidos

1. [Introducción a OOP](#-1-introducción-a-oop)
2. [Structs Básicos](#-2-structs-básicos)
3. [Acceso a Campos](#-3-acceso-a-campos)
4. [Constructores y Factories](#-4-constructores-y-factories)
5. [Métodos (Funciones Asociadas)](#-5-métodos-funciones-asociadas)
6. [RAII - Gestión de Recursos](#-6-raii---gestión-de-recursos)
7. [Patrones OOP Avanzados](#-7-patrones-oop-avanzados)
8. [Generación NASM](#-8-generación-nasm)
9. [Roadmap Futuro](#-9-roadmap-futuro)

---

## 📖 1. Introducción a OOP

### ¿Qué es OOP en ADead?

ADead implementa OOP de forma **eficiente y cercana al metal**:

```
┌─────────────────────────────────────────────────────────────┐
│                  OOP EN ADEAD                               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   Código ADead          Código NASM                         │
│   ┌─────────────┐       ┌─────────────────────────────────┐ │
│   │ struct P {  │  →    │ ; Struct en stack               │ │
│   │   x         │       │ ; [rbp-8]  = x                  │ │
│   │   y         │       │ ; [rbp-16] = y                  │ │
│   │ }           │       │                                 │ │
│   └─────────────┘       └─────────────────────────────────┘ │
│                                                             │
│   Sin overhead de runtime, sin GC, sin vtables innecesarias │
└─────────────────────────────────────────────────────────────┘
```

### Filosofía

1. **Zero-Cost Abstractions**: OOP sin overhead
2. **Stack-based**: Structs en stack, no heap
3. **Explícito**: Sin magia oculta
4. **NASM Directo**: Control total del código generado

---

## 📦 2. Structs Básicos

### 2.1 Definición de Struct

```ad
# Struct simple - un campo
struct Contador {
    valor
}

# Struct con múltiples campos
struct Punto {
    x
    y
}

# Struct complejo
struct Rectangulo {
    x
    y
    ancho
    alto
}
```

### 2.2 Creación de Instancias (Struct Literals)

```ad
# Sintaxis: NombreStruct { campo: valor, campo2: valor2 }

let c = Contador { valor: 0 }

let p = Punto { x: 10, y: 20 }

let rect = Rectangulo { 
    x: 0, 
    y: 0, 
    ancho: 100, 
    alto: 50 
}
```

### 2.3 Layout en Memoria

```
Struct Punto { x, y }:

Stack:
┌──────────────────────────────────┐
│  [rbp - 8]   │ x (8 bytes)       │  ← Campo 0
├──────────────────────────────────┤
│  [rbp - 16]  │ y (8 bytes)       │  ← Campo 1
└──────────────────────────────────┘

Cada campo ocupa 8 bytes (alineación de 64 bits)
```

---

## 🔍 3. Acceso a Campos

### 3.1 Lectura de Campos

```ad
struct Punto {
    x
    y
}

let p = Punto { x: 100, y: 200 }

# Leer campos con notación punto
let valor_x = p.x    # 100
let valor_y = p.y    # 200

# Usar en expresiones
let suma = p.x + p.y  # 300
```

### 3.2 Código NASM Generado

```asm
; Acceso a p.x (campo 0, offset 0)
mov rax, [rbp - 8]     ; Cargar base del struct
mov rax, [rax]         ; Cargar campo x

; Acceso a p.y (campo 1, offset 8)
mov rax, [rbp - 8]     ; Cargar base del struct
mov rax, [rax - 8]     ; Cargar campo y (offset -8)
```

### 3.3 Structs Anidados (Futuro)

```ad
# NOTA: Aún no implementado
struct Linea {
    inicio    # Tipo: Punto
    fin       # Tipo: Punto
}

let linea = Linea {
    inicio: Punto { x: 0, y: 0 },
    fin: Punto { x: 100, y: 100 }
}

# Acceso anidado
let x_inicio = linea.inicio.x
```

---

## 🏭 4. Constructores y Factories

### 4.1 Patrón Factory (Recomendado Actualmente)

En ADead, el patrón actual para crear structs con lógica es usar funciones factory:

```ad
struct Usuario {
    id
    edad
    activo
}

# Factory simple
fn crear_usuario(id, edad) {
    return Usuario { 
        id: id, 
        edad: edad, 
        activo: 1  # true
    }
}

# Factory con validación
fn crear_usuario_validado(id, edad) {
    # Validar edad
    if edad < 0 {
        return Usuario { id: 0, edad: 0, activo: 0 }
    }
    return Usuario { id: id, edad: edad, activo: 1 }
}

# Uso
let u1 = crear_usuario(1001, 25)
let u2 = crear_usuario_validado(1002, 30)
```

### 4.2 Constructor con self (Sintaxis Futura)

```ad
# Sintaxis planificada para el futuro
struct Usuario {
    id
    edad
    activo
    
    fn new(id, edad) {
        self.id = id
        self.edad = edad
        self.activo = 1
    }
}

# Uso futuro:
let u = Usuario.new(1001, 25)
```

### 4.3 Patrón Builder (Avanzado)

```ad
struct ConfigBuilder {
    ancho
    alto
    titulo
    fullscreen
}

fn config_builder_new() {
    return ConfigBuilder {
        ancho: 800,
        alto: 600,
        titulo: 0,
        fullscreen: 0
    }
}

fn config_set_size(c, w, h) {
    return ConfigBuilder {
        ancho: w,
        alto: h,
        titulo: c.titulo,
        fullscreen: c.fullscreen
    }
}

fn config_set_fullscreen(c, fs) {
    return ConfigBuilder {
        ancho: c.ancho,
        alto: c.alto,
        titulo: c.titulo,
        fullscreen: fs
    }
}

# Uso encadenado (inmutable)
let cfg = config_builder_new()
let cfg2 = config_set_size(cfg, 1920, 1080)
let cfg3 = config_set_fullscreen(cfg2, 1)
```

---

## 🔧 5. Métodos (Funciones Asociadas)

### 5.1 Patrón: Struct como Primer Parámetro

Como ADead aún no tiene métodos nativos, usamos funciones que reciben el struct como primer parámetro:

```ad
struct Contador {
    valor
}

# "Método" incrementar
fn contador_incrementar(self) {
    let nuevo = self.valor + 1
    return Contador { valor: nuevo }
}

# "Método" decrementar
fn contador_decrementar(self) {
    let nuevo = self.valor - 1
    return Contador { valor: nuevo }
}

# "Método" reset
fn contador_reset(self) {
    return Contador { valor: 0 }
}

# "Método" get
fn contador_valor(self) {
    return self.valor
}

# Uso
let c = Contador { valor: 10 }
print contador_valor(c)          # 10

let c2 = contador_incrementar(c)
print contador_valor(c2)         # 11

let c3 = contador_decrementar(c2)
print contador_valor(c3)         # 10
```

### 5.2 Convención de Nombres

```
┌──────────────────────────────────────────────────────────┐
│ CONVENCIÓN: structname_metodonombre(self, params...)    │
├──────────────────────────────────────────────────────────┤
│                                                          │
│ struct Vector2D { x, y }                                 │
│                                                          │
│ fn vector2d_new(x, y)           # Constructor            │
│ fn vector2d_magnitud(self)      # Getter calculado       │
│ fn vector2d_normalizar(self)    # Transforma             │
│ fn vector2d_suma(self, otro)    # Opera con otro         │
│ fn vector2d_escalar(self, s)    # Escala                 │
│ fn vector2d_to_string(self)     # Conversión             │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

### 5.3 Ejemplo Completo: Vector2D

```ad
struct Vector2D {
    x
    y
}

# Constructor
fn vector2d_new(x, y) {
    return Vector2D { x: x, y: y }
}

# Magnitud al cuadrado (evita sqrt)
fn vector2d_mag_sq(v) {
    return v.x * v.x + v.y * v.y
}

# Suma de vectores
fn vector2d_add(v1, v2) {
    return Vector2D { 
        x: v1.x + v2.x, 
        y: v1.y + v2.y 
    }
}

# Resta de vectores
fn vector2d_sub(v1, v2) {
    return Vector2D { 
        x: v1.x - v2.x, 
        y: v1.y - v2.y 
    }
}

# Escalar
fn vector2d_scale(v, s) {
    return Vector2D { 
        x: v.x * s, 
        y: v.y * s 
    }
}

# Producto punto
fn vector2d_dot(v1, v2) {
    return v1.x * v2.x + v1.y * v2.y
}

# --- Uso ---
let a = vector2d_new(3, 4)
let b = vector2d_new(1, 2)

print "Vector A:"
print a.x
print a.y

print "Magnitud^2 de A:"
print vector2d_mag_sq(a)  # 25 (3*3 + 4*4)

let c = vector2d_add(a, b)
print "A + B:"
print c.x  # 4
print c.y  # 6

print "A dot B:"
print vector2d_dot(a, b)  # 11 (3*1 + 4*2)
```

---

## 🔄 6. RAII - Gestión de Recursos

### 6.1 ¿Qué es RAII?

**Resource Acquisition Is Initialization**:
- Adquirir recursos en el constructor
- Liberar recursos en el destructor
- Garantiza limpieza automática

### 6.2 Patrón RAII Manual en ADead

```ad
struct Archivo {
    handle
    nombre
}

# Constructor: adquiere recurso
fn archivo_abrir(nombre) {
    print "Abriendo archivo..."
    # Simular apertura (en futuro: syscall real)
    return Archivo { handle: 1, nombre: 0 }
}

# Destructor: libera recurso
fn archivo_cerrar(f) {
    print "Cerrando archivo..."
    # Simular cierre
}

# Uso RAII manual
fn procesar_archivo() {
    let f = archivo_abrir("datos.txt")
    
    # ... trabajo con archivo ...
    print "Procesando..."
    
    # IMPORTANTE: Llamar destructor explícitamente
    archivo_cerrar(f)
}

procesar_archivo()
```

### 6.3 RAII Automático (Futuro)

```ad
# Sintaxis futura con destroy automático
struct Archivo {
    handle
    
    fn new(nombre) {
        self.handle = syscall_open(nombre)
        print "Archivo abierto"
    }
    
    fn destroy(self) {
        syscall_close(self.handle)
        print "Archivo cerrado"
    }
}

fn procesar() {
    let f = Archivo.new("test.txt")
    # ... uso ...
}  # destroy() se llama automáticamente aquí
```

---

## 🎯 7. Patrones OOP Avanzados

### 7.1 Patrón Singleton (Simulado)

```ad
struct Config {
    inicializado
    valor
}

# Variable "global" (en scope del módulo)
let CONFIG_INSTANCE = Config { inicializado: 0, valor: 0 }

fn config_get_instance() {
    return CONFIG_INSTANCE
}

fn config_set_valor(valor) {
    return Config { inicializado: 1, valor: valor }
}
```

### 7.2 Patrón State

```ad
struct Puerta {
    estado  # 0=cerrada, 1=abierta, 2=trabada
}

fn puerta_new() {
    return Puerta { estado: 0 }
}

fn puerta_abrir(p) {
    if p.estado == 2 {
        print "No se puede abrir: esta trabada"
        return p
    }
    return Puerta { estado: 1 }
}

fn puerta_cerrar(p) {
    return Puerta { estado: 0 }
}

fn puerta_trabar(p) {
    if p.estado == 0 {
        return Puerta { estado: 2 }
    }
    print "Debe estar cerrada para trabar"
    return p
}

fn puerta_estado_str(p) {
    if p.estado == 0 {
        print "Cerrada"
    }
    if p.estado == 1 {
        print "Abierta"
    }
    if p.estado == 2 {
        print "Trabada"
    }
}

# Uso
let p = puerta_new()
puerta_estado_str(p)      # Cerrada

let p2 = puerta_abrir(p)
puerta_estado_str(p2)     # Abierta

let p3 = puerta_cerrar(p2)
let p4 = puerta_trabar(p3)
puerta_estado_str(p4)     # Trabada
```

### 7.3 Patrón Composite (Árbol)

```ad
struct Nodo {
    valor
    tiene_hijo
    hijo_valor   # Simplificado: solo un nivel
}

fn nodo_hoja(valor) {
    return Nodo { 
        valor: valor, 
        tiene_hijo: 0, 
        hijo_valor: 0 
    }
}

fn nodo_con_hijo(valor, hijo_valor) {
    return Nodo { 
        valor: valor, 
        tiene_hijo: 1, 
        hijo_valor: hijo_valor 
    }
}

fn nodo_sumar_todo(n) {
    if n.tiene_hijo == 1 {
        return n.valor + n.hijo_valor
    }
    return n.valor
}

let arbol = nodo_con_hijo(10, 5)
print nodo_sumar_todo(arbol)  # 15
```

---

## ⚙️ 8. Generación NASM

### 8.1 Struct en Stack

```ad
struct Punto {
    x
    y
}

let p = Punto { x: 42, y: 100 }
```

**NASM Generado:**

```asm
; Reservar espacio para struct (2 campos * 8 bytes = 16 bytes)
sub rsp, 16

; Inicializar campo x (offset 0)
mov qword [rbp - 8], 42

; Inicializar campo y (offset 8)  
mov qword [rbp - 16], 100
```

### 8.2 Acceso a Campo

```ad
let valor = p.x
```

**NASM Generado:**

```asm
; Calcular dirección del campo
lea rax, [rbp - 8]     ; Dirección base del struct
mov rax, [rax]         ; Cargar campo x (offset 0)
mov [rbp - 24], rax    ; Guardar en variable 'valor'
```

### 8.3 Pasar Struct a Función

```ad
fn procesar(punto) {
    return punto.x + punto.y
}

let resultado = procesar(p)
```

**NASM Generado (Windows x64 ABI):**

```asm
; Pasar puntero al struct en RCX
lea rcx, [rbp - 16]    ; Dirección del struct
sub rsp, 32            ; Shadow space
call procesar
add rsp, 32

; En la función:
procesar:
    push rbp
    mov rbp, rsp
    sub rsp, 32
    
    ; punto está en RCX (puntero)
    mov rax, [rcx]         ; punto.x
    mov rdx, [rcx - 8]     ; punto.y
    add rax, rdx           ; suma
    
    mov rsp, rbp
    pop rbp
    ret
```

---

## 🚀 9. Roadmap Futuro

### Fase 1: OOP Básico ✅ COMPLETADO
- [x] Structs simples
- [x] Múltiples campos
- [x] Struct literals
- [x] Acceso a campos
- [x] Funciones factory

### Fase 2: Métodos Nativos (En Progreso)
- [ ] Sintaxis `fn metodo(self)`
- [ ] Llamada `objeto.metodo()`
- [ ] `self` implícito

### Fase 3: Constructores Nativos
- [ ] `fn new()` como constructor
- [ ] `NombreStruct.new()` 
- [ ] Inicialización automática

### Fase 4: RAII Completo
- [ ] `fn destroy(self)` automático
- [ ] Orden de destrucción (LIFO)
- [ ] Manejo de excepciones

### Fase 5: Herencia
- [ ] `extends` keyword
- [ ] `super.metodo()`
- [ ] Campos heredados

### Fase 6: Polimorfismo
- [ ] Virtual tables (vtables)
- [ ] Late binding
- [ ] Interfaces/Traits

### Fase 7: Avanzado
- [ ] Genéricos básicos
- [ ] Métodos estáticos
- [ ] Visibilidad (pub/priv)

---

## 📊 Comparación con Otros Lenguajes

| Característica | ADead | Rust | C++ | Python |
|---------------|-------|------|-----|--------|
| Structs | ✅ | ✅ | ✅ | ❌ |
| Classes | 🔄 | ❌ | ✅ | ✅ |
| Herencia | ⏳ | ❌ | ✅ | ✅ |
| Traits/Interfaces | ⏳ | ✅ | ✅ | ✅ |
| Zero-cost | ✅ | ✅ | ✅ | ❌ |
| No GC | ✅ | ✅ | ✅ | ❌ |
| RAII | 🔄 | ✅ | ✅ | ❌ |

---

## 💡 Tips y Mejores Prácticas

### 1. Nombrar Structs con PascalCase
```ad
struct MiEstructura { ... }  # ✅ Correcto
struct mi_estructura { ... }  # ❌ Evitar
```

### 2. Usar Prefijo en Funciones
```ad
fn miestruct_metodo(self, ...)  # ✅ Convención
fn metodo(self, ...)            # ❌ Confuso
```

### 3. Inmutabilidad por Defecto
```ad
# Preferir retornar nuevos structs
fn punto_mover(p, dx, dy) {
    return Punto { x: p.x + dx, y: p.y + dy }  # ✅
}
```

### 4. RAII Manual Siempre
```ad
let recurso = crear_recurso()
# ... uso ...
destruir_recurso(recurso)  # ✅ No olvidar
```

---

**¡Esta guía se actualizará con cada avance en OOP!**

---

*ADead Compiler - Diciembre 2025*

