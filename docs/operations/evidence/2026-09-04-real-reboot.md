<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-04-real-reboot.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-04 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Sonnet 5
TIMESTAMP: 2026-09-04 20:44:02 Europe/London
REASON FOR CREATION: Zamkniecie bramy wyjscia M2 - operator wykonal prawdziwy restart CBMS.
Ten plik zapisuje dowod, ze zapora, uslugi i API przetrwaly, zmierzony PO fakcie, nie
zalozony z gory.
==========================================
-->

# Prawdziwy restart CBMS — dowod dla M2, 2026-09-04

## Werdykt

**M2 spelnione.** Operator zrestartowal CBMS. Nowy boot_id, `darkstar-firewall.service`
wstal sam i przeladowal regulki, kontenery Darkstar/Headplane wrocily zdrowe, API i kanal
administracyjny odpowiadaja - w tym z prawdziwego klienta LAN (Windows), nie tylko z petli
CBMS. Jedno realne odkrycie: noga pamieci na porcie 8000 nie ma jednostki systemd i nie
wstala - opisane osobno, nie jest to zaleglosc M2.

## Boot ID — zmiana potwierdzona

```
poprzedni:  dbe6ad68-eff2-4954-baa4-66c7077d1c8c  (od 2026-09-01 07:15)
nowy:       e2f73b6e-7d5b-4921-aea3-085bced00a60  (od 2026-09-04 20:14:40)
```

## `darkstar-firewall.service` — wstal sam, przeladowal regulki

Z `journalctl -u darkstar-firewall`, na nowym boot ID:

```
Sep 04 20:14:57 CBMS systemd[1]: Starting darkstar-firewall.service...
Sep 04 20:14:57 CBMS darkstar-firewall-apply[1508]: darkstar-firewall: loaded inet darkstar_host_guard
Sep 04 20:14:57 CBMS systemd[1]: Finished darkstar-firewall.service.
```

`systemctl is-active darkstar-firewall` = `active`, `is-enabled` = `enabled`.

## Uslugi i kontenery — wszystkie wrocily automatycznie

```
darkstar                 active/enabled
darkstar-chroma-e-copy   active/enabled
cloudflared               active/enabled
tailscaled-headscale      active/enabled
docker                    active/enabled

docker ps:
  darkstar            Up (healthy)
  darkstar-headplane  Up (healthy)
  darkstar-headscale  Up (unhealthy - stan sprzed restartu, healthcheck kontenera
                       zle skonfigurowany; /health i /windows odpowiadaja poprawnie)
```

## API i kanal administracyjny — sprawdzone z PRAWDZIWEGO klienta LAN

Z Windowsa, po restarcie CBMS, nie z petli zwrotnej CBMS:

```
curl http://192.168.2.1:18080/world/                       -> 200
curl http://192.168.2.1:18080/v1/context/health bez tokenu -> 401 (auth przetrwala)
```

## Co NIE przetrwalo — nie jest to zaleglosc bramy M2

Noga pamieci `local_cbms` (port 8000) nie wstala po restarcie. Przyczyna zbadana osobno:
brak jednostki systemd dla tego procesu w calym repozytorium - dane na dysku sa realne
(`/var/lib/darkstar/chroma`, baza 29 MB, dziesiatki kolekcji, ostatnio dotykana
2026-09-01), ale nic nie ma instrukcji "uruchom mnie po starcie". To osobny, nowo odkryty
watek (persistencja pamieci), nie czesc zakresu M2 (zapora/API/routing).

## Powiazane

- Poprzedni zapis o brakujacej nodze 8000: ta sama rozmowa, bez osobnego pliku evidence.
- `docs/DARKSTAR_ROADMAP.md`, sekcja M2.
