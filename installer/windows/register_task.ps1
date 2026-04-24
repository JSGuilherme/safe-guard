param(
    [Parameter(Mandatory = $true)]
    [string]$ExePath,
    [string]$TaskName = "CofreApi",
    [int]$Port = 5474,
    [int]$SessionTtlSecs = 1800
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $ExePath)) {
    throw "Executavel nao encontrado: $ExePath"
}

$action = New-ScheduledTaskAction -Execute $ExePath -Argument "--port $Port --session-ttl-secs $SessionTtlSecs"
$trigger = New-ScheduledTaskTrigger -AtLogOn
$settings = New-ScheduledTaskSettingsSet -Hidden -AllowStartIfOnBatteries -StartWhenAvailable -RestartCount 999 -RestartInterval (New-TimeSpan -Minutes 1)

$existing = Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
if ($existing) {
    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false
}

Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger -Settings $settings -Description "Inicia a API local do CofreSenhaRust no logon" -RunLevel Limited | Out-Null

Start-ScheduledTask -TaskName $TaskName
Write-Host "Tarefa registrada e iniciada: $TaskName"
