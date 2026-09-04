<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-04-context-search.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-04 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Sonnet 5
TIMESTAMP: 2026-09-04 18:39:04 Europe/London
REVISION: 2026-09-04 19:05 Europe/London - polaczono z tresci docs/operations/evidence/2026-09-04-context-legs.md
przed jego usunieciem (git rm) - zachowane fakty bez dwoch nazw kolekcji, ktore tamten plik ujawnial.
REASON FOR CREATION: Zapis pomiaru odczytu pamieci (AIONS/Chroma) BEZ nazw zbiorow i BEZ tresci -
scislejsza wersja niz poprzedni plik tej samej sesji, ktory ujawnil nazwy dwoch kolekcji. Ten plik
zawiera wylacznie ksztalt i liczby, nigdy zawartosc.
==========================================
-->

# Odczyt pamięci przez Darkstara — fakty bez treści, 2026-09-04

## Werdykt

Obie nogi kontekstu odpowiadają, obie mówią Chroma v2, autoryzacja działa - potwierdzone
zapytaniem spoza pętli zwrotnej, nie tylko czytaniem kodu. Zero kodu zmienione w tej turze -
warunkowa naprawa nie miała zastosowania, bo warunek okazał się fałszywy.

## Git

```
HEAD=9b73086 (w chwili zwiadu), DIRTY=nie
```

## Kod (`crates/darkstar-server/src/context_http.rs`, `darkstar-core/src/context_client.rs`)

- Trasy: `GET /v1/context/search`, `GET /v1/context/health`. Żadna metoda zmieniająca nie jest
  zarejestrowana.
- Autoryzacja: Bearer 1:1, ten sam `state.api_token` co `/v1/gateway`, `/v1/headscale`, `/v1/git`.
- `ContextLeg::remote_e()` domyślnie mówi V1 (tak mówiła prawdziwa Chroma na dysku E:), ale
  `ContextState::new()` nadpisuje to na V2 przez `.with_api(remote_api)`, chyba że
  `DARKSTAR_CONTEXT_REMOTE_API=v1` jest jawnie ustawione - a nie jest, ani w `deploy/.env`, ani
  w `deploy/docker-compose.yml`.
- `search()` w `context_client.rs` to dopasowanie podłańcucha do *nazwy* kolekcji, bez
  rozróżniania wielkości liter - nie jest wyszukiwaniem wektorowym po treści notatek. Komentarz
  w kodzie mówi wprost, że prawdziwe wyszukiwanie semantyczne "przyjdzie osobno".

## Fakty (bez nazw zbiorów, bez treści)

```
GET /v1/context/search, z tokenem, petla CBMS:
  served_by    = local_cbms
  collections  = 2 (liczba, nie nazwy)
  api version  = v2 (potwierdzone: v1 heartbeat = 410 Gone na obu portach, v2 = 200)

GET /v1/context/health, z tokenem, petla CBMS:
  local_cbms_ok = true
  remote_e_ok   = true
```

## Warunkowa naprawa z poprzedniej tury — nie zaszła

Polecenie brzmiało: "jak `remote_e` false — domyślne V1 na 8001, dopisz `with_api(V2)` albo
zmienną środowiskową, test RED najpierw." Warunek jest fałszywy: `remote_e_ok = true`,
zmierzone bezpośrednio z ciała odpowiedzi `/v1/context/health`, nie tylko z kodu HTTP. Kod już
nadpisuje wersję nogi zdalnej na V2 w `ContextState::new()` (`context_http.rs`) - opisane i
przetestowane w poprzednim commicie tej sesji. Nic tu nie wymagało zmiany.

## Autoryzacja — potwierdzona spoza pętli zwrotnej

```
curl.exe z Windows, bez tokenu, na adres bramy (nie petla):
  http://192.168.2.1:18080/v1/context/health -> 401
```

To zamyka wątpliwość, którą budziły wcześniejsze pomiary z pętli CBMS (200/200 bez tokenu):
tamte 200 wynikały z automatycznego doklejania tokenu dla ruchu z `127.0.0.1` (warstwa
`loopback.rs`), nie z braku wymogu autoryzacji. Zapytanie z realnego adresu sieci prywatnej,
bez tokenu, dostaje uczciwe 401.

## Usunięty plik

`docs/operations/evidence/2026-09-04-context-legs.md` (commit `6763be9`) zawierał dwie
prawdziwe nazwy kolekcji z żywej bazy i został usunięty z bieżącego drzewa przez `git rm` na
wyraźną prośbę - jego niewrażliwa treść (git/kod/autoryzacja) jest teraz tutaj, w tym pliku.
Historia gita zachowuje tamten commit z nazwami; nikt jej nie przepisuje bez osobnej, wyraźnej
zgody.

## Powiązane

- Test regresyjny v2: `crates/darkstar-server/tests/context_http.rs`,
  `both_legs_default_to_chroma_v2_matching_the_live_host` (commit `3c1e930`).
