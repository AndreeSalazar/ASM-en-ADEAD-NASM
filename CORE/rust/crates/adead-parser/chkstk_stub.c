// Stub para ___chkstk_ms en Windows x64
// Zig genera código que referencia este símbolo con 3 guiones bajos

#ifdef _WIN64
// Función vacía - el nombre del símbolo debe coincidir exactamente con ___chkstk_ms
void ___chkstk_ms(void) {
    // No-op stub - nunca debería ejecutarse porque usamos -fno-stack-check
}
#else
void _chkstk(void) {
    // No-op stub for 32-bit
}
#endif

