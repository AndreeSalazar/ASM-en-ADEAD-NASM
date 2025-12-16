// Librería estándar de ADead - Funciones predefinidas comunes
// Estas funciones se generan automáticamente y están disponibles en todos los programas

pub struct StdLib;

impl StdLib {
    /// Generar funciones de la librería estándar en NASM
    pub fn generate_stdlib_nasm() -> Vec<String> {
        let mut code = Vec::new();
        
        // ============================================
        // RUNTIME: Librería Estándar (Stdlib)
        // ============================================
        // Funciones predefinidas disponibles en todos los programas
        // Parte del runtime, NO código generado del usuario
        // ============================================
        
        // min(a, b): Retorna el mínimo de dos números
        code.push("; ============================================".to_string());
        code.push("; Librería Estándar ADead - Funciones Predefinidas".to_string());
        code.push("; ============================================".to_string());
        code.push("".to_string());
        
        code.push("stdlib_min:".to_string());
        code.push("    ; min(a, b): Retorna el mínimo de dos números".to_string());
        code.push("    ; Parámetros: RCX = a, RDX = b".to_string());
        code.push("    ; Retorna: RAX = min(a, b)".to_string());
        code.push("    mov rax, rcx  ; a".to_string());
        code.push("    cmp rax, rdx  ; comparar a con b".to_string());
        code.push("    jle .min_done".to_string());
        code.push("    mov rax, rdx  ; b es menor".to_string());
        code.push(".min_done:".to_string());
        code.push("    ret".to_string());
        code.push("".to_string());
        
        // max(a, b): Retorna el máximo de dos números
        code.push("stdlib_max:".to_string());
        code.push("    ; max(a, b): Retorna el máximo de dos números".to_string());
        code.push("    ; Parámetros: RCX = a, RDX = b".to_string());
        code.push("    ; Retorna: RAX = max(a, b)".to_string());
        code.push("    mov rax, rcx  ; a".to_string());
        code.push("    cmp rax, rdx  ; comparar a con b".to_string());
        code.push("    jge .max_done".to_string());
        code.push("    mov rax, rdx  ; b es mayor".to_string());
        code.push(".max_done:".to_string());
        code.push("    ret".to_string());
        code.push("".to_string());
        
        // abs(n): Retorna el valor absoluto
        code.push("stdlib_abs:".to_string());
        code.push("    ; abs(n): Retorna el valor absoluto".to_string());
        code.push("    ; Parámetros: RCX = n".to_string());
        code.push("    ; Retorna: RAX = |n|".to_string());
        code.push("    mov rax, rcx  ; n".to_string());
        code.push("    test rax, rax  ; verificar signo".to_string());
        code.push("    jns .abs_done  ; si no es negativo, ya está bien".to_string());
        code.push("    neg rax  ; negar si es negativo".to_string());
        code.push(".abs_done:".to_string());
        code.push("    ret".to_string());
        code.push("".to_string());
        
        // pow(base, exp): Potencia (solo para exponentes pequeños)
        code.push("stdlib_pow:".to_string());
        code.push("    ; pow(base, exp): Potencia base^exp (exp >= 0)".to_string());
        code.push("    ; Parámetros: RCX = base, RDX = exp".to_string());
        code.push("    ; Retorna: RAX = base^exp".to_string());
        code.push("    push rbx  ; preservar rbx".to_string());
        code.push("    mov rax, 1  ; resultado = 1".to_string());
        code.push("    mov rbx, rcx  ; base".to_string());
        code.push("    mov rcx, rdx  ; exp (contador)".to_string());
        code.push("    test rcx, rcx  ; si exp == 0, retornar 1".to_string());
        code.push("    jz .pow_done".to_string());
        code.push(".pow_loop:".to_string());
        code.push("    imul rax, rbx  ; resultado *= base".to_string());
        code.push("    dec rcx".to_string());
        code.push("    jnz .pow_loop".to_string());
        code.push(".pow_done:".to_string());
        code.push("    pop rbx  ; restaurar rbx".to_string());
        code.push("    ret".to_string());
        code.push("".to_string());
        
        code
    }
}

