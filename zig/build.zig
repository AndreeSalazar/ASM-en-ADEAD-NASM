// build.zig - Build system de Zig para compilar los módulos Zig
// Compatible con Zig 0.16.0-dev.1484+
// NOTA: Si este archivo no funciona, usar build-zig.ps1 como alternativa

const std = @import("std");

pub fn build(b: *std.Build) void {
    // NOTA: La API de Zig 0.16.0-dev.1484 tiene cambios
    // Por ahora, usar el script build-zig.ps1 que compila directamente
    
    // Este build.zig necesita corrección según la API exacta
    // Mientras tanto, ejecutar: .\build-zig.ps1
    _ = b;
}
