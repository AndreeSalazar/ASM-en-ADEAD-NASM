# Tree-sitter Parser para ADead

Parser robusto usando Tree-sitter para ADead. Este parser resuelve problemas con estructuras anidadas complejas (while con if, etc.).

## 🚀 Instalación Completa

### Requisitos
- ✅ Node.js v24.11.1 (instalado)
- ✅ tree-sitter CLI v0.26.3 (instalado globalmente)

### Estructura
```
tree-sitter-adead/
├── grammar.js           # Grammar definition
├── package.json         # Node.js package
├── tree-sitter.json     # Configuración
├── corpus/              # Tests
│   └── statements.txt
└── src/                 # Código C generado
```

## 🔧 Uso

### Generar parser
```bash
cd tree-sitter-adead
tree-sitter generate
```

### Ejecutar tests
```bash
tree-sitter test
```

### Parsear archivo
```bash
tree-sitter parse archivo.ad
```

## 🔗 Integración con Rust

El parser está integrado en `rust/crates/adead-parser/src/tree_sitter_parser.rs`

### Uso desde Rust
```rust
use adead_parser::tree_sitter_parser::TreeSitterParser;

let mut parser = TreeSitterParser::new()?;
let tree = parser.parse(source)?;
```

## 📋 Features Implementadas

- ✅ Print statements
- ✅ Let statements
- ✅ While loops (con bloques anidados)
- ✅ If statements (con else)
- ✅ Function definitions
- ✅ Return statements
- ✅ Binary expressions (con precedencia correcta)
- ✅ Array/Field access
- ✅ Struct literals
- ✅ Comments

## 🎯 Próximos Pasos

1. Configurar build.rs para compilar biblioteca C
2. Implementar conversión AST completa
3. Integrar en flujo principal de compilación

