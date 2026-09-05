<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-05-wynik-wybor-znaczenia.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-05 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Sonnet 5
TIMESTAMP: 2026-09-06 00:20:00 Europe/London
REASON FOR CREATION: Wynik pomiaru wyboru znaczenia istniejacym MiniLM. Zbior przypadkow
i oczekiwane odpowiedzi zostaly zapisane w commicie 563fcf4 ZANIM ten pomiar zostal
uruchomiony i ZANIM istnial kod, ktory go liczy.
==========================================
-->

# Wybór znaczenia istniejącym MiniLM — wynik

Zbiór testowy i oczekiwane odpowiedzi: commit **563fcf4**, zapisany przed pomiarem.
Model: `all-MiniLM-L6-v2`, ten sam, który leży na Darkstarze od rana. Nic nowego nie dodano.

## Wynik w dwóch liczbach, nigdy zlanych w jedną

| | część A (20 słów) | część B (12 zdań) |
|---|---|---|
| poprawny **wśród kandydatów** — to mierzy słownik | **20/20 (100%)** | **11/12 (91%)** |
| poprawny **wybrany** — to mierzy model | **9/20 (45%)** | **6/12 (50%)** |
| dla porównania: branie pierwszego kandydata | 5/20 (25%) | — |

## Część B — czy model w ogóle patrzy na kontekst

To jest najważniejsza część pomiaru: sześć par, gdzie **to samo słowo** znaczy dwie różne
rzeczy.

| zdanie | wybrany | oczekiwany | |
|---|---|---|---|
| the code compiles without errors | **kodo** | kodo | ✓ |
| the message was written in a secret code | **ĉifro** | ĉifro | ✓ |
| what time is it | **horo** | horo | ✓ |
| it happened three times | **fojo** | fojo | ✓ |
| add the two vectors together | **vektoro** | vektoro | ✓ |
| the mosquito is a vector of the disease | vektora | vehiklo | ✗ |
| this painting is his finest work | **verko** | verko | ✓ |
| she went to work early | labori | laboro | ✗ |
| the computer has four gigabytes of memory | memorkapablo | memorilo | ✗ |
| he has a clear memory of that day | memorkapablo | memoro | ✗ |
| count the apples in the basket | kompti | nombri | ✗ |
| the count lived in the castle | esti konsiderata | grafo | ✗ |

**Model używa kontekstu.** W **pięciu parach na sześć** dał dwa różne rdzenie dla dwóch
różnych znaczeń. Dla `code` i `time` trafił **obie strony pary** — a to są dokładnie te
przypadki, na których wszystkie reguły mechaniczne poległy.

Jedyna para bez rozróżnienia to `memory` — ten sam rdzeń w obu zdaniach.

## Znaleziona wada metody, nie modelu

W części A dwanaście przypadków dostało ocenę **dokładnie 1.0000**. To nie jest pewność —
to znaczy, że definicja kandydata jest **dosłownie tym samym słowem** co zapytanie.

```
memorkapablo : memory                       -> ocena 1.0000, wygrywa
memoro       : memory, recollection, storage -> ocena nizsza, przegrywa
```

Czyli porównanie pojedynczego słowa z definicją **premiuje terminy wąskie i rzadkie**,
bo ich definicja to jedno słowo, a słowo podstawowe ma definicję długą. Stąd
`memory → memorkapablo`, `book → kartonalumetaro`, `hand → kampulservisto`.

**To jest wada mojej metody porównania, nie modelu.** I tłumaczy, dlaczego część A wypadła
gorzej niż część B, choć jest zadaniem łatwiejszym — w części B zapytaniem jest całe zdanie,
więc do remisu na 1.0000 nie dochodzi.

## Zaobserwowany wzorzec — NIEZMIERZONY

Wiele pomyłek to ten sam rdzeń w innej formie gramatycznej albo z innym przyrostkiem:

```
memory     memorkapablo / memorilo / memoro     -> wszystkie zawieraja memor
knowledge  sciado / scio                        -> sci
word       vorta / vorto                        -> vort
language   lingvaĵo / lingvo                    -> lingv
name       nomizi / nomo                        -> nom
work       labori / laboro                      -> labor
```

Nasuwa się, że porównywanie po rdzeniach zamiast po całych słowach podniosłoby wynik.
**Nie zostało to zmierzone i nie jest twierdzeniem.** Proste obcięcie końcówki tego nie
załatwi — `memorkapablo` bez końcówki to `memorkapabl`, a nie `memor`. Potrzebny jest
prawdziwy rozbiór morfologiczny, którego nie mamy wpiętego.

## Czego ten pomiar nie rozstrzyga

- **Nic o polskim.** MiniLM jest modelem angielskim.
- **Nic o rozumieniu przez Noworodka.** Rozbiór słowa na klocki nie jest dowodem, że model
  potrafi z tych klocków składać znaczenie. Nie mierzyliśmy tego.
- **Nic o wersji z rdzeniami.** Patrz wyżej — wzorzec zaobserwowany, nie zmierzony.

## Co zostało zmienione

Nic. Księga stoi, stary układ stoi, żaden model nie został dodany ani wpięty.

## Jak odtworzyć

```
espdic sense --dict espdic.txt --cases przypadki.tsv
```

## Powiązane

- `docs/operations/evidence/2026-09-05-zbior-testowy-wybor-znaczenia.md` — zbiór, commit 563fcf4
- `docs/operations/evidence/2026-09-05-espdic-kandydaci.md` — pomiar samego słownika
