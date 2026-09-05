<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-05-espdic-kandydaci.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-05 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Sonnet 5
TIMESTAMP: 2026-09-05 23:30:00 Europe/London
REASON FOR CREATION: Zielone swiatlo dotyczylo wylacznie wciagniecia ESPDIC jako zrodla
kandydatow, z warunkiem zachowania wszystkich znaczen. To jest pomiar tego kroku: pokrycie,
wieloznacznosc i poprawnosc podane OSOBNO, bo zlane w jedna liczbe ukrylyby caly problem.
Uruchomione na Darkstarze.
==========================================
-->

# ESPDIC jako źródło kandydatów — pomiar 2026-09-05

## Jedno zdanie

Słownik podaje **poprawny rdzeń w 20 przypadkach na 20**, ale **jako pierwszy tylko w 5** —
czyli źródło jest bez zarzutu, a wybór znaczenia jest osobnym, niezałatwionym problemem.

## Co wciągnięto

| | |
|---|---|
| plik | ESPDIC, Paul Denisowski, licencja wolna |
| haseł esperanckich | **63 894** |
| po odwróceniu, haseł angielskich | **56 001** |
| w tym jednowyrazowych | 31 486 |
| w tym wielowyrazowych | 24 515 |
| pominiętych (czyste objaśnienia w nawiasie) | 389 |

Zachowane są **wszystkie** znaczenia. Rdzeń, u którego szukane słowo stoi na dziesiątym
miejscu, też trafia na listę — odrzucenie go byłoby cichym wyborem.

## 1. Pokrycie — czy słownik w ogóle zna słowo

Lista 3000 słów zebranych z naszego materiału:

```
zna słownik      874   (29%)
nie zna         2126   (70%)
```

Dla porównania, przed wciągnięciem ESPDIC: słownik 5% + reguła 3% = **8%**.

## 2. Wieloznaczność — ile z tego wymaga wyboru

Liczona **tylko wśród znanych**, bo to inne pytanie niż pokrycie:

```
jednoznacznych   195   (22% znanych)
wymaga wyboru    679   (77% znanych)
średnio kandydatów: 3,76
```

| kandydatów | słów |
|---|---|
| brak w słowniku | 2126 |
| 1 | 195 |
| 2 | 170 |
| 3 | 129 |
| 4 | 116 |
| 5 | 89 |
| 6 lub więcej | 175 |

## 3. Poprawność — na 20 sprawdzonych parach

Standardowe, niesporne słowa esperanckie. Pytanie brzmi **„czy poprawny rdzeń jest wśród
kandydatów"**, a osobno **„czy jest pierwszy"**.

| angielskie | kand. | pozycja | co stoi przed poprawnym |
|---|---|---|---|
| error | 5 | **1** | — |
| similarity | 1 | **1** | — |
| function | 2 | **1** | — |
| direction | 2 | **1** | — |
| house | 2 | **1** | — |
| code | 4 | 2 | ĉifro (szyfr) |
| water | 5 | 2 | akvi |
| book | 3 | 2 | kartonalumetaro |
| hand | 5 | 2 | kampulservisto |
| friend | 2 | 2 | amikino |
| memory | 4 | 3 | memorilo, memorkapablo |
| vector | 3 | 3 | vehiklo (pojazd), vektora |
| word | 4 | 3 | parolo, vorta |
| language | 3 | 3 | idiomo, lingvaĵo |
| number | 4 | 3 | domnumero, nombri |
| name | 5 | 3 | nomi, nomizi |
| day | 3 | 3 | diurno, tagnokto |
| time | 5 | 5 | fojo, horo, takto, tempomezuri |
| knowledge | 7 | 7 | instruiteco, konado, kono, sciado, sciaĵo, sciaro |
| work | 14 | 11 | dungiteco, efiki, ekspluati, ellaboraĵo, funkciigi… |

```
poprawny JEST wśród kandydatów : 20/20  (100%)
poprawny jest PIERWSZY         :  5/20  ( 25%)
```

## Co z tego wynika

**Źródło jest dobre.** Stuprocentowa obecność poprawnego rdzenia na liście oznacza, że
ESPDIC nadaje się na stopień 2 bez zastrzeżeń.

**Branie pierwszego byłoby błędem w trzech przypadkach na cztery.** To nie jest wada
słownika — słownik jest ułożony alfabetycznie po esperancku, więc pozycja nie niesie żadnej
informacji o znaczeniu. `kartonalumetaro` (pudełko zapałek) stoi przed `libro` wyłącznie
dlatego, że zaczyna się na „k".

**Trudność rośnie z liczbą kandydatów, ale nie jest wprost proporcjonalna.** `work` ma 14
kandydatów i poprawny jest jedenasty; `knowledge` ma 7 i poprawny jest siódmy. Formy
pochodne (`labori`, `laborado`, `sciado`, `sciaĵo`) wypełniają listę przed formą podstawową.

## Czego ten pomiar NIE mówi

- **Nie mówi nic o polskim.** ESPDIC to wyłącznie esperanto↔angielski.
- **Nie mówi, jak wybierać.** Trzy mechaniczne reguły sprawdzone wcześniej tego dnia
  (pierwszy, ocena po pozycji, tylko formy na -o) dawały odpowiednio 25%, gorzej, i lepiej
  ale przy niższym pokryciu. Wybór wymaga rozumienia znaczenia.
- **Nie zmienia niczego, co działa.** Księga stoi nietknięta, stary układ stoi nietknięty.

## Jak odtworzyć

```
espdic report --dict espdic.txt --words lista.txt     # pokrycie + wieloznacznosc
espdic check  --dict espdic.txt --gold pary.tsv       # poprawnosc
espdic index  --dict espdic.txt --out kandydaci.tsv   # angielskie<TAB>rdzen1|rdzen2|...
```

## Powiązane

- `crates/darkstar-embed/src/bin/espdic.rs`
- `docs/operations/design/2026-09-05-tokenizer-hybryda.md` — gdzie ten stopień siedzi w całości
