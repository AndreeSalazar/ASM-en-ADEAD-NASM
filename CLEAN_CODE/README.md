# 🧹 CLEAN CODE - Post-Procesador Optimizador de ASM

**Objetivo:** Transformar ASM generado "sucio" (con overhead de C/GCC) en **ASM virgen puro** directo al CPU.

## 🎯 Filosofía

```
ASM "Sucio" (GCC/Clang generado)
  ↓
CLEAN CODE (Limpieza quirúrgica)
  ↓
ASM Virgen (30-70% menos instrucciones)
  ↓
CPU Directo ⚡
```

**Stack elegido:**
- ✅ **Rust** - Base principal (mismo proyecto, sin overhead)
- ✅ **Regex** - Limpieza quirúrgica de patrones
- ✅ **Peephole Optimizations** - Optimizaciones locales (3-5 líneas)
- ✅ **Agner Fog's objconv** (opcional) - Limpieza avanzada x86

## 🚀 Uso Rápido

### Integración en CLI

```powershell
# Compilar con limpieza automática
.\target\release\adeadc.exe compile ejemplo.ad --clean

# Solo limpiar ASM existente
.\target\release\adeadc.exe clean ejemplo.asm -o ejemplo_clean.asm
```

### Uso como Módulo

```rust
use clean_code::AsmCleaner;

let dirty_asm = std::fs::read_to_string("dirty.asm")?;
let cleaner = AsmCleaner::new();
let clean_asm = cleaner.clean(&dirty_asm)?;
std::fs::write("clean.asm", clean_asm)?;
```

## 🔥 Niveles de Optimización

### Nivel 1: Básico (Default)
- Eliminación de movimientos redundantes
- Optimización de saltos
- Simplificación básica
- Dead code básico

### Nivel 2: Avanzado
- Todo lo anterior +
- Peephole ampliado (ventanas grandes)
- Constant propagation
- Strength reduction

### Nivel 3: EXTREMO 🔥
- Todo lo anterior +
- Data flow analysis
- Integración con Agner Fog's objconv

**Ver [EXTREMO.md](EXTREMO.md) para detalles completos.**

---

## 🔧 Optimizaciones Implementadas

### 1. Eliminación de Movimientos Redundantes
```asm
; ANTES (sucio)
mov rax, rax        ; ❌ Redundante
mov rbx, rbx        ; ❌ Redundante

; DESPUÉS (limpio)
; (eliminado)
```

### 2. Optimización de Saltos
```asm
; ANTES (sucio)
jmp label1
label1:
    mov rax, 5

; DESPUÉS (limpio)
mov rax, 5
```

### 3. Eliminación de Dead Code
```asm
; ANTES (sucio)
push rbp
mov rbp, rsp
; ... código útil ...
pop rbp
ret

; DESPUÉS (limpio)
; (elimina frame setup innecesario si no se usa)
```

### 4. Simplificación de Operaciones
```asm
; ANTES (sucio)
mov rax, 0
add rax, 5

; DESPUÉS (limpio)
mov rax, 5
```

### 5. Eliminación de Labels No Referenciados
```asm
; ANTES (sucio)
unused_label:
    nop
main:
    mov rax, 42

; DESPUÉS (limpio)
main:
    mov rax, 42
```

## 📊 Resultados Esperados

- **30-70% menos instrucciones** en código simple
- **ASM más legible** y fácil de optimizar manualmente
- **Mejor uso de registros** (menos movimientos innecesarios)
- **Código más compacto** (menos overhead)

## 🛠️ Estructura del Módulo

```
CLEAN_CODE/
├── src/
│   ├── lib.rs          # API pública
│   ├── cleaner.rs      # Limpieza principal
│   ├── peephole.rs     # Optimizaciones peephole
│   └── dead_code.rs    # Eliminación de dead code
├── examples/
│   ├── before.asm      # ASM sucio (ejemplo)
│   └── after.asm       # ASM limpio (ejemplo)
├── Cargo.toml
└── README.md
```

## 🔗 Integración con ADead

El módulo se integra directamente en el proyecto Rust existente:

```toml
# En CORE/rust/Cargo.toml
[dependencies]
clean-code = { path = "../../CLEAN_CODE" }
```

## 📝 Próximos Pasos

- [ ] Integración completa con CLI
- [ ] Soporte para Agner Fog's objconv
- [ ] Análisis de data flow avanzado
- [ ] Optimizaciones específicas x86_64
- [ ] Benchmarking de mejoras

---

**Creado:** Diciembre 2025  
**Autor:** Eddi Andreé Salazar Matos  
**Stack:** Rust + Regex + Peephole Optimizations

