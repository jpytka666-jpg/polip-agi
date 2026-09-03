<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: GPT-5 Codex
TIMESTAMP: 2026-08-29 11:19:11 Europe/London
REASON FOR CREATION: Dodanie wejścia do kanonicznej dokumentacji, roadmapy i planu dalszego rozwoju Darkstar.
==========================================
-->

# polip-agi

## Darkstar development

- Canonical roadmap: docs/DARKSTAR_ROADMAP.md
- Reference architecture: docs/DARKSTAR_REFERENCE_ARCHITECTURE.md
- Network architecture: docs/DARKSTAR_NETWORK_ARCHITECTURE.md
- Current design: docs/superpowers/specs/2026-08-29-darkstar-native-gateway-private-mesh-design.md
- Current implementation plan: docs/superpowers/plans/2026-08-29-darkstar-native-gateway-headscale-headplane-plan.md

The current development handoff starts at commit
fd8099e4e3c3399ae69a885fa2ed32ab0c57d2df on
feat/darkstar-module-control.

## Stan hosta na 2026-09-03

Pracujemy na gałęzi **docs/darkstar-headscale-hotspot-plan** — to jest źródło prawdy dnia.
Gałąź feat/darkstar-module-control pozostaje starym punktem przekazania.

Roadmapa opisuje teraz **zmierzony stan maszyny**, nie plan z 29.08. Krótko, co stoi:

- Własny control server sieci prywatnej działa; Windows i CBMS widzą się bezpośrednio,
  a dostęp po tej sieci to wyłącznie SSH — usługi HTTP celowo nie są tam wystawione.
- Obie bazy Chroma odpowiadają, a warstwa kontekstu potwierdza źródło lokalne i zdalne.
- Control Room jest dostępny przez tunel SSH po sieci prywatnej i wraca po restarcie
  Windows jako ostatni krok rozruchu AIONS. Krok sprawdza właściciela gniazda, nie sam
  kod odpowiedzi — kod 200 nie dowodzi, którędy biegnie ruch.
- Sterowanie modułami przez HTTP nadal **nie wykonuje** infrastruktury: `DryRunProvider`
  pozostaje jedyną implementacją.
- Wyjście na świat jest niegotowe: skrypt istnieje, ale `cloudflared` nie jest
  zainstalowany na CBMS.
- Komercyjny login server nadal działa obok własnego. Odłączenie to osobny etap (M8).

Szczegóły i dowody: docs/DARKSTAR_ROADMAP.md, sekcja „Stan na 2026-09-03 — ZMIERZONE”.
