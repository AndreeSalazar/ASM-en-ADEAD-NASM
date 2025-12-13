# 📚 Ejemplos de ADead

Ejemplos organizados por categoría para facilitar el aprendizaje y la referencia.

## 📁 Estructura

```
ejemplos/
├── basicos/          # Ejemplos básicos del lenguaje
├── structs/          # Ejemplos de estructuras
├── oop/              # Ejemplos de OOP (encapsulación, RAII)
└── README.md         # Este archivo
```

---

## 📖 Categorías

### 🟢 Básicos (`basicos/`)

Ejemplos fundamentales del lenguaje:

- **`hello.ad`** - Hello World básico
- **`conditional.ad`** - Condicionales (if/else)
- **`factorial.ad`** - Funciones recursivas
- **`loop-infinito.ad`** - Loops básicos

**Uso:**
```powershell
.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\basicos\hello.ad
```

---

### 🏗️ Structs (`structs/`)

Ejemplos de estructuras y acceso a campos:

- **`structs.ad`** - Definición y uso básico de structs
- **`structs-metodos.ad`** - Structs con métodos

**Uso:**
```powershell
.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\structs\structs.ad
```

---

### 🎯 OOP (`oop/`)

Ejemplos de programación orientada a objetos:

- **`encapsulacion.ad`** - Encapsulación completa (init/destroy, pub/private)
- **`encapsulacion-simple.ad`** - Encapsulación básica (solo campos)
- **`raii-init-destroy.ad`** - RAII (Resource Acquisition Is Initialization)

**Uso:**
```powershell
.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\oop\encapsulacion.ad
```

---

## 🚀 Ejecutar Ejemplos

### Desde la raíz del proyecto:

```powershell
# Ejemplo básico
.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\basicos\hello.ad

# Ejemplo de structs
.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\structs\structs.ad

# Ejemplo de OOP
.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\oop\encapsulacion.ad
```

### Mantener archivos temporales (para debugging):

```powershell
.\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\basicos\hello.ad --keep-temp
```

---

## 📝 Notas

- Todos los ejemplos están probados y funcionan correctamente
- Los archivos `.exe` generados se crean en la misma carpeta que el `.ad`
- Usa `--keep-temp` para mantener `.asm`, `.obj` y otros archivos temporales

---

**Última actualización:** Diciembre 2025

