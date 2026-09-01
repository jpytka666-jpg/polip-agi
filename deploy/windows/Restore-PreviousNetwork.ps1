<#
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-01 20:35:00
REASON FOR CREATION: Przywrocenie stanu sieci Windows sprzed przelaczenia na brame Darkstar (Task 6, Step 6.5).
MECHANICS: Czyta plik JSON zapisany przez Enable-DarkstarOnly.ps1 i odtwarza adresacje karty: powrot do adresu automatycznego albo do zapisanych adresow statycznych, serwery nazw, metryke karty oraz tryb laczenia profili bezprzewodowych. Brak pliku stanu konczy prace bez zmian - fail-closed. Profil Darkstar nie jest kasowany.
SYSTEM PART: deploy/windows - strona kliencka bramy Darkstar.
ARCHITECTURE FUNCTION: Jedyna zaplanowana droga wycofania Task 6; wolana takze automatycznie przez Enable-DarkstarOnly przy nieudanym tescie.
DEPENDENCIES/LINKS: Enable-DarkstarOnly.ps1 (zrodlo pliku stanu), Test-DarkstarOnly.ps1.
TECH STACK: PowerShell 5.1+/7, swiadomie zamiast Rusta - domyslnego jezyka tego projektu.
  (1) MUSI: odtworzyc adres, brame, serwery nazw i metryke karty Windows z pliku zapisanego
      wczesniej; uruchamiany recznie przez operatora, ktory prawdopodobnie wlasnie stracil siec.
  (2) DLACZEGO NIE RUST: odtwarzane sa dokladnie te same obiekty CIM (MSFT_NetIPAddress,
      MSFT_NetIPInterface, MSFT_DNSClientServerAddress), ktore zapisal Enable. Rust wymagalby
      FFI do IP Helper albo wolania tych cmdletow - a narzedzie ratunkowe musi dzialac na
      maszynie bez toolchainu i bez sieci do pobrania czegokolwiek.
  (3) TRACIMY: typy i testy cargo. Rekompensata: plik stanu jest zwyklym JSON-em, ktory
      operator moze przeczytac i odtworzyc recznie, gdyby skrypt zawiodl.
LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
GIT COMMIT: PENDING
GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
==========================================
#>

[CmdletBinding()]
param(
    [string] $SnapshotPath,
    [string] $BackupDir = "$env:USERPROFILE\darkstar-net-backup",
    [switch] $WhatIfOnly
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Assert-Elevated {
    $id = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($id)
    if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        throw 'Wymagana konsola z uprawnieniami administratora.'
    }
}

# Brak pliku stanu = brak wiedzy o tym, do czego wracac. Konczymy bez zmian.
if (-not $SnapshotPath) {
    $latest = Join-Path $BackupDir 'latest.txt'
    if (Test-Path $latest) { $SnapshotPath = (Get-Content $latest -Raw).Trim() }
}
if (-not $SnapshotPath -or -not (Test-Path $SnapshotPath)) {
    throw "Brak pliku stanu do przywrocenia (szukano: $SnapshotPath). Nic nie zmieniono."
}

$snap = Get-Content $SnapshotPath -Raw | ConvertFrom-Json
Write-Host "Przywracam stan z $SnapshotPath (zapisany $($snap.savedAt))"
Write-Host "Karta: $($snap.adapterName), ifIndex $($snap.ifIndex), DHCP przed zmiana: $($snap.dhcpEnabled)"

if ($WhatIfOnly) {
    Write-Host 'WhatIfOnly: nic nie zmieniono. Zawartosc stanu powyzej.'
    return
}

Assert-Elevated

$nic = Get-NetAdapter -Name $snap.adapterName -ErrorAction Stop

# Czyscimy biezaca adresacje karty przed odtworzeniem zapisanej.
Get-NetIPAddress -InterfaceIndex $nic.ifIndex -AddressFamily IPv4 -ErrorAction SilentlyContinue |
    Remove-NetIPAddress -Confirm:$false -ErrorAction SilentlyContinue
Get-NetRoute -InterfaceIndex $nic.ifIndex -DestinationPrefix '0.0.0.0/0' -ErrorAction SilentlyContinue |
    Remove-NetRoute -Confirm:$false -ErrorAction SilentlyContinue

if ($snap.dhcpEnabled -eq 'Enabled') {
    Write-Host 'Powrot do adresu nadawanego automatycznie.'
    Set-NetIPInterface -InterfaceIndex $nic.ifIndex -AddressFamily IPv4 -Dhcp Enabled
    Set-DnsClientServerAddress -InterfaceIndex $nic.ifIndex -ResetServerAddresses
}
else {
    Write-Host 'Powrot do zapisanej adresacji statycznej.'
    foreach ($a in $snap.ipv4Addresses) {
        New-NetIPAddress -InterfaceIndex $nic.ifIndex -IPAddress $a.address `
            -PrefixLength $a.prefixLength -AddressFamily IPv4 -ErrorAction Stop | Out-Null
    }
    foreach ($gw in $snap.defaultGateways) {
        if ($gw) {
            New-NetRoute -InterfaceIndex $nic.ifIndex -DestinationPrefix '0.0.0.0/0' `
                -NextHop $gw -ErrorAction SilentlyContinue | Out-Null
        }
    }
    if ($snap.dnsServers -and $snap.dnsServers.Count -gt 0) {
        Set-DnsClientServerAddress -InterfaceIndex $nic.ifIndex -ServerAddresses $snap.dnsServers
    }
}

# Metryka karty wraca do zapisanej wartosci albo do trybu automatycznego.
if ($snap.automaticMetric -eq 'Enabled') {
    Set-NetIPInterface -InterfaceIndex $nic.ifIndex -AddressFamily IPv4 -AutomaticMetric Enabled
}
else {
    Set-NetIPInterface -InterfaceIndex $nic.ifIndex -AddressFamily IPv4 -InterfaceMetric $snap.interfaceMetric
}

# Profile bezprzewodowe wracaja do automatycznego laczenia. Zaden profil nie jest kasowany.
foreach ($p in $snap.wlanProfiles) {
    netsh wlan set profileparameter name="$p" connectionmode=auto | Out-Null
}

Write-Host 'Przywrocono. Sprawdz laczosc; profil Darkstar pozostal nietkniety.'
