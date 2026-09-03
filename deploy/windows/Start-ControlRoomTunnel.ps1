<#
darkstar-header-v1
po co: Start-ControlRoomTunnel.ps1
nie wolno: wskrzeszac tunelu do 100.71.8.70, ubijac cudzych sesji ssh, ruszac demona SaaS
Autor: Marcin Szul
Model: Claude
Czas: 2026-09-03T13:00:00+01:00
Po co: Stawia tunel do Sterowni przez PRYWATNY mesh i sprawdza, ze port trzyma wlasnie ten
       tunel, a nie jakikolwiek inny, ktory tez odpowiadalby 200.

THIS IS VERY IMPORTANT!!!
==========================================
MECHANICS: Trzy rzeczy, kazda z powodu wyniesionego z bledu popelnionego wczesniej tej nocy.
  1. Ubija WYLACZNIE procesy ssh z przekierowaniem portu 18080. Nie `Get-Process ssh |
     Stop-Process`, bo to zrywa takze sesje robocze operatora i agentow.
  2. Czeka, az mesh odpowie, ZANIM sprobuje postawic tunel. Bez tego pierwsza proba po
     starcie systemu trafia w niepodniesiona jeszcze siec i krok wyglada na zepsuty.
  3. Po starcie czyta PROCES TRZYMAJACY PORT, nie sam kod odpowiedzi. Kod 200 mowi tylko,
     ze cos odpowiada - nie mowi ktoredy. Dokladnie ta pomylka sprawila, ze przelozenie
     Sterowni na mesh zostalo raz zaraportowane jako udane, choc port wrocil do starej drogi.
SYSTEM PART: deploy/windows - dostep operatora do Sterowni.
ARCHITECTURE FUNCTION: Launcher dla kroku w istniejacym nadzorcy AIONS. NIE jest drugim
orkiestratorem - nadzorca `AIONS SERVER FULL.ps1` czyta kroki z JSON-a i sam opisuje ta
droge rozszerzania. Ten plik jest wylacznie tym, co ten krok uruchamia.
DEPENDENCIES/LINKS: ssh.exe z OpenSSH, wezel darkstar-cbms w prywatnym mesh,
E:\server wiedzy\runtime\aions_boot_steps.json (krok wolajacy ten plik).
TECH STACK: PowerShell 7, swiadomie zamiast Rusta - domyslnego jezyka tego projektu.
  (1) MUSI: czytac tablice procesow Windows wraz z ich liniami polecen, odczytac wlasciciela
      gniazda TCP (Get-NetTCPConnection), uruchomic proces w tle i dac sie wywolac
      bezposrednio przez nadzorce, ktory sam jest w PowerShellu.
  (2) DLACZEGO NIE RUST: caly zakres to odpytanie WMI i stosu TCP Windows oraz start
      procesu. Rust wymagalby kompilacji skrosnej i dostarczenia binarki na maszyne
      operatora, po czym siegalby po te same API systemowe - ta sama praca, wiecej krokow,
      i binarka, ktorej nadzorca nie umie zdiagnozowac. Zaufany control plane ZOSTAJE
      w Ruscie; to jest narzedzie startowe strony Windows.
  (3) TRACIMY: wspolna kontrole typow z rdzeniem i pokrycie przez cargo test.
      Rekompensata: skrypt konczy sie kodem 0 albo 1 na podstawie ZMIERZONEGO wlasciciela
      portu, wiec nadzorca dostaje uczciwa odpowiedz zamiast optymistycznej.
==========================================
#>

[CmdletBinding()]
param(
    [string]$MeshHost   = $(if ($env:DARKSTAR_MESH_HOST)   { $env:DARKSTAR_MESH_HOST }   else { 'owner@100.64.0.2' }),
    [int]   $Port       = $(if ($env:DARKSTAR_UI_PORT)     { [int]$env:DARKSTAR_UI_PORT } else { 18080 }),
    [int]   $MeshWaitSec= $(if ($env:DARKSTAR_MESH_WAIT)   { [int]$env:DARKSTAR_MESH_WAIT } else { 60 })
)

$ErrorActionPreference = 'Stop'
$meshAddress = ($MeshHost -split '@')[-1]

function Say($m) { Write-Host ("[control-room-tunnel] {0}" -f $m) }

# --- 1. Ubij TYLKO tunele tego portu. Cudze sesje ssh zostaja nietkniete. ------------
$mine = Get-CimInstance Win32_Process -Filter "Name='ssh.exe'" -ErrorAction SilentlyContinue |
        Where-Object { $_.CommandLine -match "-L\s+\S*:?$Port`:" -or $_.CommandLine -match "-L\s+$Port`:" }
foreach ($p in $mine) {
    Say "ubijam istniejacy tunel portu $Port (PID $($p.ProcessId))"
    Stop-Process -Id $p.ProcessId -Force -ErrorAction SilentlyContinue
}
if ($mine) { Start-Sleep -Seconds 3 }

# --- 2. Poczekaj, az mesh naprawde odpowie. -----------------------------------------
$deadline = (Get-Date).AddSeconds($MeshWaitSec)
$meshUp = $false
while ((Get-Date) -lt $deadline) {
    if (Test-Connection -ComputerName $meshAddress -Count 1 -Quiet -ErrorAction SilentlyContinue) { $meshUp = $true; break }
    Start-Sleep -Seconds 3
}
if (-not $meshUp) {
    Say "mesh $meshAddress nie odpowiada po $MeshWaitSec s - NIE stawiam tunelu"
    Say "swiadomie nie siegam po 100.71.8.70: cicha zamiana drogi jest gorsza niz brak tunelu"
    exit 1
}
Say "mesh $meshAddress odpowiada"

# --- 3. Postaw dokladnie jeden tunel. ------------------------------------------------
$sshArgs = @(
    '-N','-o','BatchMode=yes','-o','ExitOnForwardFailure=yes',
    '-o','ServerAliveInterval=20','-o','ServerAliveCountMax=3',
    '-L', ("127.0.0.1:{0}:127.0.0.1:{0}" -f $Port), $MeshHost
)
Start-Process -FilePath 'ssh.exe' -ArgumentList $sshArgs -WindowStyle Hidden
Start-Sleep -Seconds 8

# --- 4. Sprawdz WLASCICIELA portu, nie tylko kod odpowiedzi. -------------------------
$conn = Get-NetTCPConnection -LocalPort $Port -State Listen -ErrorAction SilentlyContinue
if (-not $conn) { Say "nikt nie nasluchuje na $Port - tunel nie wstal"; exit 1 }

$ownerCmd = (Get-CimInstance Win32_Process -Filter "ProcessId=$($conn.OwningProcess)" -ErrorAction SilentlyContinue).CommandLine
if ($ownerCmd -notlike "*$meshAddress*") {
    Say "port $Port trzyma NIE-mesh: $ownerCmd"
    Say "zostawiam jak jest i zglaszam blad - nie udaje, ze Sterownia idzie po mesh"
    exit 1
}

try {
    $r = Invoke-WebRequest -Uri ("http://127.0.0.1:{0}/health" -f $Port) -TimeoutSec 8 -UseBasicParsing
    if ($r.StatusCode -ne 200) { Say "health zwrocil $($r.StatusCode)"; exit 1 }
} catch { Say 'health nie odpowiada'; exit 1 }

Say "OK: port $Port trzyma tunel do $meshAddress, health 200"
exit 0
