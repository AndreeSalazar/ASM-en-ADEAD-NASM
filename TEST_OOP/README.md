# 🧪 Tests OOP Incrementales

Este directorio contiene tests incrementales para verificar y mejorar el sistema OOP de ADead.

## 📋 Plan de Tests

### ✅ Test 1: Struct Básico
- **Archivo:** `test_1_struct_basico.ad`
- **Objetivo:** Verificar structs básicos con campos
- **Estado:** Pendiente

### ✅ Test 2: Método Simple
- **Archivo:** `test_2_metodo_simple.ad`
- **Objetivo:** Verificar `obj.metodo()` básico
- **Estado:** Pendiente

### ✅ Test 3: Constructor
- **Archivo:** `test_3_constructor.ad`
- **Objetivo:** Verificar `fn new()` con parámetros
- **Estado:** Pendiente

### ✅ Test 4: Método con Parámetros
- **Archivo:** `test_4_metodo_con_params.ad`
- **Objetivo:** Verificar `obj.metodo(arg)`
- **Estado:** Pendiente

### ✅ Test 5: Múltiples Instancias
- **Archivo:** `test_5_multiples_instancias.ad`
- **Objetivo:** Verificar independencia de instancias
- **Estado:** Pendiente

## 🚀 Cómo Ejecutar Tests

```powershell
# Compilar el compilador (si no está compilado)
cd CORE\rust
cargo build --release

# Ejecutar un test
cd ..\..\TEST_OOP
..\CORE\rust\target\release\adeadc.exe build test_1_struct_basico.ad -o test_1.exe
.\test_1.exe

# O usar el script de compilación
..\adeadc.ps1 test_1_struct_basico.ad
```

## 📊 Progreso

- [ ] Test 1: Struct básico
- [ ] Test 2: Método simple
- [ ] Test 3: Constructor
- [ ] Test 4: Método con parámetros
- [ ] Test 5: Múltiples instancias

## 🔧 Próximos Pasos

1. Ejecutar cada test y verificar resultados
2. Identificar bugs
3. Arreglar bugs encontrados
4. Agregar más tests según sea necesario

