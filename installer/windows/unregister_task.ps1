param(
    [string]$TaskName = "CofreApi"
)

$ErrorActionPreference = "Stop"

$existing = Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
if ($existing) {
    try {
        Stop-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
    } catch {
        # Ignora erros durante parada.
    }

    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false
    Write-Host "Tarefa removida: $TaskName"
}
