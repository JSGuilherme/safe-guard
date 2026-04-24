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
        return (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
    }

    return (Get-Location).Path
}

function Register-OrUpdateStartupEntry {
    param(
        [string]$Name,
        [string]$ExePath,
        [int]$ApiPort,
        [int]$Ttl
    )

    if (-not (Test-Path $runRegistryPath)) {
        New-Item -Path $runRegistryPath -Force | Out-Null
    }

    $value = ('"{0}" --port {1} --session-ttl-secs {2}' -f $ExePath, $ApiPort, $Ttl)
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
    Write-Host "Compilando cofre_api em modo release..."
    cargo build --release --bin cofre_api
    if ($LASTEXITCODE -ne 0) {
        throw "Falha ao compilar cofre_api."
    }

    $builtExe = Join-Path $repoRoot "target\release\cofre_api.exe"
    if (-not (Test-Path $builtExe)) {
        throw "Binario nao encontrado em $builtExe"
    }

    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    $installedExe = Join-Path $InstallDir "cofre_api.exe"
    Copy-Item -Path $builtExe -Destination $installedExe -Force

    Register-OrUpdateStartupEntry -Name $TaskName -ExePath $installedExe -ApiPort $Port -Ttl $SessionTtlSecs

    if (-not $DoNotStartNow) {
        Write-Host "Iniciando API em segundo plano..."
        Start-Process -FilePath $installedExe -ArgumentList "--port $Port --session-ttl-secs $SessionTtlSecs" -WindowStyle Hidden

        $healthUrl = "http://127.0.0.1:$Port/api/v1/health"
        if (Wait-ForHealth -Url $healthUrl) {
            Write-Host "API ativa em $healthUrl"
        } else {
            Write-Warning "Startup configurado, mas o healthcheck nao respondeu ainda."
        }
    }

    Write-Host "Instalacao concluida."
    Write-Host "- Binario: $installedExe"
    Write-Host "- Entrada de inicializacao: $TaskName"
    Write-Host "- Inicializacao automatica: Ativa na lista de Aplicativos de Inicializacao do Windows"
}
finally {
    Pop-Location
}
