# Solución: Comando `adeadc` no encontrado

## Problema

El comando `adeadc` no está en el PATH, por lo que PowerShell no lo encuentra.

## Soluciones

### **Opción 1: Usar ruta completa** (Recomendado)

```powershell
& "C:\Users\andre\OneDrive\Documentos\ASM-en-ADEAD-NASM\CORE\rust\target\release\adeadc.exe" build test.ad --linker zig
```

### **Opción 2: Usar script wrapper**

Se creó un script `adeadc.ps1` en la raíz del proyecto:

```powershell
.\adeadc.ps1 build test.ad --linker zig
```

### **Opción 3: Agregar al PATH (temporal)**

Solo para la sesión actual de PowerShell:

```powershell
$workspace = "C:\Users\andre\OneDrive\Documentos\ASM-en-ADEAD-NASM"
$env:PATH += ";$workspace\CORE\rust\target\release"
adeadc build test.ad --linker zig
```

### **Opción 4: Agregar al PATH (permanente)**

1. Abrir "Variables de entorno" en Windows
2. Editar la variable `PATH` del usuario
3. Agregar: `C:\Users\andre\OneDrive\Documentos\ASM-en-ADEAD-NASM\CORE\rust\target\release`
4. Reiniciar PowerShell

### **Opción 5: Crear alias en PowerShell**

Agregar a tu perfil de PowerShell (`$PROFILE`):

```powershell
function adeadc {
    $exe = "C:\Users\andre\OneDrive\Documentos\ASM-en-ADEAD-NASM\CORE\rust\target\release\adeadc.exe"
    & $exe $args
}
```

Luego usar normalmente:
```powershell
adeadc build test.ad --linker zig
```

---

## Recomendación

**Usar el script wrapper** (`adeadc.ps1`) es la solución más simple y no requiere modificar el PATH.

