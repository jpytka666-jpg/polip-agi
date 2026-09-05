<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-05-zbior-testowy-wybor-znaczenia.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-05 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Sonnet 5
TIMESTAMP: 2026-09-05 23:55:00 Europe/London
REASON FOR CREATION: Marcin: "Przyklady i oczekiwane wyniki ustal przed pomiarem."
Ten plik powstaje i jest zapisywany w repozytorium ZANIM pomiar zostanie uruchomiony.
Kolejnosc ma byc udowodniona commitem, a nie zadeklarowana po fakcie.
==========================================
-->

# Zbiór testowy wyboru znaczenia — zapisany PRZED pomiarem

## Po co ten plik

Pomiar, którego oczekiwane wyniki ustala się po zobaczeniu odpowiedzi, nie jest pomiarem.
Ten zbiór jest zapisany i wysłany do repozytorium **zanim** cokolwiek zostało uruchomione.
Jeśli wynik wyjdzie słaby, ten plik zostaje bez zmian.

## Czego dotyczy pomiar

Dla słowa angielskiego ESPDIC podaje **kilku kandydatów** — wszystkie prawdziwe, ale tylko
jeden pasuje do zamierzonego znaczenia. Pytanie brzmi: czy `all-MiniLM-L6-v2`, **ten sam
model, który już leży na Darkstarze**, potrafi wskazać właściwego.

Mierzone są **dwie osobne rzeczy**:

1. **Obecność** — czy poprawny rdzeń w ogóle jest wśród kandydatów. To mierzy słownik,
   nie model. Zmierzone już wcześniej: 20/20.
2. **Wybór** — czy model wskazał właśnie ten. To mierzy model.

Druga liczba jest jedyną nową rzeczą, jakiej ten pomiar dotyczy.

## Część A — 20 słów bez kontekstu

Znaczenie zamierzone to znaczenie **podstawowe, słownikowe**. Rdzenie są standardowe
i niesporne.

| angielskie | oczekiwany rdzeń | znaczenie zamierzone |
|---|---|---|
| memory | memoro | pamięć jako zdolność zapamiętywania |
| error | eraro | pomyłka, błąd |
| code | kodo | kod, zapis instrukcji |
| vector | vektoro | wektor w sensie matematycznym |
| knowledge | scio | wiedza |
| similarity | simileco | podobieństwo |
| function | funkcio | funkcja |
| direction | direkto | kierunek |
| word | vorto | słowo |
| language | lingvo | język |
| book | libro | książka |
| water | akvo | woda |
| house | domo | dom |
| time | tempo | czas jako trwanie |
| number | nombro | liczba |
| name | nomo | imię, nazwa |
| day | tago | dzień |
| hand | mano | dłoń |
| friend | amiko | przyjaciel |
| work | laboro | praca jako czynność |

## Część B — to samo słowo w dwóch znaczeniach, w całych zdaniach

Tu wybór **musi** zależeć od kontekstu. Jeśli model zwróci ten sam rdzeń dla obu zdań
z pary, znaczy to, że kontekstu nie używa — i to jest wynik równie ważny jak trafienie.

| zdanie | oczekiwany rdzeń | dlaczego |
|---|---|---|
| `the code compiles without errors` | kodo | kod programu |
| `the message was written in a secret code` | ĉifro | szyfr |
| `the computer has four gigabytes of memory` | memorilo | pamięć jako urządzenie |
| `he has a clear memory of that day` | memoro | pamięć jako zdolność |
| `add the two vectors together` | vektoro | wektor matematyczny |
| `the mosquito is a vector of the disease` | vehiklo | nosiciel |
| `she went to work early` | laboro | praca jako zajęcie |
| `this painting is his finest work` | verko | dzieło |
| `what time is it` | horo | godzina |
| `it happened three times` | fojo | raz, krotność |
| `count the apples in the basket` | nombri | liczyć |
| `the count lived in the castle` | grafo | hrabia |

## Jak będzie liczone

Dla każdego przypadku:

1. wziąć **wszystkich** kandydatów z ESPDIC dla danego słowa angielskiego,
2. osadzić zapytanie — w części A samo słowo, w części B całe zdanie,
3. osadzić **angielską definicję** każdego kandydata z ESPDIC,
4. wybrać kandydata o najbliższym wektorze,
5. odnotować: czy poprawny był wśród kandydatów **oraz** czy został wybrany.

Wynik podany będzie jako dwie osobne liczby, nigdy jako jedna.

## Czego ten pomiar NIE rozstrzygnie

- **Nie dotyczy polskiego.** MiniLM jest modelem angielskim; polski wymaga osobnego pomiaru.
- **Nie dotyczy rozumienia przez Noworodka.** Rozbiór słowa na klocki nie jest dowodem, że
  model umie z tych klocków składać znaczenie. Tego nie mierzyliśmy i dopóki nie zmierzymy,
  nie wolno tego przedstawiać jako działającego.
- **Nie zmienia księgi ani niczego, co dziś działa.**

## Wynik

Wpisany zostanie do osobnego pliku po uruchomieniu, z odwołaniem do commita, w którym ten
zbiór został zapisany.
