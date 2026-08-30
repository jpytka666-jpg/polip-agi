# THIS IS VERY IMPORTANT!!!
# ==========================================
# AUTHOR: M. SZUL
# AI MODEL: GPT-5.6 Sol
# TIMESTAMP: 2026-08-29 03:34:00 Europe/London
# REASON FOR CREATION: Install the local Darkstar full-tunnel WireGuard configuration as an automatic Windows tunnel service without committing secrets.
# MECHANICS: Validates elevation, WireGuard installation, local config existence and absence of template placeholders, then installs/starts the WireGuard tunnel service.
# SYSTEM PART: Protected Windows/iOnis network edge
# ARCHITECTURE FUNCTION: Make Darkstar the automatic default Internet path on Windows.
# DEPENDENCIES/LINKS: WireGuard for Windows, local secret-bearing config derived from windows-client.conf.example.
# TECH STACK: PowerShell + WireGuard Windows tunnel service.
# LOCAL WORKSPACE: Windows client
# GIT COMMIT: PENDING
# GITHUB METADATA: jpytka666-jpg/polip-agi / feat/darkstar-gateway
# ==========================================

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$ConfigPath
)

$ErrorActionPreference = 'Stop'

$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($identity)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'Run this installer as Administrator.'
}

$wireguard = Join-Path $env:ProgramFiles 'WireGuard\wireguard.exe'
if (-not (Test-Path -LiteralPath $wireguard)) {
    throw "WireGuard for Windows not found at $wireguard"
}

$config = (Resolve-Path -LiteralPath $ConfigPath).Path
$content = Get-Content -LiteralPath $config -Raw
if ($content -match '<[^>]+>') {
    throw 'Configuration still contains <PLACEHOLDER> values. Refusing to install.'
}

$name = [IO.Path]::GetFileNameWithoutExtension($config)
if ([string]::IsNullOrWhiteSpace($name)) {
    throw 'Unable to derive a WireGuard tunnel name from the configuration file.'
}

$serviceName = "WireGuardTunnel`$$name"
$existing = Get-Service -Name $serviceName -ErrorAction SilentlyContinue
if (-not $existing) {
    & $wireguard /installtunnelservice $config
    if ($LASTEXITCODE -ne 0) {
        throw "WireGuard tunnel service installation failed with exit code $LASTEXITCODE"
    }
}

Start-Service -Name $serviceName
Get-Service -Name $serviceName | Format-Table -AutoSize
Write-Host "Darkstar WireGuard tunnel service is installed and started: $serviceName" -ForegroundColor Green
