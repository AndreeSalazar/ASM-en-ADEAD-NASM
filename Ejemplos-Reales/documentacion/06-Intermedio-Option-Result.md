# 🔧 Nivel Intermedio: Option y Result Types

**Guía sobre manejo seguro de errores y valores opcionales en ADead**

---

## 🎯 Por qué Option/Result?

**Problema tradicional:**
- `null` o `nil` causan crashes (NullPointerException, etc.)
- Errores se manejan con excepciones (difíciles de rastrear)

**Solución ADead (estilo Rust):**
- `Option<T>`: Valores que pueden no existir (sin null)
- `Result<T, E>`: Operaciones que pueden fallar (sin excepciones)

---

## 📦 Option<T> - Valores Opcionales

`Option<T>` representa un valor que puede existir (`Some`) o no (`None`).

### Uso Básico

```adead
// Función que puede no encontrar un valor
fn buscar(nombre: string) -> Option<Persona> {
    if existe_en_db(nombre) {
        return Some(Persona(nombre))  // Valor encontrado
    }
    return None  // No encontrado
}

// Usar el resultado
let persona = buscar("Juan")
match persona {
    Some(p) => {
        print "Encontrado: " + p.nombre
    }
    None => {
        print "No encontrado"
    }
}
```

### Métodos Útiles

```adead
let valor: Option<int64> = Some(42)

// unwrap: obtiene el valor o falla si es None
let x = valor.unwrap()  // 42

// unwrap_or: valor por defecto si es None
let y = None.unwrap_or(0)  // 0

// map: transformar si existe
let doble = Some(21).map(|x| x * 2)  // Some(42)

// is_some / is_none: verificar
if valor.is_some() {
    print "Tiene valor"
}
```

### Pattern Matching con Option

```adead
let resultado = buscar("María")

match resultado {
    Some(p) => {
        print p.nombre
        print p.edad
    }
    None => {
        print "No se encontró"
    }
}

// También puedes usar if-let (syntax sugar)
if let Some(p) = buscar("Juan") {
    print "Encontrado: " + p.nombre
} else {
    print "No encontrado"
}
```

---

## ✅ Result<T, E> - Manejo de Errores

`Result<T, E>` representa una operación que puede tener éxito (`Ok`) o fallar (`Err`).

### Uso Básico

```adead
// Función que puede fallar
fn dividir(a: int64, b: int64) -> Result<int64, string> {
    if b == 0 {
        return Err("División por cero")  // Error
    }
    return Ok(a / b)  // Éxito
}

// Manejar el resultado
let resultado = dividir(10, 2)
match resultado {
    Ok(valor) => {
        print "Resultado: " + valor
    }
    Err(mensaje) => {
        print "Error: " + mensaje
    }
}
```

### Propagación de Errores

```adead
// Operador ?: propaga errores automáticamente
fn calcular() -> Result<int64, string> {
    let a = dividir(10, 2)?    // Si es Err, retorna Err
    let b = dividir(20, 4)?    // Si es Ok, extrae el valor
    return Ok(a + b)
}

// Uso
match calcular() {
    Ok(valor) => print valor
    Err(e) => print "Error: " + e
}
```

### Métodos Útiles

```adead
let resultado: Result<int64, string> = Ok(42)

// unwrap: obtiene Ok o falla
let x = Ok(42).unwrap()  // 42

// unwrap_or: valor por defecto
let y = Err("error").unwrap_or(0)  // 0

// map: transformar Ok
let doble = Ok(21).map(|x| x * 2)  // Ok(42)

// map_err: transformar Err
let error_msg = Err(404).map_err(|code| "Error " + code)
```

---

## 🔄 Combinando Option y Result

```adead
// Función que busca y puede fallar
fn buscar_seguro(id: int64) -> Result<Option<Persona>, string> {
    if id < 0 {
        return Err("ID inválido")
    }
    
    let persona = buscar_por_id(id)
    return Ok(persona)  // Option<Persona> dentro de Result
}

// Manejar ambos casos
match buscar_seguro(123) {
    Ok(Some(p)) => print "Encontrado: " + p.nombre
    Ok(None) => print "No encontrado"
    Err(e) => print "Error: " + e
}
```

---

## 💡 Mejores Prácticas

### 1. Usa Option para valores opcionales

```adead
// ❌ Malo: retornar null
fn buscar() -> Persona? {
    // ...
}

// ✅ Bueno: usar Option
fn buscar() -> Option<Persona> {
    // ...
}
```

### 2. Usa Result para operaciones que pueden fallar

```adead
// ❌ Malo: lanzar excepción
fn dividir(a: int64, b: int64) {
    if b == 0 {
        throw "Error"  // Excepción
    }
}

// ✅ Bueno: usar Result
fn dividir(a: int64, b: int64) -> Result<int64, string> {
    if b == 0 {
        return Err("División por cero")
    }
    return Ok(a / b)
}
```

### 3. Propaga errores con `?`

```adead
// En lugar de anidar match
fn operacion() -> Result<int64, string> {
    let a = dividir(10, 2)?      // Propaga si error
    let b = dividir(20, 4)?      // Propaga si error
    return Ok(a + b)
}
```

---

## 📚 Ejemplos Completos

### Ejemplo 1: Sistema de Archivos

```adead
fn leer_archivo(ruta: string) -> Result<string, string> {
    if !existe(ruta) {
        return Err("Archivo no existe")
    }
    
    let contenido = leer_contenido(ruta)
    match contenido {
        Some(texto) => Ok(texto)
        None => Err("Archivo vacío")
    }
}

match leer_archivo("datos.txt") {
    Ok(texto) => print texto
    Err(e) => print "Error: " + e
}
```

### Ejemplo 2: Parser de Números

```adead
fn parsear_numero(texto: string) -> Result<int64, string> {
    // Intenta parsear
    match intentar_parsear(texto) {
        Some(num) => Ok(num)
        None => Err("No es un número válido: " + texto)
    }
}
```

---

## ✅ Ejercicios

1. Crea una función que busca en un array y retorna `Option<T>`
2. Crea una función que divide y retorna `Result<int64, string>`
3. Usa el operador `?` para propagar errores
4. Combina Option y Result en una función

---

*Siguiente: [07-Intermedio-Arrays.md](07-Intermedio-Arrays.md)*

