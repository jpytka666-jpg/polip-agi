<!-- darkstar-header-v1 -->
<!-- po co: 2026-09-05-embedder-on-darkstar.md -->
<!-- nie wolno: hotspot, ruszac wlp2s0, wracac do 10.44, gasic DARKSTAR-WiFi, haslo w gicie -->
<!-- autor: Marcin -->
<!-- powstal: 2026-09-05 -->
<!--
THIS IS VERY IMPORTANT!!!
==========================================
AUTHOR: M. SZUL
AI MODEL: Claude Sonnet 5
TIMESTAMP: 2026-09-05 11:20:00 Europe/London
REASON FOR CREATION: Silnik osadzen stoi na Darkstarze i Windows przestal byc potrzebny do
zapytania pamieci. Dokument zapisuje takze blad wykryty przez wiekszy wzorzec - bo to jest
najwazniejsza rzecz, ktora sie tu wydarzyla.
==========================================
-->

# Silnik osadzeń na Darkstarze — 2026-09-05

## Werdykt

**Darkstar liczy sam i pyta sam. Windows nie jest już potrzebny do zapytania pamięci.**
Zgodność z wektorami Chromy na Ubuntu: **1,000000000 na 120 wpisach**, teksty od 135 do
5 743 znaków.

## Błąd, który wykrył większy wzorzec — najważniejsza rzecz w tym etapie

Poprzedni dowód opierał się na **trzech** wpisach po 22–58 znaków. Wzorzec rozszerzony do
**120 wpisów** padł przy pierwszym uruchomieniu:

```
wpis 0: 1117 znakow | zgodnosc 0.918353319 | najwieksza roznica 0.059509039
FAILED - ponizej progu 0.9999
```

Przyczyna:

```python
# chromadb/utils/embedding_functions/onnx_mini_lm_l6_v2.py:213
# max_seq_length = 256, for some reason sentence-transformers uses 256
# even though the HF config has a max length of 128
tokenizer.enable_truncation(max_length=256)
tokenizer.enable_padding(pad_id=0, pad_token="[PAD]", length=256)
```

Plik `tokenizer.json` deklaruje **128**. Chroma jawnie nadpisuje to na **256**. Skoro Chroma
policzyła tak wszystkie zapisane wspomnienia, **256 jest wartością prawdziwą, a 128 jedynie
zapisaną**. Czytanie pliku było błędem.

### Dlaczego to jest groźna klasa błędu

**Teksty krótsze niż 128 kawałków zgadzają się idealnie przy OBU ustawieniach.** Trzy krótkie
zdania dawały 1,000000000 i wyglądało to na dowód zgodności. Nie było nim.

Poprzedni dokument wymieniał zgodność na długich tekstach jako **niezweryfikowaną**. Miał
rację — i to była jedyna rzecz, która stała między tym błędem a cichym rozjazdem pamięci.

### Zabezpieczenie na przyszłość

Test zgodności **odrzuca teraz wzorzec, w którym najdłuższy tekst ma poniżej 600 znaków**,
z komunikatem tłumaczącym dlaczego. Wzorzec z samych krótkich tekstów przechodzi także przy
złym obcinaniu — czyli nie sprawdza tego, co obiecuje.

## Zmierzone na Darkstarze

```
maszyna: CBMS, Ubuntu, Intel i5-7200U (2 rdzenie / 4 watki), 7,6 GB RAM, bez GPU

budowanie:   cargo build --release -p darkstar-embed -j 2
             Finished release profile in 1m 28s          exit 0

testy:       5 passed  (jednostkowe)
             3 passed  (zgodnosc, gniazdo, pusty tekst)
             ZDANE: 120 wpisow, najgorsza zgodnosc 1.000000000

  wpis   0: 1117 znakow | 1.000000000 | roznica 0.000000089   <- ten padal przy 128
  wpis   1:  402 znaki  | 1.000000000 | roznica 0.000000045
  wpis  60:  546 znakow | 1.000000000 | roznica 0.000000045
  wpis 119:  412 znakow | 1.000000000 | roznica 0.000000060

szybkosc:    start zimny (wczytanie 86 MB + jedno liczenie): 0.39 / 0.37 / 0.37 s
             liczenie w serii: 120 wpisow w 6.90 s -> okolo 55 ms na wspomnienie
```

## Zapytanie pamięci w całości na Darkstarze

```
pytanie: "jak dziala pamiec systemu"   (25 znakow, po polsku)
policzono 384 wymiary modelem all-MiniLM-L6-v2
odpowiedz z magazynu, 3 trafienia: 0.438 / 0.427 / 0.388
```

Cała droga — model, liczenie, magazyn — wykonała się na CBMS. Klucz dostępu odczytany
lokalnie z `deploy/.env`, nie przeszedł przez sieć ani przez historię poleceń.

## Co postawiono i gdzie

| Rzecz | Miejsce | Uwaga |
|---|---|---|
| Rust | `~/.cargo/bin` | **był już zainstalowany** — zwiad go nie zobaczył, bo `which` nie widzi tej ścieżki przy połączeniu nieinteraktywnym |
| Biblioteka wnioskująca | `~/onnxruntime/onnxruntime-linux-x64-1.23.2/lib/libonnxruntime.so` | 23 MB, ta sama wersja co na Windowsie |
| Pliki modelu | `~/models/all-MiniLM-L6-v2/` | `model.onnx` 90 387 606 B — co do bajta jak na Windowsie |
| Wzorzec zgodności | `~/models/parity-reference.json` | 120 wpisów, 1,1 MB, **poza repozytorium** — zawiera treść wspomnień |
| Kod | `~/polip-agi` | gałąź `docs/darkstar-headscale-hotspot-plan`, commit `ae115b7` |

Nic nie wymagało uprawnień administratora: `rustup` instaluje do katalogu użytkownika,
model i biblioteka leżą w `~`, magazyn pamięci już działał.

## Czego nadal nie ma

**Nic nie uruchamia tego samo.** Silnik działa, gdy się go wywoła — nie ma usługi, która
podnosi się przy starcie maszyny. Zapytanie pamięci wymaga dziś ręcznego polecenia.

**Zmienne środowiskowe trzeba podawać za każdym razem.** `ORT_DYLIB_PATH` i
`DARKSTAR_EMBED_MODEL_DIR` nie są nigdzie zapisane na stałe.

**Uczeń nadal nie wpięty.** Gniazdo działa, model żywy stoi — brakuje implementacji dla
Noworodka.

**Wzorzec jest jednorazowy i pochodzi z jednej kolekcji.** 120 wpisów z jednego zbioru.
Chroma na Windowsie nadal działa, więc da się go poszerzyć o inne zbiory — warto, dopóki
działa.

## Powiązane

- `docs/operations/evidence/2026-09-05-embedder-parity.md` — pierwsza zgodność, na trzech wpisach
- `docs/operations/evidence/2026-09-04-shadow-mode.md` — gniazdo dla ucznia
- commit `ae115b7` — naprawa obcinania
