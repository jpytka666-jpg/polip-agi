<#
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-01 20:30:00
REASON FOR CREATION: Sprawdzenie, czy Windows faktycznie chodzi przez brame Darkstar (Task 6, Step 6.4).
MECHANICS: Wylacznie odczyt. Sprawdza brame 192.168.2.1, hub 192.168.2.2, wyjscie na 8.8.8.8, rozwiazywanie nazwy google.com oraz to, czy istnieje dokladnie jedna uzyteczna trasa domyslna i czy jej nastepny skok to brama Darkstar. Kod wyjscia 0 = wszystko przeszlo, 1 = ktorykolwiek warunek nie zostal spelniony.
SYSTEM PART: deploy/windows - strona kliencka bramy Darkstar.
ARCHITECTURE FUNCTION: Bramka dowodowa dla Enable-DarkstarOnly.ps1; nieprzejscie testu wywoluje wycofanie zmian.
DEPENDENCIES/LINKS: Enable-DarkstarOnly.ps1, Restore-PreviousNetwork.ps1, brama 192.168.2.1, hub EE 192.168.2.2.
TECH STACK: PowerShell 5.1+/7, swiadomie zamiast Rusta - domyslnego jezyka tego projektu.
  (1) MUSI: odczytac tablice tras Windows i stan karty, wykonac cztery proby sieciowe i zwrocic
      jeden kod wyjscia; uruchamiany recznie albo przez Enable-DarkstarOnly, nie przy starcie.
  (2) DLACZEGO NIE RUST: zrodlem prawdy jest Get-NetRoute/Get-NetIPConfiguration po CIM. Rust
      musialby wolac IP Helper API przez FFI albo odpalac te same cmdlety - bez zysku, a testy
      musialyby dzialac na maszynie bez toolchainu, w chwili gdy siec wlasnie nie dziala.
  (3) TRACIMY: typy i testy cargo. Rekompensata: logika oceny jest w czystej funkcji
      Test-DarkstarRouting, ktora da sie odpytac Pesterem bez dotykania sieci.
LOCAL WORKSPACE: D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan
GIT COMMIT: PENDING
GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
==========================================
#>

[CmdletBinding()]
param(
    [string] $Gateway = '192.168.2.1',
    [string] $Hub = '192.168.2.2',
    [string] $PublicProbe = '8.8.8.8',
    [string] $DnsName = 'google.com'
)

Set-StrictMode -Version Latest

# Czysta funkcja oceny - bez dotykania sieci, testowalna Pesterem.
function Test-DarkstarRouting {
    param(
        [Parameter(Mandatory)] [AllowEmptyCollection()] [array] $DefaultRoutes,
        [Parameter(Mandatory)] [string] $ExpectedGateway
    )
    $usable = @($DefaultRoutes | Where-Object { $_.NextHop -and $_.NextHop -ne '0.0.0.0' })
    if ($usable.Count -eq 0) { return @{ ok = $false; reason = 'brak trasy domyslnej' } }
    if ($usable.Count -gt 1) {
        return @{ ok = $false; reason = "wiecej niz jedna trasa domyslna ($($usable.Count))" }
    }
    if ($usable[0].NextHop -ne $ExpectedGateway) {
        return @{ ok = $false; reason = "nastepny skok $($usable[0].NextHop), oczekiwano $ExpectedGateway" }
    }
    return @{ ok = $true; reason = 'jedna trasa domyslna przez brame Darkstar' }
}

$failures = @()

function Check {
    param([string] $Name, [scriptblock] $Probe)
    try {
        $result = & $Probe
        if ($result) { Write-Host ("{0,-28} OK" -f $Name) }
        else { Write-Host ("{0,-28} BLAD" -f $Name); $script:failures += $Name }
    }
    catch {
        Write-Host ("{0,-28} BLAD ({1})" -f $Name, $_.Exception.Message)
        $script:failures += $Name
    }
}

Check "ping brama $Gateway"      { Test-Connection -ComputerName $Gateway -Count 2 -Quiet }
Check "ping hub $Hub"            { Test-Connection -ComputerName $Hub -Count 2 -Quiet }
Check "ping publiczny $PublicProbe" { Test-Connection -ComputerName $PublicProbe -Count 2 -Quiet }
Check "nazwa $DnsName"           { [bool](Resolve-DnsName -Name $DnsName -Type A -ErrorAction Stop) }

$routes = @(Get-NetRoute -DestinationPrefix '0.0.0.0/0' -ErrorAction SilentlyContinue)
$verdict = Test-DarkstarRouting -DefaultRoutes $routes -ExpectedGateway $Gateway
if ($verdict.ok) { Write-Host ("{0,-28} OK ({1})" -f 'trasa domyslna', $verdict.reason) }
else {
    Write-Host ("{0,-28} BLAD ({1})" -f 'trasa domyslna', $verdict.reason)
    $failures += 'trasa domyslna'
}

if ($failures.Count -eq 0) {
    Write-Host 'WYNIK: ZIELONY - Windows chodzi przez brame Darkstar.'
    exit 0
}

Write-Host ("WYNIK: CZERWONY - nie przeszlo: {0}" -f ($failures -join ', '))
exit 1
