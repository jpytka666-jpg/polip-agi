<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-05-tokenizer-hybryda.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-05 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Sonnet 5
TIMESTAMP: 2026-09-05 22:40:00 Europe/London
REASON FOR CREATION: Marcin: "ZOBACZ JAK DZIALAJA WSZYSTKIE TE APKI PRZESANALIZUJ ICH
BUDOWE POCZYM ZBUDUJ Z NICH JEDNA HYBRYDE KTORA BEDZIE DLA NAS IDEALNA". To jest ta analiza
i ten projekt. Powstal, bo probowalem budowac tlumacza metoda prob i bledow na jednolinijkowcach
zamiast przeczytac, jak zrobili to ludzie, ktorzy juz to zrobili.
==========================================
-->

# Tokenizer-tłumacz: analiza istniejących narzędzi i projekt hybrydy

## Po co to jest

Warstwa esperanto **jest tokenizerem Noworodka**, nie pomocnikiem przed nim. Wejście
w dowolnym języku zamienia się na znaki CBMS i dopiero to widzi rdzeń. W drugą stronę ta sama
warstwa obrabia odpowiedź na angielski — i tylko na angielski.

## Co już istnieje i jak jest zbudowane

### Apertium — tłumaczenie regułowe, bez AI

Potok dziewięciu stopni, z których dla nas liczą się cztery:

| Stopień | Co robi |
|---|---|
| analiza morfologiczna | słowo → forma podstawowa + znaczniki |
| ujednoznacznienie | wybiera właściwy rozbiór (Constraint Grammar) |
| przekład słownikowy | forma źródłowa → forma docelowa |
| **wybór znaczenia** | rozstrzyga, gdy jedno słowo ma kilka odpowiedników |

Formaty: `.dix` (słowniki morfologiczne), `.rlx` (reguły ujednoznaczniania), `.lrx` (wybór
znaczenia). Pary z esperanto: **eo↔en, eo←ca, eo←es, eo←fr**. Polskiego nie ma.

**Do wzięcia:** kształt potoku i to, że **wybór znaczenia jest osobnym, nazwanym stopniem** —
nie doklejką do słownika. Dokładnie tam, gdzie się dziś wywróciłem.

### ESPDIC — słownik esperanto→angielski

Jeden plik tekstowy, `rdzeń : definicje`, licencja wolna. **Pobrany i sprawdzony na
Darkstarze: 63 897 haseł, 1,9 MB, kilka sekund.** Ma wszystko, na czym poległa moja reguła —
`simileco : similarity`, `memoro : memory`, `eraro : error`, `funkcio : function`.

**Do wzięcia:** cały słownik. To jest kręgosłup pojęć, za darmo.

### Morfologik — odmiana polska, bez AI

Automat skończony (FSA) ze słownikiem morfologicznym polskiego. Zwraca formę podstawową
i znaczniki. Mały i szybki, bo FSA zamiast XML-a.

**Do wzięcia:** rozwiązanie problemu, który zmierzyłem — w naszej liście siedzą
`wykonanej`, `pokazaniu`, `modyfikowane`, a żaden słownik takich form nie ma.

### Morfologia esperanta — rozbiór na klocki

Esperanto jest **celowo sklejane**. Zmierzone przez innych: **17 000 rdzeni pokrywa ponad
50 000 słów** przez afiksy i złożenia. Pierwotny zestaw Zamenhofa to 900 rdzeni, słownik
z 1894 dodał 1740. Analizator EspGram ma 4,3% błędu przy niemal pełnym pokryciu.

`malsanulejo` = `mal` + `san` + `ul` + `ej` + `o` — pięć klocków, jedno słowo.

**Do wzięcia:** to jest największa wygrana i o niej za chwilę osobno.

### Wspólny mianownik wszystkich czterech

**Automaty skończone.** Apertium, Morfologik, PReVo — wszystkie stoją na tym samym.
Wyszukanie trwa mikrosekundy, słownik waży kilka megabajtów, procesor prawie nie pracuje.
Zero AI w torze.

## Projekt hybrydy

```
WEJŚCIE — dowolny język, slang, brud, skróty
   │
   │ [1] ZDJĘCIE ODMIANY                          FSA, mikrosekundy
   │     PL: Morfologik   EN: proste reguły
   │     "wykonanej" → "wykonać"
   ▼
   │ [2] SŁOWNIK DWUJĘZYCZNY                      FSA, mikrosekundy
   │     ESPDIC + ReVo + Apertium .dix
   │     "wykonać" → plenumi | fari | efektivigi     ← kilku kandydatów
   ▼
   │ [3] WYBÓR ZNACZENIA    ← JEDYNE miejsce z modelem, i tylko gdy kandydatów >1
   │     wektory; poniżej progu → NIEZNANE, nigdy zgadnięcia
   │     wynik zapisywany do słownika NA STAŁE
   ▼
   │ [4] ROZBIÓR ESPERANTA                        FSA, mikrosekundy
   │     "malsanulejo" → mal + san + ul + ej + o
   ▼
   │ [5] ZNAKI CBMS — jeden znak na rdzeń, jeden na afiks, jeden na końcówkę
   ▼
NOWORODEK

DROGA POWROTNA:  znaki → rdzenie → ESPDIC eo→en → ANGIELSKI
```

## Co ta hybryda daje

### Słownik znaków kurczy się sześciokrotnie i pokrywa więcej

| | teraz | po hybrydzie |
|---|---|---|
| znaków w księdze | 110 354 | **~17 000** |
| pokrycie | niepełne, 90% słów bez rdzenia | ponad 50 000 słów |
| nowy język | dziesiątki tysięcy nowych znaków | **zero nowych znaków** |

Znaki przestają być „po jednym na słowo każdego języka", a stają się „po jednym na klocek,
z których wszystkie języki się składają".

### Materiał na znak rośnie sześciokrotnie bez dopisania jednego zdania

Dziś `pamięć`, `memory` i `memoro` dzielą materiał na trzy części, a `wykonanej`,
`wykonać` i `wykonany` na kolejne trzy. Po hybrydzie wszystko to trafia w te same klocki.
Ten sam korpus, sześć razy gęstszy sygnał na znak.

### Koszt sprzętowy prawie zerowy

Stopnie 1, 2, 4 i 5 to automaty skończone — mikrosekundy i kilka megabajtów pamięci.
Model jest tylko w stopniu 3, tylko przy wieloznaczności, **i tylko przy budowaniu**: wynik
idzie do słownika na stałe, więc przy pytaniu model już nie pracuje.

To spełnia warunek, który postawiliśmy rdzeniowi: odruch ma być natychmiastowy.

### Model wreszcie dostaje zadanie, które umie

Zmierzone dzisiaj: model **wymyślający** rdzeń produkuje `count→kounto` i `doc→doko`.
Model **wybierający** z listy prawdziwych kandydatów nie może wymyślić nic — najwyżej wybrać
gorzej. To jest różnica między zgadywaniem a rozstrzyganiem, i to jest ta sama zasada,
którą trzyma rdzeń kręgowy.

## Dowód, że stopień 3 jest potrzebny — zmierzone dzisiaj

Odwrócenie ESPDIC bez wyboru znaczenia, trzy próby:

| słowo | brałem pierwszy | ocena po pozycji | tylko formy na -o | poprawnie |
|---|---|---|---|---|
| similarity | simileco ✓ | simileco ✓ | simileco ✓ | simileco |
| error | eraro ✓ | prierara ✗ | eraro ✓ | eraro |
| memory | memorilo ✗ | memorilo ✗ | memoro ✓ | memoro |
| code | ĉifro ✗ | kodo de signaro ✗ | ĉifro ✗ | kodo |
| vector | vehiklo ✗ | vektora ✗ | vehiklo ✗ | vektoro |
| count | grafo ✗ | grafo ✗ | grafo ✗ | nombro |
| pokrycie 3000 słów | 24% | 24% | 16% | — |

`grafo` znaczy **hrabia**. `vehiklo` to **pojazd** (nosiciel choroby). `ĉifro` to **szyfr**.
Trzy różne mechaniczne reguły, trzy razy te same pomyłki — bo wybór znaczenia **nie jest
zadaniem mechanicznym**. Kandydaci są prawdziwi, wybór wymaga rozumienia.

## Kolejność budowania — od najtańszego

| Krok | Co | Zysk | Koszt |
|---|---|---|---|
| 1 | wciągnąć ESPDIC jako słownik stopnia 2 | pokrycie 8% → 24% | pół godziny |
| 2 | rozbiór esperanta na rdzenie i afiksy | znaków 110k → 17k | dzień |
| 3 | zdjęcie odmiany polskiej | polskie zapytania w ogóle zaczynają działać | dzień |
| 4 | wybór znaczenia na wektorach | reszta wieloznaczności | dzień |
| 5 | wybić księgę na nowo z rdzeni | to, po co całość istnieje | pół dnia |

Kroki 1–4 nie ruszają niczego, co dziś działa. Krok 5 jest jedynym, który wymienia księgę,
i dopiero po nim ma sens ponowne uczenie rdzenia.

## Czego jeszcze nie wiem

- **Polski → esperanto**: ESPDIC daje tylko angielski. Do sprawdzenia ReVo (ma tłumaczenia
  na kilkadziesiąt języków) i Wikisłownik. Obejście: polski → angielski → esperanto.
- **Morfologik jest w Javie**, a my w Rust. Do sprawdzenia, czy da się użyć samych danych
  słownika przez rustową bibliotekę automatów, zamiast portować kod.
- **Lista 17 000 rdzeni** — czy jest dostępna osobno, czy trzeba ją wyprowadzić z ESPDIC
  przez rozbiór.
- **EspGram** — czy w ogóle jest do pobrania.

## Źródła

- [Apertium — architektura systemu](https://wiki.apertium.org/wiki/Apertium_system_architecture)
- [ESPDIC — Paul Denisowski](http://www.denisowski.org/Esperanto/ESPDIC/espdic_readme.html)
- [Morfologik — narzędzia FSA i słownik polskiego](https://github.com/morfologik/morfologik-stemming)
- [Słownik morfologiczny esperanta z częstościami morfemów (LREC 2016)](https://aclanthology.org/L16-1171/)
- [Słownik ze świadomością morfologii dla esperanta — Blahuš](https://nlp.fi.muni.cz/raslan/2009/papers/15.pdf)
- [Reta Vortaro — słownik wielojęzyczny](https://en.wikipedia.org/wiki/Reta_Vortaro)
- [Słownictwo esperanta — liczba rdzeni](https://en.wikipedia.org/wiki/Esperanto_vocabulary)
