<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-05-noworodek-first-weights.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-05 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Sonnet 5
TIMESTAMP: 2026-09-05 12:15:00 Europe/London
REASON FOR CREATION: Noworodek dostal pierwsze wagi w historii projektu. Rownie wazne jest
to, CO to udowodnilo: problemem nie byla wielkosc modelu, tylko chudosc materialu.
==========================================
-->

# Noworodek — pierwsze wagi — 2026-09-05

## Werdykt

**Noworodek ma wagi. Pierwsze, jakie istnieją.** Godzinę wcześniej wyszukiwanie `*.nwrd`
na wszystkich dyskach i na Darkstarze dawało zero trafień.

**I wiadomo już, co go blokowało: materiał, nie wielkość modelu.** Ten sam model, ta sama
maszyna, te same ustawienia — zmieniony wyłącznie materiał do nauki.

## Porównanie, w którym zmieniła się jedna rzecz

| | Materiał stary | Materiał z Darkstara |
|---|---|---|
| Symboli w materiale | 142 685 | **635 306** |
| Różnych symboli | 4 589 | 5 312 |
| Wystąpień na symbol | 31 | **120** |
| **Kroków przed zatrzymaniem** | **4 z 20** | **300 z 300 — pełne** |
| Odłożone po nauce | 10,8865 | **8,5487** |
| Przewaga nad zgadywaniem | 0,008 bitu | **3,381 bitu** |
| Wciąż gorszy od częstości o | 7,974 bitu | 4,552 bitu |
| Czas | 41,7 s | 782,9 s |

Model, ustawienia i maszyna identyczne. **Jedyna zmienna to materiał.**

Przewaga nad czystym zgadywaniem urosła **czterysta dwadzieścia trzy razy**. Ale
najważniejsza jest inna liczba: **300 kroków zamiast 4**. Wcześniej nauka kończyła się po
czterech krokach, bo odłożona część danych przestawała się poprawiać. Teraz szła do końca.

## Skąd wziął się materiał

31 kolekcji magazynu Darkstara, 4 631 wspomnień, 1 358 249 bajtów tekstu. Zebrane przez
`tools/collect-corpus`, magazyn wyłącznie czytany.

Dwa wspomnienia pominięte: **wiersz 3457 i 4611 — ten sam tekst, duplikat**, dane zdrowotne.
Narzędzie kodujące odmawiało na nich odtworzenia tekstu i wolało zwrócić błąd, niż wydać
materiał, który nie odpowiada źródłu. Została setna procenta poniżej całości: **4 629
z 4 631**.

Materiał końcowy przeszedł `round trip: exact` — po zakodowaniu odtwarza się co do znaku.

## Sufit, w który uderzyła księga

Przy próbie dopisania wszystkich brakujących słów narzędzie zgłosiło:

```
NO SYMBOLS LEFT: 9354 words went unminted - widen MINT_RANGES
```

**Wyczerpały się wolne znaki.** Księga ma wyznaczone zakresy Unicode, z których bierze
symbole, i doszła do ich końca przy 34 726 wpisach. Pokrycie stanęło na 95,95%.

Przy progu 3 wystąpień (czyli bez jednorazówek, których model i tak by się nie nauczył)
mieści się bez problemu: 3 853 nowe wpisy, pokrycie 69,28% → **92,72%**, księga 30 590
wpisów, wczytuje się poprawnie.

To jest ograniczenie do zapamiętania: **im więcej materiału, tym szybciej się w nie uderza.**

## Co nadal nie działa

**Model nie używa kontekstu.** Program mówi to wprost: *uczy się dopiero, co jest częste*.
Jest o 4,552 bitu gorszy niż samo liczenie częstości słów — czyli prosta tablica
częstotliwości wciąż bije wytrenowany model.

Widać też, gdzie się zatrzymał: strata na materiale uczącym skacze między 6,4 a 8,1, a na
odłożonym stoi na 8,55–8,65 od mniej więcej kroku 270. To jest granica tego modelu przy tym
kontekście.

Kontekst wynosił **8 symboli wstecz**. To bardzo mało — stąd następny przebieg zmienia
wyłącznie tę jedną rzecz na 32, żeby dało się powiedzieć, co dokładnie pomogło.

## Gdzie co leży

| Rzecz | Miejsce |
|---|---|
| Wagi (53 MB, 12 tensorów) | `~/models/noworodek/noworodek-darkstar.nwrd` na CBMS |
| Materiał do nauki | `~/models/noworodek/korpus.u16` (1 270 612 B) |
| Księga użyta do kodowania | `~/models/noworodek/ksiega-kopia.txt` |
| Zbieracz materiału | `tools/collect-corpus` w repozytorium |

**Księga Marcina jest nietknięta.** `C:\Users\User\Desktop\AIONS-CBMS\ksiazka-wspolna.txt`
otwierany wyłącznie do odczytu; wszystkie rozszerzenia powstały na kopii. Decyzja, czy
dopisać 3 853 nowe słowa do prawdziwej księgi, **należy do Marcina**.

Wagi celowo poza repozytorium — 53 MB i zasada projektu, że wagi nie idą na GitHuba.

## Powiązane

- `tools/collect-corpus` — zbieracz materiału
- `docs/operations/evidence/2026-09-05-embedder-on-darkstar.md` — model żywy na Darkstarze
- `docs/operations/evidence/2026-09-04-shadow-mode.md` — gniazdo, w które uczeń kiedyś wejdzie
