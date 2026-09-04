<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-04-shadow-mode.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-04 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Sonnet 5
TIMESTAMP: 2026-09-04 23:50:00 Europe/London
REASON FOR CREATION: Noworodek ma sie docelowo nauczyc obslugiwac AIONS i pisac kod. Ten
dokument zapisuje, na jakiej zasadzie zaczyna patrzec, i czym udowodniono, ze patrzenie
nie moze przejsc w dzialanie.
==========================================
-->

# Tryb cienia — 2026-09-04

## Werdykt

**Uczeń ma gniazdo, w którym widzi każde zadanie, i nie ma z niego drogi do wyniku.
Zakaz działania jest wymuszony przez kompilator, nie przez komentarz.** Noworodek nie
został tknięty — jego katalog roboczy ma tę samą gałąź, ten sam commit i tę samą liczbę
niezapisanych zmian co przed pracą.

## Co powstało

`crates/darkstar-shadow` — jeden nowy crate, trzy pliki, zero nowych zależności.

| Plik | Rola |
|---|---|
| `record.rs` | Co zapisujemy: obserwacja osadzenia albo działania agenta |
| `journal.rs` | Gdzie to ląduje: plik JSONL otwierany wyłącznie w trybie dopisywania |
| `lib.rs` | `Embedder` — kontrakt silnika; `ShadowedEmbedder` — dwa silniki, jeden głos |

## Dlaczego uczeń nie może nic zepsuć

Trzy niezależne zabezpieczenia, każde sprawdzone testem:

**1. Brak drogi do ucznia.** Pole `shadow` jest prywatne i nie ma metody, która by je
zwracała. Jedyna publiczna ścieżka do wektora — `embed()` — czyta wyłącznie silnik żywy.

**2. Test, który pilnuje tej granicy.** W dokumentacji siedzi przykład sięgający po ucznia
z zewnątrz, oznaczony `compile_fail`. Rust sprawdza, że ten kod **się nie buduje**. Gdyby
ktoś kiedyś udostępnił ucznia, test zrobi się czerwony — bo zakazany kod nagle by przeszedł.

**3. Wywrotka ucznia nie przewraca systemu.** Wywołanie ucznia jest opakowane w
`catch_unwind`. Model w trakcie treningu ma prawo paść; test z uczniem, który naprawdę
panikuje, kończy się poprawną odpowiedzią z modelu żywego i zapisem wywrotki w dzienniku.

## Zmierzone

```
cargo test -p darkstar-shadow
  14 testow jednostkowych        PASS
   1 doctest compile_fail        PASS  <- proba siegniecia po ucznia sie NIE kompiluje
  test result: ok. 14 passed; 0 failed
  test result: ok. 1 passed; 0 failed

cargo build --workspace          Finished dev profile
cargo clippy -p darkstar-shadow -- -D warnings
                                 Finished, zero ostrzezen
```

Testy wartościowe merytorycznie, nie tylko liczbowo:

| Test | Co dowodzi |
|---|---|
| `the_answer_comes_from_the_live_model_only` | Uczeń zwraca `[-9,-9,-9]`, system zwraca wektor żywego. Głos ucznia nie przecieka |
| `a_shadow_that_crashes_does_not_bring_the_system_down` | Uczeń panikuje, `embed()` nadal zwraca `Ok` |
| `different_dimensions_are_recorded_without_a_fabricated_score` | 128 wobec 384 daje `cosine: None`, nie zmyśloną liczbę |
| `reopening_appends_instead_of_erasing` | Restart serwera nie kasuje materiału zbieranego tygodniami |
| `a_failing_shadow_is_recorded_not_hidden` | Błąd ucznia to obserwacja, nie awaria do przemilczenia |

## Format zbioru uczącego

JSONL — jedna obserwacja na linię, czytana przez trenery bez konwersji. Dwa rodzaje:

```
{"kind":"embedding", text, live_model, shadow_model, live_dims, shadow_dims,
                     cosine, live_ms, shadow_ms, shadow_error}
{"kind":"action",    actor, task, tool, input, outcome, summary}
```

`embedding` odpowiada na pytanie „czy uczeń już dorównuje". `action` to materiał na cel
właściwy: **obsługiwanie systemu i pisanie kodu, nie prowadzenie rozmowy** — i zbiera się
niezależnie od tego, czy jakikolwiek model osadzeń jest wpięty.

Trzy pola są tam celowo i warto wiedzieć po co:

- `cosine: None` zamiast `0.0`, gdy porównanie nie ma sensu. Zero wygląda jak pomiar, a nim
  nie jest — i po miesiącu nikt już nie pamięta, które zera były prawdziwe.
- `shadow_ms` obok `live_ms`. Uczeń trafiający lepiej, ale wolniejszy o rząd wielkości, nie
  nadaje się do zamiany. Bez pomiaru czasu wyszłoby to dopiero po wdrożeniu.
- `ActionOutcome::Unknown` jako trzeci stan. Uczeń ma widzieć także to, że system czasem nie
  wie — inaczej nauczy się, że pewność jest zawsze uzasadniona.

## Czego jeszcze nie ma

**Żaden prawdziwy silnik nie jest wpięty.** Powstało gniazdo i reguła, nie modele. Do
działania brakuje dwóch implementacji `Embedder`: dla `all-MiniLM-L6-v2` (żywy) i dla
Noworodka (uczeń). To jest następny etap.

**Uczeń siedzi w tym samym procesie.** `catch_unwind` łapie panikę, ale nie złapie
zapętlenia ani zajechania pamięci. Docelowo uczeń idzie do osobnego procesu — wtedy ten
strażnik staje się zbędny, a ochrona pełna. Dziś to znane ograniczenie, nie przeoczenie.

**Dziennik rośnie bez ograniczeń.** Brak rotacji i brak próbkowania. Przy realnym ruchu
trzeba będzie jedno albo drugie; teraz byłoby to rozwiązywanie problemu, którego nie ma.

**Docelowa ścieżka na Darkstarze:** `/var/lib/darkstar/shadow/` — ext4, nigdy
`/mnt/aionslab`; exFAT nie daje rzetelnych blokad pliku, a dziennik pisze wiele wątków.

## Noworodek — nietknięty

```
katalog:      C:\temp\aions-cbms-train
galaz:        noworodek-cbms-training   (bez zmian)
commit:       9871a33                   (bez zmian)
niezapisane:  5 plikow                  (tyle samo co przed praca)
```

Nic w tym katalogu nie było czytane poza źródłami, i nic nie było zapisywane.

## Powiązane

- `crates/darkstar-shadow/` — kontrakt, dziennik, format
- `docs/operations/evidence/2026-09-04-memory-moved.md` — przeprowadzka pamięci, etap poprzedni
- commit `d435be7` — `feat(darkstar): shadow mode - the student watches, never acts`
