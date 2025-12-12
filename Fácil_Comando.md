# ⚡ Comandos Fáciles - ADead

Guía ultra-simplificada para ejecutar código ADead en segundos.

---

## 🚀 Ejecutar Cualquier Archivo .ad

### Comando Básico (TODO de una vez)

```powershell
.\rust\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\hello.ad
```

**Eso es todo.** Esto compila (con Zig + Rust), ensambla, enlaza y ejecuta automáticamente.

---

## 🏗️ Arquitectura Actual

El flujo completo es:

```
.ad → Zig (Parsing de structs) + Rust (Parsing resto + Validación + Codegen) → .asm → NASM → .obj → link → .exe
```

**✅ Estado:** Parsing completo funcionando
- **Zig:** Parsing eficiente de structs complejos (implementado en `zig/src/parser_completo.zig`)
- **Rust:** Parsing del resto + validación + codegen (siempre activo)
- **Integración:** Zig compensa las debilidades de Rust en parsing complejo, Rust aporta seguridad y codegen
- **Fallback:** Si Zig no está disponible, usa parser Rust (robusto y completo)

---

## 📝 Ejemplos Rápidos

### Hello World
```powershell
.\rust\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\hello.ad
```

### Factorial
```powershell
.\rust\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\factorial.ad
```

### Conditional (If/Else)
```powershell
.\rust\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\conditional.ad
```

### Loop (While)
```powershell
.\rust\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\loop.ad
```

### Loop Infinito
```powershell
.\rust\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\loop-infinito.ad
```

### Encapsulación (Structs con public/private)
```powershell
.\rust\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\encapsulacion.ad
```

### RAII (Init/Destroy)
```powershell
.\rust\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\raii-init-destroy.ad
```

### Structs Básicos
```powershell
.\rust\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\structs.ad
```

---

## ▶️ Ejecutar el .exe Ya Generado

Si ya compilaste antes, simplemente ejecuta el `.exe`:

```powershell
.\Ejemplos-Reales\compilados\hello.exe
```

---

## 🔍 Ver el Proceso (Paso a Paso)

Si quieres ver cada paso del proceso:

```powershell
.\rust\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\hello.ad --keep-temp
```

Esto muestra cada paso y guarda los archivos `.asm` y `.obj` para revisarlos.

---

## 📁 Dónde Están los Ejecutables

Todos los `.exe` se guardan en:
```
Ejemplos-Reales\compilados\
```

Ejemplos:
- `Ejemplos-Reales\compilados\hello.exe`
- `Ejemplos-Reales\compilados\factorial.exe`
- `Ejemplos-Reales\compilados\conditional.exe`
- `Ejemplos-Reales\compilados\encapsulacion.exe`
- etc.

---

## 💡 Tips Rápidos

1. **Ejecutar rápido:** Usa `run` (hace todo automático: .ad → Zig/Rust → ASM → NASM → link → .exe)
2. **Ver código ASM:** Agrega `--keep-temp` y revisa `Ejemplos-Reales\compilados\*.asm`
3. **Ejecutar .exe:** Directamente `.\Ejemplos-Reales\compilados\nombre.exe`
4. **Tu propio código:** Crea `Ejemplos-Reales\ejemplos\mi-codigo.ad` y ejecútalo igual

---

## 🎯 Plantilla para Tu Código

1. Crea un archivo en `Ejemplos-Reales\ejemplos\tu-archivo.ad`
2. Escribe tu código ADead
3. Ejecuta:
   ```powershell
   .\rust\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\tu-archivo.ad
   ```
4. ¡Listo! Tu programa se ejecuta automáticamente.

---

## ⚡ Atajo de PowerShell (Opcional)

Si usas PowerShell frecuentemente, puedes crear un alias:

```powershell
Set-Alias adead ".\rust\target\release\adeadc.exe"
```

Luego solo ejecutas:
```powershell
adead run Ejemplos-Reales\ejemplos\hello.ad
```

---

## 📋 Resumen Ultra-Rápido

**Para ejecutar cualquier .ad:**
```powershell
.\rust\target\release\adeadc.exe run Ejemplos-Reales\ejemplos\NOMBRE.ad
```

**Para ejecutar el .exe:**
```powershell
.\Ejemplos-Reales\compilados\NOMBRE.exe
```

**Flujo completo automático:**
```powershell
# Compila con Zig (parsing) + Rust (validación + codegen), ensambla con NASM, enlaza y ejecuta
.\rust\target\release\adeadc.exe run tu-archivo.ad
```

**¡Eso es todo!** 🎉

---

## 🔧 Compilación Manual (Si Necesitas)

Si necesitas compilar manualmente:

1. **Compilar Zig:**
   ```powershell
   cd zig
   zig build
   ```

2. **Compilar Rust:**
   ```powershell
   cd rust
   cargo build --release
   ```

3. **Compilar ADead:**
   ```powershell
   .\rust\target\release\adeadc.exe compile tu-archivo.ad
   ```

4. **Ensamblar:**
   ```powershell
   nasm -f win64 -o tu-archivo.obj Ejemplos-Reales\compilados\tu-archivo.asm
   ```

5. **Enlazar:**
   ```powershell
   link /subsystem:console /entry:main tu-archivo.obj /out:tu-archivo.exe
   ```

Pero **normalmente no necesitas hacer esto manualmente** - usa `run` para hacerlo todo automático.
