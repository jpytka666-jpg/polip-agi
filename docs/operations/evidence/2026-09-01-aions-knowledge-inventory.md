<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-01-aions-knowledge-inventory.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-01 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-02 00:50:00
REASON FOR CREATION: Inwentarz serwerow wiedzy AIONS na E:, D:, C: i GitHubie przed decyzja, co przenosic na Darkstar CBMS.
==========================================
-->

# AIONS — inwentarz serwerow wiedzy

Data: 2026-09-01. **Wylacznie odczyt** — nic nie budowano, nie kopiowano, nie zatrzymywano.

## Trafienia

| Sciezka | Co to jest | Jezyk | Czy zyje | Gdzie dane | Zrodlo prawdy |
|---|---|---|---|---|---|
| `E:\server wiedzy` | serwer kontekstu AIONS, repo `aions-server-wiedzy`, galaz `main` (`da733db`, 2026-08-27) | Python (694 plikow `.py`, 151 `.ps1`) | **TAK** | `E:\server wiedzy\data\chroma` | **TAK** |
| `E:\server wiedzy\data\chroma` | baza wektorowa Chroma | dane | **TAK** — obslugiwana przez proces 15208 | 371 plikow, **172,8 MB** | **TAK — dane** |
| `E:\server wiedzy\venv\Scripts\chroma.exe` | proces Chroma | Python | **TAK**, `127.0.0.1:8000` | j.w. | nie (silnik) |
| `E:\server wiedzy\mcpServers\VS_CODE_MCP_CODEX` | serwer MCP `aions-context` (narzedzia dla Claude) | Python | **TAK**, proces 10592, tryb `http` | czyta Chroma powyzej | nie (fasada) |
| `uvicorn server.app:app` | API serwera wiedzy | Python | **TAK**, proces 23640, `127.0.0.1:8765` | j.w. | nie (fasada) |
| `E:\aions-chroma-watchdog` | straznik zdrowia Chroma, repo `aions-chroma-watchdog` | **Rust** | brak procesu | pilnuje Chroma | nie (narzedzie) |
| `E:\server` | `mcp-integration-system` — inne repo, nie AIONS wiedza | TypeScript/Node | brak nasluchu na typowych portach | `E:\server\docs`, `system-map` | nie |
| `D:\AIONS` | kopia robocza, **NIE jest repozytorium** (`fatal: not a git repository`) | mieszany | nie | — | **NIE — slepy zaulek** |
| `D:\aions_dev`, `D:\aions-models` | wagi modeli (`qwen3-4b-*`, `qwen3-coder-*`) | dane | nie | lokalne pliki modeli | nie (modele, nie wiedza) |
| `C:\temp\aions-*` | katalogi robocze | — | **BRAK** — dzis nie istnieja | — | nie |
| GH `jpytka666-jpg/aions-server-wiedzy` | zdalne zrodlo `E:\server wiedzy` | Python | n/d | — | **TAK — zdalny kanon** |
| GH `jpytka666-jpg/aions-mcp-server` | 40 narzedzi MCP w kontenerze, 2026-08-29 | Python | n/d | — | nie (opakowanie) |
| GH `jpytka666-jpg/aions-chroma-watchdog` | straznik, 2026-08-25 | Rust | n/d | — | nie |
| GH `jpytka666-jpg/aions-tmp-layer-catalog` | **prywatne**, katalog warstw; opis wprost: „Not a live E: dump. Not Darkstar/Codex" | Python | n/d | — | nie |

Docker: dziala 7 kontenerow (`kali-web-poc`, `kali-headless`, `kind-*`) — **zaden nie nalezy do
AIONS wiedzy**, wiec nie sa czescia tego inwentarza.

## 1. Jedno zrodlo prawdy kontekstu

**`E:\server wiedzy` + `E:\server wiedzy\data\chroma`**, z kanonem zdalnym
`github.com/jpytka666-jpg/aions-server-wiedzy` (galaz `main`).

Powod: to jedyna lokalizacja, ktora jednoczesnie **zyje** (Chroma na `127.0.0.1:8000`, API na
`:8765`, serwer MCP w trybie `http`), **ma dane** (172,8 MB w 371 plikach) i **ma kanon w Gicie**
z biezacym commitem. Reszta to fasady nad ta sama baza, narzedzia albo kopie.

Uwaga: w kopii roboczej sa **2 niezapisane zmiany** wzgledem `origin/main`.

## 2. Co musi trafic na Darkstar CBMS

1. **Dane Chroma** — 172,8 MB. To jedyna rzecz, ktorej nie da sie odtworzyc z Gita.
2. **Serwer MCP** `VS_CODE_MCP_CODEX` — fasada, ktora Darkstar bedzie wystawial jako modul.
3. **Kontrakt nasluchu**: dzis wszystko stoi na `127.0.0.1`. Na CBMS ma byc `127.0.0.1` albo
   `192.168.2.1` — **nigdy** `0.0.0.0`, tak samo jak dla Headscale.
4. **Straznik Chroma** (Rust) — jedyny element juz zgodny z jezykiem Darkstara.

Kolejnosc: najpierw dane, potem fasada, na koncu straznik. Zaden krok nie zostal wykonany.

## 3. Czego nie ruszac

- **Zywych procesow**: 15208 (Chroma), 10592 (MCP), 23640 (uvicorn) — zatrzymanie ktoregokolwiek
  odcina narzedzia AIONS w trwajacej sesji.
- **`E:\server wiedzy\data\chroma`** — kopiowac wylacznie przy zatrzymanej bazie, inaczej kopia
  bedzie niespojna.
- **`D:\AIONS`** — nie jest repozytorium, ostatnia zmiana 2026-08-28. Nie traktowac jak zrodla.
- **`E:\server`** — to inny projekt (`mcp-integration-system`), nie mieszac z wiedza.
- **`aions-tmp-layer-catalog`** — prywatne repo, ktore samo o sobie mowi, ze nie jest zrzutem E:.
- **Kontenerow Kali i kind** — nie naleza do tego watku.

Zadnego zapisu poza tym plikiem. Zadne haslo ani token nie zostaly odczytane.
