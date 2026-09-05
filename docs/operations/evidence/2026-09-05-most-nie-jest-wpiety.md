<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-05-most-nie-jest-wpiety.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-05 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Sonnet 5
TIMESTAMP: 2026-09-05 20:05:00 Europe/London
REASON FOR CREATION: Most i pelny wszczep skonczyly sie oba. Pomiar pokazal, ze wszczep
zadzialal mechanicznie, ale NIE poprawil rozrozniania znaczen - i pokazal dokladnie
dlaczego. Jedna liczba tlumaczy wszystko: 95 rdzeni na 9235 laczy dwa jezyki.
==========================================
-->

# Most nie jest wpięty w księgę — 2026-09-05

## Jedno zdanie

Mechanizm działa, wszczep wpisał 109 408 wierszy, **rozróżnianie znaczeń nie poprawiło się
ani trochę** — bo most, który miał połączyć polski z angielskim, połączył **95 rdzeni na
9235**, czyli jeden procent.

## Co zmierzono

Pięć zdań, dwie pary znaczące to samo w dwóch językach plus jedno o czymś zupełnie innym
(przepis na ciasto) jako punkt odniesienia. Te same zdania, ta sama księga (110 354 wpisy),
dwa pliki wag.

| para | przed wszczepem | po pełnym wszczepie | jak być powinno |
|---|---|---|---|
| „policz podobieństwo…" ↔ `cosine similarity…` — **to samo** | 0,6566 | 0,5957 | najwyżej |
| „błąd połączenia…" ↔ `network connection error` — **to samo** | 0,8196 | 0,5756 | najwyżej |
| „policz podobieństwo…" ↔ „błąd połączenia…" — co innego | 0,7303 | 0,6376 | nisko |
| „błąd połączenia…" ↔ **„upiecz ciasto z jabłkami"** | **0,8403** | **0,6935** | najniżej |

**Kolejność jest odwrócona i po wszczepie nadal jest odwrócona.** Przepis na ciasto leży
bliżej błędu sieci niż angielskie tłumaczenie tego samego zdania. Cały zakres zjechał w dół
(0,64–0,84 → 0,50–0,69), ale rozstęp się nie zmienił — czyli nic się nie rozdzieliło,
wszystko po prostu zmalało.

## Dlaczego — test jednego słowa

`błąd` i `error` **są** w moście i oba prowadzą do `eraro`. Sprawdzenie, jaki znak dostają
w księdze:

```
błąd     -> znak 1910
error    -> znak 2862
eraro    -> znaki 6742, 56064   (rozbity na kawałki, nie ma własnego znaku)

pamięć   -> znak 2704
memory   -> znak 1450
memoro   -> znaki 97362, 1186   (jw.)
```

**Most jest tabelką leżącą obok księgi, a nie w środku niej.** Zapisuje, że `pamięć` i
`memory` prowadzą do `memoro`, ale księga o tym nie wie i daje im dwa niezwiązane znaki.
Sam rdzeń esperancki nie ma nawet własnego znaku — rozpada się na kawałki.

Wszczep umie to obejść: formy dzielące rdzeń uśrednia **przed** rzutem i wpisuje **ten sam
wektor** pod oba numery. Zrobił to — dla **160 znaków na 54 895**. Bo tyle znalazł.

## Skąd wzięło się te 95 rdzeni

Budowa mostu przerobiła 3000 słów. Wynik:

```
ze słownika      121
z encyklopedii    47
z angielskiego    27
własne          2805   <- słowo wskazuje samo na siebie, czyli BRAK mostu
AWARIE pobrania    0   <- to nie jest problem z siecią ani ze źródłem
różnych rdzeni  2997   (zbitych form: 3)
```

**Trzy.** Trzy słowa z trzech tysięcy trafiły na wspólny rdzeń z innym słowem.

„Własne" nie zawsze jest błędem — dla zapożyczeń technicznych (`indexer`, `inject`, `status`)
polski i angielski to **to samo słowo**, więc most jest niepotrzebny. Ale dla słów, które się
różnią, to jest po prostu brak:

```
podobieństwo -> podobieństwo (własne)      similarity -> similarity (własne)
funkcja      -> funkcja      (własne)      function   -> function   (własne)
kierunek     -> kierunek     (własne)      direction  -> direction  (własne)
kod          -> kod          (własne)      code       -> code       (własne)
wektor       -> BRAK W MOŚCIE              vector     -> BRAK W MOŚCIE
```

A wszystkie te słowa **mają** zwykłe formy w esperanto: `simileco`, `funkcio`, `direkto`,
`kodo`, `vektoro`. Zero awarii pobrania znaczy, że źródła odpowiedziały i tych słów po prostu
w nich nie było — to luka w źródle, nie usterka pobierania.

## Co z tego wynika

| Twierdzenie | Stan |
|---|---|
| Wszczepianie wektorów do wag działa mechanicznie | **PRAWDA** — 109 408 z 110 354 wierszy |
| Most potrafi połączyć dwa języki na jednym rdzeniu | **PRAWDA** — `błąd`/`error` → `eraro` |
| `pamięć` i `memory` to ten sam wektor | **NIEPRAWDA** — dwa różne znaki, dwa wektory |
| Wszczep poprawił rozróżnianie znaczeń | **NIEPRAWDA** — zmierzone, kolejność nadal odwrócona |
| Rdzeń rozumie polskie zapytanie o kod | **NIEPRAWDA** — nie ma czym |

**Wąskie gardło jest jedno i jest nazwane: most pokrywa 1% rdzeni.** Nie architektura, nie
wszczep, nie księga, nie pojemność magazynu. Most.

## Najkrótsza droga dalej

Esperanto jest językiem **planowanym** — buduje się z międzynarodowych rdzeni plus regularna
końcówka. `vektoro`, `funkcio`, `kodo`, `simileco` powstają mechanicznie, bez słownika. Most
z taką regułą zapasową objąłby te słowa, których słownik nie miał, a to są dokładnie te
słowa, na których dzisiaj poległ.

Do sprawdzenia **przed** budowaniem czegokolwiek dalej — na jednej parze, nie na trzech
tysiącach: czy po dopisaniu `podobieństwo→simileco` i `similarity→simileco` do mostu i
ponownym wszczepie te dwa zdania zbliżą się do siebie bardziej niż do przepisu na ciasto.
Jeśli tak, reguła zapasowa ma sens dla całej reszty. Jeśli nie — problem leży głębiej niż
w moście i trzeba go szukać gdzie indziej.

## Jak odtworzyć

```
cbms ksiega-max.txt ids zdanie.txt zdanie.u32
noworodek-embed <wagi.nwrd> a.u32 b.u32 ... --ids32
```

`--ids32` jest **konieczne**. Księga ma 110 354 wpisy, więc numery znaków nie mieszczą się
w 16 bitach. Odczyt szerokich numerów jako wąskie nie wywala się — po cichu tnie każdy numer
i pomiar wychodzi sensownie wyglądający i nieprawdziwy.

## Powiązane

- `crates/darkstar-embed/src/bin/implant.rs` — wszczep
- `crates/darkstar-seed/src/bin/noworodek_embed.rs` — pomiar (poprawiony o `--ids32`)
- `docs/operations/evidence/2026-09-05-reflex-dictionary.md` — pomiar, który postawił pytanie
