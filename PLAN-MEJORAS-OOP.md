# 🎯 Plan Completo de Mejoras OOP - ADead

**Fecha:** Diciembre 2025  
**Objetivo:** Mejorar completamente el sistema OOP desde básico hasta avanzado

---

## 📊 ESTADO ACTUAL (Después del Fix del Optimizer)

### ✅ Lo que FUNCIONA

1. **Structs Básicos**
   - ✅ Definición de structs: `struct Nombre { campo1, campo2 }`
   - ✅ Struct literals: `let p = Punto { x: 10, y: 20 }`
   - ✅ Acceso a campos: `p.x`, `p.y`
   - ✅ Asignación a campos: `p.x = 5`
   - ✅ Múltiples instancias independientes

2. **Constructores Básicos**
   - ✅ `fn new()` en structs
   - ✅ `self.campo = valor` en constructores
   - ⚠️ Funciona pero necesita mejoras

3. **Sistema de Generación**
   - ✅ Structs se generan en stack
   - ✅ Offsets de campos calculados correctamente
   - ✅ Layout de memoria funcional

### ⚠️ Lo que FUNCIONA PARCIALMENTE

1. **Métodos de Instancia**
   - ⚠️ `MethodCall` existe pero solo para arrays
   - ⚠️ Métodos de structs/clases no están completamente implementados
   - ⚠️ No hay generación de funciones `fn_StructName_method` para métodos

2. **RAII / Destructores**
   - ⚠️ `destroy` está definido en AST pero no se llama automáticamente
   - ⚠️ No hay tracking automático de destrucción al salir de scope

3. **Constructores**
   - ⚠️ Funcionan pero el código generado puede optimizarse
   - ⚠️ No hay validación de tipos de parámetros

### ❌ Lo que FALTA

1. **Herencia**
   - ❌ `extends` no implementado
   - ❌ `super.metodo()` no implementado
   - ❌ Polimorfismo dinámico

2. **Interfaces/Traits**
   - ❌ `implements` no implementado
   - ❌ Contratos de comportamiento

3. **Métodos Estáticos**
   - ❌ `static fn` no implementado
   - ❌ Llamadas a `StructName.metodo_estatico()`

4. **Visibilidad**
   - ⚠️ `Visibility` enum existe pero no se usa completamente
   - ❌ Métodos privados (`_privado`) no están protegidos

5. **Vtables**
   - ❌ No hay vtables para dispatch dinámico
   - ❌ No hay polimorfismo real

---

## 🚀 PLAN DE MEJORAS (Priorizado)

### FASE 1: Mejorar OOP Básico (PRIORIDAD ALTA) ⚡

#### 1.1 Mejorar Métodos de Instancia

**Problema Actual:**
- `MethodCall` solo funciona para métodos de arrays (append, pop, etc.)
- Métodos de structs/clases no generan código

**Solución:**
```rust
// En generate_expr_windows, mejorar MethodCall:
Expr::MethodCall { object, method, args } => {
    // 1. Detectar si es método de array/string (mantener lógica actual)
    if es_metodo_predefinido(method) {
        // Lógica actual para append, pop, etc.
    } else {
        // 2. Es método de struct/clase
        //    Generar: fn_StructName_method(struct_ptr, args...)
        
        // Evaluar objeto (puntero al struct)
        self.generate_expr_windows(object)?;
        self.text_section.push("    mov rcx, rax  ; self (primer parámetro)".to_string());
        
        // Evaluar argumentos
        for (i, arg) in args.iter().enumerate() {
            self.generate_expr_windows(arg)?;
            let reg = match i {
                0 => "rdx",
                1 => "r8",
                2 => "r9",
                _ => break, // Más de 4 params en stack
            };
            self.text_section.push(format!("    mov {}, rax  ; arg{}", reg, i));
        }
        
        // Llamar a método: fn_StructName_method
        let struct_type = self.get_struct_type_from_expr(object)?;
        let method_label = format!("fn_{}_{}", struct_type, method);
        self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
        self.text_section.push(format!("    call {}", method_label));
        self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
    }
}
```

**Implementación:**
1. Modificar `generate_expr_windows` para manejar métodos de structs
2. Al definir struct con métodos, generar funciones `fn_StructName_method`
3. Asegurar que métodos tienen acceso a `self` como primer parámetro

#### 1.2 Generar Métodos de Struct

**Problema Actual:**
- Los métodos definidos en `Struct { methods: Vec<(String, StructMethod)> }` no se generan

**Solución:**
```rust
// En generate_stmt_windows, en Stmt::Struct:
Stmt::Struct { name, fields, init, destroy, methods } => {
    // ... código actual de registro de struct ...
    
    // Generar métodos de instancia
    for (method_name, method) in methods {
        let method_label = format!("fn_{}_{}", name, method_name);
        
        self.text_section.push(format!("{}:", method_label));
        self.generate_abi_prologue(true);
        
        // self viene en RCX (primer parámetro implícito)
        let self_offset = self.stack_offset;
        self.stack_offset += 8;
        self.variables.insert("self".to_string(), self_offset);
        self.variable_types.insert("self".to_string(), name.clone());
        self.text_section.push(format!("    mov [rbp - {}], rcx  ; guardar self", self_offset + 8));
        
        // Parámetros del método vienen en RDX, R8, R9...
        for (i, param) in method.params.iter().enumerate() {
            // ... guardar parámetros ...
        }
        
        // Generar cuerpo del método
        for s in &method.body {
            self.generate_stmt_windows(s)?;
        }
        
        // Epilogue
        self.generate_abi_epilogue(true);
        self.text_section.push(format!("{}_end:", method_label));
        
        // Limpiar variables
        self.variables.remove("self");
    }
}
```

#### 1.3 Mejorar RAII/Destructores

**Problema Actual:**
- `destroy` está definido pero no se llama automáticamente

**Solución:**
```rust
// En generate_stmt_windows, al salir de scope:
// Si hay variables_to_destroy, llamar a sus destructores

// Al final de una función o bloque:
for (var_name, struct_name) in &self.variables_to_destroy {
    if let Some(has_destroy) = self.structs_with_destroy.get(struct_name) {
        if *has_destroy {
            // Llamar a destroy
            if let Some(&offset) = self.variables.get(var_name) {
                self.text_section.push(format!("    mov rcx, [rbp - {}]  ; cargar {}", offset + 8, var_name));
                self.text_section.push("    sub rsp, 32  ; shadow space".to_string());
                self.text_section.push(format!("    call {}_destroy", struct_name));
                self.text_section.push("    add rsp, 32  ; restaurar shadow space".to_string());
            }
        }
    }
}
```

---

### FASE 2: OOP Intermedio

#### 2.1 Métodos Estáticos

**Implementación:**
```rust
// En AST, StructMethod tiene campo: pub is_static: bool

// Generar métodos estáticos sin self:
if method.is_static {
    // No incluir self en parámetros
    // Llamar como: StructName.metodo() -> fn_StructName_metodo()
} else {
    // Método de instancia con self
}
```

#### 2.2 Visibilidad

**Implementación:**
```rust
// En generate_stmt_windows, verificar visibility:
match method.visibility {
    Visibility::Public => {
        // Generar normalmente
    }
    Visibility::Private => {
        // Solo accesible desde dentro del mismo struct
        // En caller, verificar que estamos dentro del struct
        // (esto requiere contexto de "estructura actual")
    }
}
```

---

### FASE 3: OOP Avanzado

#### 3.1 Herencia Básica

**Implementación:**
```rust
// Struct { parent: Option<String> }

// Layout en memoria:
// [+0]  vtable_ptr (para dispatch dinámico futuro)
// [+8]  campos del padre
// [+16] campos del hijo

// Constructor:
// 1. Llamar a constructor del padre
// 2. Inicializar campos del hijo
```

#### 3.2 Polimorfismo con Vtables

**Implementación:**
```rust
// Vtable layout:
// vtable_Padre:
//   [0] fn_Padre_metodo1
//   [1] fn_Padre_metodo2
//
// vtable_Hijo:
//   [0] fn_Hijo_metodo1  (override)
//   [1] fn_Padre_metodo2 (heredado)

// Llamada virtual:
// obj.metodo() -> 
//   1. Cargar vtable_ptr desde [obj + 0]
//   2. Cargar función desde [vtable_ptr + offset]
//   3. call función
```

---

## 📝 ARCHIVOS A MODIFICAR

### Prioridad ALTA:
1. `CORE/rust/crates/adead-backend/src/lib.rs`
   - `generate_expr_windows`: Mejorar `MethodCall`
   - `generate_stmt_windows`: Generar métodos de struct

2. `CORE/rust/crates/adead-parser/src/lib.rs`
   - Verificar que métodos se parsean correctamente

### Prioridad MEDIA:
3. Crear `CORE/rust/crates/adead-backend/src/oop_helpers.rs`
   - Funciones helper para OOP
   - Manejo de vtables
   - Herencia

### Prioridad BAJA:
4. Tests OOP mejorados
5. Documentación actualizada

---

## ✅ CHECKLIST DE IMPLEMENTACIÓN

### OOP Básico Mejorado:
- [ ] Métodos de instancia funcionan (`obj.metodo()`)
- [ ] Métodos generan código NASM correcto
- [ ] `self` está disponible en métodos
- [ ] Constructores mejorados
- [ ] RAII/Destructores automáticos

### OOP Intermedio:
- [ ] Métodos estáticos (`StructName.metodo()`)
- [ ] Visibilidad (público/privado)
- [ ] Getters/Setters opcionales

### OOP Avanzado:
- [ ] Herencia (`extends`)
- [ ] `super.metodo()`
- [ ] Vtables
- [ ] Polimorfismo dinámico
- [ ] Interfaces/Traits

---

## 🎯 RESULTADO ESPERADO

Después de estas mejoras:

1. **OOP Básico:** Completamente funcional
   - Structs, campos, métodos, constructores, destructores
   - Todo funciona correctamente

2. **OOP Intermedio:** Implementado
   - Métodos estáticos, visibilidad
   - Mejor encapsulamiento

3. **OOP Avanzado:** Funcional
   - Herencia, polimorfismo
   - Sistema completo de OOP

---

**Próximo paso:** Implementar FASE 1 (Mejoras OOP Básico)

