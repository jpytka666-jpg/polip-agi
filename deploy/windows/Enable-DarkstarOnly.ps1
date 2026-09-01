<#
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-01 20:25:00
REASON FOR CREATION: Przelaczenie Windows na siec wylacznie przez brame Darkstar, z zapisem stanu poprzedniego do odwrocenia (Task 6, Step 6.2 i 6.3).
MECHANICS: Najpierw zapisuje pelny stan karty i tras do pliku JSON poza repozytorium, potem sprawdza osiagalnosc bramy, ustawia adres statyczny 192.168.2.50/24 z brama 192.168.2.1 i serwerem nazw 8.8.8.8, obniza metryke karty Darkstar i wylacza automatyczne laczenie profili domowych. Kazde niepowodzenie po zapisie stanu wywoluje Restore-PreviousNetwork.ps1.
SYSTEM PART: deploy/windows - strona kliencka bramy Darkstar.
ARCHITECTURE FUNCTION: Realizuje przejscie z Task 6 w sposob odwracalny; bez pliku stanu skrypt nie robi nic.
DEPENDENCIES/LINKS: Test-DarkstarOnly.ps1, Restore-PreviousNetwork.ps1, brama 192.168.2.1 (profil DARKSTAR-WiFi na CBMS), hub EE 192.168.2.2.
TECH STACK: PowerShell 5.1+/7, swiadomie zamiast Rusta - domyslnego jezyka tego projektu.
  (1) MUSI: przestawic adres, brame, serwer nazw i metryke karty Windows, odczytac profile
      bezprzewodowe i trasy domyslne, a przy bledzie cofnac wszystko; uruchamiany recznie
      z podniesionej konsoli operatora, bez startu przy systemie i bez wpiecia w Darkstar Core.
  (2) DLACZEGO NIE RUST: te operacje to cmdlety MSFT_Net* po WMI/CIM (New-NetIPAddress,
      Set-NetIPInterface, Set-DnsClientServerAddress). W Rust trzeba by wolac IP Helper API
      przez FFI albo opakowywac te same cmdlety w Command::new("powershell") - czyli i tak
      PowerShell, tylko schowany za warstwa. Operator ratujacy siec ma miec plik tekstowy,
      ktory czyta i uruchamia na miejscu, a nie binarke wymagajaca kompilacji krzyzowej na
      maszynie, ktora wlasnie stracila laczosc.
  (3) TRACIMY: kontrole typow, testy cargo, jeden artefakt. Rekompensata: brak zaleznosci,
      czytelnosc dla operatora i logika czysta wydzielona do Test-DarkstarOnly.ps1.
LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
GIT COMMIT: PENDING
GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
==========================================
#>

[CmdletBinding()]
param(
    [string] $AdapterName = 'WiFi 2',
    [string] $Ssid = 'EE-57GMTG',
    [string] $IPAddress = '192.168.2.50',
    [int]    $PrefixLength = 24,
    [string] $Gateway = '192.168.2.1',
    [string[]] $DnsServers = @('8.8.8.8'),
    # Katalog kopii stanu MUSI lezec poza repozytorium.
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

# Zapis stanu przed zmiana. Klucze Wi-Fi nie sa odczytywane ani zapisywane.
function Save-NetworkSnapshot {
    param([string] $Path, [string] $Adapter)

    $nic = Get-NetAdapter -Name $Adapter -ErrorAction Stop
    $cfg = Get-NetIPConfiguration -InterfaceIndex $nic.ifIndex -ErrorAction Stop
    $ipv4 = Get-NetIPAddress -InterfaceIndex $nic.ifIndex -AddressFamily IPv4 -ErrorAction SilentlyContinue
    $iface = Get-NetIPInterface -InterfaceIndex $nic.ifIndex -AddressFamily IPv4 -ErrorAction Stop

    $snapshot = [ordered]@{
        savedAt         = (Get-Date).ToString('s')
        adapterName     = $nic.Name
        ifIndex         = $nic.ifIndex
        adapterStatus   = [string]$nic.Status
        macAddress      = $nic.MacAddress
        dhcpEnabled     = [string]$iface.Dhcp
        interfaceMetric = $iface.InterfaceMetric
        automaticMetric = [string]$iface.AutomaticMetric
        ipv4Addresses   = @($ipv4 | ForEach-Object {
                @{ address = $_.IPAddress; prefixLength = $_.PrefixLength; origin = [string]$_.PrefixOrigin }
            })
        defaultGateways = @($cfg.IPv4DefaultGateway | ForEach-Object { $_.NextHop })
        dnsServers      = @((Get-DnsClientServerAddress -InterfaceIndex $nic.ifIndex -AddressFamily IPv4).ServerAddresses)
        # Nazwy profili bezprzewodowych; hasla NIE sa odczytywane (zadnego key=clear).
        wlanProfiles    = @(netsh wlan show profiles |
                Select-String 'All User Profile\s*:\s*(.+)$' |
                ForEach-Object { $_.Matches[0].Groups[1].Value.Trim() })
        connectedSsid   = (netsh wlan show interfaces |
                Select-String '^\s*SSID\s*:\s*(.+)$' |
                ForEach-Object { $_.Matches[0].Groups[1].Value.Trim() } | Select-Object -First 1)
        defaultRoutesV4 = @(Get-NetRoute -DestinationPrefix '0.0.0.0/0' -ErrorAction SilentlyContinue |
                ForEach-Object { @{ ifIndex = $_.ifIndex; nextHop = $_.NextHop; metric = $_.RouteMetric } })
        defaultRoutesV6 = @(Get-NetRoute -DestinationPrefix '::/0' -ErrorAction SilentlyContinue |
                ForEach-Object { @{ ifIndex = $_.ifIndex; nextHop = $_.NextHop; metric = $_.RouteMetric } })
    }

    $snapshot | ConvertTo-Json -Depth 6 | Set-Content -Path $Path -Encoding UTF8
    return $Path
}

Assert-Elevated

if (-not (Test-Path $BackupDir)) { New-Item -ItemType Directory -Path $BackupDir -Force | Out-Null }
$snapshotPath = Join-Path $BackupDir ("network-snapshot-{0:yyyyMMdd-HHmmss}.json" -f (Get-Date))

$nic = Get-NetAdapter -Name $AdapterName -ErrorAction Stop
Write-Host "Karta: $($nic.Name) (ifIndex $($nic.ifIndex)), status $($nic.Status)"

Save-NetworkSnapshot -Path $snapshotPath -Adapter $AdapterName | Out-Null
Write-Host "Stan przed zmiana zapisany: $snapshotPath"

# Wskaznik dla Restore: ostatnia kopia.
Set-Content -Path (Join-Path $BackupDir 'latest.txt') -Value $snapshotPath -Encoding UTF8

if ($WhatIfOnly) {
    Write-Host 'WhatIfOnly: zapisano wylacznie stan, nic nie zmieniono.'
    return
}

try {
    # Brama musi odpowiadac ZANIM cokolwiek przestawimy.
    if (-not (Test-Connection -ComputerName $Gateway -Count 2 -Quiet)) {
        throw "Brama $Gateway nie odpowiada - przerwano przed jakakolwiek zmiana."
    }

    # Stan docelowy moze juz obowiazywac. Wtedy NIE ruszamy dzialajacej sieci -
    # skrypt jest idempotentny i nigdy nie zrywa polaczenia, ktore jest poprawne.
    $hasAddress = [bool](Get-NetIPAddress -InterfaceIndex $nic.ifIndex -AddressFamily IPv4 `
            -ErrorAction SilentlyContinue |
            Where-Object { $_.IPAddress -eq $IPAddress -and $_.PrefixLength -eq $PrefixLength })
    $hasGateway = [bool](Get-NetRoute -InterfaceIndex $nic.ifIndex -DestinationPrefix '0.0.0.0/0' `
            -ErrorAction SilentlyContinue | Where-Object { $_.NextHop -eq $Gateway })
    $currentDns = @((Get-DnsClientServerAddress -InterfaceIndex $nic.ifIndex -AddressFamily IPv4).ServerAddresses)
    $hasDns = -not (Compare-Object -ReferenceObject @($DnsServers) -DifferenceObject $currentDns)

    if ($hasAddress -and $hasGateway -and $hasDns) {
        Write-Host 'Stan docelowy juz obowiazuje - nic nie zmieniam.'
        Write-Host "  adres   $IPAddress/$PrefixLength"
        Write-Host "  brama   $Gateway"
        Write-Host "  nazwy   $($currentDns -join ', ')"
    }
    else {
        Write-Host "Utrwalam $IPAddress/$PrefixLength, brama $Gateway"
        if (-not $hasAddress) {
            # Usuwamy wylacznie adresy INNE niz docelowy - docelowy zostaje na miejscu.
            Get-NetIPAddress -InterfaceIndex $nic.ifIndex -AddressFamily IPv4 -ErrorAction SilentlyContinue |
                Where-Object { $_.IPAddress -ne $IPAddress } |
                Remove-NetIPAddress -Confirm:$false -ErrorAction SilentlyContinue
            New-NetIPAddress -InterfaceIndex $nic.ifIndex -IPAddress $IPAddress `
                -PrefixLength $PrefixLength -AddressFamily IPv4 -ErrorAction Stop | Out-Null
        }
        if (-not $hasGateway) {
            Get-NetRoute -InterfaceIndex $nic.ifIndex -DestinationPrefix '0.0.0.0/0' -ErrorAction SilentlyContinue |
                Where-Object { $_.NextHop -ne $Gateway } |
                Remove-NetRoute -Confirm:$false -ErrorAction SilentlyContinue
            New-NetRoute -InterfaceIndex $nic.ifIndex -DestinationPrefix '0.0.0.0/0' `
                -NextHop $Gateway -ErrorAction Stop | Out-Null
        }
        if (-not $hasDns) {
            Set-DnsClientServerAddress -InterfaceIndex $nic.ifIndex -ServerAddresses $DnsServers
        }
    }

    # Karta Darkstar ma byc preferowana; metryki reszty zostaja nietkniete.
    Set-NetIPInterface -InterfaceIndex $nic.ifIndex -AddressFamily IPv4 -InterfaceMetric 10

    # Profile domowe: tylko wylaczenie automatycznego laczenia. NIGDY nie kasujemy profilu.
    foreach ($p in (netsh wlan show profiles |
            Select-String 'All User Profile\s*:\s*(.+)$' |
            ForEach-Object { $_.Matches[0].Groups[1].Value.Trim() })) {
        if ($p -ne $Ssid) {
            netsh wlan set profileparameter name="$p" connectionmode=manual | Out-Null
        }
    }

    Start-Sleep -Seconds 3
    $test = Join-Path $PSScriptRoot 'Test-DarkstarOnly.ps1'
    if (Test-Path $test) {
        & $test -Gateway $Gateway
        if ($LASTEXITCODE -ne 0) { throw "Test-DarkstarOnly zwrocil kod $LASTEXITCODE" }
    }

    Write-Host 'Gotowe: Windows chodzi przez brame Darkstar.'
}
catch {
    Write-Warning "Blad: $($_.Exception.Message)"
    Write-Warning 'Cofam zmiany z zapisanego stanu.'
    $restore = Join-Path $PSScriptRoot 'Restore-PreviousNetwork.ps1'
    if (Test-Path $restore) { & $restore -SnapshotPath $snapshotPath }
    else { Write-Warning "Brak $restore - cofnij recznie ze stanu: $snapshotPath" }
    throw
}
