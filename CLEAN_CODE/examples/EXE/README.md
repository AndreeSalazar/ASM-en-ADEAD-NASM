# 📦 Ejecutables (EXE)

## 📁 Archivos en esta Carpeta

- `test_array_funcional.exe` (258 KB) - **Ejecutable principal**
- `test_array_original.exe` (258 KB) - *Duplicado del anterior*

## 🔗 Origen

**Generado desde:** `CODIGO/test_array_original.c`

**Comando:**
```bash
gcc -O2 -o test_array_funcional.exe ../CODIGO/test_array_original.c
```

## ✅ Propósito

Este ejecutable demuestra que el código funciona correctamente.

**Ejecutar:**
```powershell
.\test_array_funcional.exe
```

**Salida esperada:**
```
1
2
3
```

## 📊 Relación con Otros Archivos

```
test_array_original.c (código fuente)
    │
    └──→ test_array_funcional.exe (este archivo)
            │
            └──→ Genera ASM → dirty.asm → (limpios) → basic.asm, advanced.asm, extreme.asm
```

## 🔍 Comparación

Este ejecutable NO se compara directamente con los ASM limpios porque:
- El ejecutable incluye librerías del sistema (258 KB)
- Los ASM limpios son solo código puro (531 bytes)
- La comparación real es entre los ASM y los OBJ

**Para comparar:** Ve a la carpeta raíz y ejecuta `.\comparar.ps1`

---

**Nota:** Los dos ejecutables son idénticos (mismo tamaño). Puedes eliminar uno si quieres.

