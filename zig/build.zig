// build.zig - Build system de Zig para compilar los módulos Zig
// Compatible con Zig 0.14.1 (instalado por winget)

const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // Crear biblioteca estática de Zig
    // Para Zig 0.14.1: usar b.path() en lugar de .{ .path = ... }
    const lib = b.addStaticLibrary(.{
        .name = "adead_zig",
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    // Instalar la biblioteca en zig-out/lib
    b.installArtifact(lib);
}
