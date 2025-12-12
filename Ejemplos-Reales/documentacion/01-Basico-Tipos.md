# 📖 Nivel Básico: Sistema de Tipos

**Guía básica sobre el sistema de tipos en ADead**

---

## 🎯 Tipos Primitivos

### Enteros

ADead soporta diferentes tamaños de enteros:

```adead
// Enteros con signo (pueden ser negativos)
let a: int8 = 127     // -128 a 127
let b: int16 = 32767  // -32,768 a 32,767
let c: int32 = 1000   // -2^31 a 2^31-1
let d: int64 = 1000000  // -2^63 a 2^63-1 (recomendado por defecto)

// Enteros sin signo (solo positivos)
let e: uint8 = 255    // 0 a 255
let f: uint16 = 65535 // 0 a 65,535
let g: uint32 = 100   // 0 a 2^32-1
let h: uint64 = 200   // 0 a 2^64-1

// Si no especificas tipo, se infiere como int64
let x = 10  // x es int64 automáticamente
```

### Punto Flotante

```adead
let pi: float32 = 3.14      // Precisión simple (32 bits)
let e: float64 = 2.71828    // Precisión doble (64 bits, recomendado)

// Inferencia automática
let altura = 1.75  // float64 por defecto
```

### Booleanos y Caracteres

```adead
let es_verdadero: bool = true
let es_falso: bool = false

let letra: char = 'A'      // Carácter Unicode
let emoji: char = '🚀'     // También soporta emojis
```

### Cadenas de Texto

```adead
let saludo: string = "Hola Mundo"
let nombre = "ADead"  // string inferido automáticamente
```

---

## 🔍 Inferencia de Tipos

ADead puede inferir tipos automáticamente:

```adead
// El compilador infiere el tipo desde el valor
let numero = 42        // int64
let texto = "Hola"     // string
let decimal = 3.14     // float64
let flag = true        // bool

// También puedes especificar el tipo explícitamente
let numero: int32 = 42  // Forzar int32
```

**Cuándo especificar tipos:**
- Cuando quieres un tipo específico diferente al inferido
- Para claridad en funciones
- Para evitar ambigüedades

---

## 📦 Tipos Compuestos

### Arrays (Arreglos)

```adead
// Array de tamaño fijo (especificado)
let numeros: Array<int64, 5> = [1, 2, 3, 4, 5]

// Array dinámico (tamaño variable)
let nombres: Array<string> = ["Juan", "María", "Pedro"]

// Inferencia
let edades = [25, 30, 35]  // Array<int64>
```

### Tuples (Tuplas)

```adead
// Tupla con tipos mixtos
let coordenada: (int64, int64) = (10, 20)
let persona: (string, int64, bool) = ("Juan", 25, true)

// Inferencia
let punto = (x: 10, y: 20)  // Tupla con nombres
```

---

## 🔄 Conversión de Tipos

```adead
// Conversión explícita entre tipos numéricos
let entero: int64 = 100
let flotante: float64 = float64(entero)  // 100.0

let decimal: float64 = 3.14
let redondeado: int64 = int64(decimal)   // 3 (trunca)

// Conversión a string
let numero = 42
let texto = string(numero)  // "42"
```

---

## ⚠️ Type Safety

ADead es type-safe: el compilador verifica tipos en tiempo de compilación.

```adead
let x: int64 = 10
let y: string = "Hola"

// Error: tipos incompatibles
// let suma = x + y  // ❌ No se puede sumar int64 + string
```

---

## ✅ Ejercicios

1. Declara variables de diferentes tipos primitivos
2. Experimenta con inferencia de tipos
3. Crea arrays y tuples
4. Prueba conversiones de tipos

---

*Siguiente: [02-Basico-Variables.md](02-Basico-Variables.md)*

