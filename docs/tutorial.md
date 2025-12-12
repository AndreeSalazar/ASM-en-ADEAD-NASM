# Tutorial de ADead

**Autor:** Eddi Andreé Salazar Matos  
**Fecha:** Diciembre 2025  
🇵🇪 *Proyecto peruano* 🇵🇪

Tutorial rápido para empezar con ADead.

## Instalación

Ver [README.md](../README.md) para instrucciones de instalación.

## Primer Programa

Crea un archivo `hello.ad`:

```adead
print "Hello, ADead!"
```

### Forma más directa (recomendada):

```bash
# Compila, ensambla, enlaza y ejecuta en un solo paso
adeadc run hello.ad
```

### Forma tradicional (pasos separados):

```powershell
# Windows
.\target\release\adeadc.exe compile hello.ad
.\target\release\adeadc.exe assemble hello.asm
.\target\release\adeadc.exe link hello.obj
.\target\release\adeadc.exe run hello.exe
```

### Opciones avanzadas:

```bash
# Mantener archivos temporales (asm, o, exe)
adeadc run hello.ad --keep-temp
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
let a = 5 + 3
let b = 10 - 2
let c = 4 * 2
let d = 16 / 2
```

Comparaciones:

```adead
let is_true = 5 > 3
let is_false = 2 == 3
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
    print "Iteration"
    i = i + 1
}
```

## Funciones

```adead
fn add(a, b) {
    return a + b
}

let result = add(5, 3)
```

## Structs

```adead
struct Persona {
    edad: int64,
    id: int64
}

let persona = Persona {
    edad: 25,
    id: 1
}

let edad = persona.edad
```

## Próximos Pasos

- Explora los [ejemplos](../Ejemplos-Reales/ejemplos/)
- Lee la [documentación completa](../Ejemplos-Reales/documentacion/)
- Ver [¿Es suficiente para juegos?](respuesta.md)

