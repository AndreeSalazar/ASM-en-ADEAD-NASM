# 🏗️ TEST_OOP: Pruebas de Programación Orientada a Objetos

**Guía completa de OOP en ADead: De Básico a Avanzado**

---

## 📊 Estado de Implementación OOP

```
╔════════════════════════════════════════════════════════════════════════╗
║                    OOP EN ADEAD - ESTADO ACTUAL                        ║
╠════════════════════════════════════════════════════════════════════════╣
║                                                                         ║
║  OOP BÁSICO                           OOP AVANZADO                     ║
║  ✅ Structs simples                   🔄 Herencia (parcial)            ║
║  ✅ Campos de structs                 ⏳ Interfaces/Traits              ║
║  ✅ Acceso a campos (struct.campo)    ⏳ Métodos estáticos              ║
║  ✅ Classes con constructor           ⏳ Polimorfismo                   ║
║  ✅ fn new() - Constructor            ⏳ Super/extends                  ║
║  ✅ self.campo = valor                ⏳ Métodos privados               ║
║  ✅ RAII básico (init/destroy)        ⏳ Getters/Setters                ║
║  ✅ Struct literals                   ⏳ Operadores sobrecargados       ║
║                                                                         ║
╚════════════════════════════════════════════════════════════════════════╝
```

---

## 📋 Orden de Tests (Progresivo)

### Nivel 1: Structs Básicos
1. `01_struct_simple.ad` - Struct con un campo
2. `02_struct_multi_campos.ad` - Struct con múltiples campos
3. `03_struct_acceso_campos.ad` - Acceso a campos `.campo`

### Nivel 2: Struct Literals
4. `04_struct_literal.ad` - Crear struct con `Nombre { campo: valor }`
5. `05_struct_multiple_instancias.ad` - Múltiples instancias

### Nivel 3: Clases con Constructor
6. `06_class_new_simple.ad` - `fn new()` básico
7. `07_class_new_params.ad` - `fn new(param1, param2)`
8. `08_class_self.ad` - Uso de `self.campo = valor`

### Nivel 4: Métodos
9. `09_class_metodo_simple.ad` - Método sin parámetros
10. `10_class_metodo_params.ad` - Método con parámetros
11. `11_class_metodo_return.ad` - Método que retorna valor

### Nivel 5: RAII (Avanzado)
12. `12_raii_init_destroy.ad` - Constructor y destructor

---

## 🚀 Ejecutar Tests

```powershell
# Ejecutar todos los tests OOP
.\ejecutar_tests.ps1

# Ejecutar test específico
..\CORE\rust\target\release\adeadc.exe build 01_struct_simple.ad -o 01_struct_simple.exe
.\01_struct_simple.exe
```

---

## 📖 Sintaxis OOP en ADead

### Structs (Datos sin Comportamiento)

```ad
# Struct simple con campos
struct Punto {
    x
    y
}

# Crear instancia
let p = Punto { x: 10, y: 20 }

# Acceder a campos
print p.x    # 10
print p.y    # 20
```

### Classes (Datos + Comportamiento)

```ad
# Clase con constructor
class Rectangulo {
    fn new(ancho, alto) {
        self.ancho = ancho
        self.alto = alto
    }
    
    fn area(self) {
        return self.ancho * self.alto
    }
    
    fn perimetro(self) {
        return 2 * (self.ancho + self.alto)
    }
}

# Crear instancia usando constructor
let rect = Rectangulo.new(5, 3)

# Llamar métodos
print rect.area()       # 15
print rect.perimetro()  # 16
```

### RAII (Resource Acquisition Is Initialization)

```ad
class Recurso {
    fn new(valor) {
        self.valor = valor
        print "Recurso creado"
    }
    
    fn destroy(self) {
        print "Recurso destruido"
    }
}

# El destructor se llama automáticamente al salir del scope
let r = Recurso.new(42)
# ... código ...
# destroy() se llama aquí automáticamente
```

---

## 🎯 Objetivo de Cada Test

| Test | Objetivo | Verifica |
|------|----------|----------|
| 01 | Struct mínimo | Parser reconoce `struct` |
| 02 | Múltiples campos | Manejo de varios campos |
| 03 | Acceso `.campo` | Generación de offsets |
| 04 | Struct literal | Sintaxis `{ campo: valor }` |
| 05 | Múltiples instancias | Independencia de datos |
| 06 | Constructor simple | `fn new()` sin params |
| 07 | Constructor params | `fn new(a, b)` con params |
| 08 | self | `self.campo = valor` |
| 09 | Método simple | Método que usa `self` |
| 10 | Método params | Método con parámetros |
| 11 | Método return | Retornar valor calculado |
| 12 | RAII | `destroy()` automático |

---

## 📁 Estructura de Archivos

```
TEST_OOP/
├── README.md                    # Esta guía
├── ejecutar_tests.ps1           # Script para ejecutar todos
│
├── # Nivel 1: Structs Básicos
├── 01_struct_simple.ad
├── 02_struct_multi_campos.ad
├── 03_struct_acceso_campos.ad
│
├── # Nivel 2: Struct Literals
├── 04_struct_literal.ad
├── 05_struct_multiple_instancias.ad
│
├── # Nivel 3: Classes
├── 06_class_new_simple.ad
├── 07_class_new_params.ad
├── 08_class_self.ad
│
├── # Nivel 4: Métodos
├── 09_class_metodo_simple.ad
├── 10_class_metodo_params.ad
├── 11_class_metodo_return.ad
│
└── # Nivel 5: RAII
    └── 12_raii_init_destroy.ad
```

---

## ⚠️ Limitaciones Actuales

1. **Sin herencia**: `class Hijo extends Padre` no implementado
2. **Sin interfaces**: `implements` no implementado
3. **Sin métodos estáticos**: `static fn` no implementado
4. **Sin visibilidad**: `pub`/privado parcialmente implementado
5. **Sin polimorfismo**: No hay vtables dinámicas

---

## 🔮 Roadmap OOP

### Próximas Implementaciones:
1. [ ] Herencia simple (`extends`)
2. [ ] Llamada a `super.metodo()`
3. [ ] Métodos estáticos (`static fn`)
4. [ ] Visibilidad (`_privado`)
5. [ ] Interfaces/Traits
6. [ ] Polimorfismo con vtables

---

**Autor:** Eddi Andreé Salazar Matos  
**Fecha:** Diciembre 2025

