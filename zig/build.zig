// build.zig - Build system de Zig para compilar los mÃ³dulos Zig
// Compatible con Zig 0.16.0+ (API actualizada)

const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // Crear biblioteca estÃ¡tica de Zig
    // En Zig 0.16.0, se requiere dereferenciar explÃ­citamente con .*
    const lib = b.addStaticLibrary(.{
        .name = "adead_zig",
        .root_source_file = .{ .path = "src/main.zig" },
        .target = target,
        .optimize = optimize,
    });
    
    // Linkear con libc (necesario para FFI con Rust)
    lib.linkLibC();

    // Instalar la biblioteca en zig-out/lib
    b.installArtifact(lib);
}
