# Guía de Contribución

**ADead** - Desarrollado por Eddi Andreé Salazar Matos  
**Fecha de creación:** 11 de Diciembre de 2025  
🇵🇪 *Proyecto peruano* 🇵🇪

¡Gracias por tu interés en contribuir a ADead!

## Cómo Contribuir

### Reportar Bugs

Por favor abre un issue en GitHub con:
- Descripción del bug
- Pasos para reproducir
- Comportamiento esperado vs. actual
- Tu sistema operativo y versión

### Sugerir Features

Abre un issue con la etiqueta `enhancement` y describe:
- Qué problema resuelve
- Cómo debería funcionar
- Ejemplos de uso

### Pull Requests

1. Fork el repositorio
2. Crea una rama (`git checkout -b feature/amazing-feature`)
3. Haz tus cambios
4. Añade tests si es necesario
5. Asegúrate de que los tests pasen (`cargo test`)
6. Commit tus cambios (`git commit -m 'Add amazing feature'`)
7. Push a la rama (`git push origin feature/amazing-feature`)
8. Abre un Pull Request

### Estilo de Código

- Sigue el estilo Rust estándar (`rustfmt`)
- Añade comentarios para código complejo
- Escribe tests para nuevas features
- Documenta funciones públicas

### Estructura del Proyecto

```
crates/
├─ adead-cli/      # CLI tool
├─ adead-parser/   # Parser (AST)
├─ adead-backend/  # Code generation (NASM)
└─ adead-common/   # Shared types
```

### Testing

```bash
# Todos los tests
cargo test --workspace

# Tests de un crate específico
cargo test -p adead-parser

# Ejecutar ejemplos
./target/release/adeadc compile examples/hello.ad -o test.asm --run
```

## Code of Conduct

Por favor sigue nuestro [Code of Conduct](CODE_OF_CONDUCT.md).

