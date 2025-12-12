# Gramática de ADead

**Autor:** Eddi Andreé Salazar Matos  
**Fecha:** 11 de Diciembre de 2025  
🇵🇪 *Proyecto peruano* 🇵🇪

## Producciones

```
program  ::= stmt*

stmt     ::= "print" expr
           | "let" IDENT "=" expr
           | "if" expr "{" stmt* "}" ["else" "{" stmt* "}"]
           | "while" expr "{" stmt* "}"
           | "fn" IDENT "(" params? ")" "{" stmt* "}"
           | expr
           | "return" expr?

params   ::= IDENT ("," IDENT)*

expr     ::= equality

equality ::= comparison (("==" | "!=") comparison)*

comparison ::= term (("<" | "<=" | ">" | ">=") term)*

term     ::= factor (("+" | "-") factor)*

factor   ::= unary (("*" | "/") unary)*

unary    ::= ("+" | "-") unary | call

call     ::= primary ("(" args? ")")?

args     ::= expr ("," expr)*

primary  ::= NUMBER | STRING | IDENT | "(" expr ")"
```

## Tokens

```
NUMBER   ::= [0-9]+
STRING   ::= '"' (.*?) '"'
IDENT    ::= [a-zA-Z_][a-zA-Z0-9_]*
```

## Keywords

```
print, let, if, else, while, fn, return
```

## Tipos (MVP)

- `int64`: Enteros de 64 bits
- `string`: Cadenas de caracteres

## Ejemplos

### Print

```adead
print "Hello"
```

### Variables

```adead
let x = 42
let name = "ADead"
```

### Expresiones

```adead
let sum = 5 + 3
let product = 2 * 4
let is_greater = 10 > 5
```

### Condicionales

```adead
if x > 0 {
    print "positive"
} else {
    print "negative or zero"
}
```

### Loops

```adead
while i < 10 {
    print i
    i = i + 1
}
```

### Funciones

```adead
fn add(a, b) {
    return a + b
}
```

