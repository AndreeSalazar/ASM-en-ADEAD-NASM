# Tutorial de ADead

**Autor:** Eddi Andreé Salazar Matos  
**Fecha:** 11 de Diciembre de 2025  
🇵🇪 *Proyecto peruano* 🇵🇪

Un tutorial rápido de 5 minutos para empezar con ADead.

## Instalación

Ver [README.md](../README.md) para instrucciones de instalación.

## Primer Programa

Crea un archivo `hello.ad`:

```adead
print "Hello, ADead!"
```

Compila y ejecuta:

```bash
adeadc compile hello.ad -o hello.asm --run
./hello
```

## Variables

```adead
let name = "ADead"
let age = 1
let total = 10 + 20
```

Las variables son inferidas como `int64` para números y `string` para cadenas.

## Expresiones

Soportamos operaciones aritméticas básicas:

```adead
let a = 5 + 3      // 8
let b = 10 - 2     // 8
let c = 4 * 2      // 8
let d = 16 / 2     // 8
```

Y comparaciones:

```adead
let is_true = 5 > 3      // true (1)
let is_false = 2 == 3    // false (0)
```

## Condicionales

```adead
let x = 10

if x > 5 {
    print "x is greater than 5"
} else {
    print "x is 5 or less"
}
```

## Loops

```adead
let i = 0
while i < 5 {
    print "Iteration: "
    print i
    i = i + 1
}
```

## Funciones

```adead
fn greet(name) {
    print "Hello, "
    print name
}

greet("World")
```

Funciones con retorno:

```adead
fn add(a, b) {
    return a + b
}

let result = add(5, 3)
```

## Ejemplo Completo: Factorial

```adead
let n = 5
let result = 1
let i = 1

while i <= n {
    result = result * i
    i = i + 1
}

print "Factorial of "
print n
print " is "
print result
```

## Próximos Pasos

- Explora los [ejemplos](../examples/)
- Lee la [gramática completa](grammar.md)
- ¡Contribuye al proyecto!

