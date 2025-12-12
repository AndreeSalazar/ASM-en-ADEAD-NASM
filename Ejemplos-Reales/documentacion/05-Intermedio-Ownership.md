# 🔧 Nivel Intermedio: Ownership y Borrowing

**Guía sobre el sistema de ownership estilo Rust en ADead**

---

## 🎯 Concepto de Ownership

En ADead, cada valor tiene un **único dueño** (owner). Cuando el dueño sale de scope, el valor se libera automáticamente.

```adead
{
    let mensaje = "Hola"  // mensaje es el owner
    print mensaje
}  // mensaje se libera aquí automáticamente
```

---

## 📦 Movimiento (Move Semantics)

Por defecto, los valores se **mueven** en lugar de copiarse:

```adead
let s1 = "Hola Mundo"
let s2 = s1           // s1 se MUEVE a s2
// print s1           // ❌ Error: s1 ya no es válido (fue movido)
print s2              // ✅ OK: s2 es el nuevo owner
```

### Valores que se Copian

Algunos tipos son **Copy** (se copian en lugar de moverse):

```adead
let x = 10
let y = x             // x se COPIA (porque int64 es Copy)
print x               // ✅ OK: x sigue siendo válido
print y               // ✅ OK: y tiene una copia
```

**Tipos Copy:**
- Todos los enteros (`int8`, `int16`, `int32`, `int64`, `uint8`, etc.)
- Punto flotante (`float32`, `float64`)
- `bool`, `char`
- Referencias (`&T`)

**Tipos Move:**
- `string`
- `Array<T>`
- Structs/Clases (en general)

---

## 🔗 Borrowing (Prestar Referencias)

En lugar de mover, puedes **prestar** una referencia:

### Referencia Inmutable (`&T`)

```adead
fn imprimir(tex: &string) {  // & = prestar (read-only)
    print tex                // Solo lectura
}

let texto = "Hola"
imprimir(&texto)      // Prestar referencia a texto
print texto           // ✅ OK: texto sigue siendo owner
```

**Reglas:**
- Puedes tener **múltiples referencias inmutables** al mismo tiempo
- No puedes modificar a través de referencia inmutable

```adead
let x = "Hola"
let r1 = &x           // ✅ OK
let r2 = &x           // ✅ OK: múltiples & permitidos
// r1.append("!")     // ❌ Error: & es inmutable
```

### Referencia Mutable (`&mut T`)

```adead
fn modificar(tex: &mut string) {  // &mut = prestar mutable
    tex.append("!")               // Puede modificar
}

let mut mensaje = "Hola"
modificar(&mut mensaje)  // Prestar mutable
print mensaje            // "Hola!" ✅ OK
```

**Reglas:**
- Solo puedes tener **UNA referencia mutable** a la vez
- No puedes tener `&` y `&mut` simultáneamente
- El owner debe ser `mut` para poder tomar `&mut`

```adead
let mut x = "Hola"
let r1 = &mut x       // ✅ OK
// let r2 = &mut x    // ❌ Error: solo un &mut a la vez
// let r3 = &x        // ❌ Error: no puedes tener & y &mut juntos
```

---

## 🔄 Ejemplos Prácticos

### Ejemplo 1: Funciones con Borrowing

```adead
// Función que no consume (usa &)
fn longitud(tex: &string) -> int64 {
    return tex.length()
}

let texto = "Hola Mundo"
let len = longitud(&texto)
print texto  // ✅ OK: texto sigue siendo válido
print len    // 10
```

### Ejemplo 2: Modificar sin Mover

```adead
fn agregar_sufijo(tex: &mut string, sufijo: string) {
    tex.append(sufijo)
}

let mut nombre = "Juan"
agregar_sufijo(&mut nombre, " Pérez")
print nombre  // "Juan Pérez"
```

### Ejemplo 3: Evitar Movimientos Innecesarios

```adead
// ❌ Malo: consume el string
fn malo(s: string) -> int64 {
    return s.length()
}

let texto = "Hola"
let len = malo(texto)   // texto se mueve y se destruye
// print texto          // ❌ Error: texto ya no existe

// ✅ Bueno: usa borrowing
fn bueno(s: &string) -> int64 {
    return s.length()
}

let texto2 = "Hola"
let len2 = bueno(&texto2)  // texto2 se presta
print texto2               // ✅ OK: texto2 sigue existiendo
```

---

## 🎯 Reglas de Ownership

1. **Cada valor tiene un único owner**
2. **Solo un owner a la vez**
3. **Cuando el owner sale de scope, el valor se libera**
4. **Puedes prestar (`&`) o prestar mutable (`&mut`) sin transferir ownership**

---

## 📚 Comparación con Otros Lenguajes

| Característica | Python/JavaScript | Rust | ADead |
|----------------|-------------------|------|-------|
| Memory Management | GC automático | Ownership manual | Ownership automático |
| Referencias | Todas son referencias | Borrowing explícito | Borrowing explícito |
| Mutabilidad | Todo es mutable | Inmutable por defecto | Inmutable por defecto |
| Velocidad | Lenta (GC) | Rápida | Rápida (sin GC) |

---

## ⚠️ Errores Comunes

### Error 1: Usar después de mover

```adead
let s = "Hola"
let s2 = s
print s  // ❌ Error: s fue movido a s2
```

**Solución:** Usa borrowing si no necesitas mover

```adead
let s = "Hola"
let s2 = &s  // Prestar en lugar de mover
print s      // ✅ OK
```

### Error 2: Múltiples &mut

```adead
let mut x = "Hola"
let r1 = &mut x
let r2 = &mut x  // ❌ Error: solo un &mut
```

**Solución:** Usa una a la vez

```adead
let mut x = "Hola"
{
    let r1 = &mut x
    // usar r1
}  // r1 termina aquí
let r2 = &mut x  // ✅ OK ahora
```

---

## ✅ Ejercicios

1. Crea una función que toma `&string` y calcula la longitud
2. Crea una función que toma `&mut string` y agrega un sufijo
3. Experimenta con move vs copy
4. Prueba las reglas de borrowing

---

*Siguiente: [06-Intermedio-Option-Result.md](06-Intermedio-Option-Result.md)*

