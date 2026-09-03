# darkstar-header-v1
# po co: push-chroma-to-cbms.ps1
# nie wolno: pisac na dysk E, ruszac zapory Windows, zmieniac profilu sieciowego karty
# Autor: Marcin Szul
# Model: Claude
# Czas: 2026-09-03T01:35:00+01:00
# Po co: Przenosi sklad Chromy z dysku E na CBMS, zeby druga noga pamieci miala WLASNE
#        dane po stronie Ubuntu i zyla takze wtedy, gdy Windows jest wylaczony.
#
# THIS IS VERY IMPORTANT!!!
# ==========================================
# MECHANICS: Kierunek jest odwrocony wzgledem pierwotnego planu i to jest sedno. CBMS NIE
# MA JAK siegnac do Windowsa: karta z adresem 192.168.2.50 ma profil sieciowy Public, wiec
# zapora Windows odrzuca wszystko przychodzace - zmierzone, ping oraz porty 445 i 8000
# milcza. Ruch WYCHODZACY z Windowsa nie jest blokowany, wiec to Windows wypycha dane
# przez SSH, ktore i tak dziala. Zadna zapora nie zostaje przy tym dotknieta.
# Kopiowanie idzie dwuetapowo: najpierw robocopy na dysk tymczasowy, dopiero z niego scp.
# Powod: sklad Chromy to zywa baza SQLite i kopiowanie jej wprost w trakcie zapisu daje
# plik rozdarty w polowie transakcji.
# SYSTEM PART: deploy/context - druga noga pamieci.
# ARCHITECTURE FUNCTION: Jedyna droga zasilania kopii na CBMS. Czyta dysk zrodlowy i nie
# zapisuje na nim ani jednego bajtu; katalog roboczy powstaje w TEMP.
# DEPENDENCIES/LINKS: scp, ssh i robocopy (wszystkie obecne w systemie - sprawdzone),
# konto owner na bramie prywatnej.
# TECH STACK: PowerShell 7, swiadomie zamiast Rusta - domyslnego jezyka tego projektu.
#   (1) MUSI: czytac sciezke Windows ze spacja w nazwie, radzic sobie z zywym plikiem
#       bazy (robocopy), uzyc scp i ssh z kluczem juz skonfigurowanym dla tego konta,
#       i dac sie uruchomic doraznie przez operatora bez etapu budowania.
#   (2) DLACZEGO NIE RUST: to trzy polecenia systemowe uruchamiane recznie na maszynie
#       operatora, a nie usluga. Rust wymagalby kompilacji skrosnej na Windows i
#       dostarczenia binarki na jego komputer, po czym i tak wywolywalby robocopy oraz
#       scp - bo napisanie w Ruscie wlasnego transportu SSH i wlasnego kopiowania
#       odpornego na otwarta baze to duza powierzchnia zaleznosci przy zerowym zysku.
#       Zaufany control plane ZOSTAJE w Ruscie; to jest jednorazowe narzedzie operatora.
#   (3) TRACIMY: wspolna kontrole typow z rdzeniem i pokrycie przez cargo test.
#       Rekompensata: skrypt nie ma logiki poza kopiowaniem i podmiana, a kazdy krok
#       sprawdza swoj kod wyjscia i przerywa zamiast brnac dalej.
# GIT COMMIT: PENDING
# GITHUB METADATA: jpytka666-jpg/polip-agi, branch docs/darkstar-headscale-hotspot-plan
#
# ZADNA sciezka nie jest wpisana w kod. Wszystkie trzy przychodza ze srodowiska, zeby ten
# sam skrypt dzialal na innej maszynie bez edycji i zeby nic nie kopiowalo sie "w ciemno"
# do miejsca, ktorego operator nie wskazal.
# ==========================================
#
# Uzycie:
#   $env:DARKSTAR_CHROMA_SOURCE = '<sciezka do skladu chroma>'
#   $env:DARKSTAR_CBMS_TARGET   = 'owner@<adres bramy prywatnej>'
#   pwsh -File deploy/context/push-chroma-to-cbms.ps1
#   pwsh -File deploy/context/push-chroma-to-cbms.ps1 -WhatIf     # sam pomiar, bez wysylki

[CmdletBinding(SupportsShouldProcess)]
param(
    [string]$Source    = $env:DARKSTAR_CHROMA_SOURCE,
    [string]$Target    = $env:DARKSTAR_CBMS_TARGET,
    [string]$RemoteDir = $env:DARKSTAR_CONTEXT_COPY
)

$ErrorActionPreference = 'Stop'

function Fail([string]$m) { Write-Error "push-chroma: $m"; exit 1 }

if ([string]::IsNullOrWhiteSpace($Source)) {
    Fail 'ustaw DARKSTAR_CHROMA_SOURCE - sciezka do skladu chroma po stronie Windows'
}
if ([string]::IsNullOrWhiteSpace($Target)) {
    Fail 'ustaw DARKSTAR_CBMS_TARGET - np. owner@<adres bramy prywatnej>'
}
if ([string]::IsNullOrWhiteSpace($RemoteDir)) {
    # Sciezka wzgledna wobec katalogu domowego konta na CBMS - nie jest wpisana na sztywno.
    $RemoteDir = 'darkstar-context-copy'
}

# Tylda w cudzyslowie NIE jest rozwijana przez zdalna powloke - zmierzone: powstal katalog
# o doslownej nazwie "~". Sciezka wzgledna daje ten sam efekt bez pulapki, bo sesja SSH
# startuje w katalogu domowym konta.
if ($RemoteDir.StartsWith('~/')) { $RemoteDir = $RemoteDir.Substring(2) }

if (-not (Test-Path -LiteralPath $Source)) { Fail "brak zrodla: $Source" }

# Zrodlo jest tylko do czytania. Katalog roboczy powstaje w TEMP - nigdy na dysku zrodlowym.
$stage = Join-Path $env:TEMP ('chroma-stage-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))

$files  = Get-ChildItem -LiteralPath $Source -Recurse -File -ErrorAction SilentlyContinue
$sizeMB = [math]::Round(($files | Measure-Object -Sum Length).Sum / 1MB, 1)
Write-Host "push-chroma: zrodlo ma $($files.Count) plikow, $sizeMB MB"

if ($PSCmdlet.ShouldProcess($Target, "wyslij $sizeMB MB")) {

    New-Item -ItemType Directory -Path $stage -Force | Out-Null
    try {
        # /MIR dziala WYLACZNIE w obrebie katalogu tymczasowego. Zrodlo jest czytane.
        robocopy $Source $stage /MIR /R:1 /W:1 /NFL /NDL /NJH /NJS | Out-Null
        if ($LASTEXITCODE -ge 8) { Fail "robocopy zwrocil $LASTEXITCODE" }

        $staged = Get-ChildItem -LiteralPath $stage -Recurse -File
        if ($staged.Count -eq 0) { Fail 'kopia robocza wyszla pusta - nie wysylam' }

        # Wysylka do katalogu OBOK biezacej kopii. Podmiana dopiero po udanym scp, zeby
        # przerwany transfer nigdy nie zostal biezacym zrodlem drugiej nogi.
        ssh $Target "rm -rf '$RemoteDir/incoming' && mkdir -p '$RemoteDir/incoming'"
        if ($LASTEXITCODE -ne 0) { Fail 'nie moge przygotowac katalogu incoming na CBMS' }

        scp -q -r "$stage\*" "${Target}:$RemoteDir/incoming/"
        if ($LASTEXITCODE -ne 0) { Fail 'scp nie powiodl sie - biezaca kopia zostaje nietknieta' }

        $swap = @"
set -eu
cd '$RemoteDir'
[ -n "`$(ls -A incoming 2>/dev/null)" ] || { echo 'incoming puste - nie podmieniam'; exit 1; }
[ -d current ] && mv current "previous.`$(date +%Y%m%d-%H%M%S)"
mv incoming current
du -sh current
"@
        ssh $Target $swap
        if ($LASTEXITCODE -ne 0) { Fail 'podmiana na CBMS nie powiodla sie' }

        Write-Host "push-chroma: gotowe -> ${Target}:$RemoteDir/current"
    }
    finally {
        if (Test-Path -LiteralPath $stage) { Remove-Item -LiteralPath $stage -Recurse -Force }
    }
}
