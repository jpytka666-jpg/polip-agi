<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-05-embedder-parity.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-05 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Sonnet 5
TIMESTAMP: 2026-09-05 02:00:00 Europe/London
REASON FOR CREATION: Etap 3 - silnik osadzen. Bez niego 4631 przeniesionych wspomnien bylo
osiagalnych tylko dla kogos, kto ma juz gotowy wektor. Ten dokument zapisuje, czym
zmierzono zgodnosc i co przy okazji wyszlo.
==========================================
-->

# Silnik osadzeń — zgodność z Chromą — 2026-09-05

## Werdykt

**Pamięci można zadać pytanie zwykłym zdaniem. Sprawdzone na żywym magazynie, nie na
atrapie.** Zgodność wektorów z tym, co policzyła Chroma: **1,000000000**, największa
różnica na pojedynczej liczbie **3 × 10⁻⁸**.

## Dlaczego akurat ta droga

Kryterium było jedno i twarde: **te same liczby co Chroma**. Silnik dający inne wektory
jest bezwartościowy, choćby najszybszy — 4 631 wspomnień przestałoby być odnajdywalnych.

Chroma liczyła te wektory silnikiem ONNX Runtime. Więc liczymy tym samym: **ten sam plik
modelu, ta sama biblioteka**. To nie jest podobieństwo, to jest tożsamość.

Dwie czysto rustowe drogi (`tract`, `candle`) odrzucone: każda musiałaby **odtworzyć**
sposób uśredniania, a różnica w tym jednym kroku to cichy rozjazd, który wychodzi dopiero
wtedy, gdy pamięć przestaje odpowiadać.

## Zmierzone, nie założone

```
cargo test -p darkstar-embed
  5 testow jednostkowych                                   PASS
  test zgodnosci na PRAWDZIWYCH wektorach z Chromy:
    wpis 0: 58 znakow | zgodnosc 1.000000000 | roznica 0.000000030
    wpis 1: 58 znakow | zgodnosc 1.000000000 | roznica 0.000000015
    wpis 2: 22 znakow | zgodnosc 1.000000000 | roznica 0.000000030
  test wpiecia w gniazdo cienia:
    silnik pasuje, 384 wymiary, wynik przez gniazdo IDENTYCZNY z bezposrednim

cargo clippy -p darkstar-embed --all-targets -- -D warnings    zero ostrzezen
cargo build --workspace                                        Finished
```

## Zapytanie żywej pamięci — dowód końcowy

Pytanie zadane po polsku, magazyn na CBMS, kolekcja z 1 269 wspomnieniami:

```
pytanie: "kompresja wag modelu jezykowego"
odpowiedz: 3 wspomnienia, oceny 0.523 / 0.483 / 0.471
tresc trafna - regula korzystania z pamieci oraz ustalenie o przepisywaniu plikow
```

Klucz dostępu **nie opuścił CBMS**: `recall-ask` świadomie nie łączy się z siecią, tylko
wypisuje treść zapytania, a ta idzie przez `ssh` na maszynę, gdzie klucz mieszka. Narzędzie,
które nie potrzebuje klucza, nie może go wypuścić.

## Co wyszło przy okazji — trafność zależy od języka wspomnienia

| Pytanie | Najlepsza ocena |
|---|---|
| kompresja wag modelu jezykowego | **0,523** |
| language model weight compression | 0,446 |
| regula pamieci sprawdzaj logi | 0,477 |
| memory rule check logs first | **0,586** |

Nie ma reguły „angielski trafia lepiej". **Trafia ten język, w którym zapisano wspomnienie.**
Zbiór jest mieszany, więc raz wygrywa jedno, raz drugie.

Oceny w przedziale 0,45–0,59 to skutek świadomego kompromisu: `all-MiniLM-L6-v2` jest
modelem przede wszystkim angielskim. Model wielojęzyczny trafiałby lepiej po polsku — ale
dawałby wektory **niezgodne** z 4 631 już zapisanymi. Zgodność była wymogiem wyższego rzędu.

## Ryzyko, które okazało się nie istnieć

Obcinanie długich tekstów było głównym podejrzanym o cichy rozjazd. Sprawdzone:

```
tokenizer.json → truncation: max_length 128, direction Right
                 padding:    Fixed 128, pad_id 0
```

Obcinanie i wypełnianie siedzą **w samym pliku tokenizera**, nie w kodzie. Ponieważ
wczytujemy dokładnie ten plik, którego używała Chroma, długie teksty tną się identycznie.

Co więcej: teksty w teście miały 22–58 znaków, czyli po dopełnieniu do 128 tokenów
**większość pozycji to wypełnienie** — a zgodność i tak wyszła idealna. To dowodzi, że
odsiewanie wypełnienia przy uśrednianiu działa poprawnie.

## Obejście, które okazało się ulepszeniem

Budowanie ze statycznym łączeniem **nie powiodło się na tym Windowsie**: 40 nierozwiązanych
symboli `__std_*`, czyli narzędzia Microsoftu starsze niż te, którymi zbudowano ONNX Runtime.

Zamiast naprawiać laptop — ładowanie gotowej biblioteki w czasie działania
(`ORT_DYLIB_PATH`). To dało coś więcej niż obejście: używamy **dokładnie tej binarki**,
którą liczyła Chroma, a nie „takiej samej". Przy okazji budowanie przestało wymagać
działającego łącznika C++, co ma znaczenie dla Darkstara, gdzie narzędzi budowania jeszcze
nie ma.

## Stan Darkstara (zmierzony 2026-09-05)

```
Rust/cargo             NIE zainstalowany
dysk glowny            47 GB wolnego ze 114 GB
/mnt/aionslab          307 GB wolnego z 477 GB
pamiec                 7,6 GB, dostepne 5,3 GB
procesor               i5-7200U, 4 watki, AVX2 tak
darkstar-recall        Up 3 hours (healthy)  <- magazyn pamieci dziala
darkstar-headscale     Up 5 hours (UNHEALTHY) <- osobna sprawa, poza tym etapem
/var/lib/darkstar/shadow   NIE ISTNIEJE
pliki modelu na CBMS       BRAK
```

## Czego jeszcze nie ma

**Silnik nie stoi na Darkstarze.** Działa na Windowsie. Do przeniesienia trzeba tam Rusta
(nie ma), plików modelu (nie ma) i biblioteki wnioskującej. To następny krok i wymaga
instalacji, czyli zgody.

**Uczeń nadal nie jest wpięty.** Gniazdo czeka, silnik żywy działa — brakuje implementacji
dla Noworodka. Ta droga jest teraz o tyle prostsza, że kontrakt jest sprawdzony w boju.

**Zbiór wzorcowy jest jednorazowy.** Trzy wpisy, w katalogu roboczym, poza repozytorium
(zawiera prawdziwą treść wspomnień). Przy zmianie sposobu liczenia trzeba go odtworzyć,
dopóki Chroma na Windowsie jeszcze chodzi.

**Test zgodności pomija się po cichu**, gdy brak zmiennych środowiskowych — wypisuje
`POMINIETO`, ale raportuje się jako zdany. W środowisku budującym bez modelu wygląda to jak
sukces. Do poprawienia.

## Powiązane

- `crates/darkstar-embed/` — silnik, test zgodności, narzędzie `recall-ask`
- `crates/darkstar-shadow/` — gniazdo, w które silnik się wpina
- `docs/operations/evidence/2026-09-04-shadow-mode.md` — gniazdo i reguła cienia
- `docs/operations/evidence/2026-09-04-memory-moved.md` — skąd wzięło się 4 631 wspomnień
- commity `00124dd`, `26d7c4b`
