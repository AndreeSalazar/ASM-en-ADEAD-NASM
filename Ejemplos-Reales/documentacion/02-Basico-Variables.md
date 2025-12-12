# 📖 Nivel Básico: Variables y Mutabilidad

**Guía sobre variables en ADead - Inmutabilidad por defecto**

---

## 📦 Declaración de Variables

### Variables Inmutables (por defecto)

En ADead, las variables son **inmutables por defecto** (como Rust):

```adead
let x = 10
// x = 20  // ❌ Error: x es inmutable

print x  // 10
```

### Variables Mutables

Usa `mut` para hacer una variable mutable:

```adead
let mut y = 10
y = 20      // ✅ OK: y es mutable
y = y + 5   // ✅ OK
print y     // 25
```

---

## 🎯 Inmutabilidad: ¿Por qué?

**Ventajas:**
- ✅ Menos bugs (valores no cambian inesperadamente)
- ✅ Código más fácil de entender
- ✅ Mejor para concurrencia (si se implementa después)

**Cuándo usar `mut`:**
- Cuando realmente necesitas modificar la variable
- En loops (contadores)
- Cuando acumulas valores

---

## 📝 Ejemplos

### Ejemplo 1: Variables Inmutables

```adead
let nombre = "Juan"           // Inmutable
let edad = 25                 // Inmutable
let es_estudiante = true      // Inmutable

// No puedes modificarlos
// nombre = "María"           // ❌ Error
```

### Ejemplo 2: Variables Mutables

```adead
let mut contador = 0
contador = contador + 1       // ✅ OK
contador = contador + 1
print contador                // 2

let mut mensaje = "Hola"
mensaje = mensaje + " Mundo"  // ✅ OK
print mensaje                 // "Hola Mundo"
```

### Ejemplo 3: En Loops

```adead
let mut i = 0
while i < 10 {
    print i
    i = i + 1                 // ✅ Necesario: mut
}
```

---

## 🔄 Shadowing (Re-declaración)

Puedes re-declarar una variable con el mismo nombre (shadowing):

```adead
let x = 5
let x = x + 1        // ✅ OK: nueva variable x (no modifica la anterior)
let x = "Hola"       // ✅ OK: incluso puede cambiar de tipo
print x              // "Hola"
```

**Nota:** Shadowing crea una nueva variable, no modifica la anterior.

---

## 📊 Scope (Alcance)

Las variables existen dentro de su scope (bloque):

```adead
let x = 10           // x existe aquí

{
    let y = 20       // y solo existe dentro de este bloque
    print x          // ✅ OK: x es accesible
    print y          // ✅ OK: y es accesible aquí
}

// print y            // ❌ Error: y no existe fuera del bloque
print x              // ✅ OK: x sigue existiendo
```

---

## ✅ Buenas Prácticas

### 1. Usa inmutabilidad por defecto

```adead
// ✅ Bueno: inmutable
let nombre = "Juan"

// ❌ Evita mut innecesario
// let mut nombre = "Juan"  // No necesitas mutar esto
```

### 2. Usa mut solo cuando es necesario

```adead
// ✅ Necesario: contador en loop
let mut i = 0
while i < 10 {
    i = i + 1
}
```

### 3. Shadowing para transformaciones

```adead
let valor = "42"
let valor = parse_int(valor)  // Transformar: string -> int
```

---

## ✅ Ejercicios

1. Declara variables inmutables y mutables
2. Prueba modificar variables mutables
3. Experimenta con shadowing
4. Crea bloques y prueba scopes

---

*Siguiente: [03-Basico-Funciones.md](03-Basico-Funciones.md)*

