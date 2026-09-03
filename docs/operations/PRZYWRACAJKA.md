<!-- darkstar-header-v1 -->
<!-- po co: PRZYWRACAJKA.md -->
<!-- nie wolno: tailscale logout, nft -f, gasic SaaS, wskrzeszac tunelu do 100.71.8.70 -->
<!--
Autor: Marcin Szul
Model: Claude
Czas: 2026-09-03T14:00:00+01:00
Po co: Zeby po restarcie maszyny dalo sie w minute odtworzyc, co gdzie stoi, co dziala,
       co czeka - bez szukania i bez zgadywania.
-->

# Przywracajka — czytaj to najpierw po restarcie

## W jednym zdaniu

Darkstar ma własną sieć prywatną, pamięć na dwóch nogach i Sterownię chodzącą po tej
własnej sieci. Chmura Tailscale zostaje jako zapas i **nie wolno jej gasić**.

## Gdzie co jest

| Rzecz | Adres / miejsce |
|---|---|
| Kopia robocza | `D:\codex-fresh-2026-08-28\worktrees\polip-agi-darkstar-plan` |
| Gałąź | `docs/darkstar-headscale-hotspot-plan` |
| GitHub | `github.com/jpytka666-jpg/polip-agi` |
| Maszyna CBMS, po kablu | `owner@192.168.2.1` |
| Maszyna CBMS, po własnej sieci | `owner@100.64.0.2` |
| Maszyna CBMS, przez chmurę (zapas) | `owner@100.71.8.70` |
| Sterownia | `http://127.0.0.1:18080/` przez tunel |

## Co ma działać po restarcie

Wszystko poniżej wstaje samo. Jeśli coś nie wstało — komendy sprawdzające niżej.

| Co | Gdzie wstaje | Sprawdzenie |
|---|---|---|
| Chroma pamięci głównej | CBMS, usługa | port 8000 |
| Chroma drugiej nogi | CBMS, `darkstar-chroma-e-copy` | port 8001 |
| Serwer Darkstar | CBMS, docker `darkstar` | port 18080 na CBMS |
| Serwer własnej sieci | CBMS, docker `darkstar-headscale` | port 8080 na 192.168.2.1 |
| Drugi demon sieci | CBMS, `tailscaled-headscale` | interfejs `headscale0` |
| Tunel Sterowni | **Windows**, krok w AIONS Conductor | port 18080 lokalnie |

## Jedna komenda, która mówi całą prawdę

```
ssh owner@192.168.2.1 'echo "chroma 8000/8001: $(curl -s -o /dev/null -w %{http_code} http://127.0.0.1:8000/api/v2/heartbeat)/$(curl -s -o /dev/null -w %{http_code} http://127.0.0.1:8001/api/v2/heartbeat)"; cd ~/polip-agi; set -a; . deploy/.env; set +a; echo "pamiec: $(curl -s http://127.0.0.1:$DARKSTAR_HOST_PORT/v1/context/health)"; echo "headscale: $(curl -s -o /dev/null -w %{http_code} http://192.168.2.1:8080/health)"; echo "wezly: $(docker exec darkstar-headscale headscale nodes list | grep -c online)"; echo "SaaS/mesh: $(systemctl is-active tailscaled.service)/$(systemctl is-active tailscaled-headscale.service)"'
```

A na Windowsie, po zalogowaniu:

```
$c = Get-NetTCPConnection -LocalPort 18080 -State Listen
(Get-CimInstance Win32_Process -Filter "ProcessId=$($c.OwningProcess)").CommandLine
```

**Ta linia MA zawierać `owner@100.64.0.2`.** Samo `200` na porcie nie wystarcza — mówi, że
coś odpowiada, nie którędy. Na tym się już raz przejechałem.

## Jeśli tunel Sterowni nie wstał

```
& 'E:\server wiedzy\runtime\control_room_tunnel_run.cmd'
```

Albo ręcznie:

```
ssh -N -L 18080:127.0.0.1:18080 owner@100.64.0.2
```

**Nigdy nie podstawiaj `100.71.8.70`** — dałoby 200 i fałszywe wrażenie, że Sterownia idzie
po własnej sieci.

## Czego NIE robić

- `tailscale logout` ani `tailscale down` na CBMS — chmura to jedyna zdalna droga, gdy
  padnie kabel i własna sieć naraz.
- `nft -f` z ręki — reguły idą wyłącznie przez `deploy/firewall/darkstar-firewall-apply`.
- Wiązać czegokolwiek na `0.0.0.0` — wystawiłoby usługę na sieć Vodafone.
- Pisać na dysk E bez przeczytania tego, co leży wokół.

## Co jest zrobione, a co czeka

Zrobione: własna sieć z dwoma węzłami, SSH po niej, pamięć na dwóch niezależnych
procesach i katalogach, Sterownia po własnej sieci, tunel wpięty w start.

Czeka:

1. **Wyjście na świat** — pliki gotowe (`deploy/headscale/world-tunnel`,
   `deploy/systemd/cloudflared-headscale.service`), `cloudflared` **nie zainstalowany**,
   nic nie uruchomione. Szczegóły w `docs/operations/world-ingress.md`.
2. **Telefon** — nie dołączony, czeka na punkt 1.
3. **Licznik dokumentów w Chromie jest zepsuty** — patrz niżej.

## Znaleziona usterka w Chromie

Zapytanie o liczbę dokumentów zwraca błąd na **każdej** kolekcji i na **obu** nogach:

```
Error executing plan: ... Rust type `u64` (as SQL type `INTEGER`)
is not compatible with SQL type `BLOB`
```

To nie jest uszkodzenie danych: **wypisywanie kolekcji i szukanie działają normalnie**
(36 kolekcji na obu nogach). Zepsuta jest sama funkcja liczenia w tej wersji Chromy przy
tym formacie bazy. Nie naprawiane — do decyzji.

## Gdzie ta sesja NIE została zapisana

W Chromie. Sprawdzone: nie powstała żadna kolekcja z dzisiejszą datą, a `memory_store`
i `conv_log` nie były w tej sesji wywołane ani razu.

Trwałe jest wyłącznie to:

1. **Commity na GitHubie** — cała robota i uzasadnienia w opisach zmian.
2. **Pliki dowodowe** w `docs/operations/evidence/`.
3. **Pamięć plikowa** w `C:\Users\User\.claude\projects\C--Windows-System32\memory\`.
4. **Ten plik.**

Nic poza tym nie przetrwa. Nowa sesja zaczyna od zera i czyta to, co tu napisane.
