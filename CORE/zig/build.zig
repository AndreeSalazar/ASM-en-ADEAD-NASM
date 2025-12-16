const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{
        .default_target = .{
            .cpu_arch = .x86_64,
            .os_tag = .windows,
            .abi = .msvc,
        },
    });

    const optimize = b.standardOptimizeOption(.{});

    // Crear biblioteca estática
    const lib = b.addStaticLibrary(.{
        .name = "adead_zig",
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    // Configurar para Windows x64
    lib.addCSourceFlags(&[_][]const u8{
        "-fno-stack-check",
    });
    
    // Linkear con libc
    lib.linkLibC();
    
    // Asegurar que las funciones se exporten correctamente
    lib.force_pic = false;
    lib.bundle_compiler_rt = false;
    
    // Instalar la biblioteca
    b.installArtifact(lib);
    
    // Crear un test opcional
    const tests = b.addTest(.{
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });
    
    const run_tests = b.addRunArtifact(tests);
    const test_step = b.step("test", "Run unit tests");
    test_step.dependOn(&run_tests.step);
}
