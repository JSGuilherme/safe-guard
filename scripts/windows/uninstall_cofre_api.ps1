param(
    [string]$TaskName = "CofreApi",
    [string]$InstallDir = "$env:LOCALAPPDATA\CofreSenhaRust\api",
    [switch]$KeepBinary
)

$ErrorActionPreference = "Stop"


# Remove tray app from autostart
$runRegistryPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
if (Test-Path $runRegistryPath) {
    if (Get-ItemProperty -Path $runRegistryPath -Name $TaskName -ErrorAction SilentlyContinue) {
        Remove-ItemProperty -Path $runRegistryPath -Name $TaskName
        Write-Host "Autostart removido: $TaskName"
    } else {
        Write-Host "Autostart nao encontrado: $TaskName"
    }
}

if (-not $KeepBinary) {
    if (Test-Path $InstallDir) {
        Remove-Item -Path $InstallDir -Recurse -Force
        Write-Host "Diretorio removido: $InstallDir"
    } else {
        Write-Host "Diretorio nao encontrado: $InstallDir"
    }
}

Write-Host "Desinstalacao concluida."
