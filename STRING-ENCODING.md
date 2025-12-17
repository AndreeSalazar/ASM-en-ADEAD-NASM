# String Encoding - ADead

**Fecha:** Diciembre 2025  
**Estado:** ✅ **OFICIAL**

---

## 🎯 Encoding de Strings

**ADead usa ASCII-only para strings.**

### Limitaciones

- ✅ **Soporta:** ASCII (0-127)
- ❌ **NO soporta:** UTF-8 completo
- ❌ **NO soporta:** Caracteres multibyte
- ❌ **NO soporta:** Emojis, caracteres especiales Unicode

### Implementación

**Strings en ADead:**
- Almacenados como bytes (1 byte por carácter)
- Null-terminated (`\0` al final)
- Length en bytes (no en caracteres)

**Operaciones:**
- `upper()` / `lower()` - Solo convierte A-Z / a-z (ASCII)
- `len()` - Retorna número de bytes
- `slice()` - Opera en bytes, no en caracteres

---

## ⚠️ Advertencias

### No usar caracteres fuera de ASCII

```ad
// ✅ CORRECTO
let s = "Hello World"

// ❌ INCORRECTO (puede causar problemas)
let s = "Hola mundo"  // 'ñ' no es ASCII
let s = "Привет"      // No es ASCII
let s = "Hello 🌍"    // Emoji no soportado
```

### Conversión ASCII

**Caracteres ASCII válidos:**
- Letras: A-Z, a-z
- Números: 0-9
- Símbolos: !@#$%^&*()_+-=[]{}|;:'",.<>?/`~

---

## 🔮 Futuro: UTF-8 Support

**Planificado para futuras versiones:**
- Soporte UTF-8 completo
- Validación de caracteres
- Operaciones en caracteres (no bytes)
- Normalización Unicode

**Por ahora:** ASCII-only es suficiente para la mayoría de casos de uso.

---

**Esta limitación está documentada y es explícita en el código.**

