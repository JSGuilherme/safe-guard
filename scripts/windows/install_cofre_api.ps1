param(
    [int]$Port = 5474,
    [int]$SessionTtlSecs = 1800,
    [string]$TaskName = "CofreApi",
    [string]$InstallDir = "$env:LOCALAPPDATA\CofreSenhaRust\api",
    [switch]$DoNotStartNow
)

$ErrorActionPreference = "Stop"
$runRegistryPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"

function Assert-CommandExists {
    param([string]$Name)

    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Comando '$Name' nao encontrado. Instale o requisito e tente novamente."
    }
}

function Get-RepoRoot {
    if ($PSScriptRoot) {
        return (Resolve-Path (Join-Path $PSScriptRoot "..\..\")).Path
    }
    return (Get-Location).Path
}


# Register tray app for autostart
function Register-OrUpdateStartupEntry {
    param(
        [string]$Name,
        [string]$TrayExePath
    )
    if (-not (Test-Path $runRegistryPath)) {
        New-Item -Path $runRegistryPath -Force | Out-Null
    }
    $value = '"' + $TrayExePath + '"'
    Set-ItemProperty -Path $runRegistryPath -Name $Name -Value $value -Type String
}

function Wait-ForHealth {
    param([string]$Url)

    $attempts = 10
    for ($i = 0; $i -lt $attempts; $i++) {
        try {
            $resp = Invoke-RestMethod -Method Get -Uri $Url -TimeoutSec 2
            if ($resp.status -eq "ok") {
                return $true
            }
        } catch {
            # Ignora e tenta de novo na proxima iteracao.
        }

        Start-Sleep -Seconds 1
    }

    return $false
}

Assert-CommandExists -Name "cargo"

$repoRoot = Get-RepoRoot
Push-Location $repoRoot

try {
    Write-Host "Compilando cofre_api e cofre_tray em modo release..."
    cargo build --release --bin cofre_api --bin cofre_tray
    if ($LASTEXITCODE -ne 0) {
        throw "Falha ao compilar cofre_api ou cofre_tray."
    }

    $builtApiExe = Join-Path $repoRoot "target\release\cofre_api.exe"
    $builtTrayExe = Join-Path $repoRoot "target\release\cofre_tray.exe"
    if (-not (Test-Path $builtApiExe)) {
        throw "Binario nao encontrado em $builtApiExe"
    }
    if (-not (Test-Path $builtTrayExe)) {
        throw "Binario nao encontrado em $builtTrayExe"
    }

    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    $installedApiExe = Join-Path $InstallDir "cofre_api.exe"
    $installedTrayExe = Join-Path $InstallDir "cofre_tray.exe"
    Copy-Item -Path $builtApiExe -Destination $installedApiExe -Force
    Copy-Item -Path $builtTrayExe -Destination $installedTrayExe -Force

    Register-OrUpdateStartupEntry -Name $TaskName -TrayExePath $installedTrayExe

    if (-not $DoNotStartNow) {
        Write-Host "Iniciando tray app em segundo plano..."
        Start-Process -FilePath $installedTrayExe -WindowStyle Hidden
    }

    Write-Host "Instalacao concluida."
    Write-Host "- Binario API: $installedApiExe"
    Write-Host "- Binario Tray: $installedTrayExe"
    Write-Host "- Entrada de inicializacao: $TaskName"
    Write-Host "- Inicializacao automatica: Ativa na lista de Aplicativos de Inicializacao do Windows"
}
finally {
    Pop-Location
}
