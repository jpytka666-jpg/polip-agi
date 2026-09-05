<!-- darkstar-header-v1 -->
<!-- po co: STAN-2026-09-06-tokenizer.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-06 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Sonnet 5
TIMESTAMP: 2026-09-06 00:45:00 Europe/London
REASON FOR CREATION: Marcin: "Zapisz stan do przejecia." Ten plik ma wystarczyc komus,
kto siada do tego bez zadnej wiedzy z poprzedniej rozmowy - i mnie samemu po skroceniu
kontekstu. Zawiera to, co ZMIERZONE, oddzielone od tego, co tylko przypuszczane.
==========================================
-->

# Stan do przejęcia — warstwa językowa Darkstara, 2026-09-06

## Zasady, które obowiązują. Przeczytaj przed pierwszą komendą

| Zasada | Skąd |
|---|---|
| **Praca powstaje na Darkstarze** (`owner@100.64.0.2`), nie na Windows | Marcin, 2026-09-05 |
| **Księga i wagi nietknięte.** Stary układ zostaje jako droga zapasowa i przełącznik języka | Marcin |
| **Nie zamrażaj jednego tłumaczenia** dla słowa wieloznacznego. Zachowaj znaczenia, gramatykę i źródła; wybór ma zależeć od konkretnego użycia | Marcin |
| **MiniLM zostaje** tam, gdzie działa, ale **nie wolno nim zatwierdzać nowych znaczeń** przy obecnej jakości (45–50%) | Marcin |
| **Morfemy to struktura wejścia, nie dowód rozumienia.** Nie opisuj składania znaczeń jako działającego, dopóki nie zmierzone | Marcin |
| Zanim nazwiesz zmianę poprawą — **porównaj obie metody na tych samych przypadkach**, potem sprawdź na przykładach **odłożonych** | Marcin |
| Zero „gotowe" bez bloku DOWÓD albo NIEZWERYFIKOWANE | stała |
| Nic destrukcyjnego bez zgody. Bez `git add -A` | stała |

## Gdzie co jest

**Darkstar** `owner@100.64.0.2` — Ubuntu, 4 rdzenie, 7 GB RAM, 46 GB wolnego.

```
~/polip-agi/                      repo, gałąź docs/darkstar-headscale-hotspot-plan
~/darkstar-artefakty-2026-09-05/  wszystkie dane robocze
~/darkstar-embed.env              ORT_DYLIB_PATH + katalog modelu; wczytaj przed uzyciem
~/models/all-MiniLM-L6-v2/        model osadzen
~/models/qwen2.5-0.5b/            maly model wielojezyczny, NIEUZYWANY
~/tools/cbms/                     zrodlo cbms; MINT_RANGES [(u32,u32); 9]
```

Rust nie jest widoczny przez `which` po nieinteraktywnym ssh — dodaj `~/.cargo/bin` do PATH.
`sudo` wymaga hasła, więc **niczego nie instalujemy w systemie**. `sqlite3` leży rozpakowany
w `~/darkstar-artefakty-2026-09-05/narzedzia/root/usr/bin/sqlite3`; przed użyciem ustaw
`LD_LIBRARY_PATH` na `.../narzedzia/root/usr/lib/x86_64-linux-gnu`.

## Dane, które zdobyliśmy 2026-09-05

| plik | co to | rozmiar |
|---|---|---|
| `espdic.txt` | słownik esperanto→angielski, **63 894 hasła**, licencja wolna | 1,9 MB |
| `most-pl-eo.tsv` | **51 775 par polski→esperanto**, 35 474 różnych słów polskich | 1,0 MB |
| `pl-eo/eo_morf.dic` | **morfologia esperanta, 29 571 wpisów**, format Hunspell | 407 KB |
| `pl-eo/eo_morf.aff` | reguły sklejania klocków | 20 KB |
| `pl-eo/pl.dict` | automat skończony dla polskiego | 654 KB |
| `pl-eo/evortaro-*.sql` | słowniki dla 10 języków (de, en, fr, ru, pl, be, hu, nl, pt, sv) | 24 MB razem |
| `kandydaci-en.tsv` | odwrócony ESPDIC: `angielskie<TAB>rdzeń1\|rdzeń2\|...` | 1,6 MB |

Wszystko z paczki **Esperantilo** (GPL) i **ESPDIC** (CC-BY). Polski słownik jest drugi
co do wielkości po niemieckim — Zamenhof był z Białegostoku i to widać w zasobach.

## Co zmierzone — liczby, nie wrażenia

### Pokrycie listy 3000 słów

```
na starcie (nasz most + regula)          8%
ESPDIC angielski                        29%
ESPDIC + slownik polski             1137/3000  = 37%
   oba slowniki       50
   tylko polski      263
   tylko angielski   824
   zaden            1863
```

### Wieloznaczność wśród znanych

```
polski    : 183 z 313 ma wiecej niz jednego kandydata
angielski : 679 z 874 ma wiecej niz jednego kandydata  (srednio 3,76)
```

### Wybór znaczenia — MiniLM, zbiór zapisany w commicie `563fcf4` PRZED pomiarem

| | część A: pojedyncze słowa | część B: całe zdania |
|---|---|---|
| poprawny **wśród kandydatów** (mierzy słownik) | 20/20 (100%) | 11/12 (91%) |
| poprawny **wybrany** (mierzy model) | 9/20 (45%) | 6/12 (50%) |
| branie pierwszego, bez modelu | 5/20 (25%) | — |

**Model używa kontekstu**: w 5 parach na 6 dał różne rdzenie dla różnych znaczeń;
dla `code` (kodo/ĉifro) i `time` (horo/fojo) trafił obie strony.

**Wada metody z części A**: 12 przypadków dostało ocenę równo `1.0000`, bo definicja
kandydata jest dosłownie tym samym słowem co zapytanie. Przez to terminy rzadkie o
jednowyrazowej definicji biją słowa podstawowe o definicji długiej
(`memorkapablo : memory` bije `memoro : memory, recollection, storage`).
**Część B tej wady nie ma**, bo zapytaniem jest zdanie.

### Wszczep do wag — układ stary, zapasowy

Policzony na Darkstarze na **kompletnym** moście (12 213 rdzeni, 116 łączących języki).
Wcześniejszy przebieg na Windows czytał most w trakcie zapisu i widział 9 235 / 95.

```
przed wszczepem  ->  po wszczepie
pl-podobienstwo <-> en-similarity   0.6566  ->  0.5957
pl-blad         <-> en-error        0.8196  ->  0.5756
pl-blad         <-> pl-ciasto       0.8403  ->  0.6935   <- PRZEPIS NA CIASTO
```

Kolejność odwrócona przed i po. **Wszczep nie poprawił rozróżniania znaczeń.**
Przyczyna zmierzona: most jest tabelką obok księgi, nie w środku niej —
`błąd` → znak 1910, `error` → znak 2862, `eraro` bez własnego znaku.

## Co zbudowane i działa

| program | co robi | testy |
|---|---|---|
| `spine` | zadanie → decyzja (odruch albo oddanie mózgowi) + zapis do dziennika | 4 |
| `spine-loop` | różnica bloku wobec pamięci, dopisanie wyniku, dziennik → materiał | 9 |
| `espdic` | ESPDIC jako **kandydaci**; tryby index / report / check / sense | 6 |
| `lathe` | obrabiarka, warstwy słownik + reguła; **bez pieczątki** | 12 |
| `tokenize` | tokenizer przez wyszukiwanie: próg **i** odstęp | 3 |
| `implant` | wszczepianie znaczeń do wag | — |
| `reflex-store`, `recall-find`, `nwrd-diff` | z wcześniejszych dni, działają | — |

Wszystko przechodzi `clippy --all-targets -- -D warnings`.

## Czego NIE wolno powiedzieć, że działa

- **Noworodek składający znaczenie z morfemów** — nigdy nie mierzone.
- **Porównywanie po rdzeniach zamiast po słowach** — wzorzec zaobserwowany
  (memorkapablo/memoro, sciado/scio, vorta/vorto), **nie zmierzony**. Obcięcie samej
  końcówki tego nie da: `memorkapablo` bez końcówki to `memorkapabl`, nie `memor`.
- **Prognoza 110k → 17k znaków** — 110 354 to WPISY obecnej księgi (formy powierzchniowe),
  17 000 to RDZENIE esperanta z cudzej publikacji. **Inne jednostki, u nas niezmierzone.**
- **Polski po stronie modelu** — MiniLM jest angielski, nie sprawdzany na polskim.

## Następne kroki, w kolejności, z rygorem

1. **Porównać metodę A i B na tych samych przypadkach.** Część B już porównuje zdania
   z definicjami i daje 6/12. Zmiana metody jest poprawą dopiero wtedy, gdy wygra na
   identycznym zbiorze — nie na innym.
2. **Odłożyć nowe przykłady** i sprawdzić na nich po każdym strojeniu. Zbiór, na którym
   się stroi, przestaje mierzyć.
3. **Wpiąć `eo_morf.dic`** jako rozbiór wejścia — i zmierzyć, ile znaków naprawdę wychodzi.
   To zastąpi prognozę liczbą.
4. **Zdjęcie odmiany polskiej** — w liście siedzą `wykonanej`, `pokazaniu`, `modyfikowane`,
   których nie ma w żadnym słowniku.
5. Dopiero po 1–4: rozważyć wybicie księgi na nowo. **Nie wcześniej.**

Wybór znaczenia zostaje **osobnym stopniem**, tak jak `lexical selection` w Apertium.
Słownik trzyma wszystkich kandydatów z gramatyką i źródłem; stopień wyboru czyta kontekst.
Żadne słowo wieloznaczne nie dostaje jednego zamrożonego tłumaczenia.

## Ślad w repozytorium

```
563fcf4  zbior testowy zapisany PRZED pomiarem
7be98b9  wynik: MiniLM 9/20 i 6/12, uzywa kontekstu
af7d907  ESPDIC: poprawny rdzen 20/20 wsrod kandydatow, 5/20 pierwszy
35770a7  projekt hybrydy z analizy Apertium / ESPDIC / Morfologik / morfologii eo
5949b1a  wyrzucenie pieczatki, ktora stemplowala 94% slow
dba8e3a  most jest tabelka obok ksiegi, nie w srodku
e30ea59  petla zwrotna + poprawka 32-bitowa
a58b566  rdzen kregowy: decyduje, nie wyszukuje
```
