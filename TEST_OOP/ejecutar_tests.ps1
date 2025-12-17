# ============================================
# Script de Ejecución de Tests OOP
# ADead Compiler - Diciembre 2025
# ============================================

$ErrorActionPreference = "Continue"
$compiler = "..\CORE\rust\target\release\adeadc.exe"

Write-Host ""
Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║          ADEAD OOP TESTS - COMPILACIÓN Y EJECUCIÓN         ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Verificar que existe el compilador
if (-not (Test-Path $compiler)) {
    Write-Host "ERROR: Compilador no encontrado en $compiler" -ForegroundColor Red
    Write-Host "Ejecuta 'cargo build --release' en CORE/rust primero" -ForegroundColor Yellow
    exit 1
}

# Lista de tests
$tests = @(
    "01_struct_simple",
    "02_struct_multi_campos",
    "03_struct_acceso_campos",
    "04_struct_literal",
    "05_struct_multiple_instancias",
    "06_class_new_simple",
    "07_class_new_params",
    "08_class_self",
    "09_class_metodo_simple",
    "10_class_metodo_params",
    "11_class_metodo_return",
    "12_raii_init_destroy"
)

$pasados = 0
$fallidos = 0

foreach ($test in $tests) {
    $adFile = "$test.ad"
    $asmFile = "$test.asm"
    
    if (-not (Test-Path $adFile)) {
        Write-Host "⏭️  SKIP: $adFile no existe" -ForegroundColor Yellow
        continue
    }
    
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor DarkGray
    Write-Host "🔧 Compilando: $adFile" -ForegroundColor White
    
    # Compilar a ASM
    $output = & $compiler compile $adFile -o $asmFile 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ Compilación exitosa → $asmFile" -ForegroundColor Green
        $pasados++
        
        # Mostrar primeras líneas del ASM generado
        if (Test-Path $asmFile) {
            Write-Host "   📄 Preview ASM (primeras 10 líneas):" -ForegroundColor DarkCyan
            Get-Content $asmFile | Select-Object -First 10 | ForEach-Object {
                Write-Host "      $_" -ForegroundColor DarkGray
            }
        }
    } else {
        Write-Host "   ❌ Error de compilación" -ForegroundColor Red
        Write-Host $output -ForegroundColor Red
        $fallidos++
    }
    
    Write-Host ""
}

Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor DarkGray
Write-Host ""
Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║                      RESUMEN DE TESTS                       ║" -ForegroundColor Cyan
Write-Host "╠════════════════════════════════════════════════════════════╣" -ForegroundColor Cyan
Write-Host "║  ✅ Pasados:  $pasados                                           ║" -ForegroundColor Green
Write-Host "║  ❌ Fallidos: $fallidos                                           ║" -ForegroundColor $(if ($fallidos -eq 0) { "Green" } else { "Red" })
Write-Host "║  📊 Total:    $($pasados + $fallidos)                                          ║" -ForegroundColor White
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

if ($fallidos -eq 0) {
    Write-Host "🎉 ¡TODOS LOS TESTS OOP PASARON!" -ForegroundColor Green
} else {
    Write-Host "⚠️  Hay $fallidos tests que necesitan revisión" -ForegroundColor Yellow
}

