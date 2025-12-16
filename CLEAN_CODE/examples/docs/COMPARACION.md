# 📊 Comparación: ASM Sucio vs ASM Limpio

## 🔧 Compilación Diferencial

Este documento muestra los resultados de compilar ambos archivos ASM (sucio y limpio) y comparar los resultados.

## Ejecutar Comparación

### Windows (PowerShell):

```powershell
cd CLEAN_CODE\examples
.\compilar_y_comparar.ps1
```

### Linux/Mac (Bash):

```bash
cd CLEAN_CODE/examples
chmod +x compilar_y_comparar.sh
./compilar_y_comparar.sh
```

## Resultados Esperados

### Tamaños de Archivos

| Archivo | Sucio | Limpio | Reducción |
|---------|-------|--------|-----------|
| ASM (.asm) | ~4,249 B | ~531 B | ~87.3% |
| Objeto (.obj) | ~X KB | ~Y KB | ~Z% |
| Ejecutable (.exe) | ~X KB | ~Y KB | ~Z% |

### Líneas de Código

- **Sucio:** 204 líneas
- **Limpio:** 26 líneas
- **Reducción:** 87.3%

## Análisis

### Lo que se eliminó:

1. ✅ Metadatos de Clang (`.def`, `.scl`, `.type`, `.endef`)
2. ✅ Comentarios de debug (`# %bb.0:`, `# -- Begin function`)
3. ✅ Secciones de debug (`.section .debug$S`)
4. ✅ Información del compilador
5. ✅ Líneas vacías y espacios redundantes

### Lo que se mantuvo:

✅ Código funcional esencial
✅ Labels importantes
✅ Instrucciones de código
✅ Estructura del programa

## Conclusión

El ASM limpio debería:
- ✅ Ser más pequeño (87% menos líneas)
- ✅ Compilar a ejecutables más pequeños
- ✅ Mantener la misma funcionalidad
- ✅ Ser más fácil de leer y optimizar

---

**Nota:** Los ejecutables pueden tener tamaños similares porque el linker agrega código de inicialización y librerías estándar. La diferencia real está en el código ASM generado.

