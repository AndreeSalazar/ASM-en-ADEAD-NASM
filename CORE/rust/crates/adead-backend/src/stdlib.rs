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
        
        // clamp(value, min, max): Limitar valor entre min y max
        code.push("stdlib_clamp:".to_string());
        code.push("    ; clamp(value, min, max): Limitar valor entre min y max".to_string());
        code.push("    ; Parámetros: RCX = value, RDX = min, R8 = max".to_string());
        code.push("    ; Retorna: RAX = clamp(value, min, max)".to_string());
        code.push("    mov rax, rcx  ; value".to_string());
        code.push("    cmp rax, rdx  ; comparar con min".to_string());
        code.push("    jge .clamp_check_max".to_string());
        code.push("    mov rax, rdx  ; value < min, retornar min".to_string());
        code.push("    ret".to_string());
        code.push(".clamp_check_max:".to_string());
        code.push("    cmp rax, r8  ; comparar con max".to_string());
        code.push("    jle .clamp_done".to_string());
        code.push("    mov rax, r8  ; value > max, retornar max".to_string());
        code.push(".clamp_done:".to_string());
        code.push("    ret".to_string());
        code.push("".to_string());
        
        // sign(n): Retorna -1, 0, o 1 según el signo
        code.push("stdlib_sign:".to_string());
        code.push("    ; sign(n): Retorna el signo de n (-1, 0, 1)".to_string());
        code.push("    ; Parámetros: RCX = n".to_string());
        code.push("    ; Retorna: RAX = -1 si n < 0, 0 si n == 0, 1 si n > 0".to_string());
        code.push("    xor rax, rax  ; rax = 0".to_string());
        code.push("    cmp rcx, 0".to_string());
        code.push("    je .sign_done  ; si n == 0, retornar 0".to_string());
        code.push("    jl .sign_neg   ; si n < 0, saltar a negativo".to_string());
        code.push("    mov rax, 1    ; n > 0, retornar 1".to_string());
        code.push("    ret".to_string());
        code.push(".sign_neg:".to_string());
        code.push("    mov rax, -1   ; n < 0, retornar -1".to_string());
        code.push(".sign_done:".to_string());
        code.push("    ret".to_string());
        code.push("".to_string());
        
        // gcd(a, b): Máximo común divisor (algoritmo de Euclides)
        code.push("stdlib_gcd:".to_string());
        code.push("    ; gcd(a, b): Máximo común divisor (Euclides)".to_string());
        code.push("    ; Parámetros: RCX = a, RDX = b".to_string());
        code.push("    ; Retorna: RAX = gcd(a, b)".to_string());
        code.push("    push rbx  ; preservar rbx".to_string());
        code.push("    mov rax, rcx  ; a".to_string());
        code.push("    mov rbx, rdx  ; b".to_string());
        code.push("    ; Hacer valores absolutos".to_string());
        code.push("    test rax, rax".to_string());
        code.push("    jns .gcd_a_pos".to_string());
        code.push("    neg rax".to_string());
        code.push(".gcd_a_pos:".to_string());
        code.push("    test rbx, rbx".to_string());
        code.push("    jns .gcd_loop".to_string());
        code.push("    neg rbx".to_string());
        code.push(".gcd_loop:".to_string());
        code.push("    test rbx, rbx  ; si b == 0, terminar".to_string());
        code.push("    jz .gcd_done".to_string());
        code.push("    xor rdx, rdx  ; limpiar para división".to_string());
        code.push("    div rbx       ; rax = rax / rbx, rdx = rax % rbx".to_string());
        code.push("    mov rax, rbx  ; a = b".to_string());
        code.push("    mov rbx, rdx  ; b = a % b".to_string());
        code.push("    jmp .gcd_loop".to_string());
        code.push(".gcd_done:".to_string());
        code.push("    pop rbx  ; restaurar rbx".to_string());
        code.push("    ret".to_string());
        code.push("".to_string());
        
        // lcm(a, b): Mínimo común múltiplo
        code.push("stdlib_lcm:".to_string());
        code.push("    ; lcm(a, b): Mínimo común múltiplo".to_string());
        code.push("    ; Parámetros: RCX = a, RDX = b".to_string());
        code.push("    ; Retorna: RAX = lcm(a, b) = |a * b| / gcd(a, b)".to_string());
        code.push("    push rbx  ; preservar rbx".to_string());
        code.push("    push r12  ; preservar r12".to_string());
        code.push("    push r13  ; preservar r13".to_string());
        code.push("    mov r12, rcx  ; guardar a".to_string());
        code.push("    mov r13, rdx  ; guardar b".to_string());
        code.push("    ; Calcular gcd(a, b) primero".to_string());
        code.push("    call stdlib_gcd".to_string());
        code.push("    mov rbx, rax  ; rbx = gcd".to_string());
        code.push("    ; Calcular |a * b| / gcd".to_string());
        code.push("    mov rax, r12  ; a".to_string());
        code.push("    imul rax, r13 ; a * b".to_string());
        code.push("    ; Hacer absoluto".to_string());
        code.push("    test rax, rax".to_string());
        code.push("    jns .lcm_pos".to_string());
        code.push("    neg rax".to_string());
        code.push(".lcm_pos:".to_string());
        code.push("    xor rdx, rdx".to_string());
        code.push("    div rbx  ; |a * b| / gcd".to_string());
        code.push("    pop r13".to_string());
        code.push("    pop r12".to_string());
        code.push("    pop rbx".to_string());
        code.push("    ret".to_string());
        code.push("".to_string());
        
        // factorial(n): Factorial de n (n!)
        code.push("stdlib_factorial:".to_string());
        code.push("    ; factorial(n): Calcula n! (factorial)".to_string());
        code.push("    ; Parámetros: RCX = n".to_string());
        code.push("    ; Retorna: RAX = n!".to_string());
        code.push("    push rbx  ; preservar rbx".to_string());
        code.push("    mov rax, 1   ; resultado = 1".to_string());
        code.push("    mov rbx, rcx ; contador = n".to_string());
        code.push("    test rbx, rbx  ; si n <= 0, retornar 1".to_string());
        code.push("    jle .factorial_done".to_string());
        code.push(".factorial_loop:".to_string());
        code.push("    imul rax, rbx  ; resultado *= contador".to_string());
        code.push("    dec rbx".to_string());
        code.push("    jnz .factorial_loop".to_string());
        code.push(".factorial_done:".to_string());
        code.push("    pop rbx  ; restaurar rbx".to_string());
        code.push("    ret".to_string());
        code.push("".to_string());
        
        // is_even(n): Verificar si n es par
        code.push("stdlib_is_even:".to_string());
        code.push("    ; is_even(n): Verifica si n es par".to_string());
        code.push("    ; Parámetros: RCX = n".to_string());
        code.push("    ; Retorna: RAX = 1 si par, 0 si impar".to_string());
        code.push("    mov rax, rcx".to_string());
        code.push("    and rax, 1   ; bit 0".to_string());
        code.push("    xor rax, 1   ; invertir (0 si impar, 1 si par)".to_string());
        code.push("    ret".to_string());
        code.push("".to_string());
        
        // is_odd(n): Verificar si n es impar
        code.push("stdlib_is_odd:".to_string());
        code.push("    ; is_odd(n): Verifica si n es impar".to_string());
        code.push("    ; Parámetros: RCX = n".to_string());
        code.push("    ; Retorna: RAX = 1 si impar, 0 si par".to_string());
        code.push("    mov rax, rcx".to_string());
        code.push("    and rax, 1   ; bit 0 = resultado".to_string());
        code.push("    ret".to_string());
        code.push("".to_string());
        
        code
    }
}

