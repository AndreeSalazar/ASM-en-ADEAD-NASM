# 🧪 TEST_OOP - Tests de Programación Orientada a Objetos

## 📊 Estado de Implementación

```
Progreso OOP: ░░░░░░░░░░ 0% → Meta: 100%

⏳ PENDIENTE PARSER     🔄 EN PROGRESO     ✅ IMPLEMENTADO
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
struct keyword          -                   -
class keyword           -                   -
self reference          -                   -
constructores (new)     -                   -
métodos                 -                   -
acceso a campos (.)     -                   -
```

## 📁 Archivos de Test

### Nivel 1: Structs Básicos
| Archivo | Descripción | Parser | Backend |
|---------|-------------|--------|---------|
| `01_struct_simple.ad` | Struct con 2 campos | ⏳ | ⏳ |
| `02_struct_multiple.ad` | Varios structs | ⏳ | ⏳ |
| `03_struct_operaciones.ad` | Operaciones con campos | ⏳ | ⏳ |

### Nivel 2: Clases con Constructor
| Archivo | Descripción | Parser | Backend |
|---------|-------------|--------|---------|
| `04_class_basica.ad` | Clase con new() | ⏳ | ⏳ |
| `05_class_metodos.ad` | Métodos con self | ⏳ | ⏳ |
| `06_class_rectangulo.ad` | Área y perímetro | ⏳ | ⏳ |

### Nivel 3: Clases Avanzadas
| Archivo | Descripción | Parser | Backend |
|---------|-------------|--------|---------|
| `07_class_pila.ad` | Stack con array interno | ⏳ | ⏳ |
| `08_class_persona.ad` | Strings en clases | ⏳ | ⏳ |
| `09_class_vector2d.ad` | Matemáticas vectoriales | ⏳ | ⏳ |
| `10_class_banco.ad` | Métodos que usan otros métodos | ⏳ | ⏳ |

## 🎯 Sintaxis Objetivo

### Structs (Datos sin métodos)
```python
struct Punto {
    x
    y
}

let p = Punto { x: 10, y: 20 }
print p.x
p.y = 30
```

### Clases (Datos + Métodos)
```python
class Rectangulo {
    fn new(ancho, alto) {
        self.ancho = ancho
        self.alto = alto
    }
    
    fn area(self) {
        return self.ancho * self.alto
    }
}

let r = Rectangulo.new(10, 5)
print r.area()
```

## 🔧 Implementación Requerida

### 1. Parser (`adead-parser/src/lib.rs`)

```rust
// Nuevos tokens
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // ... existentes ...
    Struct,      // "struct"
    Class,       // "class"
    Self_,       // "self"
    Dot,         // "."
}

// Nuevas expresiones
#[derive(Debug, Clone)]
pub enum Expr {
    // ... existentes ...
    FieldAccess { object: Box<Expr>, field: String },
    MethodCall { object: Box<Expr>, method: String, args: Vec<Expr> },
    StructLiteral { name: String, fields: Vec<(String, Expr)> },
    SelfRef,
}

// Nuevos statements
#[derive(Debug, Clone)]
pub enum Stmt {
    // ... existentes ...
    StructDef { name: String, fields: Vec<String> },
    ClassDef { name: String, methods: Vec<FnDef> },
}
```

### 2. Backend (`adead-backend/src/lib.rs`)

```rust
// Generar estructura en memoria
fn generate_struct_def(&mut self, name: &str, fields: &[String]) {
    // Calcular offsets
    // Generar constructor implícito
}

// Generar clase con vtable
fn generate_class_def(&mut self, name: &str, methods: &[FnDef]) {
    // Generar vtable
    // Generar métodos
    // Generar constructor
}
```

### 3. Memoria NASM

```asm
; Estructura Punto (16 bytes)
; [+0]  x (qword)
; [+8]  y (qword)

; Clase Rectangulo (24 bytes)  
; [+0]  vtable_ptr (puntero a métodos)
; [+8]  ancho (qword)
; [+16] alto (qword)

; Vtable Rectangulo
Rectangulo_vtable:
    dq Rectangulo_area
    dq Rectangulo_perimetro
```

## 📋 Plan de Implementación

### Fase 1: Structs Simples (1-2 días)
1. [ ] Agregar token `struct` al lexer
2. [ ] Parser para `struct Name { fields }`
3. [ ] Parser para `Struct { field: value }`
4. [ ] Parser para `obj.field`
5. [ ] Generar layout en memoria
6. [ ] Generar acceso a campos
7. [ ] Tests 01-03

### Fase 2: Clases Básicas (2-3 días)
1. [ ] Agregar token `class` al lexer
2. [ ] Parser para `class Name { methods }`
3. [ ] Parser para `self.field`
4. [ ] Parser para `Class.new(args)`
5. [ ] Generar vtable
6. [ ] Generar métodos con self
7. [ ] Tests 04-06

### Fase 3: Clases Avanzadas (2-3 días)
1. [ ] Clases con arrays internos
2. [ ] Métodos que llaman otros métodos
3. [ ] Métodos con múltiples parámetros
4. [ ] Optimizaciones
5. [ ] Tests 07-10

## 🚀 Cómo Ejecutar

```powershell
# Cuando esté implementado:
cd TEST_OOP
..\CORE\rust\target\release\adeadc.exe build 01_struct_simple.ad -o test.exe
.\test.exe
```

## 📊 Salidas Esperadas

### 01_struct_simple.ad
```
10
20
30
```

### 05_class_metodos.ad
```
0
3
2
```

### 10_class_banco.ad
```
1000
500
1200
900
500
900
```

---

**Última actualización:** Diciembre 2025
**Estado:** ⏳ Pendiente implementación de parser y backend

