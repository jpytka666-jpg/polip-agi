<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-08-29 18:00:00 Europe/London
REASON FOR CREATION: Evidence ręcznej aktywacji darkstar-firewall.service na Ubuntu Darkstar.
==========================================
-->

# Darkstar firewall — manual activation evidence

Data: 2026-08-29 17:50 UTC  
Host: CBMS (`owner@100.71.8.70`)  
Zakres: wyłącznie host guard; bez rebootu i bez zmian firmware SH32B.

## Repozytorium i walidacja

Repo Ubuntu wykonano fast-forward do `4a54d93099c704e25a60a43b96b83f6abf1b4c71` (`feat/darkstar-module-control`); drzewo było czyste.

| Komenda | Wynik / kod |
|---|---:|
| `git fetch origin feat/darkstar-module-control` + fast-forward | 0 |
| `sudo nft -c -f deploy/firewall/host-guard.nft` (bez usunięcia istniejącej tabeli) | 1 — tabela już istniała |
| walidacja kandydata z `delete table inet darkstar_host_guard` | 0 |
| `systemd-analyze verify deploy/systemd/darkstar-firewall.service` | 0 |
| `sh -n deploy/firewall/darkstar-firewall-apply` | 0 |
| `sh -n deploy/firewall/darkstar-firewall-verify` | 0 |
| `shellcheck` | niedostępny |

## Stan przed zmianą

- `tailscale0`: `100.71.8.70/32`; sesja SSH była aktywna.
- Trasa domyślna: `wlp2s0`.
- `curl -fsS http://127.0.0.1:18080/health`: API `status=ok`.
- `darkstar-firewall.service`: inactive przed aktywacją.

## Backup i instalacja

Backup root-owned 0700 utworzono w:
`/var/backups/darkstar-firewall/20260829T175054Z`

| Plik | Tryb | SHA-256 |
|---|---:|---|
| `host-guard.nft` | 0644 | `f8b918528d7998a26cc00fd5902cdf23d80f3465cb85f5b7db66269370cd6cf4` |
| `darkstar-firewall-apply` | 0755 | `cb655a7d610fe71afb931e623d68041259d742ee1527bd59633c236f931dee75` |
| `darkstar-firewall.service` | 0644 | `84e865510c0b759959cd93a9fd78e6759cc5c4d004139112d3bb6f0209b0461e` |

Artefakty zainstalowano jako root:root do `/etc/darkstar/host-guard.nft`, `/usr/local/sbin/darkstar-firewall-apply`, `/usr/local/sbin/darkstar-firewall-verify` i `/etc/systemd/system/darkstar-firewall.service`. Wykonano `daemon-reload`.

## Wyniki po aktywacji

- `systemctl start darkstar-firewall.service`: kod 0; `active (exited)`, enabled.
- `sudo /usr/local/sbin/darkstar-firewall-verify`: kod 0.
- `nft list table inet darkstar_host_guard`: named sets, input/forward policy drop oraz postrouting masquerade obecne.
- Drugi start i restart usługi: zakończone poprawnie; dokładnie jedna tabela `inet darkstar_host_guard`.
- IPv4 forwarding = `1`; IPv6 forwarding = `1`.
- API health: `{"service":"darkstar","api_version":"darkstar.core/v1","status":"ok"}`.
- Egress `api.ipify.org`: `84.70.26.249`.
- Windows `Test-NetConnection 100.71.8.70 -Port 22`: `True`.
- Istniejąca sesja SSH pozostała aktywna; `hostnamectl --static`: `CBMS`.

## Rollback review

Sprawdzono dokładne ścieżki backupu, właściciela, tryb i checksumy. Komendy rollbacku przejrzano, lecz rollbacku nie wykonano, ponieważ sesja zarządzająca i testy po aktywacji działały poprawnie. Nie wykonano rebootu, nie dotknięto firmware SH32B, nie flushowano obcych tabel ani nie zmieniano innych usług.
