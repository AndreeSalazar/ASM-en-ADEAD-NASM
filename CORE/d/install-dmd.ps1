# Script de instalación automática de DMD (D Language)
# Para Windows

Write-Host "=== Instalador de D Language (DMD) ===" -ForegroundColor Cyan

# Verificar si ya está instalado
$dmdInstalled = Get-Command dmd -ErrorAction SilentlyContinue
if ($dmdInstalled) {
    Write-Host "✅ DMD ya está instalado:" -ForegroundColor Green
    dmd --version
    exit 0
}

Write-Host "`nDMD no está instalado. Instalando..." -ForegroundColor Yellow

# URL de descarga para Windows (64-bit)
$downloadUrl = "https://downloads.dlang.org/releases/2024/1/dmd.2.107.1.windows-x86_64.msi"
$installerPath = "$env:TEMP\dmd-installer.msi"

Write-Host "`nDescargando DMD desde dlang.org..." -ForegroundColor Yellow
Write-Host "URL: $downloadUrl" -ForegroundColor Gray

try {
    # Descargar usando Invoke-WebRequest
    Invoke-WebRequest -Uri $downloadUrl -OutFile $installerPath -UseBasicParsing
    Write-Host "✅ Descarga completada" -ForegroundColor Green
    
    Write-Host "`nEjecutando instalador..." -ForegroundColor Yellow
    Write-Host "Por favor sigue las instrucciones del instalador." -ForegroundColor White
    Write-Host "Asegúrate de agregar DMD al PATH durante la instalación." -ForegroundColor White
    
    # Ejecutar instalador
    Start-Process msiexec.exe -ArgumentList "/i `"$installerPath`" /quiet /norestart" -Wait
    
    Write-Host "`n✅ Instalación completada" -ForegroundColor Green
    Write-Host "`nReinicia PowerShell y verifica la instalación con: dmd --version" -ForegroundColor Yellow
    
    # Limpiar
    Remove-Item $installerPath -ErrorAction SilentlyContinue
    
} catch {
    Write-Host "`n❌ Error durante la instalación:" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    Write-Host "`nInstalación manual:" -ForegroundColor Yellow
    Write-Host "1. Descarga desde: https://dlang.org/download.html" -ForegroundColor White
    Write-Host "2. Ejecuta el instalador" -ForegroundColor White
    Write-Host "3. Agrega DMD al PATH" -ForegroundColor White
}

