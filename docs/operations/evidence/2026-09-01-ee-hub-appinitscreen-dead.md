<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-01-ee-hub-appinitscreen-dead.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-01 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-01 19:30:00
REASON FOR CREATION: Zamkniecie proby konfiguracji huba EE przez jego panel i usuniecie tymczasowej trasy diagnostycznej.
==========================================
-->

# Hub EE — panel martwy, watek odroczony

Data: 2026-09-01. Host CBMS (`owner@100.71.8.70`), downstream `enp1s0`.

## Co ustalono

Hub EE `192.168.1.254` (`a0:2d:db:d7:a9:20`, `SerialNumber ":114404:2525118982"`, `Brand
Consumer`) jest osiagalny i odpowiada, ale **jego panel nie nadaje sie do uzycia**.

| Pomiar | Wynik |
|---|---|
| `ping 192.168.1.254` | 2/2, 0% loss, rtt ~0.9 ms |
| `curl -k -sI https://192.168.1.254/` | `200 OK`, `Server: nginx` |
| wszystkie zasoby aplikacji (`main.chunk.js` 1.6 MB, `4.chunk.js` 1.0 MB, CSS 507 kB, `manifest.json`, `config.json`) | kazdy `200`, pelny rozmiar |
| konsola przegladarki | `Initialising common scripts`, `i18next: initialized` — **zero bledow** |
| render (`--dump-dom`, 15 s budzetu) | wylacznie tekst `Hub Manager` |
| zrzuty `hub.png`, `hub-login.png` (1280x900) | **calkowicie biale**, po 5290 B kazdy |

Aplikacja startuje, pobiera sie w calosci i nie zglasza bledu, ale **`AppInitScreen` nie oddaje
sterowania** — ekran poczatkowy wisi i nie renderuje interfejsu. Utrzymywalo sie to przez caly
dzien, przy kazdej probie i na obu adresach (`/` oraz `/login`).

Sterowanie hubem przez panel zostaje **porzucone**. Nie probowano zadnego hasla; hasel nie
zgadywano i nie zapisywano.

## Skutek dla planu

- **Extender / hub EE: ODROCZONY.** Bez panelu nie da sie wylaczyc jego serwera adresow ani
  przelaczyc go w tryb mostka. Klucze sterujace zostaly zidentyfikowane w kodzie panelu
  (`dhcp_enabled_togglebtn`, `dhcpServerEnabled`, `Device.DHCPv4.Server.`, `AccessPoint`), wiec
  gdyby panel kiedys ozyl, wiadomo gdzie klikac.
- **Brama na Ubuntu ZOSTAJE.** `DARKSTAR-WiFi` na `enp1s0`, `192.168.2.1/24`, `ipv4.method=shared`,
  `ipv6.method=disabled`. Zapora `darkstar_host_guard` dopasowana do `192.168.2.0/24`
  (`2026-09-01-host-guard-192-168-2-apply.md`), trwalosc po reboocie udowodniona
  (`2026-09-01-firewall-reboot-persistence.md`).

## Sprzatanie — stan koncowy

Tymczasowe waskie przejscie diagnostyczne usuniete w calosci:

| Element | Stan |
|---|---|
| `ip route del 192.168.1.254/32 dev enp1s0` | wykonane; `ip route show 192.168.1.254` pusty |
| `ip addr del 192.168.1.253/24 dev enp1s0` | wykonane; `enp1s0` ma juz tylko `192.168.2.1/24` |
| tunel `8443` (Windows) | proces ubity, petla wznawiajaca zatrzymana; port martwy |
| `default via 192.168.1.1 dev wlp2s0` | bez zmian przez caly czas |
| `DARKSTAR-WiFi` / `wlp2s0` / `tailscale0` | wszystkie `activated` |

Nic trwalego nie zostalo na hoscie po tej probie. Zrzutow ekranu nie dolaczono — sa puste i
niczego nie dokumentuja poza sama pustka.
