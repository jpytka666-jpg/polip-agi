<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-01 07:15:52
REASON FOR CREATION: Evidence prawdziwego testu trwalosci darkstar-firewall.service po reboocie (Task 3).
==========================================
-->

# Darkstar firewall — reboot persistence evidence

Data: 2026-09-01 07:13–07:16 UTC
Host: CBMS (`owner@100.71.8.70`, tailscale0, relay `lhr`)
Zakres: wylacznie host guard. Bez zmian plikow, bez apply bramy, bez firmware SH32B.

## Bramki przed rebootem

| Pomiar | Wynik |
|---|---|
| `sudo -S true` | `SUDO=0` |
| `sudo darkstar-firewall-verify` | `host guard checks passed`, `VERIFY=0` |
| `sudo nft list table inet darkstar_host_guard` | tabela obecna; sety `loopback_ifaces={lo}`, `management_ifaces={tailscale0}`, `upstream_ifaces={wlp2s0}` |
| `systemctl is-enabled` / `is-active` | `enabled` / `active` |
| `hostname` | `CBMS` |
| `uptime` | `up 4 days, 3:00` |
| boot_id przed | `58f8dc66-4647-4b79-8edd-00424c35c90b` |

## Reboot

`sudo systemctl reboot` — kod wyjscia ssh 0.
Odczekano 45 s, nastepnie petla powrotu co 15 s (limit 8 prob).
Host odpowiedzial w **1. probie**.

## Stan po reboocie

| Pomiar | Wynik |
|---|---|
| `hostname` | `CBMS` |
| `uptime` | `up 0 min` (07:15:52) |
| boot_id po | `dbe6ad68-eff2-4954-baa4-66c7077d1c8c` |
| boot_id zmieniony | **TAK** — restart byl realny, nie samo `enabled` |
| `sudo darkstar-firewall-verify` | `host guard checks passed`, `VERIFY_AFTER=0` |
| `systemctl is-enabled` / `is-active` | `enabled` / `active` |
| `nft list table inet darkstar_host_guard` | tabela obecna po starcie |
| IPv4 / IPv6 forwarding | `1` / `1` |
| Droga zarzadzania | sesja SSH przez tailscale0 wrocila bez interwencji |

## Werdykt

**ZIELONY.** Host guard jest odporny na reboot: tabela odtwarza sie z jednostki systemd przy
starcie, a nie z recznego zaladowania. Warunek wejscia Task 4 (`Real reboot persistence from
Task 3 is proven`) jest spelniony.

## Czego ten przebieg NIE udowodnil

- Step 3.3 — osobne potwierdzenie drogi ratunkowej z Windows nie bylo wykonane przed rebootem.
- Step 3.5 — pomiar portu 22 po 45 s pokazal `True`; upadku hosta nie udowodniono pomiarem
  portu, lecz posrednio przez `uptime 0 min` i zmiane boot_id.
- Step 3.9 — egress z Windows nie byl mierzony w tym przebiegu.
- Step 3.10 — nie dotyczy, zadna warstwa nie zawiodla.

Zadne haslo nie zostalo zapisane w repozytorium ani w tym pliku.
