<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-01-cbms-chroma-leg-live.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-01 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Opus 5
TIMESTAMP: 2026-09-02 03:10:00
REASON FOR CREATION: Evidence uruchomienia drugiej nogi Chroma na CBMS i jednokierunkowej migracji danych z E:.
==========================================
-->

# Druga noga Chroma na CBMS — ZYWA

Data: 2026-09-01/02. **Zrodlo na `E:\server wiedzy` nietkniete** — nie kasowane, nie zatrzymywane,
Chroma na Windows dziala nieprzerwanie na `127.0.0.1:8000`.

## 1. Instalacja

| Krok | Wynik |
|---|---|
| venv | `/home/owner/darkstar/venv`, Python `3.12.3` |
| pakiet | `chromadb 1.5.9` (+ 74 zaleznosci), `PIP_EXIT=0` |
| katalog danych | `/var/lib/darkstar/chroma`, `0750 owner:owner` |

## 2. Health PUSTEJ nogi

Pierwsza proba startu **nie powiodla sie** i zostalo to naprawione, a nie ukryte:

~~~text
error: unexpected argument '--log-path' found
~~~

`chromadb 1.5.9` nie przyjmuje juz `--log-path`; skrypt zbiera dziennik przekierowaniem.
Po poprawce, na pustej bazie: `port_state=listening`, `collections=0`, plik `chroma.sqlite3`
188 416 B.

## 3. Rozjazd wersji API — fakt, nie szczegol

| Noga | Wersja | Heartbeat | Sciezka kolekcji |
|---|---|---|---|
| `E:` (Windows) | **v1** | `/api/v1/heartbeat` → 200 | `/api/v1/collections` |
| CBMS (1.5.9) | **v2** | `/api/v2/heartbeat` → 200 | `/api/v2/tenants/{tenant}/databases/{db}/collections` |

Stare API na CBMS zwraca `410 Gone` z komunikatem `The v1 API is deprecated`. `ContextLeg` w
`darkstar-core` niesie teraz `api_version` i sam buduje wlasciwe adresy; test
`each_leg_uses_the_api_version_its_server_speaks` to utrwala.

## 4. Migracja — jednokierunkowa

`scp -r` z `E:\server wiedzy\data\chroma` do `/var/lib/darkstar/chroma`, przy **zatrzymanej**
nodze lokalnej. Zrodlo tylko czytane.

| Pomiar | Zrodlo E: | CBMS po migracji |
|---|---|---|
| pliki | 371 | **371** |
| rozmiar | 172,8 MB | **175 MB** (`181 201 696 B`) |
| `chroma.sqlite3` | 28,2 MB | `29 556 736 B` |

## 5. Health PELNEJ nogi

~~~text
data_dir_files=371
data_dir_bytes=181201696
chroma_binary=venv:/home/owner/darkstar/venv
chroma_version=1.5.9
port_state=listening
heartbeat_127.0.0.1=200
heartbeat_192.168.2.1=000
api_version=v2
collections=36
~~~

Nowa Chroma odczytala baze starszej wersji bez migracji recznej.

## 6. Rozbieznosc do wyjasnienia — NIE zamiatam

**CBMS raportuje 36 kolekcji, a zywe API na E: — 33.** Liczba plikow i rozmiar sie zgadzaja, wiec
kopia nie jest niekompletna; roznica dotyczy tego, co kazda wersja **pokazuje**. Prawdopodobne
przyczyny: inne domyslne limity odpowiedzi w v1 i v2, albo wpisy obecne w SQLite, ktorych starsza
wersja juz nie listuje. **NIEZWERYFIKOWANE** — wymaga porownania nazwa po nazwie.

Kopia byla robiona przy **zywym** zrodle na Windows, wiec moze byc niespojna w czasie. Nie
stanowi to problemu dzis: noga CBMS jest zapasowa, a prawda pozostaje na E:.

## 7. Wiazanie

Chroma slucha **wylacznie na `127.0.0.1`**. `heartbeat_192.168.2.1=000` jest **zamierzony**:
baza wektorowa nie ma wlasnego uwierzytelniania, wiec kazdy adres poza petla zwrotna to otwarte
drzwi. Dostep z sieci prywatnej ma isc przez `darkstar-server`, ktory ma token i wylacznie
sciezki odczytu. Skrypt `darkstar-chroma-run` odmawia startu na adresie spoza listy dozwolonych —
zabezpieczenie jest w kodzie, nie w pamieci operatora.

## 8. darkstar-server zbudowany i uruchomiony NA CBMS

| Krok | Wynik |
|---|---|
| toolchain | `rustup` minimal, `cargo 1.98.0`, `rustc 1.98.0` |
| zrodla | `tar` przez ssh do `/home/owner/darkstar/src` (bez `target`, `node_modules`, `.git`) |
| budowa | `cargo build --release -p darkstar-server` → `Finished`, binarka **3 973 976 B** |
| start | `127.0.0.1:18082`, `DARKSTAR_CONTEXT_LOCAL=http://127.0.0.1:8000` |

### Napotkana przeszkoda — port 18080 zajety

Pierwsza proba startu na `18080` skonczyla sie panika:

~~~text
bind Darkstar listener: Os { code: 98, kind: AddrInUse }
~~~

Port trzyma **inna, juz dzialajaca** usluga: proces `3158 /usr/local/bin/darkstar-server`.
**Nie zostala zatrzymana ani podmieniona.** Nasza instancja testowa stoi obok, na `18082`.

### Odpowiedzi z hosta

~~~text
GET /health                → {"service":"darkstar","api_version":"darkstar.core/v1","status":"ok"}
GET /v1/context/health     → {"local_cbms_ok":true,"remote_e_ok":false}   HTTP 200
GET /v1/context/search?q=session&limit=3 → HTTP 200, served_by "local_cbms", 3 kolekcje:
   session_AIONS_ARCHITECTURE_MASTERPLAN, session_aions_skills_v4_diag2, session_marcin_health_2026
~~~

`remote_e_ok:false` jest **poprawne**: Chroma na Windows slucha wylacznie na `127.0.0.1`, wiec z
CBMS jest nieosiagalna. Noga lokalna obsluzyla zapytanie i to widac w polu `served_by`.

### Poprawka wymuszona przez rzeczywistosc

Adresy nog byly wpisane na sztywno, co dawaloby `503` na CBMS. Teraz pochodza ze zmiennych
`DARKSTAR_CONTEXT_LOCAL` i `DARKSTAR_CONTEXT_REMOTE`, a noga lokalna jest pierwsza — gdy dane sa
na miejscu, nie ma powodu isc przez siec.

Zadne haslo nie zostalo zapisane.
