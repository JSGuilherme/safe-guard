param(
    [string]$TaskName = "CofreApi",
    [string]$InstallDir = "$env:LOCALAPPDATA\CofreSenhaRust\api",
    [switch]$KeepBinary
)

$ErrorActionPreference = "Stop"

$task = Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
if ($task) {
    try {
        Stop-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
    } catch {
        # Ignora erro ao parar tarefa inexistente/em transicao.
    }

    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false
    Write-Host "Tarefa removida: $TaskName"
} else {
    Write-Host "Tarefa nao encontrada: $TaskName"
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
