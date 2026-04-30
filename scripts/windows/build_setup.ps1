param(
    [string]$Version = "0.1.0",
    [int]$Port = 5474,
    [int]$SessionTtlSecs = 1800,
    [string]$TaskName = "CofreApi",
    [string]$IsccPath = ""
)

$ErrorActionPreference = "Stop"

function Resolve-IsccPath {
    param([string]$ExplicitPath)

    if ($ExplicitPath -and (Test-Path $ExplicitPath)) {
        return (Resolve-Path $ExplicitPath).Path
    }

    $cmd = Get-Command iscc -ErrorAction SilentlyContinue
    if ($cmd) {
        return $cmd.Path
    }

    $cmdUpper = Get-Command ISCC -ErrorAction SilentlyContinue
    if ($cmdUpper) {
        return $cmdUpper.Path
    }

    $commonPaths = @(
        "$env:ProgramFiles(x86)\Inno Setup 6\ISCC.exe",
        "$env:ProgramFiles\Inno Setup 6\ISCC.exe",
        "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe",
        "$env:ProgramFiles(x86)\Inno Setup 5\ISCC.exe",
        "$env:ProgramFiles\Inno Setup 5\ISCC.exe",
        "$env:LOCALAPPDATA\Programs\Inno Setup 5\ISCC.exe"
    )

    foreach ($path in $commonPaths) {
        if (Test-Path $path) {
            return $path
        }
    }

    $registryKeys = @(
        "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\ISCC.exe",
        "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\App Paths\ISCC.exe",
        "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\ISCC.exe",
        "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Inno Setup 6_is1",
        "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Inno Setup 6_is1",
        "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Inno Setup 6_is1",
        "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Inno Setup 5_is1",
        "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Inno Setup 5_is1",
        "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Inno Setup 5_is1"
    )

    foreach ($key in $registryKeys) {
        if (-not (Test-Path $key)) {
            continue
        }

        $item = Get-ItemProperty -Path $key -ErrorAction SilentlyContinue
        if ($null -eq $item) {
            continue
        }

        $candidates = @()
        if ($item.PSObject.Properties.Match("Path").Count -gt 0 -and $item.Path) {
            $candidates += (Join-Path $item.Path "ISCC.exe")
        }
        if ($item.PSObject.Properties.Match("InstallLocation").Count -gt 0 -and $item.InstallLocation) {
            $candidates += (Join-Path $item.InstallLocation "ISCC.exe")
        }
        if ($item.PSObject.Properties.Match("DisplayIcon").Count -gt 0 -and $item.DisplayIcon) {
            $candidates += (($item.DisplayIcon -split ",")[0])
        }

        foreach ($candidate in $candidates) {
            if ($candidate -and (Test-Path $candidate)) {
                return $candidate
            }
        }
    }

    throw "ISCC.exe nao encontrado. Instale Inno Setup 6 ou informe -IsccPath."
}

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$issFile = Join-Path $repoRoot "installer\windows\cofre_api.iss"

if (-not (Test-Path $issFile)) {
    throw "Arquivo .iss nao encontrado: $issFile"
}

Push-Location $repoRoot

try {
    Write-Host "Compilando cofre_api e cofre_tray em release..."
    cargo build --release --bin cofre_api --bin cofre_tray
    if ($LASTEXITCODE -ne 0) {
        throw "Falha ao compilar cofre_api ou cofre_tray."
    }

    $iscc = Resolve-IsccPath -ExplicitPath $IsccPath
    Write-Host "Usando ISCC: $iscc"

    & $iscc "/DMyAppVersion=$Version" "/DApiPort=$Port" "/DSessionTtlSecs=$SessionTtlSecs" "/DTaskName=$TaskName" $issFile
    if ($LASTEXITCODE -ne 0) {
        throw "Falha ao gerar setup.exe com Inno Setup."
    }

    $outputDir = Join-Path $repoRoot "dist\windows"
    Write-Host "Setup gerado em: $outputDir"
}
finally {
    Pop-Location
}
