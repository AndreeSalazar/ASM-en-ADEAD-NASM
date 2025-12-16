//! Ejemplo de uso de CLEAN CODE

use clean_code::AsmCleaner;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ASM sucio de ejemplo
    let dirty_asm = r#"
section .text
    global main

main:
    ; Movimientos redundantes
    mov rax, rax
    mov rbx, rbx
    
    ; Movimiento a cero seguido de add
    mov rax, 0
    add rax, 5
    
    ; Jmp a label inmediatamente siguiente
    jmp label1
label1:
    mov rax, 10
    
    ; Label no usado
unused_label:
    nop
    
    ; Código después de ret (inalcanzable)
    ret
    mov rax, 999
    
    ret
"#;

    println!("=== ASM SUCIO (ANTES) ===");
    println!("{}", dirty_asm);
    println!("\nLíneas: {}\n", dirty_asm.lines().count());

    // Limpiar ASM
    let cleaner = AsmCleaner::new();
    let clean_asm = cleaner.clean(dirty_asm)?;

    println!("=== ASM VIRGEN (DESPUÉS) ===");
    println!("{}", clean_asm);
    println!("\nLíneas: {}", clean_asm.lines().count());

    let reduction = (1.0 - clean_asm.lines().count() as f64 / dirty_asm.lines().count() as f64) * 100.0;
    println!("\nReducción: {:.2}%", reduction);

    Ok(())
}

