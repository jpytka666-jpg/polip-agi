<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-04-context-legs.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-04 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Sonnet 5
TIMESTAMP: 2026-09-04 18:07:08 Europe/London
REASON FOR CREATION: Zwiad przed spieciem odczytu pamieci (AIONS/Chroma) przez Darkstar. Zero
kodu w tej fazie - wylacznie odczyt git, kodu i zywego hosta przez SSH, plus jeden test z
Windowsa spoza petli zwrotnej.
==========================================
-->

# Zwiad nóg kontekstu (AIONS/Chroma) — 2026-09-04

## Werdykt

Obie nogi kontekstu (`local_cbms` na 8000, `remote_e` na 8001) żyją, mówią Chroma API v2,
i odpowiadają na `/v1/context/*` prawdziwymi danymi. Autoryzacja działa poprawnie - potwierdzone
zapytaniem spoza pętli zwrotnej, nie tylko czytaniem kodu. Żadna zmiana kodu nie była tu
potrzebna: to, co brzmiało jak podejrzenie błędu (klient v1 kontra host v2), okazało się już
poprawnie obsłużone we wcześniejszej sesji.

## ZWIAD A — git

```
HEAD=9b73086
DIRTY=nie
```

## ZWIAD B — kod (`crates/darkstar-server/src/context_http.rs`)

- Trasy: `GET /v1/context/search`, `GET /v1/context/health`. Żadna metoda zmieniająca nie
  jest zarejestrowana (`context_router`, linia ~230).
- Autoryzacja: TAK, Bearer 1:1 (`authenticated()`), ten sam `state.api_token` co
  `/v1/gateway`, `/v1/headscale`, `/v1/git` - wszystkie klonowane z jednego `AppState::from_env()`
  w `main.rs`.
- Domyślne adresy nóg (kod): `DARKSTAR_CONTEXT_LOCAL` → `http://127.0.0.1:8000`,
  `DARKSTAR_CONTEXT_REMOTE` → `http://127.0.0.1:8001`.
- `deploy/.env` na CBMS zawiera wyłącznie `DARKSTAR_HOST_PORT` i `DARKSTAR_API_TOKEN` - żadna
  z dwóch zmiennych adresu nóg nie jest tam ustawiona, więc działają domyślne wartości z kodu.

### Wersja Chroma API — sprawdzone, nie zgadnięte

`crates/darkstar-core/src/context_client.rs`:

- `ContextLeg::remote_e()` domyślnie ustawia `ChromaApi::V1` (tak mówiła prawdziwa Chroma na
  dysku E:).
- `ContextLeg::local_cbms()` domyślnie ustawia `ChromaApi::V2`.
- W `ContextState::new()` (`context_http.rs`) noga `remote_e` jest jednak **nadpisana** przez
  `.with_api(remote_api)`, gdzie `remote_api` domyślnie to `ChromaApi::V2` (branch `_ =>`),
  chyba że `DARKSTAR_CONTEXT_REMOTE_API=v1` jest jawnie ustawione - a nie jest, ani w
  `deploy/.env`, ani w `deploy/docker-compose.yml`.

**Wniosek: w rzeczywistej konfiguracji serwera obie nogi już dziś mówią v2.** Warunkowe
polecenie "jak klient v1 — test RED, potem fix" nie miało zastosowania: klient nie jest v1.
Dopisany został wyłącznie test regresyjny (patrz niżej) - nie było niczego do naprawienia.

## ZWIAD C — CBMS przez SSH (owner@100.64.0.2, klucz `id_ed25519`)

```
SS 8000/8001/18080: LISTEN 127.0.0.1:18080, 192.168.2.1:18080, 127.0.0.1:8000, 127.0.0.1:8001
darkstar-chroma-e-copy = active
context/health z petli CBMS: bez tokenu = 200, z tokenem = 200 (petla ma auto-doklejany token)
heartbeat 8000: v1=410 Gone, v2=200
heartbeat 8001: v1=410 Gone, v2=200
```

### `GET /v1/context/search?q=test&limit=3` z tokenem, na pętli CBMS

Skrót ciała odpowiedzi (bez treści notatek - to same nazwy kolekcji, nie zawartość pamięci):

```
dlugosc: 242 bajty
klucze najwyzszego poziomu: collections, served_by
served_by: local_cbms
collections: 2 pozycje, kazda {id, name, dimension:384}
```

Ważne odkrycie architektoniczne: `search()` w `context_client.rs` **nie jest wyszukiwaniem
wektorowym po treści notatek**. To dopasowanie podłańcucha do *nazwy kolekcji*, bez
rozróżniania wielkości liter (`collections.filter(|c| c.name.to_lowercase().contains(needle))`).
Prawdziwe wyszukiwanie semantyczne po treści (z embeddingiem) nie istnieje jeszcze w tym
kliencie - komentarz w kodzie mówi wprost, że "przyjdzie osobno". To nie jest błąd - to jest
opis obecnego zakresu.

## ZWIAD D — Windows, spoza pętli zwrotnej

Kluczowy test bezpieczeństwa: żądanie z prawdziwego klienta LAN (Windows), bez tokenu, na
adres bramy zamiast pętli:

```
curl.exe http://192.168.2.1:18080/v1/context/health  ->  401
```

To zamyka wątpliwość z poprzedniego zwiadu: 200/200 zmierzone na pętli CBMS wynikało
wyłącznie z automatycznego doklejania tokenu dla ruchu z `127.0.0.1` (warstwa `loopback.rs`),
nie z braku wymogu autoryzacji. Zapytanie z realnego adresu LAN, bez tokenu, dostaje uczciwe
401 - autoryzacja działa tak, jak opisuje kod.

Porty Windows (`8000`, `8765`, `8787`) i sprawdzenie montowania E: na CBMS - patrz poprzedni
zwiad w tej samej rozmowie; bez zmian: `E:\server wiedzy` nie jest zamontowany na CBMS,
`darkstar-chroma-e-copy` to lokalna kopia, nie żywy mount.

## Zastane, nietknięte w tej turze

- `darkstar-headscale` kontener nadal `unhealthy` mimo HTTP 200 - nieznana przyczyna, stan
  sprzed tej sesji, poza zakresem tego zwiadu.
- `/world/` kafelek "Pamięć kontekstu" zostaje zamknięty (`<div>`, bez `href`) - zweryfikowane,
  nie zmienione.

## Powiązane

- Test regresyjny v1/v2: `crates/darkstar-server/tests/context_http.rs`,
  `both_legs_default_to_chroma_v2_matching_the_live_host`.
- Poprzedni zwiad (uwierzytelnienie, wcześniejsze ustalenia): ta sama rozmowa, bez osobnego pliku.
