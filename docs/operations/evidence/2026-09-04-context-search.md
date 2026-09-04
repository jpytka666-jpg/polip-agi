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
REASON FOR CREATION: Zapis pomiaru odczytu pamieci (AIONS/Chroma) BEZ nazw zbiorow i BEZ tresci -
scislejsza wersja niz poprzedni plik tej samej sesji, ktory ujawnil nazwy dwoch kolekcji. Ten plik
zawiera wylacznie ksztalt i liczby, nigdy zawartosc.
==========================================
-->

# Odczyt pamięci przez Darkstara — fakty bez treści, 2026-09-04

## Werdykt

Obie nogi kontekstu odpowiadają, obie mówią Chroma v2, autoryzacja działa. Zero kodu zmienione
w tej turze - warunkowa naprawa nie miała zastosowania, bo warunek okazał się fałszywy.

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

## Ostrzeżenie o poprzednim pliku tej sesji

`docs/operations/evidence/2026-09-04-context-legs.md` (commit `6763be9`, już wysłany) zawiera
dwie prawdziwe nazwy kolekcji z żywej bazy. To nie sekret w sensie hasła, ale to więcej niż
wymaga ta nowa, ściślejsza zasada. Historia gita jest nieodwracalna bez przepisania jej, a tego
nikt tu nie robi bez wyraźnej zgody - więc ten plik nie usuwa tamtych nazw, tylko je nie
powtarza. Jeśli chcesz, żebym wyczyścił tamten plik (nowym commitem, bez dotykania historii),
powiedz - zrobię to jako osobny, świadomy krok.

## Powiązane

- Poprzedni, mniej ścisły zapis: `docs/operations/evidence/2026-09-04-context-legs.md`.
- Test regresyjny v2: `crates/darkstar-server/tests/context_http.rs`,
  `both_legs_default_to_chroma_v2_matching_the_live_host` (commit `3c1e930`).
