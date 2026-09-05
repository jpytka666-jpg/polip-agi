<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-05-reflex-dictionary.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-05 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Sonnet 5
TIMESTAMP: 2026-09-05 19:45:00 Europe/London
REASON FOR CREATION: Slownik odruchow dziala - i pomiar pokazal DOKLADNIE, czego brakuje,
zeby dzialal po polsku. To jest najkrotsza droga do wyjasnienia, po co istnieje CBMS.
==========================================
-->

# Słownik odruchów — 2026-09-05

## Architektura, którą postawił Marcin

> Qwen 30B albo agent = **MÓZG**. Noworodek = **RDZEŃ KRĘGOWY**. AIONS i Darkstar = **CIAŁO**.

**Rdzeń kręgowy nie myśli — reaguje odruchowo, i zawsze tak samo.**

To przewartościowuje każdy wcześniejszy pomiar. Dziewięć różnych odpowiedzi na dwieście
kontekstów jest fatalne dla rozmówcy i **właściwe dla odruchu**, o ile te dziewięć to
właściwe bloki. Dłuższy kontekst nie jest potrzebny. Generowanie płynnego tekstu nie jest
potrzebne **w ogóle**.

Zadanie brzmi: **wziąć gotowy blok kodu, podmienić co trzeba, dopasować.**

## Co zbudowano

`reflex-store` — słownik odruchów. Wyciąga z kodu źródłowego funkcje z opisem, osadza
**opis** (nie ciało — szukanie zaczyna się od „czego potrzebuję", a to jest zdanie o
zadaniu), zapisuje do osobnej kolekcji magazynu. Tryb `--find` szuka.

Zmierzone na tym repozytorium: **579 funkcji, 84 z użytecznym opisem, wszystkie 84 osadzone,
zero pominiętych.**

## Pierwsza próba: rdzeń nie rozróżnia znaczeń

Osadzenia liczone silnikiem Noworodka, czyli w przestrzeni CBMS:

```
"policz podobienstwo dwoch wektorow"  -> validate_unit      0.9450
                                         from_env           0.9296
                                         headscale_router   0.9286
"zapisz punkty do magazynu wektorow"  -> cosine_similarity  0.8986
```

Wyniki losowe względem znaczenia, a **wszystkie oceny między 0,89 a 0,95** — wektory
wskazują niemal w tym samym kierunku niezależnie od treści.

**Przyczyna leży w moim sposobie zasiewu, nie w modelu:** wektory znaków powstają z
**losowego rzutu** sąsiedztwa, a przy 128 wymiarach i 33 tysiącach znaków ten rzut jest za
ciasny — średnia kilku takich wektorów zbiega do średniej ogólnej.

## Druga próba: silnik rozróżniający — i odkrycie

Ten sam słownik, osadzenia `all-MiniLM-L6-v2`:

| Pytanie | Pierwszy wynik | Drugi |
|---|---|---|
| **po angielsku** `cosine similarity between two vectors` | **`cosine_similarity` 0,4702** | 0,2265 |
| po polsku `zgodnosc kierunku dwoch wektorow` | `embed` 0,5997 | 0,5916 |

**System działa — po angielsku.** Zwróć uwagę na odstęp: pierwszy wynik jest **dwa razy
lepszy od drugiego**, model jest pewny. Po polsku wszystko siedzi koło 0,59 — zgadywanie.

Drugie pytanie użyło słów **wprost z opisu funkcji** (`Zgodnosc kierunku dwoch wektorow`)
i mimo to nie trafiło. To nie jest kwestia sformułowania. **To jest język.**

## Po co istnieje CBMS — najkrótsze wyjaśnienie, jakie znam

MiniLM jest modelem angielskim. Polskie „podobieństwo" i „zgodność kierunku" to dla niego
dwie niepowiązane rzeczy.

**W CBMS ten problem nie istnieje z definicji** — polskie i angielskie słowo prowadzą do
tego samego znaku, bo most esperanto sprowadza je do jednego rdzenia.

Czyli: **Noworodek jest właściwym narzędziem do tej roboty, nie MiniLM.** Brakuje mu
wyłącznie osadzeń, które rozróżniają znaczenia — a to jest wada mojej metody zasiewu,
nie modelu ani nie CBMS.

## Stan i droga dalej

| Część | Stan |
|---|---|
| Słownik odruchów, wyszukiwanie | **działa**, kolekcja `reflex_minilm`, 84 bloki |
| Rozróżnianie po angielsku | **działa**, pewne |
| Rozróżnianie po polsku | **nie działa** — wina modelu angielskiego |
| Osadzenia rdzenia | **za słabe** — losowy rzut zbyt ciasny |

**Najkrótsza droga do celu:** poprawić osadzenia Noworodka tak, żeby rozróżniały znaczenia.
Wtedy język przestaje mieć znaczenie, bo w CBMS go nie ma.

`--core` przełącza słownik na rdzeń, więc „czy już dorównał" pozostaje pytaniem, na które
odpowiada **jedno polecenie**, a nie przebudowa.

## Powiązane

- `crates/darkstar-embed/src/bin/reflex_store.rs`
- `docs/operations/evidence/2026-09-05-written-not-trained.md` — jak powstały wagi rdzenia
- `docs/operations/evidence/2026-09-04-shadow-mode.md` — gniazdo, w którym rdzeń czeka
